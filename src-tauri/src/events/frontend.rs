use crate::store::profiles::{PROFILE_STORES, DEVICE_STORES, get_device_profiles};
use crate::devices::DEVICES;
use crate::shared::{Action, ActionContext, ActionInstance, CATEGORIES};

use std::collections::HashMap;

use tauri::Manager;

#[derive(serde::Serialize, serde::Deserialize)]
struct Error {
	pub description: String
}

async fn serialise_mutex_hashmap<T>(map: &tokio::sync::Mutex<HashMap<String, T>>) -> String where T: serde::Serialize {
	// Here, we "duplicate" the HashMap so it isn't captured in a MutexGuard, allowing it to be serialised
	let mut hash_map: HashMap<String, &T> = HashMap::new();
	let locked = map.lock().await;

	for key in locked.keys() {
		hash_map.insert(key.to_owned(), locked.get(key).unwrap());
	}
	serde_json::to_string(&hash_map).unwrap()
}

// Strings are returned from many of these commands as their return values often reference static Mutexes.

#[tauri::command]
pub async fn get_devices() -> String {
	serialise_mutex_hashmap(&*DEVICES).await
}

#[tauri::command]
pub async fn get_categories() -> String {
	serialise_mutex_hashmap(&*CATEGORIES).await
}

#[tauri::command]
pub fn get_profiles(app: tauri::AppHandle, device: &str) -> String {
	match get_device_profiles(device, &app) {
		Ok(profiles) => serde_json::to_string(&profiles).unwrap(),
		Err(error) => serde_json::to_string(&Error { description: error.to_string() }).unwrap()
	}
}

#[tauri::command]
pub async fn get_selected_profile(app: tauri::AppHandle, device: String) -> String {
	match DEVICE_STORES.lock().await.get_device_store(&device, &app) {
		Ok(store) => {
			match PROFILE_STORES.lock().await.get_profile_store(
				DEVICES.lock().await.get(&device).unwrap(),
				&store.value.selected_profile,
				&app
			) {
				Ok(store) => serde_json::to_string(&store.value).unwrap(),
				Err(error) => serde_json::to_string(&Error { description: error.to_string() }).unwrap()
			}
		},
		Err(error) => serde_json::to_string(&Error { description: error.to_string() }).unwrap()
	}
}

#[tauri::command]
pub async fn set_selected_profile(app: tauri::AppHandle, device: String, id: String) -> String {
	let mut device_stores = DEVICE_STORES.lock().await;
	let devices = DEVICES.lock().await;
	let mut profile_stores = PROFILE_STORES.lock().await;
	let store = device_stores.get_device_store(&device, &app).unwrap();

	if store.value.selected_profile != id {
		let old_profile = match profile_stores.get_profile_store(devices.get(&device).unwrap(), &store.value.selected_profile, &app) {
			Ok(store) => &store.value,
			Err(error) => return serde_json::to_string(&Error { description: error.to_string() }).unwrap()
		};
		for instance in (&old_profile.keys).into_iter().chain(&old_profile.sliders) {
			if let Some(instance) = instance {
				let _ = crate::events::outbound::will_appear::will_disappear(instance).await;
			}
		}
	}

	let new_profile = match profile_stores.get_profile_store(devices.get(&device).unwrap(), &id, &app) {
		Ok(store) => &store.value,
		Err(error) => return serde_json::to_string(&Error { description: error.to_string() }).unwrap()
	};
	for instance in (&new_profile.keys).into_iter().chain(&new_profile.sliders) {
		if let Some(instance) = instance {
			let _ = crate::events::outbound::will_appear::will_appear(instance).await;
		}
	}

	store.value.selected_profile = id.to_owned();
	if let Err(error) = store.save() {
		return serde_json::to_string(&Error { description: error.to_string() }).unwrap()
	}

	serde_json::to_string(&store.value).unwrap()
}

#[tauri::command]
pub async fn delete_profile(app: tauri::AppHandle, device: String, profile: String) {
	let mut profile_stores = PROFILE_STORES.lock().await;
	profile_stores.remove_profile(&device, &profile, &app);
}

