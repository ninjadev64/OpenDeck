use super::Store;
use crate::shared::Profile;

use std::collections::HashMap;
use std::fs;
use std::iter::repeat_with;

use serde::{Deserialize, Serialize};

use anyhow::Context;
use lazy_static::lazy_static;
use tokio::sync::{Mutex, MutexGuard};

pub struct ProfileStores {
	stores: HashMap<String, Store<Profile>>,
}

impl ProfileStores {
	pub fn get_profile_store(&mut self, device: &crate::devices::DeviceInfo, id: &str, app: &tauri::AppHandle) -> Result<&mut Store<Profile>, anyhow::Error> {
		let path = format!("profiles/{}/{}", device.id, id);

		if self.stores.contains_key(&path) {
			Ok(self.stores.get_mut(&path).unwrap())
		} else {
			let default = Profile {
				device: device.id.clone(),
				id: id.to_owned(),
				keys: repeat_with(|| None).take((device.rows * device.columns).into()).collect(),
				sliders: repeat_with(|| None).take(device.sliders.into()).collect(),
			};

			let store = Store::new(&path, app.path_resolver().app_config_dir().unwrap(), default).with_context(|| format!("Failed to create store for profile {}", path))?;

			store.save().with_context(|| format!("Failed to save store for profile {}", path))?;

			self.stores.insert(path.clone(), store);
			Ok(self.stores.get_mut(&path).unwrap())
		}
	}

	pub fn remove_profile(&mut self, device: &str, id: &str, app: &tauri::AppHandle) {
		self.stores.remove(id);
		let config_dir = app.path_resolver().app_config_dir().unwrap();
		let path = config_dir.join(format!("profiles/{}/{}.json", device, id));
		let _ = fs::remove_file(path);
	}

	pub fn all_from_plugin(&self, plugin: &str) -> Vec<crate::shared::ActionContext> {
		let mut all = vec![];
		for store in self.stores.values() {
			for instance in store.value.keys.iter().chain(&store.value.sliders).flatten() {
				if instance.action.plugin == plugin {
					all.push(instance.context.clone());
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

			let store =
				Store::new(&format!("profiles/{}", device), app.path_resolver().app_config_dir().unwrap(), default).with_context(|| format!("Failed to create config store for device {}", device))?;

			store.save().with_context(|| format!("Failed to save config store for device {}", device))?;

			self.stores.insert(device.to_owned(), store);
			Ok(self.stores.get_mut(device).unwrap())
		}
	}
}

pub fn get_device_profiles(device: &str, app: &tauri::AppHandle) -> Result<Vec<String>, anyhow::Error> {
	let mut profiles: Vec<String> = vec![];

	let device_path = app.path_resolver().app_config_dir().unwrap().join(format!("profiles/{}", device));
	fs::create_dir_all(&device_path).with_context(|| format!("Failed to create directories for device {}", device))?;

	let entries = fs::read_dir(device_path).with_context(|| format!("Failed to read directory for device {}", device))?;

	for entry in entries.flatten() {
		if entry.metadata().unwrap().is_file() && entry.file_name() != "config.json" {
			profiles.push(entry.file_name().to_string_lossy()[..entry.file_name().len() - 5].to_owned());
		}
	}

	if profiles.is_empty() {
		profiles.push("Default".to_owned());
	}

	Ok(profiles)
}

lazy_static! {
	/// A singleton object to contain all active Store instances that hold a profile.
	pub static ref PROFILE_STORES: Mutex<ProfileStores> = Mutex::new(ProfileStores { stores: HashMap::new() });

	/// A singleton object to manage Store instances for device configurations.
	pub static ref DEVICE_STORES: Mutex<DeviceStores> = Mutex::new(DeviceStores { stores: HashMap::new() });
}

pub async fn lock_mutexes() -> (
	MutexGuard<'static, Option<tauri::AppHandle>>,
	MutexGuard<'static, DeviceStores>,
	MutexGuard<'static, HashMap<String, crate::devices::DeviceInfo>>,
	MutexGuard<'static, ProfileStores>,
) {
	let app = crate::APP_HANDLE.lock().await;
	let device_stores = DEVICE_STORES.lock().await;
	let devices = crate::devices::DEVICES.lock().await;
	let profile_stores = PROFILE_STORES.lock().await;
	(app, device_stores, devices, profile_stores)
}

pub async fn get_instance(device: &str, position: u8, controller: &str) -> Result<Option<crate::shared::ActionInstance>, anyhow::Error> {
	let (app, mut device_stores, devices, mut profile_stores) = lock_mutexes().await;

	let selected_profile = &device_stores.get_device_store(device, app.as_ref().unwrap())?.value.selected_profile;
	let device = devices.get(device).unwrap();
	let store = profile_stores.get_profile_store(device, selected_profile, app.as_ref().unwrap())?;
	let profile = &store.value;

	let configured = match controller {
		"Encoder" => profile.sliders[position as usize].as_ref(),
		_ => profile.keys[position as usize].as_ref(),
	};

	match configured {
		Some(configured) => Ok(Some(configured.clone())),
		None => Ok(None),
	}
}
