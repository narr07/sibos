use std::path::PathBuf;

use tauri::State;

use crate::error::{validate_year, AppError, AppResult};
use crate::export::xlsx;
use crate::pengaturan;
use crate::repo::arkas::{bku, queries};
use crate::services::book::{build_book_with_manual, Book, BookKind, BookRequest, ManualTaxLine};
use crate::state::AppState;

fn request(kind: BookKind, year: i32, month: Option<u32>, fund: Option<i64>) -> AppResult<BookRequest> {
	let year = validate_year(year)?;
	if let Some(m) = month {
		if !(1..=12).contains(&m) {
			return Err(AppError::InvalidInput(format!("bulan {m} tidak valid")));
		}
	}
	Ok(BookRequest { kind, year, month, until: None, fund: fund.filter(|f| *f != 0) })
}

/// Bangun buku: data ARKAS dibaca dulu (kunci ARKAS dilepas), baru data SIBOS.
pub(crate) fn load_book(state: &AppState, req: &BookRequest) -> AppResult<Book> {
	let (rows, balances) = state.with_arkas(|db| Ok((bku::kas_rows(db, req.year)?, bku::month_balances(db, req.year)?)))?;
	let funds = state.with_arkas(|db| queries::fund_sources(db, req.year))?;
	let db = state.app_db();
	let overrides = db.uraian_overrides(req.year)?;
	let manual: Vec<ManualTaxLine> = if req.kind == BookKind::Pajak {
		db.manual_taxes(req.year)?
			.into_iter()
			.map(|m| ManualTaxLine {
				fund_name: funds.iter().find(|f| f.id == m.sumber_dana).map(|f| f.name.clone()).unwrap_or_default(),
				id: m.id,
				tanggal: m.tanggal,
				no_bukti: m.no_bukti,
				uraian: m.uraian,
				jenis_pajak: m.jenis_pajak,
				pungut: m.arah == "pungut",
				nominal: m.nominal,
				fund_id: m.sumber_dana,
			})
			.collect()
	} else {
		Vec::new()
	};
	drop(db);
	Ok(build_book_with_manual(req, &rows, &balances, &overrides, &manual))
}

#[tauri::command]
pub fn book(
	state: State<'_, AppState>,
	kind: BookKind,
	year: i32,
	month: Option<u32>,
	fund: Option<i64>,
) -> AppResult<Book> {
	load_book(&state, &request(kind, year, month, fund)?)
}

#[tauri::command]
pub fn last_active_month(state: State<'_, AppState>, year: i32) -> AppResult<Option<u32>> {
	let year = validate_year(year)?;
	state.with_arkas(|db| bku::last_active_month(db, year))
}

fn validate_id(id: &str) -> AppResult<&str> {
	let id = id.trim();
	if id.is_empty() || id.len() > 64 {
		return Err(AppError::InvalidInput("id transaksi tidak valid".into()));
	}
	Ok(id)
}

/// Ubah uraian tampilan (disimpan di sibos.db). Uraian kosong = kembalikan ke uraian ARKAS.
#[tauri::command]
pub fn uraian_override_set(state: State<'_, AppState>, year: i32, id: String, uraian: String) -> AppResult<()> {
	let year = validate_year(year)?;
	let id = validate_id(&id)?;
	let uraian = uraian.trim();
	if uraian.chars().count() > 500 {
		return Err(AppError::InvalidInput("uraian maksimal 500 karakter".into()));
	}
	let db = state.app_db();
	if uraian.is_empty() {
		db.uraian_override_delete(id)
	} else {
		db.uraian_override_set(year, id, uraian)
	}
}

#[tauri::command]
pub fn uraian_override_delete(state: State<'_, AppState>, id: String) -> AppResult<()> {
	state.app_db().uraian_override_delete(validate_id(&id)?)
}

/// Export buku ke file Excel di path yang dipilih pengguna lewat dialog simpan.
#[tauri::command]
pub fn export_book_xlsx(
	state: State<'_, AppState>,
	kind: BookKind,
	year: i32,
	month: Option<u32>,
	fund: Option<i64>,
	path: String,
) -> AppResult<String> {
	let req = request(kind, year, month, fund)?;
	let mut path = PathBuf::from(path.trim());
	if path.as_os_str().is_empty() {
		return Err(AppError::InvalidInput("lokasi file belum dipilih".into()));
	}
	if path.extension().is_none_or(|e| !e.eq_ignore_ascii_case("xlsx")) {
		path.set_extension("xlsx");
	}

	let book = load_book(&state, &req)?;
	let (school, funds) = state.with_arkas(|db| Ok((queries::school_info(db, Some(req.year))?, queries::fund_sources(db, req.year)?)))?;
	let fund_label = req
		.fund
		.and_then(|f| funds.iter().find(|s| s.id == f).map(|s| s.name.clone()))
		.unwrap_or_else(|| "Semua sumber dana".into());
	let settings = pengaturan::gabung(&pengaturan::load(&state.app_db())?, &pengaturan::bawaan(&school));

	xlsx::write_book(&path, &book, &school, &settings, &fund_label)?;
	Ok(path.display().to_string())
}
