use crate::built_info;
use crate::devices::DEVICES;
use crate::shared::{Action, ActionContext, ActionInstance, Context, CATEGORIES};
use crate::store::profiles::{acquire_locks, acquire_locks_mut, get_device_profiles, get_instance_mut, get_slot_mut, save_profile, LocksMut, DEVICE_STORES, PROFILE_STORES};

use std::collections::HashMap;

use tauri::{command, AppHandle, Manager};
#[cfg(not(debug_assertions))]
use tauri_plugin_autostart::ManagerExt;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Error {
	pub description: String,
}

impl std::fmt::Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.description)
	}
}
impl std::error::Error for Error {}

impl Error {
	fn new(description: String) -> Self {
		log::error!("{}", description);
		Self { description }
	}
}

impl From<anyhow::Error> for Error {
	fn from(error: anyhow::Error) -> Self {
		Self::new(error.to_string())
	}
}

#[command]
pub async fn get_devices() -> HashMap<std::string::String, crate::devices::DeviceInfo> {
	DEVICES.read().await.clone()
}

pub async fn update_devices() {
	let app = crate::APP_HANDLE.get().unwrap();
	let _ = app.get_window("main").unwrap().emit("devices", DEVICES.read().await.clone());
}

#[command]
pub async fn rescan_devices() {
	let devices = DEVICES.read().await;
	if devices.len() > 0 {
		return;
	}
	crate::devices::initialise_devices();
}

#[command]
pub async fn get_categories() -> HashMap<std::string::String, Vec<Action>> {
	CATEGORIES.read().await.clone()
}

#[command]
pub fn get_profiles(app: AppHandle, device: &str) -> Result<Vec<String>, Error> {
	Ok(get_device_profiles(device, &app)?)
}

