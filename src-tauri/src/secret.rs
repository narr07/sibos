//! Kunci database ARKAS: dari Windows Credential Manager, atau kunci bawaan yang ditanam
//! (tersamar) saat build dari file `.env` (lihat build.rs). Kunci tidak pernah ditulis ke log atau repo.

use crate::error::{AppError, AppResult};

const SERVICE: &str = "sibos";
const ACCOUNT: &str = "arkas-db-key";
/// Untuk pengembangan/test: kunci bisa diberikan lewat variabel lingkungan.
const ENV_KEY: &str = "SIBOS_ARKAS_KEY";

fn entry() -> AppResult<keyring::Entry> {
	keyring::Entry::new(SERVICE, ACCOUNT).map_err(|e| AppError::Secret(e.to_string()))
}

include!(concat!(env!("OUT_DIR"), "/embedded_key.rs"));

/// Kunci bawaan installer; kosong bila build tanpa `.env`.
fn embedded_key() -> Option<String> {
	if EMBED_DATA.is_empty() {
		return None;
	}
	let bytes: Vec<u8> = EMBED_DATA.iter().zip(EMBED_PAD).map(|(d, p)| d ^ p).collect();
	String::from_utf8(bytes).ok()
}

/// Urutan: variabel lingkungan (pengembangan) → Credential Manager → kunci bawaan installer.
pub fn load_key() -> AppResult<Option<String>> {
	if let Ok(key) = std::env::var(ENV_KEY) {
		if !key.is_empty() {
			return Ok(Some(key));
		}
	}
	match entry()?.get_password() {
		Ok(key) => Ok(Some(key)),
		Err(keyring::Error::NoEntry) => Ok(embedded_key()),
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
