fn main() {
	built::write_built_file().expect("failed to acquire build-time information");
	tauri_build::build();
}
