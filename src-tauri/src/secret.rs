//! Penyimpanan kunci database ARKAS di Windows Credential Manager.
//! Kunci tidak pernah ditulis ke file, log, atau repo.

use crate::error::{AppError, AppResult};

const SERVICE: &str = "sibos";
const ACCOUNT: &str = "arkas-db-key";
/// Untuk pengembangan/test: kunci bisa diberikan lewat variabel lingkungan.
const ENV_KEY: &str = "SIBOS_ARKAS_KEY";

fn entry() -> AppResult<keyring::Entry> {
	keyring::Entry::new(SERVICE, ACCOUNT).map_err(|e| AppError::Secret(e.to_string()))
}

pub fn load_key() -> AppResult<Option<String>> {
	if let Ok(key) = std::env::var(ENV_KEY) {
		if !key.is_empty() {
			return Ok(Some(key));
		}
	}
	match entry()?.get_password() {
		Ok(key) => Ok(Some(key)),
		Err(keyring::Error::NoEntry) => Ok(None),
		Err(e) => Err(AppError::Secret(e.to_string())),
	}
}

pub fn save_key(key: &str) -> AppResult<()> {
	entry()?.set_password(key).map_err(|e| AppError::Secret(e.to_string()))
}

pub fn clear_key() -> AppResult<()> {
	match entry()?.delete_credential() {
		Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
		Err(e) => Err(AppError::Secret(e.to_string())),
	}
}