#[tauri::command]
pub async fn create_instance(app: tauri::AppHandle, action: Action, context: ActionContext) -> String {
	let instance = ActionInstance {
		action: action.clone(),
		context: context.clone(),
		states: action.states.clone(),
		current_state: 0,
		settings: serde_json::Value::Object(serde_json::Map::new())
	};

	let mut profile_stores = PROFILE_STORES.lock().await;
	let store = match profile_stores.get_profile_store(
		DEVICES.lock().await.get(&context.device).unwrap(),
		&context.profile,
		&app
	) {
		Ok(store) => store,
		Err(error) => return serde_json::to_string(&Error { description: error.to_string() }).unwrap()
	};

	let instance_ref: &Option<ActionInstance>;
	if context.controller == "Encoder" {
		if let Some(instance) = &store.value.sliders[context.position as usize] {
			let _ = crate::events::outbound::will_appear::will_disappear(instance).await;
		}
		store.value.sliders[context.position as usize] = Some(instance);
		instance_ref = &store.value.sliders[context.position as usize];
	} else {
		if let Some(instance) = &store.value.keys[context.position as usize] {
			let _ = crate::events::outbound::will_appear::will_disappear(instance).await;
		}
		store.value.keys[context.position as usize] = Some(instance);
		instance_ref = &store.value.keys[context.position as usize];
	}

	let _ = crate::events::outbound::will_appear::will_appear(instance_ref.as_ref().unwrap()).await;

	if let Err(error) = store.save() {
		return serde_json::to_string(&Error { description: error.to_string() }).unwrap();
	}

	serde_json::to_string(instance_ref).unwrap()
}

#[tauri::command]
pub async fn clear_slot(app: tauri::AppHandle, context: ActionContext) -> String {
	let mut profile_stores = PROFILE_STORES.lock().await;
	let store = match profile_stores.get_profile_store(
		DEVICES.lock().await.get(&context.device).unwrap(),
		&context.profile,
		&app
	) {
		Ok(store) => store,
		Err(error) => return serde_json::to_string(&Error { description: error.to_string() }).unwrap()
	};

	let instance_ref: &Option<ActionInstance>;
	if context.controller == "Encoder" {
		if let Some(instance) = &store.value.sliders[context.position as usize] {
			let _ = crate::events::outbound::will_appear::will_disappear(instance).await;
		}
		store.value.sliders[context.position as usize] = None;
		instance_ref = &store.value.sliders[context.position as usize];
	} else {
		if let Some(instance) = &store.value.keys[context.position as usize] {
			let _ = crate::events::outbound::will_appear::will_disappear(instance).await;
		}
		store.value.keys[context.position as usize] = None;
		instance_ref = &store.value.keys[context.position as usize];
	}

	if let Err(error) = store.save() {
		return serde_json::to_string(&Error { description: error.to_string() }).unwrap();
	}

	serde_json::to_string(instance_ref).unwrap()
}

#[tauri::command]
pub async fn make_info(app: tauri::AppHandle, plugin: String) -> String {
	let mut path = app.path_resolver().app_config_dir().unwrap();
	path.push("plugins");
	path.push(&plugin);
	path.push("manifest.json");

	let manifest = match std::fs::read(&path) {
		Ok(data) => data,
		Err(error) => return serde_json::to_string(&Error { description: error.to_string() }).unwrap()
	};

	let manifest: crate::plugins::manifest::PluginManifest = match serde_json::from_slice(&manifest) {
		Ok(manifest) => manifest,
		Err(error) => return serde_json::to_string(&Error { description: error.to_string() }).unwrap()
	};

	serde_json::to_string(&crate::plugins::info_param::make_info(plugin, manifest.version).await).unwrap()
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

pub async fn update_state(app: &tauri::AppHandle, instance: &ActionInstance) -> Result<(), anyhow::Error> {
	let window = app.get_window("main").unwrap();
	window.emit("update_state", serde_json::to_string(instance).unwrap())?;
	Ok(())
}
