//! Penganggaran RKAS: draft lokal dengan versi Awal, Perubahan, dan Pergeseran.

use std::collections::HashMap;

use rust_xlsxwriter::Workbook;
use serde::Serialize;
use tauri::State;

use super::export::kertas_kerja_sheet;
use super::rkas::KertasKerja;
use crate::error::{validate_year, AppError, AppResult};
use crate::export::xlsx::add_table_sheet;
use crate::repo::app::rkas_draft::{Draft, DraftItem};
use crate::repo::arkas::{queries, rkas};
use crate::state::AppState;

fn validate_id(id: &str) -> AppResult<&str> {
	let id = id.trim();
	if id.is_empty() || id.len() > 64 || !id.chars().all(|c| c.is_ascii_alphanumeric()) {
		return Err(AppError::InvalidInput("id draft tidak valid".into()));
	}
	Ok(id)
}

fn clean_name(nama: &str) -> AppResult<String> {
	let n = nama.trim();
	if n.is_empty() || n.chars().count() > 100 {
		return Err(AppError::InvalidInput("nama draft wajib diisi (maks. 100 karakter)".into()));
	}
	Ok(n.to_string())
}

#[tauri::command]
pub fn rkas_draft_list(state: State<'_, AppState>, year: i32) -> AppResult<Vec<Draft>> {
	state.app_db().drafts(validate_year(year)?)
}

/// Draft "Awal" dari RKAS yang berlaku di ARKAS (ARKAS hanya dibaca).
#[tauri::command]
pub fn rkas_draft_from_arkas(state: State<'_, AppState>, year: i32, fund: i64, nama: String) -> AppResult<String> {
	let year = validate_year(year)?;
	let nama = clean_name(&nama)?;
	let items = state.with_arkas(|db| rkas::rkas_items(db, year))?;
	let items: Vec<DraftItem> = items
		.into_iter()
		.filter(|i| i.fund_id == fund)
		.map(|i| {
			let mut volume_bulan = i.volume_bulan;
			// Bila volume per bulan kosong tetapi ada rupiah, turunkan dari rupiah / harga.
			if volume_bulan.iter().sum::<f64>() == 0.0 && i.harga_satuan > 0 {
				for (v, rp) in volume_bulan.iter_mut().zip(i.bulan.iter()) {
					*v = *rp as f64 / i.harga_satuan as f64;
				}
			}
			DraftItem {
				id: String::new(),
				urutan: 0,
				kode_kegiatan: i.kode_kegiatan,
				kode_rekening: i.kode_rekening,
				uraian: i.uraian,
				satuan: i.satuan,
				harga_satuan: i.harga_satuan,
				volume_bulan,
				sumber_id_rapbs: Some(i.id_rapbs),
			}
		})
		.collect();
	if items.is_empty() {
		return Err(AppError::InvalidInput("RKAS sumber dana ini belum ada atau belum disahkan di ARKAS".into()));
	}
	state.app_db().draft_create(year, fund, &nama, "awal", None, &items)
}

