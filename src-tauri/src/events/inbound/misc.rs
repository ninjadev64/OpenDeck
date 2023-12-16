use serde::Deserialize;

#[derive(Deserialize)]
pub struct RegisterEvent {
	pub uuid: String
}

#[derive(Deserialize)]
pub struct OpenUrlEvent {
	pub url: String
}
