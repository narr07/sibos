//! Fitur belanja SIPLah dan pencocokan harga barang dengan RKAS / standar harga.
//! Scrape profil dan produk toko dari SIPLah Toko Ladang, simpan di sibos.db,
//! dan lakukan pencocokan nama & harga.

use std::collections::HashMap;
use std::os::windows::process::CommandExt;
use std::path::Path;

use rust_xlsxwriter::{Color, Format, FormatBorder, Workbook};
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::error::{validate_year, AppError, AppResult};
use crate::repo::app::siplah::{SiplahProduk, SiplahToko};
use crate::repo::arkas::bku;
use crate::repo::arkas::rkas::{self, RkasItem, StandarHarga};
use crate::state::AppState;

const CREATE_NO_WINDOW: u32 = 0x0800_0000;

const STOP_WORDS: &[&str] = &[
	"dan", "atau", "untuk", "dari", "pada", "ke", "di", "dengan",
	"yang", "ini", "itu", "isi", "dus", "pcs", "pack", "lembar",
	"buah", "rim", "kotak", "botol", "tahun", "anggaran", "belanja",
	"pengadaan", "biaya", "kegiatan", "sub", "keg",
];

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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SiplahMatchCandidate {
	pub produk: SiplahProduk,
	pub score: u32,
	pub matched_words: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SiplahMatchItem {
	pub source_id: String,
	pub nama_rkas: String,
	pub satuan: Option<String>,
	pub volume: f64,
	pub harga_rkas: i64,
	pub total_rkas: i64,
	pub realisasi_rkas: i64,
	/// "belum" | "sebagian" | "lunas"
	pub status_belanja: String,
	pub kode_rekening: Option<String>,
	pub sumber_dana: Option<String>,
	pub matched_product: Option<SiplahProduk>,
	pub score: u32,
	/// "sesuai" | "lebih_murah" | "lebih_mahal" | "tidak_cocok"
	pub status: String,
	pub selisih: i64,
	pub persen_selisih: f64,
	pub candidates: Vec<SiplahMatchCandidate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SiplahMatchSummary {
	pub total_items: usize,
	pub total_cocok: usize,
	pub total_tidak_cocok: usize,
	pub total_sesuai: usize,
	pub total_lebih_murah: usize,
	pub total_lebih_mahal: usize,
	pub total_anggaran_rkas: i64,
	pub total_estimasi_toko: i64,
	pub total_hemat: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SiplahMatchReport {
	pub toko: SiplahToko,
	pub source_mode: String,
	pub year: i32,
	pub month: Option<u32>,
	pub fund_id: Option<i64>,
	pub only_unspent: bool,
	pub summary: SiplahMatchSummary,
	pub items: Vec<SiplahMatchItem>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SiplahSyncResult {
	pub toko_id: i64,
	pub total: i64,
	pub terakhir_sync: String,
}

/// Ekstrak ID toko SIPLah dari URL atau string (mis. "kpr-rajagaluh.92186" -> 92186).
pub fn extract_toko_id(input: &str) -> Option<i64> {
	let trimmed = input.trim();
	if let Ok(id) = trimmed.parse::<i64>() {
		return Some(id);
	}
	let path_part = trimmed.split('?').next().unwrap_or(trimmed).trim_end_matches('/');
	let last_seg = path_part.rsplit('/').next().unwrap_or(path_part);
	if let Some(pos) = last_seg.rfind('.') {
		if let Ok(id) = last_seg[pos + 1..].parse::<i64>() {
			return Some(id);
		}
	}
	let re_digits: String = last_seg.chars().filter(|c| c.is_ascii_digit()).collect();
	re_digits.parse::<i64>().ok()
}

fn fetch_json_with_curl(url: &str) -> AppResult<String> {
	let mut cmd = std::process::Command::new("C:\\Windows\\System32\\curl.exe");
	cmd.creation_flags(CREATE_NO_WINDOW);
	cmd.args([
		"-s",
		"-L",
		"--max-time",
		"30",
		"-H",
		"Accept: application/json, text/plain, */*",
		"-H",
		"User-Agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
		url,
	]);
	let output = cmd.output().map_err(|e| AppError::Io(std::io::Error::other(format!("gagal menjalankan curl: {e}"))))?;
	if !output.status.success() {
		return Err(AppError::InvalidInput(format!("Gagal menghubungi server SIPLah (exit code {:?})", output.status.code())));
	}
	let body = String::from_utf8_lossy(&output.stdout).to_string();
	Ok(body)
}

fn normalize_image_url(raw: Option<String>) -> Option<String> {
	let img = raw?.trim().to_string();
	if img.is_empty() {
		return None;
	}
	if img.starts_with("http://") || img.starts_with("https://") {
		if img.contains("worker-siplah.tokoladang.co.id") {
			Some(img.replace("worker-siplah.tokoladang.co.id", "siplah-oss.tokoladang.co.id"))
		} else {
			Some(img)
		}
	} else {
		Some(format!("https://siplah-oss.tokoladang.co.id/{}", img.trim_start_matches('/')))
	}
}

#[tauri::command(async)]
pub fn siplah_inspect_url(url: String) -> AppResult<SiplahToko> {
	let id = extract_toko_id(&url).ok_or_else(|| AppError::InvalidInput("Link toko SIPLah tidak valid. Contoh: https://siplah.tokoladang.co.id/m/kpr-rajagaluh.92186".into()))?;
	let endpoint = format!("https://api-siplah.tokoladang.co.id/public/merchants/{id}");
	let json_str = fetch_json_with_curl(&endpoint)?;
	let raw: MerchantRaw = serde_json::from_str(&json_str)
		.map_err(|e| AppError::InvalidInput(format!("Toko SIPLah ID {id} tidak ditemukan atau respons tidak valid: {e}")))?;

	let formatted_url = if url.trim().starts_with("http") {
		url.trim().to_string()
	} else {
		format!("https://siplah.tokoladang.co.id/m/{}.{}", raw.slug.as_deref().unwrap_or("toko"), raw.id)
	};

	let toko = SiplahToko {
		id: raw.id,
		nama: raw.name,
		slug: raw.slug,
		url: formatted_url,
		tipe: raw.tipe,
		slogan: raw.slogan,
		alamat: raw.address,
		provinsi: raw.province.and_then(|p| p.name),
		kabupaten: raw.city.and_then(|c| c.name),
		kecamatan: raw.district.and_then(|d| d.name),
		kodepos: raw.postal_code,
		gambar: normalize_image_url(raw.image),
		total_produk: 0,
		terakhir_sync: None,
		created_at: String::new(),
		updated_at: String::new(),
	};
	Ok(toko)
}

#[tauri::command]
pub fn siplah_toko_list(state: State<'_, AppState>) -> AppResult<Vec<SiplahToko>> {
	state.app_db().siplah_toko_list()
}

#[tauri::command]
pub fn siplah_toko_save(state: State<'_, AppState>, toko: SiplahToko) -> AppResult<()> {
	if toko.nama.trim().is_empty() {
		return Err(AppError::InvalidInput("Nama toko wajib diisi".into()));
	}
	state.app_db().siplah_toko_save(&toko)
}

#[tauri::command]
pub fn siplah_toko_delete(state: State<'_, AppState>, id: i64) -> AppResult<()> {
	state.app_db().siplah_toko_delete(id)
}

#[tauri::command]
pub fn siplah_produk_list(state: State<'_, AppState>, toko_id: i64, keyword: Option<String>) -> AppResult<Vec<SiplahProduk>> {
	state.app_db().siplah_produk_list(toko_id, keyword.as_deref())
}

#[tauri::command(async)]
pub fn siplah_sync_products(state: State<'_, AppState>, toko_id: i64) -> AppResult<SiplahSyncResult> {
	// Pastikan profil toko diperbarui
	let merchant_endpoint = format!("https://api-siplah.tokoladang.co.id/public/merchants/{toko_id}");
	if let Ok(json_str) = fetch_json_with_curl(&merchant_endpoint) {
		if let Ok(raw) = serde_json::from_str::<MerchantRaw>(&json_str) {
			let _ = state.app_db().siplah_toko_save(&SiplahToko {
				id: raw.id,
				nama: raw.name,
				slug: raw.slug.clone(),
				url: format!("https://siplah.tokoladang.co.id/m/{}.{}", raw.slug.as_deref().unwrap_or("toko"), raw.id),
				tipe: raw.tipe,
				slogan: raw.slogan,
				alamat: raw.address,
				provinsi: raw.province.and_then(|p| p.name),
				kabupaten: raw.city.and_then(|c| c.name),
				kecamatan: raw.district.and_then(|d| d.name),
				kodepos: raw.postal_code,
				gambar: normalize_image_url(raw.image),
				total_produk: 0,
				terakhir_sync: None,
				created_at: String::new(),
				updated_at: String::new(),
			});
		}
	}

	let mut all_products: Vec<SiplahProduk> = Vec::new();
	// Ambil semua halaman produk sampai habis
	for page in 1..=100 {
		let endpoint = format!("https://api-siplah.tokoladang.co.id/public/products/merchant/{toko_id}?page={page}&sort=");
		let json_str = fetch_json_with_curl(&endpoint)?;
		let items: Vec<ProductRaw> = serde_json::from_str(&json_str)
			.map_err(|e| AppError::InvalidInput(format!("Gagal membaca data produk halaman {page}: {e}")))?;

		if items.is_empty() {
			break;
		}

		let count = items.len();
		for it in items {
			let harga = it.price.map(|p| p.round() as i64).unwrap_or(0);
			all_products.push(SiplahProduk {
				id: it.id,
				toko_id,
				nama: it.name.trim().to_string(),
				harga,
				stok: it.qty_available.unwrap_or(0),
				terjual: it.qty_sell.unwrap_or(0),
				gambar: normalize_image_url(it.image),
				kena_pajak: it.taxed.unwrap_or(true),
			});
		}

		if count < 30 {
			break;
		}
	}

	let total = state.app_db().siplah_produk_save_batch(toko_id, &all_products)?;
	let (y, m, d, hh, mm, ss) = crate::util::now_wib();
	let synced_at = format!("{y:04}-{m:02}-{d:02} {hh:02}:{mm:02}:{ss:02}");

	Ok(SiplahSyncResult {
		toko_id,
		total,
		terakhir_sync: synced_at,
	})
}

fn tokenize(text: &str) -> Vec<String> {
	let lower = text.to_lowercase();
	let mut words = Vec::new();
	let mut current = String::new();
	for c in lower.chars() {
		if c.is_alphanumeric() {
			current.push(c);
		} else if !current.is_empty() {
			if current.chars().count() >= 2 {
				words.push(current.clone());
			}
			current.clear();
		}
	}
	if current.chars().count() >= 2 {
		words.push(current);
	}
	words
}

fn normalize_text(text: &str) -> String {
	let words = tokenize(text);
	words.join(" ")
}

fn score_match(src_words: &[String], prod_words: &[String], src_norm: &str, prod_norm: &str) -> (u32, Vec<String>) {
	if src_norm == prod_norm {
		return (100, src_words.to_vec());
	}
	if !src_norm.is_empty() && !prod_norm.is_empty() && (src_norm.contains(prod_norm) || prod_norm.contains(src_norm)) {
		let matched: Vec<String> = src_words.iter().filter(|w| prod_words.contains(w)).cloned().collect();
		return (90, matched);
	}

	let mut matched = Vec::new();
	let mut weight: u32 = 0;

	for w in src_words {
		if prod_words.contains(w) {
			matched.push(w.clone());
			if STOP_WORDS.contains(&w.as_str()) {
				weight += 5;
			} else {
				let len = w.chars().count() as u32;
				weight += 10 + len * 2;
			}
		}
	}

	if matched.is_empty() {
		return (0, Vec::new());
	}

	// Aturan: Asal 1 kata sama bisa diasumsikan nama produknya itu
	let score = (40 + weight).min(85);
	(score, matched)
}

#[tauri::command]
pub fn siplah_match_prices(
	state: State<'_, AppState>,
	year: i32,
	toko_id: i64,
	source_mode: String,
	month: Option<u32>,
	fund: Option<i64>,
	only_unspent: Option<bool>,
) -> AppResult<SiplahMatchReport> {
	let year = validate_year(year)?;
	let fund = fund.filter(|f| *f != 0);
	let month = month.filter(|m| (1..=12).contains(m));
	let only_unspent_flag = only_unspent.unwrap_or(true);
	let toko = state
		.app_db()
		.siplah_toko_get(toko_id)?
		.ok_or_else(|| AppError::InvalidInput(format!("Toko dengan ID {toko_id} belum tersimpan.")))?;

	let store_products = state.app_db().siplah_produk_list(toko_id, None)?;
	let store_products_tokenized: Vec<(&SiplahProduk, Vec<String>, String)> = store_products
		.iter()
		.map(|p| (p, tokenize(&p.nama), normalize_text(&p.nama)))
		.collect();

	struct ItemToMatch {
		id: String,
		nama: String,
		satuan: Option<String>,
		volume: f64,
		harga: i64,
		total: i64,
		realisasi: i64,
		status_belanja: String,
		kode_rekening: Option<String>,
		sumber_dana: Option<String>,
	}

	let items_to_match: Vec<ItemToMatch> = match source_mode.as_str() {
		"standar_harga" => {
			let sh_list: Vec<StandarHarga> = state.with_arkas(|db| rkas::standar_harga_search(db, year, "", 2000))?;
			sh_list
				.into_iter()
				.enumerate()
				.map(|(idx, s)| {
					let h = s.harga.unwrap_or(0);
					ItemToMatch {
						id: format!("sh-{idx}"),
						nama: s.nama_barang,
						satuan: s.satuan,
						volume: 1.0,
						harga: h,
						total: h,
						realisasi: 0,
						status_belanja: "belum".into(),
						kode_rekening: s.kode_rekening,
						sumber_dana: None,
					}
				})
				.collect()
		}
		_ => {
			let (rkas_items, kas_rows) = state.with_arkas(|db| {
				let items = rkas::rkas_items(db, year)?;
				let krows = bku::kas_rows(db, year)?;
				Ok((items, krows))
			})?;

			let mut spent_by_month: HashMap<String, [i64; 12]> = HashMap::new();
			let mut spent_total: HashMap<String, i64> = HashMap::new();
			for r in kas_rows.iter().filter(|k| matches!(k.id_ref_bku, 4 | 15 | 24 | 35)) {
				if let Some(id) = &r.id_rapbs {
					let m = r.tanggal.get(5..7).and_then(|s| s.parse::<usize>().ok()).unwrap_or(1).clamp(1, 12);
					spent_by_month.entry(id.clone()).or_insert([0; 12])[m - 1] += r.saldo;
					*spent_total.entry(id.clone()).or_insert(0) += r.saldo;
				}
			}

			rkas_items
				.into_iter()
				.filter(|r| fund.is_none_or(|f| f == r.fund_id))
				.filter_map(|r| {
					if let Some(m) = month {
						let idx = (m - 1) as usize;
						let vol_m = r.volume_bulan[idx];
						let jml_m = r.bulan[idx];
						if jml_m > 0 || vol_m > 0.0 {
							let spent_m = spent_by_month.get(&r.id_rapbs).map(|s| s[idx]).unwrap_or(0);
							let sisa_m = jml_m.saturating_sub(spent_m);
							let status_belanja = if spent_m >= jml_m && jml_m > 0 {
								"lunas"
							} else if spent_m > 0 {
								"sebagian"
							} else {
								"belum"
							};

							if only_unspent_flag && status_belanja == "lunas" {
								None
							} else {
								let sisa_volume = if status_belanja == "sebagian" && jml_m > 0 && r.harga_satuan > 0 {
									(sisa_m as f64 / r.harga_satuan as f64).max(0.1)
								} else if vol_m > 0.0 {
									vol_m
								} else {
									r.volume
								};

								Some(ItemToMatch {
									id: r.id_rapbs,
									nama: r.uraian,
									satuan: r.satuan,
									volume: sisa_volume,
									harga: r.harga_satuan,
									total: if only_unspent_flag && status_belanja == "sebagian" { sisa_m } else { jml_m },
									realisasi: spent_m,
									status_belanja: status_belanja.to_string(),
									kode_rekening: r.kode_rekening,
									sumber_dana: Some(r.fund_name),
								})
							}
						} else {
							None
						}
					} else {
						let total_spent = spent_total.get(&r.id_rapbs).copied().unwrap_or(0);
						let sisa_total = r.jumlah.saturating_sub(total_spent);
						let status_belanja = if total_spent >= r.jumlah && r.jumlah > 0 {
							"lunas"
						} else if total_spent > 0 {
							"sebagian"
						} else {
							"belum"
						};

						if only_unspent_flag && status_belanja == "lunas" {
							None
						} else {
							Some(ItemToMatch {
								id: r.id_rapbs,
								nama: r.uraian,
								satuan: r.satuan,
								volume: if status_belanja == "sebagian" && r.harga_satuan > 0 {
									(sisa_total as f64 / r.harga_satuan as f64).max(0.1)
								} else {
									r.volume
								},
								harga: r.harga_satuan,
								total: if only_unspent_flag && status_belanja == "sebagian" { sisa_total } else { r.jumlah },
								realisasi: total_spent,
								status_belanja: status_belanja.to_string(),
								kode_rekening: r.kode_rekening,
								sumber_dana: Some(r.fund_name),
							})
						}
					}
				})
				.collect()
		}
	};

	let mut match_items: Vec<SiplahMatchItem> = Vec::with_capacity(items_to_match.len());

	for it in items_to_match {
		let src_words = tokenize(&it.nama);
		let src_norm = normalize_text(&it.nama);

		let mut candidate_list: Vec<SiplahMatchCandidate> = Vec::new();

		for (prod, prod_words, prod_norm) in &store_products_tokenized {
			let (score, matched_words) = score_match(&src_words, prod_words, &src_norm, prod_norm);
			if score > 0 {
				candidate_list.push(SiplahMatchCandidate {
					produk: (*prod).clone(),
					score,
					matched_words,
				});
			}
		}

		candidate_list.sort_by(|a, b| b.score.cmp(&a.score));
		let best_match = candidate_list.first().cloned();
		let top_candidates: Vec<SiplahMatchCandidate> = candidate_list.into_iter().take(5).collect();

		let (matched_product, score, status, selisih, persen_selisih) = match best_match {
			Some(best) => {
				let diff = best.produk.harga - it.harga;
				let pct = if it.harga > 0 {
					(diff as f64 / it.harga as f64) * 100.0
				} else {
					0.0
				};
				let st = if diff == 0 {
					"sesuai".to_string()
				} else if diff < 0 {
					"lebih_murah".to_string()
				} else {
					"lebih_mahal".to_string()
				};
				(Some(best.produk), best.score, st, diff, pct)
			}
			None => (None, 0, "tidak_cocok".to_string(), 0, 0.0),
		};

		match_items.push(SiplahMatchItem {
			source_id: it.id,
			nama_rkas: it.nama,
			satuan: it.satuan,
			volume: it.volume,
			harga_rkas: it.harga,
			total_rkas: it.total,
			realisasi_rkas: it.realisasi,
			status_belanja: it.status_belanja,
			kode_rekening: it.kode_rekening,
			sumber_dana: it.sumber_dana,
			matched_product,
			score,
			status,
			selisih,
			persen_selisih,
			candidates: top_candidates,
		});
	}

	let total_items = match_items.len();
	let total_cocok = match_items.iter().filter(|i| i.matched_product.is_some()).count();
	let total_tidak_cocok = total_items - total_cocok;
	let total_sesuai = match_items.iter().filter(|i| i.status == "sesuai").count();
	let total_lebih_murah = match_items.iter().filter(|i| i.status == "lebih_murah").count();
	let total_lebih_mahal = match_items.iter().filter(|i| i.status == "lebih_mahal").count();

	let mut total_anggaran_rkas: i64 = 0;
	let mut total_estimasi_toko: i64 = 0;

	for it in &match_items {
		total_anggaran_rkas += it.total_rkas;
		if let Some(p) = &it.matched_product {
			let vol = if it.volume > 0.0 { it.volume } else { 1.0 };
			total_estimasi_toko += (p.harga as f64 * vol).round() as i64;
		}
	}

	let total_hemat = total_anggaran_rkas - total_estimasi_toko;

	let summary = SiplahMatchSummary {
		total_items,
		total_cocok,
		total_tidak_cocok,
		total_sesuai,
		total_lebih_murah,
		total_lebih_mahal,
		total_anggaran_rkas,
		total_estimasi_toko,
		total_hemat,
	};

	Ok(SiplahMatchReport {
		toko,
		source_mode,
		year,
		month,
		fund_id: fund,
		only_unspent: only_unspent_flag,
		summary,
		items: match_items,
	})
}

#[tauri::command(async)]
pub fn siplah_export_match_xlsx(
	state: State<'_, AppState>,
	year: i32,
	toko_id: i64,
	source_mode: String,
	month: Option<u32>,
	fund: Option<i64>,
	only_unspent: Option<bool>,
	path: String,
) -> AppResult<String> {
	let report = siplah_match_prices(state, year, toko_id, source_mode, month, fund, only_unspent)?;
	let mut workbook = Workbook::new();
	let worksheet = workbook.add_worksheet();
	worksheet.set_name("Pencocokan Harga SIPLah")?;

	let title_format = Format::new().set_bold().set_font_size(14);
	let subtitle_format = Format::new().set_italic().set_font_size(10).set_font_color(Color::RGB(0x666666));
	let header_format = Format::new()
		.set_bold()
		.set_background_color(Color::RGB(0xEAEEF3))
		.set_border(FormatBorder::Thin);

	let border_format = Format::new().set_border(FormatBorder::Thin);
	let num_format = Format::new()
		.set_border(FormatBorder::Thin)
		.set_num_format("#,##0");

	let hijau_format = Format::new()
		.set_border(FormatBorder::Thin)
		.set_font_color(Color::RGB(0x166534))
		.set_num_format("#,##0");

	let merah_format = Format::new()
		.set_border(FormatBorder::Thin)
		.set_font_color(Color::RGB(0x991B1B))
		.set_num_format("#,##0");

	worksheet.write_string_with_format(0, 0, "LAPORAN PENCOCOKAN HARGA BELANJA SIPLAH", &title_format)?;
	const BULAN_NAMA: [&str; 12] = [
		"Januari", "Februari", "Maret", "April", "Mei", "Juni",
		"Juli", "Agustus", "September", "Oktober", "November", "Desember",
	];
	let sub = match report.month {
		Some(m) if (1..=12).contains(&m) => format!(
			"Toko: {} | Tahun Anggaran: {} | Bulan: {} | Total Item: {}",
			report.toko.nama, report.year, BULAN_NAMA[(m - 1) as usize], report.items.len()
		),
		_ => format!(
			"Toko: {} | Tahun Anggaran: {} | Total Item: {}",
			report.toko.nama, report.year, report.items.len()
		),
	};
	worksheet.write_string_with_format(1, 0, &sub, &subtitle_format)?;

	let headers = [
		"No",
		"Uraian Belanja / RKAS",
		"Satuan",
		"Volume",
		"Harga Satuan RKAS",
		"Total RKAS",
		"Produk SIPLah Toko",
		"Harga Satuan SIPLah",
		"Selisih Satuan",
		"Status",
	];

	for (col, h) in headers.iter().enumerate() {
		worksheet.write_string_with_format(3, col as u16, *h, &header_format)?;
	}

	for (row_idx, it) in report.items.iter().enumerate() {
		let r = (row_idx + 4) as u32;
		worksheet.write_number_with_format(r, 0, (row_idx + 1) as f64, &border_format)?;
		worksheet.write_string_with_format(r, 1, &it.nama_rkas, &border_format)?;
		worksheet.write_string_with_format(r, 2, it.satuan.as_deref().unwrap_or("-"), &border_format)?;
		worksheet.write_number_with_format(r, 3, it.volume, &border_format)?;
		worksheet.write_number_with_format(r, 4, it.harga_rkas as f64, &num_format)?;
		worksheet.write_number_with_format(r, 5, it.total_rkas as f64, &num_format)?;

		match &it.matched_product {
			Some(p) => {
				worksheet.write_string_with_format(r, 6, &p.nama, &border_format)?;
				worksheet.write_number_with_format(r, 7, p.harga as f64, &num_format)?;

				let diff_fmt = if it.selisih <= 0 { &hijau_format } else { &merah_format };
				worksheet.write_number_with_format(r, 8, it.selisih as f64, diff_fmt)?;

				let status_label = match it.status.as_str() {
					"sesuai" => "Sesuai",
					"lebih_murah" => "Lebih Murah",
					_ => "Lebih Mahal",
				};
				worksheet.write_string_with_format(r, 9, status_label, &border_format)?;
			}
			None => {
				worksheet.write_string_with_format(r, 6, "(Tidak ada kecocokan)", &border_format)?;
				worksheet.write_string_with_format(r, 7, "-", &border_format)?;
				worksheet.write_string_with_format(r, 8, "-", &border_format)?;
				worksheet.write_string_with_format(r, 9, "Belum Cocok", &border_format)?;
			}
		}
	}

	worksheet.autofit();
	workbook.save(Path::new(&path))?;
	Ok(path)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn extracts_id_from_url_and_slug() {
		assert_eq!(extract_toko_id("https://siplah.tokoladang.co.id/m/kpr-rajagaluh.92186?page=1&etalase=&keyword="), Some(92186));
		assert_eq!(extract_toko_id("kpr-rajagaluh.92186"), Some(92186));
		assert_eq!(extract_toko_id("92186"), Some(92186));
		assert_eq!(extract_toko_id("https://siplah.tokoladang.co.id/m/toko-abc.12345/"), Some(12345));
	}

	#[test]
	fn tokenizes_indonesian_text() {
		let tokens = tokenize("KERTAS HVS A4 70 GR (SINAR DUNIA)");
		assert_eq!(tokens, vec!["kertas", "hvs", "a4", "70", "gr", "sinar", "dunia"]);
	}

	#[test]
	fn matches_with_at_least_one_word() {
		let src_words = tokenize("Spidol Whiteboard Snowman");
		let prod_words = tokenize("SPIDOL KECIL");
		let (score, matched) = score_match(&src_words, &prod_words, "spidol whiteboard snowman", "spidol kecil");
		assert!(score >= 40);
		assert_eq!(matched, vec!["spidol"]);
	}

	#[test]
	fn exact_match_gets_100() {
		let src_words = tokenize("KERTAS HVS A4");
		let prod_words = tokenize("KERTAS HVS A4");
		let (score, _) = score_match(&src_words, &prod_words, "kertas hvs a4", "kertas hvs a4");
		assert_eq!(score, 100);
	}
}
