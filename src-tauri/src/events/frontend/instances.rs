use super::Error;

use crate::shared::{config_dir, Action, ActionContext, ActionInstance, Context};
use crate::store::profiles::{acquire_locks_mut, get_instance_mut, get_slot_mut, save_profile, LocksMut};

use tauri::{command, AppHandle, Emitter, Manager};
use tokio::fs::remove_dir_all;

#[command]
pub async fn create_instance(app: AppHandle, action: Action, context: Context) -> Result<Option<ActionInstance>, Error> {
	if !action.controllers.contains(&context.controller) {
		return Ok(None);
	}

	let mut locks = acquire_locks_mut().await;
	let slot = get_slot_mut(&context, &mut locks).await?;

	if let Some(parent) = slot {
		let Some(ref mut children) = parent.children else { return Ok(None) };
		let index = match children.last() {
			None => 1,
			Some(instance) => instance.context.index + 1,
		};

		let instance = ActionInstance {
			action: action.clone(),
			context: ActionContext::from_context(context.clone(), index),
			states: action.states.clone(),
			current_state: 0,
			settings: serde_json::Value::Object(serde_json::Map::new()),
			children: None,
		};
		children.push(instance.clone());

		if parent.action.uuid == "opendeck.toggleaction" && parent.states.len() < children.len() {
			parent.states.push(crate::shared::ActionState {
				image: "opendeck/toggle-action.png".to_owned(),
				..Default::default()
			});
			let _ = update_state(&app, parent.context.clone(), &mut locks).await;
		}

		save_profile(&context.device, &mut locks).await?;
		let _ = crate::events::outbound::will_appear::will_appear(&instance, true).await;

		Ok(Some(instance))
	} else {
		let instance = ActionInstance {
			action: action.clone(),
			context: ActionContext::from_context(context.clone(), 0),
			states: action.states.clone(),
			current_state: 0,
			settings: serde_json::Value::Object(serde_json::Map::new()),
			children: if matches!(action.uuid.as_str(), "opendeck.multiaction" | "opendeck.toggleaction") {
				Some(vec![])
			} else {
				None
			},
		};

		*slot = Some(instance.clone());
		let slot = slot.clone();

		save_profile(&context.device, &mut locks).await?;
		let _ = crate::events::outbound::will_appear::will_appear(&instance, false).await;

		Ok(slot)
	}
}

fn instance_images_dir(instance: &ActionInstance) -> std::path::PathBuf {
	config_dir()
		.join("images")
		.join(&instance.context.device)
		.join(&instance.context.profile)
		.join(format!("{}.{}.{}", instance.context.controller, instance.context.position, instance.context.index))
}

#[command]
pub async fn move_instance(source: Context, destination: Context, retain: bool) -> Result<Option<ActionInstance>, Error> {
	if source.controller != destination.controller {
		return Ok(None);
	}

	let mut locks = acquire_locks_mut().await;
	let src = get_slot_mut(&source, &mut locks).await?;

	let Some(mut new) = src.clone() else {
		return Ok(None);
	};
	new.context = ActionContext::from_context(destination.clone(), 0);
	if let Some(ref mut children) = new.children {
		for (index, instance) in children.iter_mut().enumerate() {
			instance.context = ActionContext::from_context(destination.clone(), index as u16 + 1);
		}
	}

	if let Ok(files) = instance_images_dir(&src.clone().unwrap()).read_dir() {
		let new_dir = instance_images_dir(&new);
		for file in files.flatten() {
			let _ = tokio::fs::copy(file.path(), new_dir.join(file.file_name())).await;
		}
	}

	let dst = get_slot_mut(&destination, &mut locks).await?;
	if dst.is_some() {
		return Ok(None);
	}
	*dst = Some(new.clone());

	if !retain {
		let src = get_slot_mut(&source, &mut locks).await?;
		if let Some(old) = src {
			let _ = crate::events::outbound::will_appear::will_disappear(old, false, true).await;
			let _ = remove_dir_all(instance_images_dir(old)).await;
		}
		*src = None;
	}

	let _ = crate::events::outbound::will_appear::will_appear(&new, false).await;

	save_profile(&destination.device, &mut locks).await?;

	Ok(Some(new))
}

#[command]
pub async fn remove_instance(context: ActionContext) -> Result<(), Error> {
	let mut locks = acquire_locks_mut().await;
	let slot = get_slot_mut(&(&context).into(), &mut locks).await?;
	let Some(instance) = slot else {
		return Ok(());
	};

	if instance.context == context {
		let _ = crate::events::outbound::will_appear::will_disappear(instance, false, true).await;
		if let Some(children) = &instance.children {
			for child in children {
				let _ = crate::events::outbound::will_appear::will_disappear(child, false, true).await;
				let _ = remove_dir_all(instance_images_dir(child)).await;
			}
		}
		let _ = remove_dir_all(instance_images_dir(instance)).await;
		*slot = None;
	} else {
		let children = instance.children.as_mut().unwrap();
		for (index, instance) in children.iter().enumerate() {
			if instance.context == context {
				let _ = crate::events::outbound::will_appear::will_disappear(instance, true, true).await;
				let _ = remove_dir_all(instance_images_dir(instance)).await;
				children.remove(index);
				break;
			}
		}
		if instance.action.uuid == "opendeck.toggleaction" {
			if instance.current_state as usize >= children.len() {
				instance.current_state = if children.is_empty() { 0 } else { children.len() as u16 - 1 };
			}
			if !children.is_empty() {
				instance.states.pop();
				let _ = update_state(crate::APP_HANDLE.get().unwrap(), instance.context.clone(), &mut locks).await;
			}
		}
	}

	save_profile(&context.device, &mut locks).await?;

	Ok(())
}

#[derive(Clone, serde::Serialize)]
struct UpdateStateEvent {
	context: ActionContext,
	contents: Option<ActionInstance>,
}

pub async fn update_state(app: &AppHandle, context: ActionContext, locks: &mut LocksMut<'_>) -> Result<(), anyhow::Error> {
	let window = app.get_webview_window("main").unwrap();
	window.emit(
		"update_state",
		UpdateStateEvent {
			contents: get_instance_mut(&context, locks).await?.cloned(),
			context,
		},
	)?;
	Ok(())
}

#[command]
pub async fn set_state(instance: ActionInstance, state: u16) -> Result<(), Error> {
	let mut locks = acquire_locks_mut().await;
	let reference = get_instance_mut(&instance.context, &mut locks).await?.unwrap();
	*reference = instance.clone();
	save_profile(&instance.context.device, &mut locks).await?;
	crate::events::outbound::states::title_parameters_did_change(&instance, state).await?;
	Ok(())
}

#[command]
pub async fn update_image(context: Context, image: String) {
	if Some(&context.profile) != crate::store::profiles::DEVICE_STORES.write().await.get_selected_profile(&context.device).ok().as_ref() {
		return;
	}

	if let Err(error) = crate::events::outbound::devices::update_image(context, Some(image)).await {
		log::warn!("Failed to update device image: {}", error);
	}
}

#[derive(Clone, serde::Serialize)]
struct KeyMovedEvent {
	context: Context,
	pressed: bool,
}

pub async fn key_moved(app: &AppHandle, context: Context, pressed: bool) -> Result<(), anyhow::Error> {
	let window = app.get_webview_window("main").unwrap();
	window.emit("key_moved", KeyMovedEvent { context, pressed })?;
	Ok(())
}
