//! SIPLah Intan Pariwara (siplah.intanonline.com): halaman toko HTML publik (Yii),
//! `/frontend/web/store/detail/index?slug=<slug>&page=N`, 15 produk per halaman (tetap).

use super::{html, http, kabupaten_provinsi, stable_id, toko_dasar, Halaman, Platform};
use crate::error::{AppError, AppResult};
use crate::repo::app::siplah::{SiplahProduk, SiplahToko};

const BASE: &str = "https://siplah.intanonline.com/frontend/web";
const KARTU: &str = "<div class=\"ps-product\">";

fn ambil(kode: &str, n: u32) -> AppResult<String> {
	http::get(&format!("{BASE}/store/detail/index?slug={kode}&page={n}&per-page=15"), "text/html")
}

pub fn toko(kode: &str) -> AppResult<SiplahToko> {
	parse_toko(&ambil(kode, 1)?, kode)
}

fn parse_toko(page: &str, kode: &str) -> AppResult<SiplahToko> {
	let info = page.find("ps-vendor-store").map(|i| &page[i..]).ok_or_else(|| {
		AppError::InvalidInput("Halaman toko Intan tidak ditemukan atau formatnya berubah.".into())
	})?;
	let nama = html::between(info, "<h4>", "</h4>").map(html::text).filter(|n| !n.is_empty()).unwrap_or_else(|| "Toko Intan".into());
	let mut t = toko_dasar(Platform::Intan, kode, nama, format!("{BASE}/store/{kode}"))?;
	t.slug = Some(kode.into());
	t.gambar = html::between(info, "<img src=\"", "\"").map(str::to_string);
	if let Some(alamat) = html::between(info, "<strong>Alamat</strong>", "</p>").map(html::text) {
		(t.kabupaten, t.provinsi) = kabupaten_provinsi(&alamat);
		t.alamat = Some(alamat);
	}
	t.total_produk = page
		.find("</strong> Produk ditemukan")
		.and_then(|end| page[..end].rfind("<strong>").map(|start| &page[start + 8..end]))
		.and_then(|n| n.trim().parse().ok())
		.unwrap_or(0);
	Ok(t)
}

pub fn halaman(kode: &str, toko_id: i64, n: u32) -> AppResult<Halaman> {
	Ok(parse_halaman(&ambil(kode, n)?, toko_id, n))
}

fn parse_halaman(page: &str, toko_id: i64, n: u32) -> Halaman {
	let mut produk = Vec::new();
	for card in page.split(KARTU).skip(1) {
		let Some(sku) = html::between(card, "detail?sku=", "\"") else { continue };
		let Some((harga, harga_max)) = html::between(card, "<div class=\"d-flex flex-column\">", "<small").and_then(|t| html::rentang_rupiah(&html::text(t))) else {
			continue;
		};
		let nama = card
			.find("ps-product__title")
			.and_then(|i| html::between(&card[i..], ">", "</a>"))
			.map(html::text)
			.unwrap_or_default();
		produk.push(SiplahProduk {
			id: stable_id(Platform::Intan, sku),
			toko_id,
			nama,
			harga,
			harga_max,
			stok: 0,
			terjual: 0,
			gambar: html::between(card, "<img src=\"", "\"").map(str::to_string),
			kena_pajak: card.contains("Ber-PPn"),
			url: Some(format!("{BASE}/product/detail?sku={sku}")),
		});
	}
	let ada_lagi = !produk.is_empty() && page.contains(&format!("page={}&amp;per-page", n + 1));
	Halaman { produk, ada_lagi }
}

#[cfg(test)]
mod tests {
	use super::*;

	const PAGE: &str = include_str!("fixtures/intan_toko.html");

	#[test]
	fn parses_store_profile() {
		let t = parse_toko(PAGE, "ptnusasaranapratama").unwrap();
		assert_eq!(t.nama, "PT Nusa Sarana Pratama");
		assert_eq!(t.kabupaten.as_deref(), Some("Kab. Kupang"));
		assert_eq!(t.provinsi.as_deref(), Some("Nusa Tenggara Timur"));
		assert_eq!(t.total_produk, 14198);
		assert_eq!(t.gambar.as_deref(), Some("https://siplah.intanonline.com/storage/web/store/logo_2026-10-02_105515.png"));
		assert_eq!(t.url, "https://siplah.intanonline.com/frontend/web/store/ptnusasaranapratama");
	}

	#[test]
	fn parses_product_cards() {
		let h = parse_halaman(PAGE, -7, 1);
		assert!(h.ada_lagi);
		assert_eq!(h.produk.len(), 2);
		let a = &h.produk[0];
		assert_eq!(a.nama, "Paket Sarana Keterampilan Menulis");
		assert_eq!((a.harga, a.harga_max), (130_000, None));
		assert!(a.kena_pajak);
		assert_eq!(a.url.as_deref(), Some("https://siplah.intanonline.com/frontend/web/product/detail?sku=031004041PEMTS_NSPG11"));
		assert!(a.gambar.as_deref().unwrap().starts_with("https://storage.googleapis.com/"));
		assert_eq!(h.produk[1].harga, 125_000);
		assert_ne!(a.id, h.produk[1].id);
	}

	#[test]
	fn last_page_has_no_next_link() {
		// Fixture adalah halaman 1 dengan tautan sampai halaman 10.
		assert!(parse_halaman(PAGE, -7, 9).ada_lagi);
		assert!(!parse_halaman(PAGE, -7, 10).ada_lagi);
	}
}
