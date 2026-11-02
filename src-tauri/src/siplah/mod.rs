//! Crawler katalog toko dari situs mitra SIPLah (pasar daring resmi Kemendikdasmen).
//!
//! Hanya halaman dan API publik yang dibaca (tanpa login), mengikuti robots.txt tiap situs.
//! Mitra yang memasang anti-bot atau melarang akses otomatis (Telkom, Eureka, Blibli) tidak didukung.
//!
//! Tiap platform punya modul sendiri yang menghasilkan profil toko dan satu halaman produk.

mod gramedia;
pub mod html;
pub mod http;
mod intan;
mod masmedia;
mod temprina;
mod tokoladang;

use crate::error::{AppError, AppResult};
use crate::repo::app::siplah::{SiplahProduk, SiplahToko};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
	TokoLadang,
	Gramedia,
	Intan,
	Temprina,
	Masmedia,
}

impl Platform {
	pub const ALL: [Platform; 5] = [Platform::TokoLadang, Platform::Gramedia, Platform::Intan, Platform::Temprina, Platform::Masmedia];

	/// Kode yang disimpan di kolom `siplah_toko.platform`.
	pub fn kode(self) -> &'static str {
		match self {
			Platform::TokoLadang => "tokoladang",
			Platform::Gramedia => "gramedia",
			Platform::Intan => "intan",
			Platform::Temprina => "temprina",
			Platform::Masmedia => "masmedia",
		}
	}

	pub fn from_kode(kode: &str) -> Option<Platform> {
		Platform::ALL.into_iter().find(|p| p.kode() == kode)
	}

	/// Batas halaman per sinkronisasi agar toko raksasa (ribuan produk) tidak membuat sinkron berjam-jam.
	pub fn max_halaman(self) -> u32 {
		match self {
			Platform::TokoLadang => 100, // 30 produk per halaman
			Platform::Gramedia => 150,   // 20
			Platform::Intan => 200,      // 15
			Platform::Temprina => 150,   // 20
			Platform::Masmedia => 100,   // 30
		}
	}

	/// Jeda antar halaman supaya tidak membebani situs mitra.
	pub fn jeda_ms(self) -> u64 {
		match self {
			Platform::TokoLadang | Platform::Masmedia | Platform::Temprina => 150,
			Platform::Gramedia | Platform::Intan => 400,
		}
	}
}

/// Satu halaman katalog toko.
pub struct Halaman {
	pub produk: Vec<SiplahProduk>,
	pub ada_lagi: bool,
}

/// Contoh link yang diterima, ditampilkan saat link tidak dikenali.
pub const CONTOH_LINK: &str = "Link toko dari Toko Ladang (siplah.tokoladang.co.id/m/...), Gramedia (siplahgramedia.id/main/penyedia/...), \
	Intan (siplah.intanonline.com/frontend/web/store/...), Temprina (siplah.temprina.co.id/store/...), \
	atau Masmedia (siplahmasmedia.co.id/vendor/...).";

/// Kenali platform dan kode toko dari link (atau ID angka Toko Ladang, untuk kompatibilitas).
pub fn kenali(input: &str) -> AppResult<(Platform, String)> {
	let raw = input.trim();
	let invalid = || AppError::InvalidInput(format!("Link toko SIPLah tidak dikenali. {CONTOH_LINK}"));
	if !raw.is_empty() && raw.chars().all(|c| c.is_ascii_digit()) {
		return Ok((Platform::TokoLadang, raw.to_string()));
	}
	let lower = raw.to_lowercase();
	let without_scheme = lower.split("://").last().unwrap_or(&lower);
	let (host, rest) = without_scheme.split_once('/').unwrap_or((without_scheme, ""));
	let host = host.trim_start_matches("www.");
	let (path, query) = rest.split_once('?').unwrap_or((rest, ""));
	let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
	let after = |key: &str| segments.iter().position(|s| *s == key).and_then(|i| segments.get(i + 1)).copied();

	let (platform, kode) = if host.ends_with("tokoladang.co.id") {
		(Platform::TokoLadang, tokoladang::kode_dari_link(raw).ok_or_else(invalid)?)
	} else if host.ends_with("siplahgramedia.id") {
		(Platform::Gramedia, after("penyedia").ok_or_else(invalid)?.to_string())
	} else if host.ends_with("intanonline.com") {
		let slug = query
			.split('&')
			.find_map(|kv| kv.strip_prefix("slug="))
			.or_else(|| after("store").filter(|s| *s != "detail"))
			.ok_or_else(invalid)?;
		(Platform::Intan, slug.to_string())
	} else if host.ends_with("temprina.co.id") {
		(Platform::Temprina, after("store").ok_or_else(invalid)?.to_string())
	} else if host.ends_with("siplahmasmedia.co.id") {
		(Platform::Masmedia, masmedia::normalisasi_kode(after("vendor").ok_or_else(invalid)?).ok_or_else(invalid)?)
	} else {
		return Err(invalid());
	};

	if kode.is_empty() || kode.len() > 120 || !kode.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
		return Err(invalid());
	}
	Ok((platform, kode))
}

/// ID internal stabil untuk toko/produk platform selain Toko Ladang. Selalu negatif (tidak bentrok
/// dengan ID Toko Ladang) dan di bawah 2^52 agar aman sebagai angka JavaScript di frontend.
pub fn stable_id(platform: Platform, key: &str) -> i64 {
	// FNV-1a 64-bit: deterministik di semua versi Rust (tidak seperti DefaultHasher).
	let mut h: u64 = 0xcbf2_9ce4_8422_2325;
	for b in platform.kode().bytes().chain([b':']).chain(key.bytes()) {
		h ^= u64::from(b);
		h = h.wrapping_mul(0x0100_0000_01b3);
	}
	-((h & ((1 << 52) - 1)) as i64) - 1
}

