use crate::events::outbound::settings as outbound;
use crate::store::profiles::{get_instance, lock_mutexes, save_profile};

pub async fn set_settings(event: super::ContextAndPayloadEvent<serde_json::Value>, from_property_inspector: bool) -> Result<(), anyhow::Error> {
	let mut locks = lock_mutexes().await;

	if let Some(instance) = get_instance(&event.context, &mut locks).await? {
		instance.settings = event.payload;
		outbound::did_receive_settings(instance, !from_property_inspector).await?;
		save_profile(&event.context.device, &mut locks).await?;
	}

	Ok(())
}

pub async fn get_settings(event: super::ContextEvent, from_property_inspector: bool) -> Result<(), anyhow::Error> {
	let mut locks = lock_mutexes().await;

	if let Some(instance) = get_instance(&event.context, &mut locks).await? {
		outbound::did_receive_settings(instance, from_property_inspector).await?;
	}

	Ok(())
}

pub async fn set_global_settings(event: super::ContextAndPayloadEvent<serde_json::Value, String>, from_property_inspector: bool) -> Result<(), anyhow::Error> {
	{
		let app = crate::APP_HANDLE.get().unwrap();

		let settings_dir = app.path_resolver().app_config_dir().unwrap().join("settings/");
		tokio::fs::create_dir_all(&settings_dir).await?;

		let path = settings_dir.join(event.context.clone() + ".json");
		tokio::fs::write(path, event.payload.to_string()).await?;
	}

	outbound::did_receive_global_settings(&event.context, !from_property_inspector).await?;

	Ok(())
}

pub async fn get_global_settings(event: super::ContextEvent<String>, from_property_inspector: bool) -> Result<(), anyhow::Error> {
	outbound::did_receive_global_settings(&event.context, from_property_inspector).await
}