#[command]
pub async fn get_selected_profile(device: String) -> Result<crate::shared::Profile, Error> {
	let mut device_stores = DEVICE_STORES.write().await;
	let profile_stores = PROFILE_STORES.read().await;

	let selected_profile = device_stores.get_selected_profile(&device)?;
	let profile = profile_stores.get_profile_store(DEVICES.read().await.get(&device).unwrap(), selected_profile)?;

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
		let old_profile = &profile_stores.get_profile_store(devices.get(&device).unwrap(), selected_profile)?.value;
		for slot in old_profile.keys.iter().chain(&old_profile.sliders) {
			for instance in slot {
				let _ = crate::events::outbound::will_appear::will_disappear(instance, slot.len() > 1).await;
			}
		}
	}

	// We must use the mutable version of get_profile_store in order to create the store if it does not exist.
	let store = profile_stores.get_profile_store_mut(devices.get(&device).unwrap(), &id, &app)?;
	let new_profile = &mut store.value;
	if let Some(profile) = profile {
		*new_profile = profile;
	}
	for slot in new_profile.keys.iter().chain(&new_profile.sliders) {
		for instance in slot {
			let _ = crate::events::outbound::will_appear::will_appear(instance, slot.len() > 1).await;
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

#[command]
pub async fn create_instance(action: Action, context: Context) -> Result<Option<Vec<ActionInstance>>, Error> {
	if !action.controllers.contains(&context.controller) {
		return Ok(None);
	}

	let mut locks = acquire_locks_mut().await;
	let slot = get_slot_mut(&context, &mut locks).await?;
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

#[command]
pub async fn move_slot(source: Context, destination: Context, retain: bool) -> Result<Option<Vec<ActionInstance>>, Error> {
	if source.controller != destination.controller {
		return Ok(None);
	}

	let mut locks = acquire_locks_mut().await;
	let src = get_slot_mut(&source, &mut locks).await?;
	let multi_action = src.len() > 1;

	let mut vec: Vec<ActionInstance> = vec![];

	for (index, instance) in src.iter_mut().enumerate() {
		let mut new = instance.clone();
		new.context = ActionContext::from_context(destination.clone(), index as u16);
		vec.push(new);
	}

	let dst = get_slot_mut(&destination, &mut locks).await?;
	if !dst.is_empty() {
		return Ok(None);
	}
	dst.clone_from(&vec);

	if !retain {
		let src = get_slot_mut(&source, &mut locks).await?;
		for old in &*src {
			let _ = crate::events::outbound::will_appear::will_disappear(old, multi_action).await;
		}
		*src = vec![];
	}

	for new in &vec {
		let _ = crate::events::outbound::will_appear::will_appear(new, multi_action).await;
	}

	save_profile(&destination.device, &mut locks).await?;

	Ok(Some(vec))
}

#[command]
pub async fn clear_slot(context: Context) -> Result<(), Error> {
	let mut locks = acquire_locks_mut().await;
	let slot = get_slot_mut(&context, &mut locks).await?;

	for instance in &*slot {
		let _ = crate::events::outbound::will_appear::will_disappear(instance, slot.len() > 1).await;
	}

	*slot = vec![];
	save_profile(&context.device, &mut locks).await?;

	Ok(())
}

#[command]
pub async fn remove_instance(context: ActionContext) -> Result<(), Error> {
	let mut locks = acquire_locks_mut().await;
	let slot = get_slot_mut(&(&context).into(), &mut locks).await?;

	for (index, instance) in slot.iter().enumerate() {
		if instance.context == context {
			let _ = crate::events::outbound::will_appear::will_disappear(instance, slot.len() > 1).await;
			slot.remove(index);
			break;
		}
	}

	save_profile(&context.device, &mut locks).await?;

	Ok(())
}

#[command]
pub async fn make_info(app: AppHandle, plugin: String) -> Result<crate::plugins::info_param::Info, Error> {
	let mut path = app.path_resolver().app_config_dir().unwrap();
	path.push("plugins");
	path.push(&plugin);
	path.push("manifest.json");

	let manifest = match tokio::fs::read(&path).await {
		Ok(data) => data,
		Err(error) => return Err(anyhow::Error::from(error).into()),
	};

	let manifest: crate::plugins::manifest::PluginManifest = match serde_json::from_slice(&manifest) {
		Ok(manifest) => manifest,
		Err(error) => return Err(anyhow::Error::from(error).into()),
	};

	Ok(crate::plugins::info_param::make_info(plugin, manifest.version, false).await)
}

#[command]
pub async fn switch_property_inspector(old: Option<ActionContext>, new: Option<ActionContext>) {
	if let Some(context) = old {
		let _ = crate::events::outbound::property_inspector::property_inspector_did_appear(context, "propertyInspectorDidDisappear").await;
	}
	if let Some(context) = new {
		let _ = crate::events::outbound::property_inspector::property_inspector_did_appear(context, "propertyInspectorDidAppear").await;
	}
}

#[command]
pub async fn open_url(app: AppHandle, url: String) -> Result<(), Error> {
	if let Err(error) = tauri::api::shell::open(&app.shell_scope(), url, None) {
		return Err(anyhow::Error::from(error).into());
	}
	Ok(())
}

#[command]
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

pub async fn update_state(app: &AppHandle, context: Context, locks: &mut LocksMut<'_>) -> Result<(), anyhow::Error> {
	let window = app.get_window("main").unwrap();
	window.emit(
		"update_state",
		UpdateStateEvent {
			contents: get_slot_mut(&context, locks).await?.clone(),
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
pub async fn install_plugin(app: AppHandle, id: String, url: Option<String>) -> Result<(), Error> {
	let resp = match reqwest::get(url.unwrap_or(format!("https://plugins.amansprojects.com/rezipped/{id}.zip"))).await {
		Ok(resp) => resp,
		Err(error) => return Err(anyhow::Error::from(error).into()),
	};
	let bytes = match resp.bytes().await {
		Ok(bytes) => bytes,
		Err(error) => return Err(anyhow::Error::from(error).into()),
	};

	let _ = crate::plugins::deactivate_plugin(&app, &format!("{}.sdPlugin", id)).await;

	let config_dir = app.path_resolver().app_config_dir().unwrap();
	let _ = tokio::fs::create_dir_all(config_dir.join("temp")).await;

	let temp = config_dir.join("temp").join(format!("{id}.sdPlugin"));
	let actual = config_dir.join("plugins").join(format!("{id}.sdPlugin"));

	let _ = tokio::fs::rename(&actual, &temp).await;
	if let Err(error) = crate::zip_extract::extract(std::io::Cursor::new(bytes), &config_dir.join("plugins")) {
		log::error!("Failed to unzip file: {}", error.to_string());
		let _ = tokio::fs::rename(&temp, &actual).await;
		let _ = crate::plugins::initialise_plugin(&actual).await;
		return Err(anyhow::Error::from(error).into());
	}
	let _ = crate::plugins::initialise_plugin(&actual).await;
	let _ = tokio::fs::remove_dir_all(temp).await;

	Ok(())
}

#[derive(serde::Serialize)]
pub struct PluginInfo {
	id: String,
	name: String,
	author: String,
	icon: String,
	version: String,
}

#[command]
pub async fn list_plugins(app: AppHandle) -> Result<Vec<PluginInfo>, Error> {
	let mut plugins = vec![];

	let mut entries = match tokio::fs::read_dir(&app.path_resolver().app_config_dir().unwrap().join("plugins")).await {
		Ok(entries) => entries,
		Err(error) => return Err(anyhow::Error::from(error).into()),
	};

	while let Ok(Some(entry)) = entries.next_entry().await {
		let path = match entry.metadata().await.unwrap().is_symlink() {
			true => tokio::fs::read_link(entry.path()).await.unwrap(),
			false => entry.path(),
		};
		let metadata = tokio::fs::metadata(&path).await.unwrap();
		if metadata.is_dir() {
			let Ok(manifest) = tokio::fs::read(path.join("manifest.json")).await else { continue };
			let Ok(manifest): Result<crate::plugins::manifest::PluginManifest, _> = serde_json::from_slice(&manifest) else {
				continue;
			};
			plugins.push(PluginInfo {
				id: path.file_name().unwrap().to_str().unwrap().to_owned(),
				name: manifest.name,
				author: manifest.author,
				icon: crate::shared::convert_icon(path.join(manifest.icon).to_str().unwrap().to_owned()),
				version: manifest.version,
			});
		}
	}

	Ok(plugins)
}

#[command]
pub async fn remove_plugin(app: AppHandle, id: String) -> Result<(), Error> {
	let locks = acquire_locks().await;
	let all = locks.profile_stores.all_from_plugin(&id);
	drop(locks);

	for context in all {
		remove_instance(context).await?;
	}

	crate::plugins::deactivate_plugin(&app, &id).await?;
	if let Err(error) = tokio::fs::remove_dir_all(app.path_resolver().app_config_dir().unwrap().join("plugins").join(&id)).await {
		return Err(anyhow::Error::from(error).into());
	}

	let mut categories = CATEGORIES.write().await;
	for category in categories.values_mut() {
		category.retain(|v| v.plugin != id);
	}
	categories.retain(|_, v| !v.is_empty());

	Ok(())
}

#[command]
pub async fn get_settings(app: AppHandle) -> Result<crate::store::Settings, Error> {
	let store = crate::store::get_settings(app).await;
	match store {
		Ok(store) => Ok(store.value),
		Err(error) => Err(error.into()),
	}
}

#[command]
pub async fn set_settings(app: AppHandle, settings: crate::store::Settings) -> Result<(), Error> {
	#[cfg(not(debug_assertions))]
	let _ = match settings.autolaunch {
		true => app.autolaunch().enable(),
		false => app.autolaunch().disable(),
	};

	crate::devices::elgato::set_brightness(settings.brightness).await;
	
	let mut store = match crate::store::get_settings(app).await {
		Ok(store) => store,
		Err(error) => return Err(error.into()),
	};

	store.value = settings;
	store.save()?;
	Ok(())
}

#[command]
pub async fn get_localisations(app: AppHandle, locale: &str) -> Result<HashMap<String, serde_json::Value>, Error> {
	let mut localisations: HashMap<String, serde_json::Value> = HashMap::new();

	let mut entries = match tokio::fs::read_dir(&app.path_resolver().app_config_dir().unwrap().join("plugins")).await {
		Ok(entries) => entries,
		Err(error) => return Err(anyhow::Error::from(error).into()),
	};

	while let Ok(Some(entry)) = entries.next_entry().await {
		let path = match entry.metadata().await.unwrap().is_symlink() {
			true => tokio::fs::read_link(entry.path()).await.unwrap(),
			false => entry.path(),
		};
		let metadata = tokio::fs::metadata(&path).await.unwrap();
		if metadata.is_dir() {
			let Ok(locale) = tokio::fs::read(path.join(format!("{locale}.json"))).await else { continue };
			let Ok(locale): Result<serde_json::Value, _> = serde_json::from_slice(&locale) else {
				continue;
			};
			localisations.insert(path.file_name().unwrap().to_str().unwrap().to_owned(), locale);
		}
	}

	Ok(localisations)
}

#[command]
pub fn open_config_directory(app: AppHandle) {
	#[cfg(target_os = "windows")]
	let command = "explorer";
	#[cfg(target_os = "macos")]
	let command = "open";
	#[cfg(target_os = "linux")]
	let command = "xdg-open";
	std::process::Command::new(command).arg(app.path_resolver().app_config_dir().unwrap()).spawn().unwrap();
}

#[command]
pub fn get_build_info() -> String {
	format!(
		r#"
		<details>
			<summary> OpenDeck v{} ({}) on {} </summary>
			{}
		</details>
		"#,
		built_info::PKG_VERSION,
		built_info::GIT_COMMIT_HASH_SHORT.unwrap_or("commit hash unknown"),
		built_info::TARGET,
		built_info::DIRECT_DEPENDENCIES_STR
	)
}
