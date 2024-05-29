use super::Store;
use crate::shared::Profile;

use std::collections::HashMap;
use std::fs;
use std::iter::repeat_with;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use anyhow::Context;
use once_cell::sync::Lazy;
use tokio::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

pub struct ProfileStores {
	stores: HashMap<String, Store<Profile>>,
}

impl ProfileStores {
	pub fn get_profile_store(&self, device: &crate::devices::DeviceInfo, id: &str) -> Result<&Store<Profile>, anyhow::Error> {
		let path = PathBuf::from("profiles").join(&device.id).join(id);
		let path = path.to_str().unwrap();

		if self.stores.contains_key(path) {
			Ok(self.stores.get(path).unwrap())
		} else {
			Err(anyhow::anyhow!("profile not found"))
		}
	}

	pub fn get_profile_store_mut(&mut self, device: &crate::devices::DeviceInfo, id: &str, app: &tauri::AppHandle) -> Result<&mut Store<Profile>, anyhow::Error> {
		#[cfg(target_os = "windows")]
		let path = PathBuf::from("profiles").join(&device.id).join(id.replace('/', "\\"));
		#[cfg(not(target_os = "windows"))]
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
		#[cfg(target_os = "windows")]
		let id = &id.replace('/', "\\");
		let path = config_dir.join("profiles").join(device).join(format!("{id}.json"));
		let _ = fs::remove_file(&path);
		// This is safe as `remove_dir` errors if the directory is not empty.
		let _ = fs::remove_dir(path.parent().unwrap());
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
	pub fn get_selected_profile(&self, device: &str) -> &str {
		if self.stores.contains_key(device) {
			&self.stores.get(device).unwrap().value.selected_profile
		} else {
			"Default"
		}
	}

	pub fn set_selected_profile(&mut self, device: &str, id: String, app: &tauri::AppHandle) -> Result<(), anyhow::Error> {
		if self.stores.contains_key(device) {
			self.stores.get_mut(device).unwrap().value.selected_profile = id;
		} else {
			let default = DeviceConfig { selected_profile: id };

			let path = PathBuf::from("profiles").join(device);
			let store = Store::new(path.to_str().unwrap(), app.path_resolver().app_config_dir().unwrap(), default).context(format!("Failed to create store for device config {}", device))?;
			store.save()?;

			self.stores.insert(device.to_owned(), store);
		}
		Ok(())
	}
}

pub fn get_device_profiles(device: &str, app: &tauri::AppHandle) -> Result<Vec<String>, anyhow::Error> {
	let mut profiles: Vec<String> = vec![];

	let device_path = app.path_resolver().app_config_dir().unwrap().join("profiles").join(device);
	fs::create_dir_all(&device_path)?;
	let entries = fs::read_dir(device_path)?;

	for entry in entries.flatten() {
		if entry.metadata()?.is_file() {
			profiles.push(entry.file_name().to_string_lossy()[..entry.file_name().len() - 5].to_owned());
		} else if entry.metadata()?.is_dir() {
			let entries = fs::read_dir(entry.path())?;
			for subentry in entries.flatten() {
				if subentry.metadata()?.is_file() {
					profiles.push(format!(
						"{}/{}",
						entry.file_name().to_string_lossy(),
						subentry.file_name().to_string_lossy()[..subentry.file_name().len() - 5].to_owned()
					));
				}
			}
		}
	}

	if profiles.is_empty() {
		profiles.push("Default".to_owned());
	}

	Ok(profiles)
}

/// A singleton object to contain all active Store instances that hold a profile.
pub static PROFILE_STORES: Lazy<RwLock<ProfileStores>> = Lazy::new(|| RwLock::new(ProfileStores { stores: HashMap::new() }));

/// A singleton object to manage Store instances for device configurations.
pub static DEVICE_STORES: Lazy<RwLock<DeviceStores>> = Lazy::new(|| RwLock::new(DeviceStores { stores: HashMap::new() }));

pub struct Locks<'a> {
	pub device_stores: RwLockReadGuard<'a, DeviceStores>,
	pub devices: RwLockReadGuard<'a, HashMap<String, crate::devices::DeviceInfo>>,
	pub profile_stores: RwLockReadGuard<'a, ProfileStores>,
}

pub async fn acquire_locks() -> Locks<'static> {
	let device_stores = DEVICE_STORES.read().await;
	let devices = crate::devices::DEVICES.read().await;
	let profile_stores = PROFILE_STORES.read().await;
	Locks {
		device_stores,
		devices,
		profile_stores,
	}
}

pub struct LocksMut<'a> {
	pub device_stores: RwLockWriteGuard<'a, DeviceStores>,
	pub devices: RwLockWriteGuard<'a, HashMap<String, crate::devices::DeviceInfo>>,
	pub profile_stores: RwLockWriteGuard<'a, ProfileStores>,
}

pub async fn acquire_locks_mut() -> LocksMut<'static> {
	let device_stores = DEVICE_STORES.write().await;
	let devices = crate::devices::DEVICES.write().await;
	let profile_stores = PROFILE_STORES.write().await;
	LocksMut {
		device_stores,
		devices,
		profile_stores,
	}
}

pub async fn get_slot<'a>(context: &crate::shared::Context, locks: &'a Locks<'_>) -> Result<&'a Vec<crate::shared::ActionInstance>, anyhow::Error> {
	let device = locks.devices.get(&context.device).unwrap();
	let store = locks.profile_stores.get_profile_store(device, &context.profile)?;

	let configured = match &context.controller[..] {
		"Encoder" => &store.value.sliders[context.position as usize],
		_ => &store.value.keys[context.position as usize],
	};

	Ok(configured)
}

pub async fn get_slot_mut<'a>(context: &crate::shared::Context, locks: &'a mut LocksMut<'_>) -> Result<&'a mut Vec<crate::shared::ActionInstance>, anyhow::Error> {
	let device = locks.devices.get(&context.device).unwrap();
	let store = locks.profile_stores.get_profile_store_mut(device, &context.profile, crate::APP_HANDLE.get().unwrap())?;

	let configured = match &context.controller[..] {
		"Encoder" => &mut store.value.sliders[context.position as usize],
		_ => &mut store.value.keys[context.position as usize],
	};

	Ok(configured)
}

pub async fn get_instance<'a>(context: &crate::shared::ActionContext, locks: &'a Locks<'_>) -> Result<Option<&'a crate::shared::ActionInstance>, anyhow::Error> {
	let slot = get_slot(&(context.into()), locks).await?;
	for instance in slot {
		if instance.context == *context {
			return Ok(Some(instance));
		}
	}
	Ok(None)
}

pub async fn get_instance_mut<'a>(context: &crate::shared::ActionContext, locks: &'a mut LocksMut<'_>) -> Result<Option<&'a mut crate::shared::ActionInstance>, anyhow::Error> {
	let slot = get_slot_mut(&(context.into()), locks).await?;
	for instance in slot {
		if instance.context == *context {
			return Ok(Some(instance));
		}
	}
	Ok(None)
}

pub async fn save_profile<'a>(device: &str, locks: &'a mut LocksMut<'_>) -> Result<(), anyhow::Error> {
	let selected_profile = locks.device_stores.get_selected_profile(device);
	let device = locks.devices.get(device).unwrap();
	let store = locks.profile_stores.get_profile_store(device, selected_profile)?;
	store.save()
}
