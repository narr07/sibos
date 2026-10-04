use std::collections::HashMap;

use serde::Serialize;
use tauri::State;

use crate::error::{validate_year, AppError, AppResult};
use crate::repo::arkas::rkas::{self, AnggaranInfo, RkasItem, StandarHarga};
use crate::repo::arkas::bku;
use crate::services::book::{build_book_with_manual, BookKind, BookRequest, ManualTaxLine};
use crate::services::realisasi::{build_realisasi, Realisasi, RealisasiInput};
use crate::services::summary::PeriodSummary;
use crate::state::AppState;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KertasKerja {
	pub year: i32,
	pub anggaran: Vec<AnggaranInfo>,
	pub items: Vec<RkasItem>,
	/// Nama untuk setiap kode program/komponen/kegiatan yang dipakai item.
	pub kode_names: HashMap<String, String>,
	pub rekening_names: HashMap<String, String>,
}

/// Semua awalan kode: "05.02.08." -> ["05.", "05.02.", "05.02.08."].
fn prefixes(kode: &str) -> Vec<String> {
	let parts: Vec<&str> = kode.split('.').filter(|p| !p.is_empty()).collect();
	(1..=parts.len()).map(|n| format!("{}.", parts[..n].join("."))).collect()
}

#[tauri::command]
pub fn kertas_kerja(state: State<'_, AppState>, year: i32, fund: Option<i64>) -> AppResult<KertasKerja> {
	let year = validate_year(year)?;
	let fund = fund.filter(|f| *f != 0);
	let (anggaran, items, kode, rek) = state.with_arkas(|db| {
		Ok((rkas::anggaran_list(db, year)?, rkas::rkas_items(db, year)?, rkas::kode_names(db, year)?, rkas::rekening_names(db, year)?))
	})?;
	let items: Vec<RkasItem> = items.into_iter().filter(|i| fund.is_none_or(|f| f == i.fund_id)).collect();
	let mut kode_names = HashMap::new();
	let mut rekening_names = HashMap::new();
	for it in &items {
		for p in it.kode_kegiatan.as_deref().map(prefixes).unwrap_or_default() {
			if let Some(n) = kode.get(&p) {
				kode_names.insert(p, n.clone());
			}
		}
		if let Some(k) = &it.kode_rekening {
			if let Some(n) = rek.get(k) {
				rekening_names.insert(k.clone(), n.clone());
			}
		}
	}
	Ok(KertasKerja { year, anggaran, items, kode_names, rekening_names })
}

pub(crate) fn load_realisasi(state: &AppState, year: i32, upto: u32, fund: Option<i64>) -> AppResult<Realisasi> {
	let (items, rows, kode, rek) = state.with_arkas(|db| {
		Ok((rkas::rkas_items(db, year)?, bku::kas_rows(db, year)?, rkas::kode_names(db, year)?, rkas::rekening_names(db, year)?))
	})?;
	let input = RealisasiInput { items: &items, rows: &rows, kode_names: &kode, rekening_names: &rek };
	Ok(build_realisasi(year, upto, fund, &input))
}

#[tauri::command]
pub fn realisasi(state: State<'_, AppState>, year: i32, upto: Option<u32>, fund: Option<i64>) -> AppResult<Realisasi> {
	let year = validate_year(year)?;
	let upto = upto.unwrap_or(12);
	if !(1..=12).contains(&upto) {
		return Err(AppError::InvalidInput("bulan tidak valid".into()));
	}
	load_realisasi(&state, year, upto, fund.filter(|f| *f != 0))
}

