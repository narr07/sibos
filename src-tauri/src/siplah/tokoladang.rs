//! Toko Ladang (siplah.tokoladang.co.id): API JSON publik `api-siplah.tokoladang.co.id/public`.

use serde::Deserialize;

use super::{http, toko_dasar, Halaman, Platform};
use crate::error::{AppError, AppResult};
use crate::repo::app::siplah::{SiplahProduk, SiplahToko};

const API: &str = "https://api-siplah.tokoladang.co.id/public";
const PER_HALAMAN: usize = 30;

#[derive(Debug, Deserialize)]
struct MerchantRaw {
	id: i64,
	name: String,
	slug: Option<String>,
	address: Option<String>,
	postal_code: Option<String>,
	slogan: Option<String>,
	#[serde(rename = "type")]
	tipe: Option<String>,
	image: Option<String>,
	province: Option<NamedRaw>,
	city: Option<NamedRaw>,
	district: Option<NamedRaw>,
}

#[derive(Debug, Deserialize)]
struct NamedRaw {
	name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ProductRaw {
	id: i64,
	name: String,
	price: Option<f64>,
	qty_available: Option<i64>,
	qty_sell: Option<i64>,
	image: Option<String>,
	taxed: Option<bool>,
}

/// ID merchant dari link (mis. ".../m/kpr-rajagaluh.92186" -> "92186").
pub fn kode_dari_link(input: &str) -> Option<String> {
	let path = input.trim().split('?').next()?.trim_end_matches('/');
	let last = path.rsplit('/').next()?;
	let id = match last.rfind('.') {
		Some(pos) if last[pos + 1..].chars().all(|c| c.is_ascii_digit()) => &last[pos + 1..],
		_ => last,
	};
	let digits: String = id.chars().filter(char::is_ascii_digit).collect();
	(!digits.is_empty()).then_some(digits)
}

pub fn normalize_image_url(raw: Option<String>) -> Option<String> {
	let img = raw?.trim().to_string();
	if img.is_empty() {
		return None;
	}
	if img.starts_with("http://") || img.starts_with("https://") {
		Some(img.replace("worker-siplah.tokoladang.co.id", "siplah-oss.tokoladang.co.id"))
	} else {
		Some(format!("https://siplah-oss.tokoladang.co.id/{}", img.trim_start_matches('/')))
	}
}

pub fn toko(kode: &str) -> AppResult<SiplahToko> {
	let body = http::get(&format!("{API}/merchants/{kode}"), "application/json")?;
	let raw: MerchantRaw = serde_json::from_str(&body)
		.map_err(|e| AppError::InvalidInput(format!("Toko Toko Ladang ID {kode} tidak ditemukan atau respons tidak valid: {e}")))?;
	let url = format!("https://siplah.tokoladang.co.id/m/{}.{}", raw.slug.as_deref().unwrap_or("toko"), raw.id);
	let mut t = toko_dasar(Platform::TokoLadang, &raw.id.to_string(), raw.name, url)?;
	t.slug = raw.slug;
	t.tipe = raw.tipe;
	t.slogan = raw.slogan;
	t.alamat = raw.address;
	t.provinsi = raw.province.and_then(|p| p.name);
	t.kabupaten = raw.city.and_then(|c| c.name);
	t.kecamatan = raw.district.and_then(|d| d.name);
	t.kodepos = raw.postal_code;
	t.gambar = normalize_image_url(raw.image);
	Ok(t)
}

pub fn halaman(kode: &str, toko_id: i64, n: u32) -> AppResult<Halaman> {
	let body = http::get(&format!("{API}/products/merchant/{kode}?page={n}&sort="), "application/json")?;
	parse_halaman(&body, toko_id).map_err(|e| AppError::InvalidInput(format!("Gagal membaca produk Toko Ladang halaman {n}: {e}")))
}

fn parse_halaman(body: &str, toko_id: i64) -> Result<Halaman, serde_json::Error> {
	let items: Vec<ProductRaw> = serde_json::from_str(body)?;
	let ada_lagi = items.len() >= PER_HALAMAN;
	let produk = items
		.into_iter()
		.map(|it| SiplahProduk {
			url: Some(format!("https://siplah.tokoladang.co.id/p/produk.{}", it.id)),
			id: it.id,
			toko_id,
			nama: it.name.trim().to_string(),
			harga: it.price.map(|p| p.round() as i64).unwrap_or(0),
			harga_max: None,
			stok: it.qty_available.unwrap_or(0),
			terjual: it.qty_sell.unwrap_or(0),
			gambar: normalize_image_url(it.image),
			kena_pajak: it.taxed.unwrap_or(true),
		})
		.collect();
	Ok(Halaman { produk, ada_lagi })
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn extracts_id_from_url_and_slug() {
		assert_eq!(kode_dari_link("https://siplah.tokoladang.co.id/m/kpr-rajagaluh.92186"), Some("92186".into()));
		assert_eq!(kode_dari_link("kpr-rajagaluh.92186/"), Some("92186".into()));
		assert_eq!(kode_dari_link("https://siplah.tokoladang.co.id/m/toko"), None);
	}

	#[test]
	fn parses_product_page() {
		let body = r#"[{"id":11,"name":" Pensil 2B ","price":2500.4,"qty_available":10,"qty_sell":3,"image":"/img/a.png","taxed":false}]"#;
		let h = parse_halaman(body, 92186).unwrap();
		assert!(!h.ada_lagi);
		let p = &h.produk[0];
		assert_eq!((p.id, p.nama.as_str(), p.harga, p.harga_max, p.kena_pajak), (11, "Pensil 2B", 2500, None, false));
		assert_eq!(p.gambar.as_deref(), Some("https://siplah-oss.tokoladang.co.id/img/a.png"));
		assert_eq!(p.url.as_deref(), Some("https://siplah.tokoladang.co.id/p/produk.11"));
	}
}
