//! Duplicates of many structs to facilitate saving profiles to disk without having device or profile IDs in context fields.

use crate::shared::{Action, ActionContext, ActionInstance, ActionState, Profile};

use std::path::Path;

use serde::{Deserialize, Serialize};

#[derive(serde_with::SerializeDisplay, serde_with::DeserializeFromStr)]
pub struct DiskActionContext {
	pub controller: String,
	pub position: u8,
	pub index: u16,
}

impl std::fmt::Display for DiskActionContext {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}.{}.{}", self.controller, self.position, self.index)
	}
}

impl std::str::FromStr for DiskActionContext {
	type Err = std::num::ParseIntError;
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let segments: Vec<&str> = s.split('.').collect();
		let mut offset: usize = 0;
		if segments.len() == 5 {
			offset = 2;
		}
		let controller = segments[offset].to_owned();
		let position = u8::from_str(segments[1 + offset])?;
		let index = u16::from_str(segments[2 + offset])?;
		Ok(Self { controller, position, index })
	}
}

impl From<ActionContext> for DiskActionContext {
	fn from(value: ActionContext) -> Self {
		Self {
			controller: value.controller,
			position: value.position,
			index: value.index,
		}
	}
}

impl DiskActionContext {
	fn into_action_context(self, path: &Path) -> ActionContext {
		let config_dir = crate::APP_HANDLE.get().unwrap().path_resolver().app_config_dir().unwrap();
		let mut iter = path.strip_prefix(config_dir).unwrap().iter();
		let device = iter.nth(1).unwrap().to_string_lossy().into_owned();
		let mut profile = iter.map(|x| x.to_string_lossy()).collect::<Vec<_>>().join("/");
		profile = profile[..profile.len() - 5].to_owned();
		ActionContext {
			device,
			profile,
			controller: self.controller,
			position: self.position,
			index: self.index,
		}
	}
}

#[derive(Serialize, Deserialize)]
pub struct DiskActionInstance {
	pub action: Action,
	pub context: DiskActionContext,
	pub states: Vec<ActionState>,
	pub current_state: u16,
	pub settings: serde_json::Value,
	pub children: Option<Vec<DiskActionInstance>>,
}

impl From<ActionInstance> for DiskActionInstance {
	fn from(value: ActionInstance) -> Self {
		Self {
			context: value.context.into(),
			action: value.action,
			states: value.states,
			current_state: value.current_state,
			settings: value.settings,
			children: value.children.map(|c| c.into_iter().map(|v| v.into()).collect()),
		}
	}
}

impl DiskActionInstance {
	fn into_action_instance(self, path: &Path) -> ActionInstance {
		ActionInstance {
			context: self.context.into_action_context(path),
			action: self.action,
			states: self.states,
			current_state: self.current_state,
			settings: self.settings,
			children: self.children.map(|c| c.into_iter().map(|v| v.into_action_instance(path)).collect()),
		}
	}
}

#[derive(Serialize, Deserialize)]
pub struct DiskProfile {
	pub keys: Vec<Option<DiskActionInstance>>,
	pub sliders: Vec<Option<DiskActionInstance>>,
}

impl From<&Profile> for DiskProfile {
	fn from(value: &Profile) -> Self {
		Self {
			keys: value.keys.clone().into_iter().map(|x| x.map(|v| v.into())).collect(),
			sliders: value.sliders.clone().into_iter().map(|x| x.map(|v| v.into())).collect(),
		}
	}
}

impl DiskProfile {
	fn into_profile(self, path: &Path) -> Profile {
		let config_dir = crate::APP_HANDLE.get().unwrap().path_resolver().app_config_dir().unwrap();
		let mut iter = path.strip_prefix(config_dir).unwrap().iter();
		let _ = iter.nth(1);
		let mut id = iter.map(|x| x.to_string_lossy()).collect::<Vec<_>>().join("/");
		id = id[..id.len() - 5].to_owned();
		Profile {
			id,
			keys: self.keys.into_iter().map(|x| x.map(|v| v.into_action_instance(path))).collect(),
			sliders: self.sliders.into_iter().map(|x| x.map(|v| v.into_action_instance(path))).collect(),
		}
	}
}

impl super::FromAndIntoDiskValue for Profile {
	fn into_value(&self) -> Result<serde_json::Value, serde_json::Error> {
		let disk: DiskProfile = self.into();
		serde_json::to_value(disk)
	}
	fn from_value(value: serde_json::Value, path: &Path) -> Result<Profile, serde_json::Error> {
		let disk: DiskProfile = serde_json::from_value(value)?;
		Ok(disk.into_profile(path))
	}
}
