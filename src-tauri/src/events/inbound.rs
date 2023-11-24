use serde::Deserialize;

#[derive(Deserialize)]
pub struct RegisterEvent {
	pub uuid: String
}
