//! SIPLah Temprina (siplah.temprina.co.id, PT Temprina Media Grafika): API JSON publik
//! `/api/...` yang juga dipakai situsnya untuk pengunjung tanpa login (token Bearer kosong).
//! Produk toko: `produk/cari_produk`, 20 per halaman. Harga bisa angka atau `{min, max}` (zona).

use serde_json::Value;

use super::{http, stable_id, toko_dasar, Halaman, Platform};
use crate::error::{AppError, AppResult};
use crate::repo::app::siplah::{SiplahProduk, SiplahToko};

const BASE: &str = "https://siplah.temprina.co.id";
// Situsnya sendiri mengirim header ini dengan token kosong untuk pengunjung yang belum login.
const AUTH: &str = "Authorization: Bearer ";

fn api(path: &str, fields: &[(&str, &str)]) -> AppResult<Value> {
	let body = http::post_form(&format!("{BASE}/api/{path}"), fields, &[AUTH])?;
	serde_json::from_str(&body).map_err(|e| AppError::InvalidInput(format!("Respons Temprina tidak valid: {e}")))
}

fn teks(v: &Value) -> Option<String> {
	match v {
		Value::String(s) if !s.trim().is_empty() => Some(s.trim().to_string()),
		Value::Number(n) => Some(n.to_string()),
		_ => None,
	}
}

fn angka(v: &Value) -> Option<i64> {
	match v {
		Value::Number(n) => n.as_f64().map(|f| f.round() as i64),
		Value::String(s) => s.trim().parse::<f64>().ok().map(|f| f.round() as i64),
		_ => None,
	}
}

pub fn toko(kode: &str) -> AppResult<SiplahToko> {
	parse_toko(&api("mitra_data/detail", &[("slug", kode), ("mkt_kode", "")])?, kode)
}

fn parse_toko(res: &Value, kode: &str) -> AppResult<SiplahToko> {
	let data = res.get("data").filter(|d| d.is_object() && res["status"] == true).ok_or_else(|| {
		AppError::InvalidInput(format!("Toko Temprina \"{kode}\" tidak ditemukan."))
	})?;
	let nama = teks(&data["nama"]).unwrap_or_else(|| "Toko Temprina".into());
	let mut t = toko_dasar(Platform::Temprina, kode, nama, format!("{BASE}/store/{kode}"))?;
	t.slug = Some(kode.into());
	t.provinsi = teks(&data["nama_provinsi"]);
	t.kabupaten = teks(&data["nama_kabupaten"]);
	t.kecamatan = teks(&data["nama_kecamatan"]);
	t.gambar = teks(&data["foto"]);
	t.total_produk = angka(&data["total_produk"]).unwrap_or(0);
	Ok(t)
}

pub fn halaman(kode: &str, toko_id: i64, n: u32) -> AppResult<Halaman> {
	let page = n.to_string();
	let res = api(
		"produk/cari_produk",
		&[("cari", ""), ("kategori", ""), ("slug", kode), ("page", &page), ("kelas", ""), ("store", "1"), ("paket", "0")],
	)?;
	Ok(parse_halaman(&res, kode, toko_id, n))
}

fn parse_halaman(res: &Value, kode: &str, toko_id: i64, n: u32) -> Halaman {
	let items = res["data"].as_array().cloned().unwrap_or_default();
	let mut produk = Vec::with_capacity(items.len());
	for it in &items {
		let Some(id) = teks(&it["id"]) else { continue };
		let (harga, harga_max) = match &it["harga"] {
			Value::Object(z) => {
				let min = z.get("min").and_then(angka).unwrap_or(0);
				let max = z.get("max").and_then(angka).filter(|max| *max > min);
				(min, max)
			}
			v => (angka(v).unwrap_or(0), None),
		};
		// Harga 0 = "Tidak Tayang" di situsnya.
		if harga <= 0 {
			continue;
		}
		let gambar = match &it["foto"] {
			Value::Array(list) => list.iter().find_map(teks),
			v => teks(v),
		};
		let slug = teks(&it["slug"]).unwrap_or_else(|| "produk".into());
		produk.push(SiplahProduk {
			id: stable_id(Platform::Temprina, &id),
			toko_id,
			nama: teks(&it["nama"]).unwrap_or_default(),
			harga,
			harga_max,
			stok: angka(&it["stok"]).unwrap_or(0),
			terjual: angka(&it["terjual"]).unwrap_or(0),
			gambar,
			kena_pajak: teks(&it["ppn"]).as_deref() == Some("1"),
			url: Some(format!("{BASE}/product/{kode}/{id}/{slug}")),
		});
	}
	let total_halaman = angka(&res["pagination"]["jml_halaman"]).unwrap_or(0);
	Halaman { ada_lagi: !items.is_empty() && i64::from(n) < total_halaman, produk }
}

#[cfg(test)]
mod tests {
	use super::*;

	fn json(s: &str) -> Value {
		serde_json::from_str(s).unwrap()
	}

	#[test]
	fn parses_store_profile() {
		let t = parse_toko(&json(include_str!("fixtures/temprina_toko.json")), "cv-futo-indonesia").unwrap();
		assert_eq!(t.nama, "CV FUTO INDONESIA");
		assert_eq!(t.provinsi.as_deref(), Some("Sumatera Utara"));
		assert_eq!(t.kabupaten.as_deref(), Some("Medan"));
		assert_eq!(t.url, "https://siplah.temprina.co.id/store/cv-futo-indonesia");
		assert!(parse_toko(&json(r#"{"status":false,"message":"tidak ada","data":null}"#), "x").is_err());
	}

	#[test]
	fn parses_products_with_zone_prices() {
		let h = parse_halaman(&json(include_str!("fixtures/temprina_produk.json")), "cv-futo-indonesia", -9, 1);
		assert!(h.ada_lagi);
		assert_eq!(h.produk.len(), 2);
		let a = &h.produk[0];
		assert_eq!((a.harga, a.harga_max, a.kena_pajak), (65_000, None, false));
		assert!(a.gambar.as_deref().unwrap().starts_with("https://files.temprina.co.id/"));
		assert!(a.url.as_deref().unwrap().starts_with("https://siplah.temprina.co.id/product/cv-futo-indonesia/171626/"));
		let b = &h.produk[1];
		assert_eq!((b.harga, b.harga_max, b.kena_pajak), (65_000, Some(81_000), true));
		assert_eq!(b.gambar, None);
	}

	#[test]
	fn hidden_products_and_last_page() {
		let res = json(r#"{"status":true,"data":[{"id":"1","nama":"X","harga":0}],"pagination":{"jml_halaman":1}}"#);
		let h = parse_halaman(&res, "t", -1, 1);
		assert!(h.produk.is_empty());
		assert!(!h.ada_lagi);
	}
}
