use crate::devices::DEVICES;
use crate::shared::{Action, ActionContext, ActionInstance, CATEGORIES};
use crate::store::profiles::{get_device_profiles, DEVICE_STORES, PROFILE_STORES};

use std::collections::HashMap;

use tauri::Manager;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Error {
	pub description: String,
}

impl From<anyhow::Error> for Error {
	fn from(error: anyhow::Error) -> Self {
		Self { description: error.to_string() }
	}
}

#[tauri::command]
pub async fn get_devices() -> HashMap<std::string::String, crate::devices::DeviceInfo> {
	DEVICES.lock().await.clone()
}

pub async fn update_devices() {
	let app = crate::APP_HANDLE.lock().await;
	let app = app.as_ref().unwrap();

	let _ = app.get_window("main").unwrap().emit("devices", DEVICES.lock().await.clone());
}

#[tauri::command]
pub async fn get_categories() -> HashMap<std::string::String, Vec<Action>> {
	CATEGORIES.lock().await.clone()
}

#[tauri::command]
pub fn get_profiles(app: tauri::AppHandle, device: &str) -> Result<Vec<String>, Error> {
	Ok(get_device_profiles(device, &app)?)
}

#[tauri::command]
pub async fn get_selected_profile(app: tauri::AppHandle, device: String) -> Result<crate::shared::Profile, Error> {
	let mut device_stores = DEVICE_STORES.lock().await;
	let mut profile_stores = PROFILE_STORES.lock().await;

	let device_store = device_stores.get_device_store(&device, &app)?;
	let profile = profile_stores.get_profile_store(DEVICES.lock().await.get(&device).unwrap(), &device_store.value.selected_profile, &app)?;

	Ok(profile.value.clone())
}

#[allow(clippy::flat_map_identity)]
#[tauri::command]
pub async fn set_selected_profile(app: tauri::AppHandle, device: String, id: String) -> Result<(), Error> {
	let mut device_stores = DEVICE_STORES.lock().await;
	let devices = DEVICES.lock().await;
	let mut profile_stores = PROFILE_STORES.lock().await;
	let store = device_stores.get_device_store(&device, &app)?;

	if store.value.selected_profile != id {
		let old_profile = &profile_stores.get_profile_store(devices.get(&device).unwrap(), &store.value.selected_profile, &app)?.value;
		for slot in old_profile.keys.iter().chain(&old_profile.sliders).flat_map(|x| x) {
			for instance in slot {
				let _ = crate::events::outbound::will_appear::will_disappear(instance).await;
			}
		}
	}

	let new_profile = &profile_stores.get_profile_store(devices.get(&device).unwrap(), &id, &app)?.value;
	for slot in new_profile.keys.iter().chain(&new_profile.sliders).flat_map(|x| x) {
		for instance in slot {
			let _ = crate::events::outbound::will_appear::will_appear(instance).await;
		}
	}

	store.value.selected_profile = id.to_owned();
	store.save()?;

	Ok(())
}

#[tauri::command]
pub async fn delete_profile(app: tauri::AppHandle, device: String, profile: String) {
	let mut profile_stores = PROFILE_STORES.lock().await;
	profile_stores.remove_profile(&device, &profile, &app);
}

#[tauri::command]
pub async fn create_instance(app: tauri::AppHandle, action: Action, context: ActionContext) -> Result<Option<ActionInstance>, Error> {
	if !action.controllers.contains(&context.controller) {
		return Ok(None);
	}

	let instance = ActionInstance {
		action: action.clone(),
		context: context.clone(),
		states: action.states.clone(),
		current_state: 0,
		settings: serde_json::Value::Object(serde_json::Map::new()),
	};

	let mut profile_stores = PROFILE_STORES.lock().await;
	let store = profile_stores.get_profile_store(DEVICES.lock().await.get(&context.device).unwrap(), &context.profile, &app)?;

	if context.controller == "Encoder" {
		if let Some(old) = &store.value.sliders[context.position as usize] {
			for instance in old {
				let _ = crate::events::outbound::will_appear::will_disappear(instance).await;
			}
		}
		store.value.sliders[context.position as usize] = Some(vec![instance.clone()]);
	} else {
		if let Some(old) = &store.value.keys[context.position as usize] {
			for instance in old {
				let _ = crate::events::outbound::will_appear::will_disappear(instance).await;
			}
		}
		store.value.keys[context.position as usize] = Some(vec![instance.clone()]);
	}

	let _ = crate::events::outbound::will_appear::will_appear(&instance).await;

	store.save()?;

	Ok(Some(instance))
}

