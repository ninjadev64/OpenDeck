use std::collections::HashMap;
use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_inline_default::serde_inline_default;

use once_cell::sync::Lazy;
use tokio::sync::Mutex;

/// Convert an icon specified in a plugin manifest to its full path.
pub fn convert_icon(path: String) -> String {
	if Path::new(&(path.clone() + ".svg")).exists() {
		path + ".svg"
	} else if Path::new(&(path.clone() + "@2x.png")).exists() {
		path + "@2x.png"
	} else {
		path + ".png"
	}
}

/// A state of an action.
#[serde_inline_default]
#[derive(Clone, Serialize, Deserialize)]
pub struct ActionState {
	#[serde(alias = "Image")]
	pub image: String,

	#[serde_inline_default(String::from(""))]
	#[serde(alias = "Name")]
	pub name: String,

	#[serde_inline_default(String::from(""))]
	#[serde(alias = "Title")]
	pub text: String,

	#[serde_inline_default(true)]
	#[serde(alias = "ShowTitle")]
	pub show: bool,

	#[serde_inline_default(String::from("#f2f2f2"))]
	#[serde(alias = "TitleColor")]
	pub colour: String,

	#[serde_inline_default(String::from("middle"))]
	#[serde(alias = "TitleAlignment")]
	pub alignment: String,

	#[serde_inline_default(String::from("Regular"))]
	#[serde(alias = "FontStyle")]
	pub style: String,

	#[serde_inline_default(String::from("16"))]
	#[serde(alias = "FontSize")]
	pub size: String,

	#[serde_inline_default(false)]
	#[serde(alias = "FontUnderline")]
	pub underline: bool,
}

/// An action, deserialised from the plugin manifest.
#[serde_inline_default]
#[derive(Clone, Serialize, Deserialize)]
pub struct Action {
	#[serde(alias = "Name")]
	pub name: String,

	#[serde(alias = "UUID")]
	pub uuid: String,

	#[serde_inline_default(String::from(""))]
	pub plugin: String,

	#[serde(alias = "Tooltip")]
	pub tooltip: String,

	#[serde(alias = "Icon")]
	pub icon: String,

	#[serde_inline_default(true)]
	#[serde(alias = "VisibleInActionsList")]
	pub visible_in_action_list: bool,

	#[serde_inline_default(String::from(""))]
	#[serde(alias = "PropertyInspectorPath")]
	pub property_inspector: String,

	#[serde_inline_default(vec![String::from("Keypad")])]
	#[serde(alias = "Controllers")]
	pub controllers: Vec<String>,

	#[serde(alias = "States")]
	pub states: Vec<ActionState>,
}

/// Location metadata of a slot.
#[derive(Clone, Serialize, Deserialize)]
pub struct Context {
	pub device: String,
	pub profile: String,
	pub controller: String,
	pub position: u8,
}

/// Information about the slot and index an instance is located in.
#[derive(Clone, serde_with::SerializeDisplay, serde_with::DeserializeFromStr)]
pub struct ActionContext {
	pub device: String,
	pub profile: String,
	pub controller: String,
	pub position: u8,
	pub index: u16,
}

impl std::fmt::Display for ActionContext {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}.{}.{}.{}.{}", self.device, self.profile, self.controller, self.position, self.index)
	}
}

impl std::str::FromStr for ActionContext {
	type Err = std::num::ParseIntError;
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let segments: Vec<&str> = s.split('.').collect();
		let device = segments[0].to_owned();
		let profile = segments[1].to_owned();
		let controller = segments[2].to_owned();
		let position = u8::from_str(segments[3])?;
		let index = u16::from_str(segments[4])?;
		Ok(Self {
			device,
			profile,
			controller,
			position,
			index,
		})
	}
}

impl ActionContext {
	pub fn from_context(context: Context, index: u16) -> Self {
		Self {
			device: context.device,
			profile: context.profile,
			controller: context.controller,
			position: context.position,
			index,
		}
	}
}

impl From<ActionContext> for Context {
	fn from(value: ActionContext) -> Self {
		Self {
			device: value.device,
			profile: value.profile,
			controller: value.controller,
			position: value.position,
		}
	}
}

impl From<&ActionContext> for Context {
	fn from(value: &ActionContext) -> Self {
		Self::from(value.clone())
	}
}

/// An instance of an action.
#[derive(Clone, Serialize, Deserialize)]
pub struct ActionInstance {
	pub action: Action,
	pub context: ActionContext,
	pub states: Vec<ActionState>,
	pub current_state: u16,
	pub settings: serde_json::Value,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Profile {
	pub id: String,
	pub keys: Vec<Vec<ActionInstance>>,
	pub sliders: Vec<Vec<ActionInstance>>,
}

/// A map of category names to a list of actions in that category.
pub static CATEGORIES: Lazy<Mutex<HashMap<String, Vec<Action>>>> = Lazy::new(|| Mutex::new(HashMap::new()));
