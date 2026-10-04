//! Export Excel untuk kertas kerja, realisasi, dan semua laporan sekaligus.

use std::path::PathBuf;

use rust_xlsxwriter::Workbook;
use tauri::State;

use super::rkas::{kertas_kerja, load_realisasi, KertasKerja};
use crate::error::{validate_year, AppError, AppResult};
use crate::export::xlsx::{add_book_sheet, add_table_sheet, Cell, TableSheet};
use crate::pengaturan;
use crate::repo::arkas::queries::{self, SchoolInfo};
use crate::services::book::{month_name, BookKind, BookRequest};
use crate::services::realisasi::Realisasi;
use crate::state::AppState;

fn target(path: &str) -> AppResult<PathBuf> {
	let mut p = PathBuf::from(path.trim());
	if p.as_os_str().is_empty() {
		return Err(AppError::InvalidInput("lokasi file belum dipilih".into()));
	}
	if p.extension().is_none_or(|e| !e.eq_ignore_ascii_case("xlsx")) {
		p.set_extension("xlsx");
	}
	Ok(p)
}

fn context(state: &AppState, year: i32, fund: Option<i64>) -> AppResult<(SchoolInfo, String)> {
	let (school, funds) = state.with_arkas(|db| Ok((queries::school_info(db, Some(year))?, queries::fund_sources(db, year)?)))?;
	let label = fund
		.and_then(|f| funds.iter().find(|s| s.id == f).map(|s| s.name.clone()))
		.unwrap_or_else(|| "Semua sumber dana".into());
	Ok((school, label))
}

fn info(school: &SchoolInfo, fund_label: &str) -> Vec<(&'static str, String)> {
	vec![
		("Nama Sekolah", school.nama.clone().unwrap_or_default()),
		("NPSN", school.npsn.clone().unwrap_or_default()),
		("Kabupaten/Kota", school.kabupaten.clone().unwrap_or_default()),
		("Sumber Dana", fund_label.to_string()),
	]
}

pub(crate) fn kertas_kerja_sheet<'a>(kk: &KertasKerja, school: &SchoolInfo, fund_label: &str) -> TableSheet<'a> {
	let mut headers = vec![
		("Kode Kegiatan", 13.0),
		("Nama Kegiatan", 30.0),
		("Kode Rekening", 18.0),
		("Uraian", 34.0),
		("Volume", 8.0),
		("Satuan", 9.0),
		("Harga Satuan", 13.0),
		("Jumlah", 14.0),
	];
	headers.extend(["Jan", "Feb", "Mar", "Apr", "Mei", "Jun", "Jul", "Agu", "Sep", "Okt", "Nov", "Des"].map(|m| (m, 11.0)));
	let mut items: Vec<_> = kk.items.iter().collect();
	items.sort_by(|a, b| (a.kode_kegiatan.as_deref(), a.kode_rekening.as_deref()).cmp(&(b.kode_kegiatan.as_deref(), b.kode_rekening.as_deref())));
	let mut month_totals = [0i64; 12];
	let rows = items
		.iter()
		.map(|it| {
			let kode = it.kode_kegiatan.clone().unwrap_or_default();
			let mut row = vec![
				Cell::Text(kode.clone()),
				Cell::Text(kk.kode_names.get(&kode).cloned().unwrap_or_default()),
				Cell::Text(it.kode_rekening.clone().unwrap_or_default()),
				Cell::Text(it.uraian.clone()),
				Cell::Num(it.volume),
				Cell::Text(it.satuan.clone().unwrap_or_default()),
				Cell::Money(it.harga_satuan),
				Cell::Money(it.jumlah),
			];
			for (m, v) in it.bulan.iter().enumerate() {
				month_totals[m] += v;
				row.push(if *v == 0 { Cell::Empty } else { Cell::Money(*v) });
			}
			row
		})
		.collect();
	let mut totals = vec![Cell::Text("JUMLAH".into()), Cell::Empty, Cell::Empty, Cell::Empty, Cell::Empty, Cell::Empty, Cell::Empty];
	totals.push(Cell::Money(kk.items.iter().map(|i| i.jumlah).sum()));
	totals.extend(month_totals.iter().map(|v| Cell::Money(*v)));
	TableSheet {
		name: "Kertas Kerja",
		title: "KERTAS KERJA RKAS",
		subtitle: format!("TAHUN ANGGARAN {}", kk.year),
		info: info(school, fund_label),
		headers,
		rows,
		totals: Some(totals),
	}
}

fn status_label(s: &str) -> &'static str {
	match s {
		"lunas" => "Lunas",
		"sebagian" => "Sebagian",
		"belum" => "Belum dibelanjakan",
		"melampaui" => "Melampaui pagu",
		_ => "Belum jatuh tempo",
	}
}

