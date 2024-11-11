use crate::store::{NotProfile, Store};

use std::collections::HashMap;

use active_win_pos_rs::get_active_window;
use once_cell::sync::Lazy;
use tauri::{Emitter, Manager};
use tokio::sync::RwLock;

pub type ApplicationProfiles = HashMap<String, HashMap<String, String>>;
impl NotProfile for ApplicationProfiles {}

pub static APPLICATIONS: RwLock<Vec<String>> = RwLock::const_new(Vec::new());
pub static APPLICATION_PROFILES: Lazy<RwLock<Store<ApplicationProfiles>>> = Lazy::new(|| RwLock::const_new(Store::new("applications", &crate::shared::config_dir(), HashMap::new()).unwrap()));

#[derive(Clone, serde::Serialize)]
pub struct SwitchProfileEvent {
	device: String,
	profile: String,
}

pub fn init_application_watcher() {
	tokio::spawn(async move {
		let mut previous = String::new();
		let app_handle = crate::APP_HANDLE.get().unwrap();
		loop {
			if let Ok(win) = get_active_window() {
				let mut applications = APPLICATIONS.write().await;
				if !applications.contains(&win.app_name) && win.app_name.to_lowercase() != "opendeck" {
					applications.push(win.app_name.clone());
					let _ = app_handle.get_webview_window("main").unwrap().emit("applications", applications.clone());
				}
				if win.app_name != previous {
					let application_profiles = &APPLICATION_PROFILES.read().await.value;
					if application_profiles.contains_key(&win.app_name) {
						let devices = application_profiles.get(&win.app_name).unwrap();
						for (device, profile) in devices.iter() {
							let _ = app_handle.get_webview_window("main").unwrap().emit(
								"switch_profile",
								SwitchProfileEvent {
									device: device.clone(),
									profile: profile.clone(),
								},
							);
						}
					}
				}
				previous = win.app_name;
			}

			tokio::time::sleep(std::time::Duration::from_millis(250)).await;
		}
	});
}
