use super::Error;

use crate::store::profiles::{acquire_locks_mut, get_device_profiles, PROFILE_STORES};

use tauri::command;

#[command]
pub fn get_profiles(device: &str) -> Result<Vec<String>, Error> {
	Ok(get_device_profiles(device)?)
}

#[command]
pub async fn get_selected_profile(device: String) -> Result<crate::shared::Profile, Error> {
	let mut locks = acquire_locks_mut().await;
	if !locks.devices.contains_key(&device) {
		return Err(Error::new(format!("device {device} not found")));
	}

	let selected_profile = locks.device_stores.get_selected_profile(&device)?;
	let profile = locks.profile_stores.get_profile_store(locks.devices.get(&device).unwrap(), &selected_profile)?;

	Ok(profile.value.clone())
}

#[allow(clippy::flat_map_identity)]
#[command]
pub async fn set_selected_profile(device: String, id: String) -> Result<(), Error> {
	let mut locks = acquire_locks_mut().await;
	if !locks.devices.contains_key(&device) {
		return Err(Error::new(format!("device {device} not found")));
	}

	let selected_profile = locks.device_stores.get_selected_profile(&device)?;

	if selected_profile != id {
		let old_profile = &locks.profile_stores.get_profile_store(locks.devices.get(&device).unwrap(), &selected_profile)?.value;
		for instance in old_profile.keys.iter().flatten().chain(&mut old_profile.sliders.iter().flatten()) {
			if !matches!(instance.action.uuid.as_str(), "opendeck.multiaction" | "opendeck.toggleaction") {
				let _ = crate::events::outbound::will_appear::will_disappear(instance, false, false).await;
			} else {
				for child in instance.children.as_ref().unwrap() {
					let _ = crate::events::outbound::will_appear::will_disappear(child, true, false).await;
				}
			}
		}
		let _ = crate::events::outbound::devices::clear_screen(device.clone()).await;
	}

	// We must use the mutable version of get_profile_store in order to create the store if it does not exist.
	let store = locks.profile_stores.get_profile_store_mut(locks.devices.get(&device).unwrap(), &id).await?;
	let new_profile = &store.value;
	for instance in new_profile.keys.iter().flatten().chain(&mut new_profile.sliders.iter().flatten()) {
		if !matches!(instance.action.uuid.as_str(), "opendeck.multiaction" | "opendeck.toggleaction") {
			let _ = crate::events::outbound::will_appear::will_appear(instance, false).await;
		} else {
			for child in instance.children.as_ref().unwrap() {
				let _ = crate::events::outbound::will_appear::will_appear(child, true).await;
			}
		}
	}
	store.save()?;

	locks.device_stores.set_selected_profile(&device, id)?;

	Ok(())
}

#[command]
pub async fn delete_profile(device: String, profile: String) {
	let mut profile_stores = PROFILE_STORES.write().await;
	profile_stores.remove_profile(&device, &profile);
}
