use super::ContextAndPayloadEvent;

use crate::events::frontend::update_state;
use crate::store::profiles::{get_instance, lock_mutexes, save_profile};

use serde::Deserialize;

#[derive(Deserialize)]
pub struct SetTitlePayload {
	title: Option<String>,
	state: Option<u16>,
}

#[derive(Deserialize)]
pub struct SetImagePayload {
	image: Option<String>,
	state: Option<u16>,
}

#[derive(Deserialize)]
pub struct SetStatePayload {
	state: u16,
}

pub async fn set_title(event: ContextAndPayloadEvent<SetTitlePayload>) -> Result<(), anyhow::Error> {
	let mut locks = lock_mutexes().await;

	if let Some(instance) = get_instance(&event.context.device, &event.context.controller, event.context.position, event.context.index, &mut locks).await? {
		if let Some(state) = event.payload.state {
			instance.states[state as usize].text = event.payload.title.unwrap_or(instance.action.states[state as usize].text.clone());
		} else {
			for (index, state) in instance.states.iter_mut().enumerate() {
				state.text = event.payload.title.clone().unwrap_or(instance.action.states[index].text.clone());
			}
		}
		update_state(crate::APP_HANDLE.get().unwrap(), instance).await?;
	}
	save_profile(&event.context.device, &mut locks).await?;

	Ok(())
}

pub async fn set_image(event: ContextAndPayloadEvent<SetImagePayload>) -> Result<(), anyhow::Error> {
	let mut locks = lock_mutexes().await;

	if let Some(instance) = get_instance(&event.context.device, &event.context.controller, event.context.position, event.context.index, &mut locks).await? {
		if let Some(state) = event.payload.state {
			instance.states[state as usize].image = event.payload.image.unwrap_or(instance.action.states[state as usize].image.clone());
		} else {
			for (index, state) in instance.states.iter_mut().enumerate() {
				state.image = event.payload.image.clone().unwrap_or(instance.action.states[index].image.clone());
			}
		}
		update_state(crate::APP_HANDLE.get().unwrap(), instance).await?;
	}
	save_profile(&event.context.device, &mut locks).await?;

	Ok(())
}

pub async fn set_state(event: ContextAndPayloadEvent<SetStatePayload>) -> Result<(), anyhow::Error> {
	let mut locks = lock_mutexes().await;

	if let Some(instance) = get_instance(&event.context.device, &event.context.controller, event.context.position, event.context.index, &mut locks).await? {
		instance.current_state = event.payload.state;
		update_state(crate::APP_HANDLE.get().unwrap(), instance).await?;
	}
	save_profile(&event.context.device, &mut locks).await?;

	Ok(())
}
