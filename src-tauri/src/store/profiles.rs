use super::Store;
use crate::shared::Profile;

use std::collections::HashMap;
use std::fs;
use std::iter::repeat_with;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use anyhow::Context;
use once_cell::sync::Lazy;
use tokio::sync::{Mutex, MutexGuard};

pub struct ProfileStores {
	stores: HashMap<String, Store<Profile>>,
}

impl ProfileStores {
	pub fn get_profile_store(&mut self, device: &crate::devices::DeviceInfo, id: &str, app: &tauri::AppHandle) -> Result<&mut Store<Profile>, anyhow::Error> {
		let path = PathBuf::from("profiles").join(&device.id).join(id);
		let path = path.to_str().unwrap();

		if self.stores.contains_key(path) {
			Ok(self.stores.get_mut(path).unwrap())
		} else {
			let default = Profile {
				id: id.to_owned(),
				keys: repeat_with(Vec::new).take((device.rows * device.columns).into()).collect(),
				sliders: repeat_with(Vec::new).take(device.sliders.into()).collect(),
			};

			let store = Store::new(path, app.path_resolver().app_config_dir().unwrap(), default).context(format!("Failed to create store for profile {}", path))?;
			store.save()?;

			self.stores.insert(path.to_owned(), store);
			Ok(self.stores.get_mut(path).unwrap())
		}
	}

	pub fn remove_profile(&mut self, device: &str, id: &str, app: &tauri::AppHandle) {
		self.stores.remove(id);
		let config_dir = app.path_resolver().app_config_dir().unwrap();
		let path = config_dir.join("profiles").join(device).join(id);
		let _ = fs::remove_file(path);
	}

	pub fn all_from_plugin(&self, plugin: &str) -> Vec<crate::shared::ActionContext> {
		let mut all = vec![];
		for store in self.stores.values() {
			for slot in store.value.keys.iter().chain(&store.value.sliders) {
				for instance in slot {
					if instance.action.plugin == plugin {
						all.push(instance.context.clone());
					}
				}
			}
		}
		all
	}
}

#[derive(Serialize, Deserialize)]
pub struct DeviceConfig {
	pub selected_profile: String,
}

pub struct DeviceStores {
	stores: HashMap<String, Store<DeviceConfig>>,
}

impl DeviceStores {
	pub fn get_device_store(&mut self, device: &str, app: &tauri::AppHandle) -> Result<&mut Store<DeviceConfig>, anyhow::Error> {
		if self.stores.contains_key(device) {
			Ok(self.stores.get_mut(device).unwrap())
		} else {
			let default = DeviceConfig {
				selected_profile: String::from("Default"),
			};

			let path = PathBuf::from("profiles").join(device);
			let store = Store::new(path.to_str().unwrap(), app.path_resolver().app_config_dir().unwrap(), default).context(format!("Failed to create store for device config {}", device))?;
			store.save()?;

			self.stores.insert(device.to_owned(), store);
			Ok(self.stores.get_mut(device).unwrap())
		}
	}
}

pub fn get_device_profiles(device: &str, app: &tauri::AppHandle) -> Result<Vec<String>, anyhow::Error> {
	let mut profiles: Vec<String> = vec![];

	let device_path = app.path_resolver().app_config_dir().unwrap().join("profiles").join(device);
	fs::create_dir_all(&device_path)?;
	let entries = fs::read_dir(device_path)?;

	for entry in entries.flatten() {
		if entry.metadata()?.is_file() && entry.file_name() != "config.json" {
			profiles.push(entry.file_name().to_string_lossy()[..entry.file_name().len() - 5].to_owned());
		}
	}

	if profiles.is_empty() {
		profiles.push("Default".to_owned());
	}

	Ok(profiles)
}

/// A singleton object to contain all active Store instances that hold a profile.
pub static PROFILE_STORES: Lazy<Mutex<ProfileStores>> = Lazy::new(|| Mutex::new(ProfileStores { stores: HashMap::new() }));

/// A singleton object to manage Store instances for device configurations.
pub static DEVICE_STORES: Lazy<Mutex<DeviceStores>> = Lazy::new(|| Mutex::new(DeviceStores { stores: HashMap::new() }));

pub struct Locks<'a> {
	pub device_stores: MutexGuard<'a, DeviceStores>,
	pub devices: MutexGuard<'a, HashMap<String, crate::devices::DeviceInfo>>,
	pub profile_stores: MutexGuard<'a, ProfileStores>,
}

pub async fn lock_mutexes() -> Locks<'static> {
	let device_stores = DEVICE_STORES.lock().await;
	let devices = crate::devices::DEVICES.lock().await;
	let profile_stores = PROFILE_STORES.lock().await;
	Locks {
		device_stores,
		devices,
		profile_stores,
	}
}

pub async fn get_slot<'a>(context: &crate::shared::Context, locks: &'a mut Locks<'_>) -> Result<&'a mut Vec<crate::shared::ActionInstance>, anyhow::Error> {
	let device = locks.devices.get(&context.device).unwrap();
	let store = locks.profile_stores.get_profile_store(device, &context.profile, crate::APP_HANDLE.get().unwrap())?;
	let profile = &mut store.value;

	let configured = match &context.controller[..] {
		"Encoder" => &mut profile.sliders[context.position as usize],
		_ => &mut profile.keys[context.position as usize],
	};

	Ok(configured)
}

pub async fn get_instance<'a>(context: &crate::shared::ActionContext, locks: &'a mut Locks<'_>) -> Result<Option<&'a mut crate::shared::ActionInstance>, anyhow::Error> {
	let slot = get_slot(&(context.into()), locks).await?;
	for instance in slot {
		if instance.context == *context {
			return Ok(Some(instance));
		}
	}
	Ok(None)
}

pub async fn save_profile<'a>(device: &str, locks: &'a mut Locks<'_>) -> Result<(), anyhow::Error> {
	let selected_profile = &locks.device_stores.get_device_store(device, crate::APP_HANDLE.get().unwrap())?.value.selected_profile;
	let device = locks.devices.get(device).unwrap();
	let store = locks.profile_stores.get_profile_store(device, selected_profile, crate::APP_HANDLE.get().unwrap())?;
	store.save()
}
