use crate::shared::Action;

use serde::Deserialize;
use serde_inline_default::serde_inline_default;

#[derive(Deserialize)]
pub struct OS {
	#[serde(alias = "Platform")]
	pub platform: String,
}

#[allow(dead_code)]
#[serde_inline_default]
#[derive(Deserialize)]
pub struct PluginManifest {
	#[serde(alias = "Name")]
	pub name: String,

	#[serde(alias = "Author")]
	pub author: String,

	#[serde(alias = "Version")]
	pub version: String,

	#[serde(alias = "Icon")]
	pub icon: String,

	#[serde_inline_default("Custom".to_owned())]
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
	pub code_path_linux: Option<String>,

	#[serde(alias = "PropertyInspectorPath")]
	pub property_inspector_path: Option<String>,
}