#[tauri::command]
pub fn standar_harga_search(state: State<'_, AppState>, year: i32, keyword: String) -> AppResult<Vec<StandarHarga>> {
	let year = validate_year(year)?;
	let keyword = keyword.trim();
	if keyword.chars().count() < 2 {
		return Ok(Vec::new());
	}
	if keyword.chars().count() > 60 {
		return Err(AppError::InvalidInput("kata kunci terlalu panjang".into()));
	}
	state.with_arkas(|db| rkas::standar_harga_search(db, year, keyword, 200))
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Dashboard {
	pub year: i32,
	pub upto: u32,
	pub summary: PeriodSummary,
	pub pagu: i64,
	pub total_realisasi: i64,
	pub persen: f64,
	pub total_tertunda: i64,
	pub item_belum: usize,
	pub item_melampaui: usize,
	pub rencana_bulan: [i64; 12],
	pub realisasi_bulan: [i64; 12],
	pub anggaran: Vec<AnggaranInfo>,
	pub bulan_ditutup: Vec<u32>,
	pub pajak_belum_disetor: i64,
}

#[tauri::command]
pub fn dashboard(state: State<'_, AppState>, year: i32, fund: Option<i64>) -> AppResult<Dashboard> {
	let year = validate_year(year)?;
	let fund = fund.filter(|f| *f != 0);
	let (last, anggaran, balances, rows) = state.with_arkas(|db| {
		Ok((bku::last_active_month(db, year)?, rkas::anggaran_list(db, year)?, bku::month_balances(db, year)?, bku::kas_rows(db, year)?))
	})?;
	// Bulan acuan: bulan berjalan untuk tahun ini, Desember untuk tahun lalu.
	let now = current_year_month();
	let upto = if year < now.0 { 12 } else { now.1.max(last.unwrap_or(1)) }.clamp(1, 12);

	let summary = super::laporan::load_summary(&state, year, 1, upto, fund)?;
	let real = load_realisasi(&state, year, upto, fund)?;

	let manual: Vec<ManualTaxLine> = state
		.app_db()
		.manual_taxes(year)?
		.into_iter()
		.map(|m| ManualTaxLine {
			id: m.id,
			tanggal: m.tanggal,
			no_bukti: m.no_bukti,
			uraian: m.uraian,
			jenis_pajak: m.jenis_pajak,
			pungut: m.arah == "pungut",
			nominal: m.nominal,
			fund_id: m.sumber_dana,
			fund_name: String::new(),
		})
		.collect();
	let pajak = build_book_with_manual(
		&BookRequest { kind: BookKind::Pajak, year, month: None, until: None, fund },
		&rows,
		&balances,
		&HashMap::new(),
		&manual,
	);

	let mut bulan_ditutup: Vec<u32> =
		balances.iter().filter(|b| b.finished && fund.is_none_or(|f| f == b.fund_id)).map(|b| b.month).collect();
	bulan_ditutup.sort_unstable();
	bulan_ditutup.dedup();

	let pagu = rkas::active_anggaran(&anggaran)
		.iter()
		.filter(|a| fund.is_none_or(|f| f == a.fund_id))
		.map(|a| a.jumlah)
		.sum();

	Ok(Dashboard {
		year,
		upto,
		pagu,
		total_realisasi: real.total_realisasi,
		persen: real.persen,
		total_tertunda: real.total_tertunda,
		item_belum: real.items.iter().filter(|i| i.status == "belum").count(),
		item_melampaui: real.items.iter().filter(|i| i.status == "melampaui").count(),
		rencana_bulan: real.rencana_bulan,
		realisasi_bulan: real.realisasi_bulan,
		summary,
		anggaran,
		bulan_ditutup,
		pajak_belum_disetor: pajak.closing,
	})
}

fn current_year_month() -> (i32, u32) {
	let (y, m, ..) = crate::util::now_wib();
	(y, m)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn code_prefixes() {
		assert_eq!(prefixes("05.02.08."), vec!["05.", "05.02.", "05.02.08."]);
	}

	#[test]
	fn current_date_is_sane() {
		let (y, m) = current_year_month();
		assert!((2024..2100).contains(&y));
		assert!((1..=12).contains(&m));
	}
}