fn realisasi_sheet<'a>(r: &Realisasi, school: &SchoolInfo, fund_label: &str) -> TableSheet<'a> {
	let headers = vec![
		("Kode Kegiatan", 13.0),
		("Nama Kegiatan", 28.0),
		("Kode Rekening", 18.0),
		("Uraian", 34.0),
		("Pagu", 14.0),
		("Rencana s.d. Bulan", 14.0),
		("Realisasi", 14.0),
		("Sisa", 14.0),
		("%", 7.0),
		("Status", 16.0),
	];
	let mut rows: Vec<Vec<Cell>> = r
		.items
		.iter()
		.map(|i| {
			vec![
				Cell::Text(i.kode_kegiatan.clone().unwrap_or_default()),
				Cell::Text(i.nama_kegiatan.clone().unwrap_or_default()),
				Cell::Text(i.kode_rekening.clone().unwrap_or_default()),
				Cell::Text(i.uraian.clone()),
				Cell::Money(i.pagu),
				Cell::Money(i.rencana_sd),
				Cell::Money(i.total_realisasi),
				Cell::Money(i.sisa),
				Cell::Num(i.persen),
				Cell::Text(status_label(&i.status).into()),
			]
		})
		.collect();
	for o in &r.luar_rkas {
		rows.push(vec![
			Cell::Empty,
			Cell::Text("Di luar RKAS aktif".into()),
			Cell::Text(o.kode_rekening.clone().unwrap_or_default()),
			Cell::Text(o.uraian.clone()),
			Cell::Empty,
			Cell::Empty,
			Cell::Money(o.nominal),
			Cell::Empty,
			Cell::Empty,
			Cell::Text(o.tanggal.clone()),
		]);
	}
	let totals = vec![
		Cell::Text("JUMLAH".into()),
		Cell::Empty,
		Cell::Empty,
		Cell::Empty,
		Cell::Money(r.total_pagu),
		Cell::Money(r.items.iter().map(|i| i.rencana_sd).sum()),
		Cell::Money(r.total_realisasi),
		Cell::Money(r.total_pagu - r.total_realisasi),
		Cell::Num(r.persen),
		Cell::Empty,
	];
	TableSheet {
		name: "Realisasi",
		title: "REALISASI BELANJA",
		subtitle: format!("TAHUN ANGGARAN {} (ACUAN S.D. {})", r.year, month_name(r.upto).to_uppercase()),
		info: info(school, fund_label),
		headers,
		rows,
		totals: Some(totals),
	}
}

#[tauri::command]
pub fn export_kertas_kerja_xlsx(state: State<'_, AppState>, year: i32, fund: Option<i64>, path: String) -> AppResult<String> {
	let year = validate_year(year)?;
	let fund = fund.filter(|f| *f != 0);
	let path = target(&path)?;
	let kk = kertas_kerja(state.clone(), year, fund)?;
	let (school, label) = context(&state, year, fund)?;
	let mut wb = Workbook::new();
	add_table_sheet(&mut wb, &kertas_kerja_sheet(&kk, &school, &label))?;
	wb.save(&path)?;
	Ok(path.display().to_string())
}

#[tauri::command]
pub fn export_realisasi_xlsx(state: State<'_, AppState>, year: i32, upto: Option<u32>, fund: Option<i64>, path: String) -> AppResult<String> {
	let year = validate_year(year)?;
	let fund = fund.filter(|f| *f != 0);
	let path = target(&path)?;
	let r = load_realisasi(&state, year, upto.unwrap_or(12).clamp(1, 12), fund)?;
	let (school, label) = context(&state, year, fund)?;
	let mut wb = Workbook::new();
	add_table_sheet(&mut wb, &realisasi_sheet(&r, &school, &label))?;
	wb.save(&path)?;
	Ok(path.display().to_string())
}

/// Semua laporan satu tahun dalam satu file Excel: BKU, buku pembantu, realisasi, kertas kerja.
#[tauri::command]
pub fn export_all_xlsx(state: State<'_, AppState>, year: i32, fund: Option<i64>, path: String) -> AppResult<String> {
	let year = validate_year(year)?;
	let fund = fund.filter(|f| *f != 0);
	let path = target(&path)?;
	let (school, label) = context(&state, year, fund)?;
	let settings = pengaturan::gabung(&pengaturan::load(&state.app_db())?, &pengaturan::bawaan(&school));

	let mut wb = Workbook::new();
	for kind in [BookKind::Umum, BookKind::Bank, BookKind::Tunai, BookKind::Pajak] {
		let book = super::bku::load_book(&state, &BookRequest { kind, year, month: None, until: None, fund })?;
		add_book_sheet(&mut wb, &book, &school, &settings, &label)?;
	}
	let r = load_realisasi(&state, year, 12, fund)?;
	add_table_sheet(&mut wb, &realisasi_sheet(&r, &school, &label))?;
	let kk = kertas_kerja(state.clone(), year, fund)?;
	add_table_sheet(&mut wb, &kertas_kerja_sheet(&kk, &school, &label))?;
	wb.save(&path)?;
	Ok(path.display().to_string())
}