#[tauri::command]
pub async fn move_instance(app: tauri::AppHandle, mut instance: ActionInstance, context: ActionContext) -> Result<Option<ActionInstance>, Error> {
	if !instance.action.controllers.contains(&context.controller) {
		return Ok(None);
	}

	let mut profile_stores = PROFILE_STORES.lock().await;
	let store = profile_stores.get_profile_store(DEVICES.lock().await.get(&context.device).unwrap(), &context.profile, &app)?;

	let _ = crate::events::outbound::will_appear::will_disappear(&instance).await;
	if instance.context.controller == "Encoder" {
		store.value.sliders[instance.context.position as usize] = None;
	} else {
		store.value.keys[instance.context.position as usize] = None;
	}

	if context.controller == "Encoder" {
		if let Some(old) = &store.value.sliders[context.position as usize] {
			for instance in old {
				let _ = crate::events::outbound::will_appear::will_disappear(instance).await;
			}
		}
		instance.context = context.clone();
		store.value.sliders[context.position as usize] = Some(vec![instance.clone()]);
	} else {
		if let Some(old) = &store.value.keys[context.position as usize] {
			for instance in old {
				let _ = crate::events::outbound::will_appear::will_disappear(instance).await;
			}
		}
		instance.context = context.clone();
		store.value.keys[context.position as usize] = Some(vec![instance.clone()]);
	}

	let _ = crate::events::outbound::will_appear::will_appear(&instance).await;

	store.save()?;

	Ok(Some(instance))
}

#[tauri::command]
pub async fn clear_slot(app: tauri::AppHandle, context: ActionContext) -> Result<(), Error> {
	let mut profile_stores = PROFILE_STORES.lock().await;
	let store = profile_stores.get_profile_store(DEVICES.lock().await.get(&context.device).unwrap(), &context.profile, &app)?;

	if context.controller == "Encoder" {
		if let Some(slot) = &store.value.sliders[context.position as usize] {
			for instance in slot {
				let _ = crate::events::outbound::will_appear::will_disappear(instance).await;
			}
		}
		store.value.sliders[context.position as usize] = None;
	} else {
		if let Some(slot) = &store.value.keys[context.position as usize] {
			for instance in slot {
				let _ = crate::events::outbound::will_appear::will_disappear(instance).await;
			}
		}
		store.value.keys[context.position as usize] = None;
	}

	store.save()?;

	Ok(())
}

#[tauri::command]
pub async fn make_info(app: tauri::AppHandle, plugin: String) -> Result<crate::plugins::info_param::Info, Error> {
	let mut path = app.path_resolver().app_config_dir().unwrap();
	path.push("plugins");
	path.push(&plugin);
	path.push("manifest.json");

	let manifest = match std::fs::read(&path) {
		Ok(data) => data,
		Err(error) => return Err(anyhow::Error::from(error).into()),
	};

	let manifest: crate::plugins::manifest::PluginManifest = match serde_json::from_slice(&manifest) {
		Ok(manifest) => manifest,
		Err(error) => return Err(anyhow::Error::from(error).into()),
	};

	Ok(crate::plugins::info_param::make_info(plugin, manifest.version).await)
}

#[tauri::command]
pub async fn switch_property_inspector(old: Option<ActionContext>, new: Option<ActionContext>) {
	if let Some(context) = old {
		let _ = crate::events::outbound::property_inspector::property_inspector_did_appear(context, "propertyInspectorDidDisappear").await;
	}
	if let Some(context) = new {
		let _ = crate::events::outbound::property_inspector::property_inspector_did_appear(context, "propertyInspectorDidAppear").await;
	}
}

#[tauri::command]
pub async fn update_image(context: ActionContext, image: String) {
	if context.device.starts_with("sd-") {
		if let Err(error) = crate::devices::elgato::update_image(&context, &image).await {
			log::warn!("Failed to update device image at context {} with image {}: {}", context, image, error);
		}
	}
}

pub async fn update_state(app: &tauri::AppHandle, instance: &ActionInstance) -> Result<(), tauri::Error> {
	let window = app.get_window("main").unwrap();
	window.emit("update_state", instance)?;
	Ok(())
}
