use super::Error;

use crate::devices::DEVICES;
use crate::store::profiles::{get_device_profiles, DEVICE_STORES, PROFILE_STORES};

use tauri::{command, AppHandle};

#[command]
pub fn get_profiles(app: AppHandle, device: &str) -> Result<Vec<String>, Error> {
	Ok(get_device_profiles(device, &app)?)
}

#[command]
pub async fn get_selected_profile(device: String) -> Result<crate::shared::Profile, Error> {
	let mut device_stores = DEVICE_STORES.write().await;
	let profile_stores = PROFILE_STORES.read().await;

	let selected_profile = device_stores.get_selected_profile(&device)?;
	let profile = profile_stores.get_profile_store(DEVICES.read().await.get(&device).unwrap(), &selected_profile)?;

	Ok(profile.value.clone())
}

#[allow(clippy::flat_map_identity)]
#[command]
pub async fn set_selected_profile(app: AppHandle, device: String, id: String, profile: Option<crate::shared::Profile>) -> Result<(), Error> {
	let mut device_stores = DEVICE_STORES.write().await;
	let devices = DEVICES.read().await;
	let mut profile_stores = PROFILE_STORES.write().await;
	let selected_profile = device_stores.get_selected_profile(&device)?;

	if selected_profile != id {
		let old_profile = &profile_stores.get_profile_store(devices.get(&device).unwrap(), &selected_profile)?.value;
		for instance in old_profile.keys.iter().flatten().chain(&mut old_profile.sliders.iter().flatten()) {
			if !matches!(instance.action.uuid.as_str(), "com.amansprojects.opendeck.multiaction" | "com.amansprojects.opendeck.toggleaction") {
				let _ = crate::events::outbound::will_appear::will_disappear(instance, false).await;
			} else {
				for child in instance.children.as_ref().unwrap() {
					let _ = crate::events::outbound::will_appear::will_disappear(child, true).await;
				}
			}
		}
	}

	// We must use the mutable version of get_profile_store in order to create the store if it does not exist.
	let store = profile_stores.get_profile_store_mut(devices.get(&device).unwrap(), &id, &app).await?;
	let new_profile = &mut store.value;
	if let Some(profile) = profile {
		*new_profile = profile;
	}
	for instance in new_profile.keys.iter().flatten().chain(&mut new_profile.sliders.iter().flatten()) {
		if !matches!(instance.action.uuid.as_str(), "com.amansprojects.opendeck.multiaction" | "com.amansprojects.opendeck.toggleaction") {
			let _ = crate::events::outbound::will_appear::will_appear(instance, false).await;
		} else {
			for child in instance.children.as_ref().unwrap() {
				let _ = crate::events::outbound::will_appear::will_appear(child, true).await;
			}
		}
	}
	store.save()?;

	device_stores.set_selected_profile(&device, id, &app)?;

	Ok(())
}

#[command]
pub async fn delete_profile(app: AppHandle, device: String, profile: String) {
	let mut profile_stores = PROFILE_STORES.write().await;
	profile_stores.remove_profile(&device, &profile, &app);
}
