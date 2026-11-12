//! Build script.
//!
//! Kunci bawaan ARKAS: diambil saat build dari file `.env` (`SIBOS_ARKAS_KEY=...`), variabel
//! lingkungan `SIBOS_ARKAS_KEY`, atau Windows Credential Manager mesin pengembang (`sibos` /
//! `arkas-db-key`). Kunci kemudian dienkripsi AES-256-GCM dengan kunci AES acak sekali-pakai
//! yang dibuat ulang di setiap build. Kunci AES itu dipecah menjadi tiga bagian acak (3-way XOR
//! secret sharing) yang disimpan terpisah di exe dan baru digabung sesaat di RAM saat dipakai,
//! sehingga baik kunci ARKAS maupun kunci AES tidak pernah ada utuh di dalam file.

use std::fmt::Write as _;
use std::path::PathBuf;

use aes_gcm::aead::{Aead, AeadCore, KeyInit, OsRng, Payload};
use aes_gcm::Aes256Gcm;

const ENV_NAME: &str = "SIBOS_ARKAS_KEY";
const KEYRING_SERVICE: &str = "sibos";
const KEYRING_ACCOUNT: &str = "arkas-db-key";
/// Harus sama dengan `AAD` di src/secret.rs.
const AAD: &[u8] = b"lajur/arkas-key/v2";

/// Ambil nilai `SIBOS_ARKAS_KEY` dari file `.env` (format `NAMA=nilai`, boleh diberi tanda kutip).
fn key_from_env_file(path: &PathBuf) -> Option<String> {
	let text = std::fs::read_to_string(path).ok()?;
	text.lines().find_map(|line| {
		let (name, value) = line.trim().split_once('=')?;
		if name.trim() != ENV_NAME {
			return None;
		}
		let value = value.trim().trim_matches('"').trim_matches('\'').to_string();
		(!value.is_empty()).then_some(value)
	})
}

/// Urutan sumber kunci saat build: `.env` -> variabel lingkungan -> Windows Credential Manager.
fn load_build_key(env_file: &PathBuf) -> Option<String> {
	if let Some(k) = key_from_env_file(env_file) {
		return Some(k);
	}
	if let Ok(k) = std::env::var(ENV_NAME) {
		let k = k.trim().to_string();
		if !k.is_empty() {
			return Some(k);
		}
	}
	keyring::Entry::new(KEYRING_SERVICE, KEYRING_ACCOUNT)
		.ok()
		.and_then(|entry| entry.get_password().ok())
		.filter(|k| !k.is_empty())
}

fn random_bytes<const N: usize>() -> [u8; N] {
	let mut out = [0u8; N];
	aes_gcm::aead::rand_core::RngCore::fill_bytes(&mut OsRng, &mut out);
	out
}

fn emit(out: &mut String, name: &str, bytes: &[u8]) {
	writeln!(out, "static {name}: [u8; {}] = {bytes:?};", bytes.len()).unwrap();
}

fn main() {
	let manifest = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
	let env_file = manifest.parent().unwrap().join(".env");
	println!("cargo:rerun-if-changed={}", env_file.display());
	println!("cargo:rerun-if-env-changed={ENV_NAME}");
	println!("cargo:rerun-if-env-changed=SIBOS_ALLOW_NO_KEY");

	let mut code = String::new();
	match load_build_key(&env_file) {
		Some(key) => {
			let aes_key = Aes256Gcm::generate_key(&mut OsRng);
			let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
			let sealed = Aes256Gcm::new(&aes_key)
				.encrypt(&nonce, Payload { msg: key.as_bytes(), aad: AAD })
				.expect("enkripsi kunci ARKAS gagal");

			let part_a: [u8; 32] = random_bytes();
			let part_b: [u8; 32] = random_bytes();
			let part_c: Vec<u8> =
				aes_key.iter().zip(&part_a).zip(&part_b).map(|((k, a), b)| k ^ a ^ b).collect();

			// Urutan dan nama sengaja tidak berurutan supaya bagian-bagian tidak bersebelahan.
			emit(&mut code, "EMBED_B", &part_b);
			emit(&mut code, "EMBED_SEALED", &sealed);
			emit(&mut code, "EMBED_A", &part_a);
			emit(&mut code, "EMBED_NONCE", &nonce);
			emit(&mut code, "EMBED_C", &part_c);
		}
		None => {
			// Installer tanpa kunci hanya jalan di komputer yang sudah menyimpan kunci di
			// Credential Manager; di laptop lain aplikasi gagal membuka ARKAS. Build rilis
			// ditolak kecuali memang disengaja (SIBOS_ALLOW_NO_KEY=1).
			let release = std::env::var("PROFILE").as_deref() == Ok("release");
			if release && std::env::var_os("SIBOS_ALLOW_NO_KEY").is_none() {
				panic!(
					"Build rilis tanpa kunci ARKAS: buat {} berisi {ENV_NAME}=<kunci>, \
					 atau set SIBOS_ALLOW_NO_KEY=1 bila memang ingin installer tanpa kunci bawaan.",
					env_file.display()
				);
			}
			for name in ["EMBED_A", "EMBED_B", "EMBED_C"] {
				emit(&mut code, name, &[0u8; 32]);
			}
			emit(&mut code, "EMBED_NONCE", &[0u8; 12]);
			emit(&mut code, "EMBED_SEALED", &[]);
		}
	}

	let out = PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("embedded_key.rs");
	std::fs::write(out, code).unwrap();

	tauri_build::build()
}