/// Salin draft menjadi versi Perubahan atau Pergeseran.
#[tauri::command]
pub fn rkas_draft_copy(state: State<'_, AppState>, id: String, jenis: String, nama: String) -> AppResult<String> {
	let id = validate_id(&id)?;
	if jenis != "perubahan" && jenis != "pergeseran" {
		return Err(AppError::InvalidInput("jenis harus perubahan atau pergeseran".into()));
	}
	let nama = clean_name(&nama)?;
	let mut db = state.app_db();
	let source = db.draft_get(id)?.ok_or_else(|| AppError::InvalidInput("draft tidak ditemukan".into()))?;
	let items = db.draft_items(id)?;
	db.draft_create(source.tahun, source.sumber_dana, &nama, &jenis, Some(id), &items)
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DraftDetail {
	pub draft: Draft,
	pub items: Vec<DraftItem>,
	pub parent: Option<Draft>,
	pub kode_names: HashMap<String, String>,
	pub rekening_names: HashMap<String, String>,
	pub fund_name: String,
}

#[tauri::command]
pub fn rkas_draft_detail(state: State<'_, AppState>, id: String) -> AppResult<DraftDetail> {
	let id = validate_id(&id)?;
	let (draft, items, parent) = {
		let db = state.app_db();
		let draft = db.draft_get(id)?.ok_or_else(|| AppError::InvalidInput("draft tidak ditemukan".into()))?;
		let parent = match &draft.dari {
			Some(p) => db.draft_get(p)?,
			None => None,
		};
		(draft.clone(), db.draft_items(id)?, parent)
	};
	let (kode_names, rekening_names, fund_name) = state
		.with_arkas(|db| {
			let funds = queries::fund_sources(db, draft.tahun)?;
			Ok((
				rkas::kode_names(db, draft.tahun)?,
				rkas::rekening_names(db, draft.tahun)?,
				funds.into_iter().find(|f| f.id == draft.sumber_dana).map(|f| f.name).unwrap_or_default(),
			))
		})
		.unwrap_or_default();
	Ok(DraftDetail { draft, items, parent, kode_names, rekening_names, fund_name })
}

#[tauri::command]
pub fn rkas_draft_item_save(state: State<'_, AppState>, draft_id: String, item: DraftItem) -> AppResult<String> {
	state.app_db().draft_item_save(validate_id(&draft_id)?, &item)
}

#[tauri::command]
pub fn rkas_draft_item_delete(state: State<'_, AppState>, draft_id: String, item_id: String) -> AppResult<()> {
	state.app_db().draft_item_delete(validate_id(&draft_id)?, validate_id(&item_id)?)
}

#[tauri::command]
pub fn rkas_draft_rename(state: State<'_, AppState>, id: String, nama: String) -> AppResult<()> {
	state.app_db().draft_rename(validate_id(&id)?, &clean_name(&nama)?)
}

#[tauri::command]
pub fn rkas_draft_delete(state: State<'_, AppState>, id: String) -> AppResult<()> {
	state.app_db().draft_delete(validate_id(&id)?)
}

#[tauri::command]
pub fn rkas_draft_export_xlsx(state: State<'_, AppState>, id: String, path: String) -> AppResult<String> {
	let detail = rkas_draft_detail(state.clone(), id)?;
	let mut target = std::path::PathBuf::from(path.trim());
	if target.extension().is_none_or(|e| !e.eq_ignore_ascii_case("xlsx")) {
		target.set_extension("xlsx");
	}
	let items = detail
		.items
		.iter()
		.map(|i| {
			let bulan = i.volume_bulan.map(|v| (v * i.harga_satuan as f64).round() as i64);
			rkas::RkasItem {
				id_rapbs: i.id.clone(),
				fund_id: detail.draft.sumber_dana,
				fund_name: detail.fund_name.clone(),
				kode_kegiatan: i.kode_kegiatan.clone(),
				kode_rekening: i.kode_rekening.clone(),
				uraian: i.uraian.clone(),
				satuan: i.satuan.clone(),
				volume: i.volume(),
				harga_satuan: i.harga_satuan,
				jumlah: i.jumlah(),
				bulan,
				volume_bulan: i.volume_bulan,
			}
		})
		.collect();
	let kk = KertasKerja {
		year: detail.draft.tahun,
		anggaran: Vec::new(),
		items,
		kode_names: detail.kode_names.clone(),
		rekening_names: detail.rekening_names.clone(),
	};
	let school = state.with_arkas(|db| queries::school_info(db, Some(detail.draft.tahun))).unwrap_or_default();
	let mut sheet = kertas_kerja_sheet(&kk, &school, &detail.fund_name);
	sheet.title = "DRAFT RKAS";
	sheet.subtitle = format!("{} — TAHUN ANGGARAN {}", detail.draft.nama.to_uppercase(), detail.draft.tahun);
	let mut wb = Workbook::new();
	add_table_sheet(&mut wb, &sheet)?;
	wb.save(&target)?;
	Ok(target.display().to_string())
}