/// ID toko di sibos.db untuk pasangan platform + kode.
pub fn toko_id(platform: Platform, kode: &str) -> AppResult<i64> {
	match platform {
		Platform::TokoLadang => kode.parse().map_err(|_| AppError::InvalidInput("ID toko Toko Ladang tidak valid".into())),
		_ => Ok(stable_id(platform, kode)),
	}
}

/// Toko kosong dengan field umum terisi; modul platform melengkapi sisanya.
fn toko_dasar(platform: Platform, kode: &str, nama: String, url: String) -> AppResult<SiplahToko> {
	Ok(SiplahToko {
		id: toko_id(platform, kode)?,
		platform: platform.kode().into(),
		kode: Some(kode.into()),
		nama,
		slug: None,
		url,
		tipe: None,
		slogan: None,
		alamat: None,
		provinsi: None,
		kabupaten: None,
		kecamatan: None,
		kodepos: None,
		gambar: None,
		total_produk: 0,
		terakhir_sync: None,
		created_at: String::new(),
		updated_at: String::new(),
	})
}

/// "..., KAB. MALUKU TENGAH, MALUKU" -> (Some("KAB. MALUKU TENGAH"), Some("MALUKU")).
fn kabupaten_provinsi(alamat: &str) -> (Option<String>, Option<String>) {
	let parts: Vec<&str> = alamat.split(',').map(str::trim).filter(|p| !p.is_empty()).collect();
	let prov = parts.last().map(|p| p.trim_start_matches("Prov.").trim().to_string());
	let kab = (parts.len() >= 2).then(|| parts[parts.len() - 2].to_string());
	(kab, prov)
}

/// Profil toko terbaru dari situs asal.
pub fn ambil_toko(platform: Platform, kode: &str) -> AppResult<SiplahToko> {
	match platform {
		Platform::TokoLadang => tokoladang::toko(kode),
		Platform::Gramedia => gramedia::toko(kode),
		Platform::Intan => intan::toko(kode),
		Platform::Temprina => temprina::toko(kode),
		Platform::Masmedia => masmedia::toko(kode),
	}
}

/// Satu halaman produk toko (mulai dari 1).
pub fn ambil_halaman(platform: Platform, kode: &str, toko_id: i64, halaman: u32) -> AppResult<Halaman> {
	match platform {
		Platform::TokoLadang => tokoladang::halaman(kode, toko_id, halaman),
		Platform::Gramedia => gramedia::halaman(kode, toko_id, halaman),
		Platform::Intan => intan::halaman(kode, toko_id, halaman),
		Platform::Temprina => temprina::halaman(kode, toko_id, halaman),
		Platform::Masmedia => masmedia::halaman(kode, toko_id, halaman),
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn recognizes_store_links() {
		let cases = [
			("https://siplah.tokoladang.co.id/m/kpr-rajagaluh.92186", Platform::TokoLadang, "92186"),
			("92186", Platform::TokoLadang, "92186"),
			("https://siplahgramedia.id/main/penyedia/274a734f75894f359d958a2a4c1f76a8?page=2", Platform::Gramedia, "274a734f75894f359d958a2a4c1f76a8"),
			("https://siplah.intanonline.com/frontend/web/store/ptnusasaranapratama", Platform::Intan, "ptnusasaranapratama"),
			("https://siplah.intanonline.com/frontend/web/store/detail/index?slug=naulibasa&page=3&per-page=15", Platform::Intan, "naulibasa"),
			("siplah.temprina.co.id/store/cv-futo-indonesia/23", Platform::Temprina, "cv-futo-indonesia"),
			("https://siplahmasmedia.co.id/vendor/3347_7411", Platform::Masmedia, "3347_7411"),
			("https://siplahmasmedia.co.id/vendor/7411", Platform::Masmedia, "3347_7411"),
		];
		for (link, platform, kode) in cases {
			assert_eq!(kenali(link).unwrap(), (platform, kode.to_string()), "{link}");
		}
	}

	#[test]
	fn rejects_unknown_or_unsafe_links() {
		for link in [
			"https://siplahtelkom.com/toko/abc",
			"https://siplah.blibli.com/merchant/X",
			"https://siplah.temprina.co.id/",
			"https://siplah.temprina.co.id/store/abc%22;rm",
			"",
		] {
			assert!(kenali(link).is_err(), "{link}");
		}
	}

	#[test]
	fn stable_ids_are_negative_deterministic_and_js_safe() {
		let a = stable_id(Platform::Gramedia, "abc");
		assert_eq!(a, stable_id(Platform::Gramedia, "abc"));
		assert_ne!(a, stable_id(Platform::Intan, "abc"));
		assert!(a < 0 && a >= -(1_i64 << 52));
		assert_eq!(toko_id(Platform::TokoLadang, "92186").unwrap(), 92186);
	}

	#[test]
	fn splits_regency_and_province() {
		assert_eq!(
			kabupaten_provinsi("Baumata Timur, Taebenu, Kab. Kupang, Prov. Nusa Tenggara Timur"),
			(Some("Kab. Kupang".into()), Some("Nusa Tenggara Timur".into()))
		);
		assert_eq!(kabupaten_provinsi("MALUKU"), (None, Some("MALUKU".into())));
	}
}
