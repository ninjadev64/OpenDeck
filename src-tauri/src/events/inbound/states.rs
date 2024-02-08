use super::ContextAndPayloadEvent;

use crate::events::frontend::update_state;

use serde::Deserialize;

#[derive(Deserialize)]
pub struct SetTitlePayload {
	title: Option<String>,
	state: Option<u16>
}

#[derive(Deserialize)]
pub struct SetImagePayload {
	image: Option<String>,
	state: Option<u16>
}

#[derive(Deserialize)]
pub struct SetStatePayload {
	state: u16
}

pub async fn set_title(event: ContextAndPayloadEvent<SetTitlePayload>) -> Result<(), anyhow::Error> {
	let (
		app,
		mut device_stores,
		devices,
		mut profile_stores
	) = crate::store::profiles::lock_mutexes().await;

	let selected_profile = &device_stores.get_device_store(&event.context.device, app.as_ref().unwrap())?.value.selected_profile;
	let device = devices.get(&event.context.device).unwrap();
	let store = profile_stores.get_profile_store(device, selected_profile, app.as_ref().unwrap())?;
	let profile = &mut store.value;

	let instance = match event.context.controller.as_str() {
		"Encoder" => profile.sliders[event.context.position as usize].as_mut(),
		_ => profile.keys[event.context.position as usize].as_mut()
	};

	if let Some(instance) = instance {
		if let Some(state) = event.payload.state {
			instance.states[state as usize].text = event.payload.title.unwrap_or(
				instance.action.states[state as usize].text.clone()
			);
		} else {
			for (index, state) in instance.states.iter_mut().enumerate() {
				state.text = event.payload.title.clone().unwrap_or(instance.action.states[index].text.clone());
			}
		}
		update_state(app.as_ref().unwrap(), instance).await?;
	}

	Ok(())
}

pub async fn set_image(event: ContextAndPayloadEvent<SetImagePayload>) -> Result<(), anyhow::Error> {
	let (
		app,
		mut device_stores,
		devices,
		mut profile_stores
	) = crate::store::profiles::lock_mutexes().await;

	let selected_profile = &device_stores.get_device_store(&event.context.device, app.as_ref().unwrap())?.value.selected_profile;
	let device = devices.get(&event.context.device).unwrap();
	let store = profile_stores.get_profile_store(device, selected_profile, app.as_ref().unwrap())?;
	let profile = &mut store.value;

	let instance = match event.context.controller.as_str() {
		"Encoder" => profile.sliders[event.context.position as usize].as_mut(),
		_ => profile.keys[event.context.position as usize].as_mut()
	};

	if let Some(instance) = instance {
		if let Some(state) = event.payload.state {
			instance.states[state as usize].image = event.payload.image.unwrap_or(
				instance.action.states[state as usize].image.clone()
			);
		} else {
			for (index, state) in instance.states.iter_mut().enumerate() {
				state.image = event.payload.image.clone().unwrap_or(instance.action.states[index].image.clone());
			}
		}
		update_state(app.as_ref().unwrap(), instance).await?;
	}

	Ok(())
}

pub async fn set_state(event: ContextAndPayloadEvent<SetStatePayload>) -> Result<(), anyhow::Error> {
	let (
		app,
		mut device_stores,
		devices,
		mut profile_stores
	) = crate::store::profiles::lock_mutexes().await;

	let selected_profile = &device_stores.get_device_store(&event.context.device, app.as_ref().unwrap())?.value.selected_profile;
	let device = devices.get(&event.context.device).unwrap();
	let store = profile_stores.get_profile_store(device, selected_profile, app.as_ref().unwrap())?;
	let profile = &mut store.value;

	let instance = match event.context.controller.as_str() {
		"Encoder" => profile.sliders[event.context.position as usize].as_mut(),
		_ => profile.keys[event.context.position as usize].as_mut()
	};

	if let Some(instance) = instance {
		instance.current_state = event.payload.state;
		update_state(app.as_ref().unwrap(), instance).await?;
	}

	Ok(())
}
