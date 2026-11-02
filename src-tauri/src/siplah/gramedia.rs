//! SIPLah Gramedia (siplahgramedia.id, PT Mitra Edukasi Nusantara): halaman toko HTML publik
//! `/main/penyedia/<id>?page=N`, 20 produk per halaman. Harga berupa rentang harga zona.

use super::{html, http, kabupaten_provinsi, stable_id, toko_dasar, Halaman, Platform};
use crate::error::{AppError, AppResult};
use crate::repo::app::siplah::{SiplahProduk, SiplahToko};

const BASE: &str = "https://siplahgramedia.id";
const NAMA: &str = "nama-product\">";

fn url_toko(kode: &str) -> String {
	format!("{BASE}/main/penyedia/{kode}")
}

fn ambil(kode: &str, n: u32) -> AppResult<String> {
	http::get(&format!("{}?page={n}", url_toko(kode)), "text/html")
}

pub fn toko(kode: &str) -> AppResult<SiplahToko> {
	parse_toko(&ambil(kode, 1)?, kode)
}

fn parse_toko(page: &str, kode: &str) -> AppResult<SiplahToko> {
	let info = page.find("info-penyedia-right").map(|i| &page[i..]).ok_or_else(|| {
		AppError::InvalidInput("Halaman toko Gramedia tidak ditemukan atau formatnya berubah.".into())
	})?;
	let nama = html::between(info, "<h2>", "</h2>").map(html::text).filter(|n| !n.is_empty()).unwrap_or_else(|| "Toko Gramedia".into());
	let mut t = toko_dasar(Platform::Gramedia, kode, nama, url_toko(kode))?;
	if let Some(alamat) = html::between(info, "Alamat :", "</span>").map(html::text) {
		(t.kabupaten, t.provinsi) = kabupaten_provinsi(&alamat);
		t.alamat = Some(alamat);
	}
	Ok(t)
}

pub fn halaman(kode: &str, toko_id: i64, n: u32) -> AppResult<Halaman> {
	Ok(parse_halaman(&ambil(kode, n)?, toko_id, n))
}

fn parse_halaman(page: &str, toko_id: i64, n: u32) -> Halaman {
	let starts: Vec<usize> = page.match_indices(NAMA).map(|(i, _)| i).collect();
	let mut produk = Vec::with_capacity(starts.len());
	for (idx, &start) in starts.iter().enumerate() {
		let end = starts.get(idx + 1).copied().unwrap_or(page.len());
		let card = &page[start..end];
		let Some(detail) = html::between(card, "/main/detail/", "\"") else { continue };
		let Some((harga, harga_max)) = html::between(card, "<span class=\"text-dark\">Harga</span>", "</span>").and_then(|t| html::rentang_rupiah(&html::text(t))) else {
			continue;
		};
		let nama = html::between(card, NAMA, "</span>").map(html::text).unwrap_or_default();
		// Gambar ada di bagian atas kartu, sebelum nama produk.
		let mut from = start.saturating_sub(3000);
		while !page.is_char_boundary(from) {
			from -= 1;
		}
		let before = &page[from..start];
		let gambar = before.rfind("<img").and_then(|i| html::attr(&before[i..], "src")).map(str::to_string);
		produk.push(SiplahProduk {
			id: stable_id(Platform::Gramedia, detail),
			toko_id,
			nama,
			harga,
			harga_max,
			stok: 0,
			terjual: 0,
			gambar,
			kena_pajak: !card.contains("Bebas Pajak"),
			url: Some(format!("{BASE}/main/detail/{detail}")),
		});
	}
	let ada_lagi = !produk.is_empty() && page.contains(&format!("?page={}\"", n + 1));
	Halaman { produk, ada_lagi }
}

#[cfg(test)]
mod tests {
	use super::*;

	const PAGE: &str = include_str!("fixtures/gramedia_toko.html");
	const KODE: &str = "274a734f75894f359d958a2a4c1f76a8";

	#[test]
	fn parses_store_profile() {
		let t = parse_toko(PAGE, KODE).unwrap();
		assert_eq!(t.nama, "MUHTAR");
		assert_eq!(t.platform, "gramedia");
		assert_eq!(t.kabupaten.as_deref(), Some("KAB. MALUKU TENGAH"));
		assert_eq!(t.provinsi.as_deref(), Some("MALUKU"));
		assert_eq!(t.url, format!("https://siplahgramedia.id/main/penyedia/{KODE}"));
		assert!(t.id < 0);
	}

	#[test]
	fn parses_product_cards_with_zone_price_range() {
		let h = parse_halaman(PAGE, -5, 1);
		assert!(h.ada_lagi);
		assert_eq!(h.produk.len(), 2);
		let a = &h.produk[0];
		assert_eq!(a.nama, "BUKU GURU PRAKARYA BUDI DAYA KURIKULUM MERDEKA KELAS 9");
		assert_eq!((a.harga, a.harga_max), (86_900, Some(181_100)));
		assert!(!a.kena_pajak);
		assert_eq!(a.url.as_deref(), Some("https://siplahgramedia.id/main/detail/00a9b4a438cd4300b43626baa89d5cf3"));
		assert_eq!(a.gambar.as_deref(), Some("https://siplahgramedia.id/assets/image/foto-produk/muhtar-00147_1759620509_1.png"));
		let b = &h.produk[1];
		assert_eq!((b.harga, b.harga_max), (12_200, Some(25_300)));
		assert!(b.kena_pajak);
		assert_ne!(a.id, b.id);
	}

	#[test]
	fn last_page_has_no_next_link() {
		assert!(!parse_halaman(PAGE, -5, 2).ada_lagi);
		assert!(parse_halaman("<html>Belum ada produk</html>", -5, 1).produk.is_empty());
	}

	#[test]
	fn rejects_non_store_page() {
		assert!(parse_toko("<html>404</html>", KODE).is_err());
	}
}
