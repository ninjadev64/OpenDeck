pub mod profiles;

use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Allows for easy persistence of values using JSON files.
pub struct Store<T>
where
	T: Serialize + for<'a> Deserialize<'a>,
{
	pub value: T,
	path: PathBuf,
}

impl<T> Store<T>
where
	T: Serialize + for<'a> Deserialize<'a>,
{
	/// Create a new Store given an ID and storage directory.
	pub fn new(id: &str, config_dir: &PathBuf, default: T) -> Result<Self, anyhow::Error> {
		let path = config_dir.join(format!("{}.json", id));

		if path.exists() {
			let file_contents = fs::read(&path)?;
			let existing_value: T = serde_json::from_slice(&file_contents)?;

			Ok(Self { path, value: existing_value })
		} else {
			Ok(Self { path, value: default })
		}
	}

	/// Save the relevant Store as a file.
	pub fn save(&self) -> Result<(), anyhow::Error> {
		fs::create_dir_all(self.path.parent().unwrap())?;
		fs::write(&self.path, serde_json::to_string_pretty(&self.value)?)?;
		Ok(())
	}
}

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
	pub language: String,
	pub autolaunch: bool,
	pub darktheme: bool,
	pub brightness: u8,
	pub developer: bool,
}

impl Default for Settings {
	fn default() -> Self {
		Self {
			language: "en".to_owned(),
			autolaunch: false,
			darktheme: true,
			brightness: 50,
			developer: false,
		}
	}
}

pub fn get_settings(app_handle: &tauri::AppHandle) -> Result<Store<Settings>, anyhow::Error> {
	Store::new("settings", &app_handle.path_resolver().app_config_dir().unwrap(), Settings::default())
}
