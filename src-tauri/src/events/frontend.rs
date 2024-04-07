use crate::devices::DEVICES;
use crate::shared::{Action, ActionContext, ActionInstance, Context, CATEGORIES};
use crate::store::profiles::{get_device_profiles, get_slot, lock_mutexes, save_profile, Locks, DEVICE_STORES, PROFILE_STORES};

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
	let app = crate::APP_HANDLE.get().unwrap();
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
		for slot in old_profile.keys.iter().chain(&old_profile.sliders) {
			for instance in slot {
				let _ = crate::events::outbound::will_appear::will_disappear(instance, slot.len() > 1).await;
			}
		}
	}

	let new_profile = &profile_stores.get_profile_store(devices.get(&device).unwrap(), &id, &app)?.value;
	for slot in new_profile.keys.iter().chain(&new_profile.sliders) {
		for instance in slot {
			let _ = crate::events::outbound::will_appear::will_appear(instance, slot.len() > 1).await;
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
pub async fn create_instance(action: Action, context: Context) -> Result<Option<Vec<ActionInstance>>, Error> {
	if !action.controllers.contains(&context.controller) {
		return Ok(None);
	}

	let mut locks = lock_mutexes().await;
	let slot = get_slot(&context, &mut locks).await?;
	let index = match slot.last() {
		None => 0,
		Some(instance) => instance.context.index + 1,
	};

	let instance = ActionInstance {
		action: action.clone(),
		context: ActionContext::from_context(context.clone(), index),
		states: action.states.clone(),
		current_state: 0,
		settings: serde_json::Value::Object(serde_json::Map::new()),
	};

	slot.push(instance.clone());
	let slot = slot.clone();

	save_profile(&context.device, &mut locks).await?;
	let _ = crate::events::outbound::will_appear::will_appear(&instance, index != 0).await;

	Ok(Some(slot))
}

#[tauri::command]
pub async fn move_slot(source: Context, destination: Context) -> Result<Option<Vec<ActionInstance>>, Error> {
	if source.controller != destination.controller {
		return Ok(None);
	}

	let mut locks = lock_mutexes().await;
	let src = get_slot(&source, &mut locks).await?;
	let multi_action = src.len() > 1;

	let mut vec: Vec<ActionInstance> = vec![];

	for (index, instance) in src.iter_mut().enumerate() {
		let mut new = instance.clone();
		new.context = ActionContext::from_context(destination.clone(), index as u16);
		vec.push(new);
	}

	let dst = get_slot(&destination, &mut locks).await?;
	if !dst.is_empty() {
		return Ok(None);
	}
	*dst = vec.clone();

	let src = get_slot(&source, &mut locks).await?;
	for old in &*src {
		let _ = crate::events::outbound::will_appear::will_disappear(old, multi_action).await;
	}
	*src = vec![];
	for new in &vec {
		let _ = crate::events::outbound::will_appear::will_appear(new, multi_action).await;
	}

	save_profile(&destination.device, &mut locks).await?;

	Ok(Some(vec))
}

#[tauri::command]
pub async fn clear_slot(context: Context) -> Result<(), Error> {
	let mut locks = lock_mutexes().await;
	let slot = get_slot(&context, &mut locks).await?;

	for instance in &*slot {
		let _ = crate::events::outbound::will_appear::will_disappear(instance, slot.len() > 1).await;
	}

	*slot = vec![];
	save_profile(&context.device, &mut locks).await?;

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
pub async fn update_image(context: Context, image: String) {
	if context.device.starts_with("sd-") {
		if let Err(error) = crate::devices::elgato::update_image(&context, &image).await {
			log::warn!("Failed to update device image: {}", error);
		}
	}
}

#[derive(Clone, serde::Serialize)]
struct UpdateStateEvent {
	context: Context,
	contents: Vec<ActionInstance>,
}

pub async fn update_state(app: &tauri::AppHandle, context: Context, locks: &mut Locks<'_>) -> Result<(), anyhow::Error> {
	let window = app.get_window("main").unwrap();
	window.emit(
		"update_state",
		UpdateStateEvent {
			contents: get_slot(&context, locks).await?.clone(),
			context,
		},
	)?;
	Ok(())
}
