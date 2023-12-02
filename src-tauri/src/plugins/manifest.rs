use crate::shared::Action;

use serde::Deserialize;
use serde_inline_default::serde_inline_default;

#[derive(Deserialize)]
pub(super) struct OS {
	#[serde(alias = "Platform")]
	pub platform: String
}

#[allow(dead_code)]
#[serde_inline_default]
#[derive(Deserialize)]
pub(super) struct PluginManifest {
	#[serde(alias = "Version")]
	pub version: String,

	#[serde_inline_default(String::from("Custom"))]
	#[serde(alias = "Category")]
	pub category: String,

	#[serde(alias = "Actions")]
	pub actions: Vec<Action>,

	#[serde(alias = "OS")]
	pub os: Vec<OS>,

	#[serde(alias = "CodePath")]
	pub code_path: Option<String>,

	#[serde(alias = "CodePathWin")]
	pub code_path_windows: Option<String>,

	#[serde(alias = "CodePathMac")]
	pub code_path_macos: Option<String>,

	#[serde(alias = "CodePathLin")]
	pub code_path_linux: Option<String>
}
