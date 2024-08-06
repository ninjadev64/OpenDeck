//! Original source from https://github.com/MCOfficer/zip-extract/blob/master/src/lib.rs, modified to better suit extracting OpenAction plugins.

#![forbid(unsafe_code)]

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

use log::{debug, trace};
use std::io::{Read, Seek};
use std::path::Path;
use std::{fs, io};

#[derive(Debug)]
#[allow(dead_code)]
pub enum ZipExtractError {
	Io(io::Error),
	Zip(zip::result::ZipError),
}
impl From<io::Error> for ZipExtractError {
	fn from(value: io::Error) -> Self {
		Self::Io(value)
	}
}
impl From<zip::result::ZipError> for ZipExtractError {
	fn from(value: zip::result::ZipError) -> Self {
		Self::Zip(value)
	}
}
impl std::fmt::Display for ZipExtractError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{:?}", self)
	}
}
impl std::error::Error for ZipExtractError {}

/// Extracts a zip archive into `target_dir`.
pub fn extract<S: Read + Seek>(source: S, target_dir: &Path) -> Result<(), ZipExtractError> {
	if !target_dir.exists() {
		fs::create_dir(target_dir)?;
	}

	let mut archive = zip::ZipArchive::new(source)?;

	// OpenAction plugins should always contain multiple files, so if there is only one, assume it's a nested archive.
	if archive.len() == 1 {
		let file = archive.by_index(0)?;
		if file.name().to_lowercase().ends_with(".streamdeckplugin") || file.name().to_lowercase().ends_with(".zip") {
			return extract(std::io::Cursor::new(file.bytes().flatten().collect::<Vec<u8>>()), target_dir);
		}
	}

	debug!("Extracting to {}", target_dir.to_string_lossy());
	for i in 0..archive.len() {
		let mut file = archive.by_index(i)?;
		let relative_path = file.mangled_name();

		if relative_path.to_string_lossy().is_empty() {
			// Top-level directory
			continue;
		}

		let mut outpath = target_dir.to_path_buf();
		outpath.push(relative_path);

		let name = file.name().replace('\\', "/");

		trace!("Extracting {} to {}", name, outpath.to_string_lossy());
		if name.ends_with('/') {
			fs::create_dir_all(&outpath)?;
		} else {
			if let Some(p) = outpath.parent() {
				if !p.exists() {
					fs::create_dir_all(p)?;
				}
			}
			let mut outfile = fs::File::create(&outpath)?;
			io::copy(&mut file, &mut outfile)?;
		}

		#[cfg(unix)]
		set_unix_mode(&file, &outpath)?;
	}

	debug!("Extracted {} files", archive.len());
	Ok(())
}

#[cfg(unix)]
fn set_unix_mode(file: &zip::read::ZipFile, outpath: &Path) -> io::Result<()> {
	if let Some(m) = file.unix_mode() {
		fs::set_permissions(outpath, PermissionsExt::from_mode(m))?
	}
	Ok(())
}
