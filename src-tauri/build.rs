//! Build script.
//!
//! Kunci bawaan ARKAS (opsional): bila file `.env` di root proyek berisi `SIBOS_ARKAS_KEY=...`,
//! kunci ditanam ke exe dalam bentuk tersamar (XOR dengan pad acak yang berbeda tiap build),
//! sehingga teks aslinya tidak ada di dalam exe. `.env` tidak ikut repo (lihat .gitignore);
//! tanpa `.env`, exe tetap meminta kunci di halaman Koneksi ARKAS.
//! Ini penyamaran, bukan enkripsi kuat: exe yang dibongkar tetap bisa mengungkap kunci.

use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

const ENV_NAME: &str = "SIBOS_ARKAS_KEY";

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

/// Pad acak sepanjang `len` (xorshift64 dengan benih waktu build).
fn random_pad(len: usize) -> Vec<u8> {
	let mut seed = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_nanos() as u64).unwrap_or(0x9E37_79B9) | 1;
	(0..len)
		.map(|_| {
			seed ^= seed << 13;
			seed ^= seed >> 7;
			seed ^= seed << 17;
			(seed >> 24) as u8
		})
		.collect()
}

fn main() {
	let manifest = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
	let env_file = manifest.parent().unwrap().join(".env");
	println!("cargo:rerun-if-changed={}", env_file.display());

	let (pad, data) = match key_from_env_file(&env_file) {
		Some(key) => {
			let pad = random_pad(key.len());
			let data: Vec<u8> = key.bytes().zip(&pad).map(|(b, p)| b ^ p).collect();
			(pad, data)
		}
		None => (Vec::new(), Vec::new()),
	};
	let out = PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("embedded_key.rs");
	std::fs::write(out, format!("const EMBED_PAD: &[u8] = &{pad:?};\nconst EMBED_DATA: &[u8] = &{data:?};\n")).unwrap();

	tauri_build::build()
}
