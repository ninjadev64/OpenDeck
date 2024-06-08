use std::collections::HashMap;
use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_inline_default::serde_inline_default;

use once_cell::sync::Lazy;
use tokio::sync::RwLock;

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
#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ActionState {
	#[serde(alias = "Image")]
	pub image: String,
	#[serde(alias = "Name")]
	pub name: String,
	#[serde(alias = "Title")]
	pub text: String,
	#[serde(alias = "ShowTitle")]
	pub show: bool,
	#[serde(alias = "TitleColor")]
	pub colour: String,
	#[serde(alias = "TitleAlignment")]
	pub alignment: String,
	#[serde(alias = "FontFamily")]
	pub family: String,
	#[serde(alias = "FontStyle")]
	pub style: String,
	#[serde(alias = "FontSize")]
	pub size: String,
	#[serde(alias = "FontUnderline")]
	pub underline: bool,
}

impl Default for ActionState {
	fn default() -> Self {
		Self {
			image: "actionDefaultImage".to_owned(),
			name: String::new(),
			text: String::new(),
			show: true,
			colour: "#FFFFFF".to_owned(),
			alignment: "middle".to_owned(),
			family: "Liberation Sans".to_owned(),
			style: "Regular".to_owned(),
			size: "16".to_owned(),
			underline: false,
		}
	}
}

/// An action, deserialised from the plugin manifest.
#[serde_inline_default]
#[derive(Clone, Serialize, Deserialize)]
pub struct Action {
	#[serde(alias = "Name")]
	pub name: String,

	#[serde(alias = "UUID")]
	pub uuid: String,

	#[serde_inline_default(String::new())]
	pub plugin: String,

	#[serde_inline_default(String::new())]
	#[serde(alias = "Tooltip")]
	pub tooltip: String,

	#[serde_inline_default(String::new())]
	#[serde(alias = "Icon")]
	pub icon: String,

	#[serde_inline_default(false)]
	#[serde(alias = "DisableAutomaticStates")]
	pub disable_automatic_states: bool,

	#[serde_inline_default(true)]
	#[serde(alias = "VisibleInActionsList")]
	pub visible_in_action_list: bool,

	#[serde_inline_default(true)]
	#[serde(alias = "SupportedInMultiActions")]
	pub supported_in_multi_actions: bool,

	#[serde_inline_default(true)]
	#[serde(alias = "UserTitleEnabled")]
	pub user_title_enabled: bool,

	#[serde_inline_default(String::new())]
	#[serde(alias = "PropertyInspectorPath")]
	pub property_inspector: String,

	#[serde_inline_default(vec!["Keypad".to_owned()])]
	#[serde(alias = "Controllers")]
	pub controllers: Vec<String>,

	#[serde(alias = "States")]
	pub states: Vec<ActionState>,
}

/// Location metadata of a slot.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Context {
	pub device: String,
	pub profile: String,
	pub controller: String,
	pub position: u8,
}

/// Information about the slot and index an instance is located in.
#[derive(Clone, PartialEq, Eq, serde_with::SerializeDisplay, serde_with::DeserializeFromStr)]
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
pub static CATEGORIES: Lazy<RwLock<HashMap<String, Vec<Action>>>> = Lazy::new(|| RwLock::new(HashMap::new()));
