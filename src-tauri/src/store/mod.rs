pub mod profiles;

use std::fs;
use std::path::PathBuf;

use serde::{Serialize, Deserialize};

use anyhow::Context;

/// Allows for easy persistence of values using JSON files.
pub struct Store<T> where T: Serialize + for<'a> Deserialize<'a> {
	pub id: String,
	pub value: T,
	path: PathBuf
}

impl<T> Store<T> where T: Serialize + for<'a> Deserialize<'a> {
	/// Create a new Store given an ID and storage directory.
	pub fn new(id: &str, config_dir: PathBuf, default: T) -> Result<Self, anyhow::Error> {
		let mut path = config_dir.clone();
		path.push(format!("{}.json", id));

		if path.exists() {
			let file_contents = fs::read(&path)
				.with_context(|| { format!("Failed to read contents of file at {}", &path.display()) })?;
			let existing_value: T = serde_json::from_slice(&file_contents)
				.with_context(|| { format!("Failed to parse config file at {}", &path.display()) })?;
			
			Ok(Self {
				path,
				id: String::from(id),
				value: existing_value
			})
		} else {
			Ok(Self {
				path,
				id: String::from(id),
				value: default
			})
		}
	}

	/// Save the relevant Store as a file.
	pub fn save(&self) -> anyhow::Result<(), anyhow::Error> {
		fs::create_dir_all(self.path.parent().unwrap())
			.with_context(|| { format!("Failed to create directories at {}", self.path.display()) })?;
		fs::write(&self.path, serde_json::to_string_pretty(&self.value).unwrap())
			.with_context(|| { format!("Failed to write to file at {}", self.path.display()) })?;
		Ok(())
	}
}
