//! Modul kredensial dan perlindungan runtime.

use std::hint::black_box;

use aes_gcm::aead::{Aead, KeyInit, Payload};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use zeroize::Zeroizing;

use crate::error::{AppError, AppResult};

const SERVICE: &str = "sibos";
const ACCOUNT: &str = "arkas-db-key";
/// Untuk pengembangan/test: kunci bisa diberikan lewat variabel lingkungan.
const ENV_KEY: &str = "SIBOS_ARKAS_KEY";

fn entry() -> AppResult<keyring::Entry> {
	keyring::Entry::new(SERVICE, ACCOUNT).map_err(|e| AppError::Secret(e.to_string()))
}

include!(concat!(env!("OUT_DIR"), "/embedded_key.rs"));

/// Harus sama dengan `AAD` di build.rs.
const AAD: &[u8] = b"lajur/arkas-key/v2";

/// Deteksi debugger aktif di Windows (mencegah ekstraksi kunci lewat breakpoint memori).
#[cfg(all(target_os = "windows", not(test)))]
fn is_debugger_present() -> bool {
	#[link(name = "kernel32")]
	extern "system" {
		fn IsDebuggerPresent() -> i32;
	}
	// SAFETY: IsDebuggerPresent adalah fungsi Win32 tanpa argumen yang selalu aman dipanggil.
	unsafe { IsDebuggerPresent() != 0 }
}

#[cfg(any(not(target_os = "windows"), test))]
fn is_debugger_present() -> bool {
	false
}

/// Apakah binary ini memiliki kunci bawaan yang tertanam saat build.
pub fn has_embedded_key() -> bool {
	!black_box(&EMBED_SEALED).is_empty()
}

/// Kunci bawaan installer; didekripsi sesaat di RAM dalam pembungkus `Zeroizing`.
pub fn embedded_key() -> Option<Zeroizing<String>> {
	if is_debugger_present() {
		return None;
	}
	// black_box: cegah kompiler menggabungkan ketiga bagian menjadi satu konstanta di exe.
	let sealed: &[u8] = black_box(&EMBED_SEALED);
	if sealed.is_empty() {
		return None;
	}
	let (a, b, c) = (black_box(&EMBED_A), black_box(&EMBED_B), black_box(&EMBED_C));
	let mut aes_key = Zeroizing::new([0u8; 32]);
	for i in 0..32 {
		aes_key[i] = a[i] ^ b[i] ^ c[i];
	}
	let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&aes_key[..]));
	let plain = Zeroizing::new(
		cipher
			.decrypt(Nonce::from_slice(black_box(&EMBED_NONCE)), Payload { msg: sealed, aad: AAD })
			.ok()?,
	);
	let text = String::from_utf8(plain.to_vec()).ok()?;
	(!text.is_empty()).then(|| Zeroizing::new(text))
}

/// Urutan: variabel lingkungan (pengembangan) → kunci bawaan terenkripsi → Credential Manager.
pub fn load_key() -> AppResult<Option<Zeroizing<String>>> {
	if let Ok(key) = std::env::var(ENV_KEY) {
		if !key.is_empty() {
			return Ok(Some(Zeroizing::new(key)));
		}
	}
	if let Some(key) = embedded_key() {
		return Ok(Some(key));
	}
	match entry().and_then(|e| e.get_password().map_err(|err| AppError::Secret(err.to_string()))) {
		Ok(key) if !key.is_empty() => Ok(Some(Zeroizing::new(key))),
		_ => Ok(None),
	}
}

pub fn save_key(key: &str) -> AppResult<()> {
	// Bila kunci bawaan sudah tertanam di binary, jangan simpan kunci ke Windows Credential Manager
	// di laptop pengguna agar tidak bisa diintip lewat Control Panel -> Credential Manager.
	if has_embedded_key() {
		return Ok(());
	}
	entry()?.set_password(key).map_err(|e| AppError::Secret(e.to_string()))
}

pub fn clear_key() -> AppResult<()> {
	match entry()?.delete_credential() {
		Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
		Err(e) => Err(AppError::Secret(e.to_string())),
	}
}

#[cfg(test)]
mod tests {
	/// Kunci bawaan harus terbuka persis sama dengan sumber kunci saat build.
	/// Memakai `assert!`, bukan `assert_eq!`, agar kunci tidak tercetak saat test gagal.
	#[test]
	fn embedded_key_matches_env_file() {
		let env = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../.env");
		let from_file = std::fs::read_to_string(env).ok().and_then(|text| {
			text.lines().find_map(|line| {
				let (name, value) = line.trim().split_once('=')?;
				(name.trim() == super::ENV_KEY)
					.then(|| value.trim().trim_matches('"').trim_matches('\'').to_string())
					.filter(|v| !v.is_empty())
			})
		});
		let from_env = std::env::var(super::ENV_KEY).ok().filter(|v| !v.trim().is_empty());
		let from_keyring = keyring::Entry::new(super::SERVICE, super::ACCOUNT)
			.ok()
			.and_then(|e| e.get_password().ok())
			.filter(|v| !v.is_empty());
		let expected = from_file.or(from_env).or(from_keyring);
		let embedded = super::embedded_key().map(|z| z.as_str().to_string());
		assert!(embedded == expected, "kunci bawaan tidak cocok dengan sumber build");
	}
}
