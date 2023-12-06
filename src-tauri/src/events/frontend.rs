use crate::store::profiles::{PROFILE_STORES, DEVICE_STORES, get_device_profiles};
use crate::devices::DEVICES;
use crate::shared::{Action, ActionContext, ActionInstance, CATEGORIES};

use std::collections::HashMap;

#[derive(serde::Serialize, serde::Deserialize)]
struct Error {
	pub description: String
}

fn serialise_mutex_hashmap<T>(map: &std::sync::Mutex<HashMap<String, T>>) -> String where T: serde::Serialize {
	// Here, we "duplicate" the HashMap so it isn't captured in a MutexGuard, allowing it to be serialised
	let mut hash_map: HashMap<String, &T> = HashMap::new();
	let locked = map.lock().unwrap();

	for key in locked.keys() {
		hash_map.insert(key.to_owned(), locked.get(key).unwrap());
	}
	serde_json::to_string(&hash_map).unwrap()
}

// Strings are returned from many of these commands as their return values often reference static Mutexes.

#[tauri::command]
pub fn get_devices() -> String {
	serialise_mutex_hashmap(&*DEVICES)
}

#[tauri::command]
pub fn get_categories() -> String {
	serialise_mutex_hashmap(&*CATEGORIES)
}

#[tauri::command]
pub fn get_profiles(app: tauri::AppHandle, device: &str) -> String {
	match get_device_profiles(device, &app) {
		Ok(profiles) => serde_json::to_string(&profiles).unwrap(),
		Err(error) => serde_json::to_string(&Error { description: error.to_string() }).unwrap()
	}
}

#[tauri::command]
pub fn get_selected_profile(app: tauri::AppHandle, device: &str) -> String {
	match DEVICE_STORES.lock().unwrap().get_device_store(device, &app) {
		Ok(store) => {
			match PROFILE_STORES.lock().unwrap().get_profile_store(
				DEVICES.lock().unwrap().get(device).unwrap(),
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
pub fn set_selected_profile(app: tauri::AppHandle, device: &str, profile: &str) {
	let mut device_stores = DEVICE_STORES.lock().unwrap();
	let store = device_stores.get_device_store(device, &app).unwrap();
	store.value.selected_profile = profile.to_owned();
	let _ = store.save();
}

#[tauri::command]
pub fn create_instance(app: tauri::AppHandle, action: Action, context: ActionContext) -> String {
	let instance = ActionInstance {
		action: action.clone(),
		context: context.clone(),
		states: action.states.clone(),
		current_state: 0,
		settings: serde_json::Value::Null
	};

	let mut profile_stores = PROFILE_STORES.lock().unwrap();
	let store = match profile_stores.get_profile_store(
		DEVICES.lock().unwrap().get(&context.device).unwrap(),
		&context.profile,
		&app
	) {
		Ok(store) => store,
		Err(error) => return serde_json::to_string(&Error { description: error.to_string() }).unwrap()
	};

	store.value.keys[context.position as usize] = Some(instance);
	if let Err(error) = store.save() {
		return serde_json::to_string(&Error { description: error.to_string() }).unwrap();
	}

	serde_json::to_string(&store.value.keys[context.position as usize]).unwrap()
}
