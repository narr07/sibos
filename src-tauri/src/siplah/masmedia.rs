//! SIPLah Masmedia (siplahmasmedia.co.id, PT Masmedia Buana Pustaka): API JSON publik
//! `api.siplahmasmedia.co.id/3347` (tanpa login): `item?member_id=...` (30 per halaman) dan
//! `member/search?member_id=...`. Nomor HP penjual di respons sengaja tidak disimpan.

use serde_json::Value;

use super::{http, stable_id, toko_dasar, Halaman, Platform};
use crate::error::{AppError, AppResult};
use crate::repo::app::siplah::{SiplahProduk, SiplahToko};

const API: &str = "https://api.siplahmasmedia.co.id/3347";
const SITE: &str = "https://siplahmasmedia.co.id";
const KOLOM_ZONA: [&str; 5] = ["price2", "price3", "price4", "price5a", "price5b"];

/// ID toko di link bisa "3347_7411" atau "7411"; API memakai bentuk berawalan "3347_".
pub fn normalisasi_kode(raw: &str) -> Option<String> {
	let digits = raw.strip_prefix("3347_").unwrap_or(raw);
	(!digits.is_empty() && digits.chars().all(|c| c.is_ascii_digit())).then(|| format!("3347_{digits}"))
}

fn get(path: &str) -> AppResult<Value> {
	let body = http::get(&format!("{API}/{path}"), "application/json")?;
	serde_json::from_str(&body).map_err(|e| AppError::InvalidInput(format!("Respons Masmedia tidak valid: {e}")))
}

fn teks(v: &Value) -> Option<String> {
	match v {
		Value::String(s) if !s.trim().is_empty() => Some(s.trim().to_string()),
		Value::Number(n) => Some(n.to_string()),
		_ => None,
	}
}

fn angka(v: &Value) -> f64 {
	match v {
		Value::Number(n) => n.as_f64().unwrap_or(0.0),
		Value::String(s) => s.trim().parse().unwrap_or(0.0),
		_ => 0.0,
	}
}

pub fn toko(kode: &str) -> AppResult<SiplahToko> {
	parse_toko(&get(&format!("member/search?member_id={kode}"))?, kode)
}

fn parse_toko(res: &Value, kode: &str) -> AppResult<SiplahToko> {
	let data = res["data"]
		.as_array()
		.and_then(|list| list.iter().find(|m| m["member_id"] == kode))
		.ok_or_else(|| AppError::InvalidInput(format!("Toko Masmedia {kode} tidak ditemukan.")))?;
	let nama = teks(&data["name"]).unwrap_or_else(|| "Toko Masmedia".into());
	let mut t = toko_dasar(Platform::Masmedia, kode, nama, format!("{SITE}/vendor/{kode}"))?;
	t.provinsi = teks(&data["provinsi"]);
	t.kabupaten = teks(&data["kabupaten"]);
	Ok(t)
}

pub fn halaman(kode: &str, toko_id: i64, n: u32) -> AppResult<Halaman> {
	Ok(parse_halaman(&get(&format!("item?member_id={kode}&page={n}&limit=30"))?, toko_id, n))
}

fn parse_halaman(res: &Value, toko_id: i64, n: u32) -> Halaman {
	let items = res["data"].as_array().cloned().unwrap_or_default();
	let mut produk = Vec::with_capacity(items.len());
	for it in &items {
		let Some(page_id) = teks(&it["page_id"]) else { continue };
		let kena_pajak = angka(&it["has_tax"]) > 0.0;
		let dasar = angka(&it["price"]);
		let dengan_pajak = angka(&it["price_with_tax"]);
		// Harga yang dibayar sekolah: termasuk pajak bila barangnya kena pajak.
		let utama = if kena_pajak && dengan_pajak > 0.0 { dengan_pajak } else { dasar };
		if utama <= 0.0 {
			continue;
		}
		// Harga zona disimpan tanpa pajak seperti `price`; samakan dengan rasio pajak barang ini.
		let rasio = if dasar > 0.0 { utama / dasar } else { 1.0 };
		let mut semua = vec![utama];
		semua.extend(KOLOM_ZONA.iter().map(|k| angka(&it[*k]) * rasio).filter(|h| *h > 0.0));
		let min = semua.iter().copied().fold(f64::MAX, f64::min).round() as i64;
		let max = semua.iter().copied().fold(0.0, f64::max).round() as i64;
		let gambar = it["image_url"].as_array().and_then(|l| l.iter().find_map(teks));
		produk.push(SiplahProduk {
			id: stable_id(Platform::Masmedia, &page_id),
			toko_id,
			nama: teks(&it["name"]).unwrap_or_default(),
			harga: min,
			harga_max: (max > min).then_some(max),
			stok: angka(&it["stock"]) as i64,
			terjual: 0,
			gambar,
			kena_pajak,
			url: Some(format!("{SITE}/detail/{page_id}")),
		});
	}
	let total_halaman = angka(&res["meta_data"]["page_total"]) as u32;
	Halaman { ada_lagi: !items.is_empty() && n < total_halaman, produk }
}

#[cfg(test)]
mod tests {
	use super::*;

	fn json(s: &str) -> Value {
		serde_json::from_str(s).unwrap()
	}

	#[test]
	fn normalizes_vendor_ids() {
		assert_eq!(normalisasi_kode("7411").as_deref(), Some("3347_7411"));
		assert_eq!(normalisasi_kode("3347_7411").as_deref(), Some("3347_7411"));
		assert_eq!(normalisasi_kode("abc"), None);
	}

	#[test]
	fn parses_store_profile_without_phone() {
		let t = parse_toko(&json(include_str!("fixtures/masmedia_toko.json")), "3347_7411").unwrap();
		assert_eq!(t.nama, "Agni Jaya 88");
		assert_eq!(t.provinsi.as_deref(), Some("Banten"));
		assert_eq!(t.url, "https://siplahmasmedia.co.id/vendor/3347_7411");
		assert!(parse_toko(&json(r#"{"status":"ok","data":[]}"#), "3347_1").is_err());
	}

	#[test]
	fn parses_items_with_tax_and_zone_prices() {
		let h = parse_halaman(&json(include_str!("fixtures/masmedia_produk.json")), -3, 1);
		assert_eq!(h.produk.len(), 2);
		let a = &h.produk[0];
		// price 41.441 + PPN = price_with_tax 46.000; tidak berzona.
		assert_eq!((a.harga, a.harga_max, a.kena_pajak), (46_000, None, true));
		assert!(a.gambar.as_deref().unwrap().starts_with("https://system.siplahmasmedia.co.id/"));
		assert!(a.url.as_deref().unwrap().starts_with("https://siplahmasmedia.co.id/detail/3347_"));
		// Produk kedua di fixture diberi harga zona 50.000 dan 60.000 (sebelum pajak).
		let b = &h.produk[1];
		assert!(b.harga_max.is_some());
		assert!(b.harga_max.unwrap() > b.harga);
		assert_eq!(b.gambar, None);
	}
}
