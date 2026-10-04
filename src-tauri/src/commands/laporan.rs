use std::collections::HashMap;

use serde::Serialize;
use tauri::State;

use crate::error::{validate_year, AppError, AppResult};
use crate::repo::app::laporan::{CashRegister, ManualTax};
use crate::repo::arkas::{bku, queries, rkas};
use crate::services::book::{build_book, BookKind, BookRequest};
use crate::services::summary::{summarize, PeriodSummary, SummaryInput};
use crate::state::AppState;

fn validate_range(start: u32, end: u32) -> AppResult<(u32, u32)> {
	if !(1..=12).contains(&start) || !(1..=12).contains(&end) || start > end {
		return Err(AppError::InvalidInput("rentang bulan tidak valid".into()));
	}
	Ok((start, end))
}

fn fund_opt(fund: Option<i64>) -> Option<i64> {
	fund.filter(|f| *f != 0)
}

pub(crate) fn load_summary(state: &AppState, year: i32, start: u32, end: u32, fund: Option<i64>) -> AppResult<PeriodSummary> {
	let (rows, balances, kode, rek) = state.with_arkas(|db| {
		Ok((bku::kas_rows(db, year)?, bku::month_balances(db, year)?, rkas::kode_names(db, year)?, rkas::rekening_names(db, year)?))
	})?;
	let input = SummaryInput { rows: &rows, balances: &balances, kode_names: &kode, rekening_names: &rek };
	Ok(summarize(year, start, end, fund, &input))
}

/// Ringkasan keuangan periode: dasar SPTJM, BA Rekonsiliasi, dan K7.
#[tauri::command]
pub fn period_summary(state: State<'_, AppState>, year: i32, start: u32, end: u32, fund: Option<i64>) -> AppResult<PeriodSummary> {
	let year = validate_year(year)?;
	let (start, end) = validate_range(start, end)?;
	load_summary(&state, year, start, end, fund_opt(fund))
}

/// Ringkasan per sumber dana (kolom-kolom laporan K7).
#[tauri::command]
pub fn period_summary_by_fund(
	state: State<'_, AppState>,
	year: i32,
	start: u32,
	end: u32,
) -> AppResult<Vec<(queries::FundSource, PeriodSummary)>> {
	let year = validate_year(year)?;
	let (start, end) = validate_range(start, end)?;
	let funds = state.with_arkas(|db| queries::fund_sources(db, year))?;
	funds
		.into_iter()
		.map(|f| {
			let s = load_summary(&state, year, start, end, Some(f.id))?;
			Ok((f, s))
		})
		.collect()
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RekonBankMonth {
	pub month: u32,
	pub saldo_bank: i64,
	pub saldo_tunai: i64,
	pub penerimaan: i64,
	pub belanja: i64,
	pub rekening_koran: Option<i64>,
	pub selisih: Option<i64>,
	pub keterangan: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RekonBank {
	pub year: i32,
	pub fund: Option<i64>,
	pub saldo_awal_bank: i64,
	pub months: Vec<RekonBankMonth>,
}

#[tauri::command]
pub fn rekon_bank(state: State<'_, AppState>, year: i32, fund: Option<i64>) -> AppResult<RekonBank> {
	let year = validate_year(year)?;
	let fund = fund_opt(fund);
	let summary = load_summary(&state, year, 1, 12, fund)?;
	let statements: HashMap<u32, (i64, Option<String>)> = state
		.app_db()
		.bank_statements(year, fund.unwrap_or(0))?
		.into_iter()
		.map(|s| (s.bulan, (s.saldo, s.keterangan)))
		.collect();
	let months = summary
		.per_bulan
		.iter()
		.map(|m| {
			let st = statements.get(&m.month);
			RekonBankMonth {
				month: m.month,
				saldo_bank: m.saldo_akhir_bank,
				saldo_tunai: m.saldo_akhir_tunai,
				penerimaan: m.penerimaan,
				belanja: m.belanja,
				rekening_koran: st.map(|s| s.0),
				selisih: st.map(|s| s.0 - m.saldo_akhir_bank),
				keterangan: st.and_then(|s| s.1.clone()),
			}
		})
		.collect();
	Ok(RekonBank { year, fund, saldo_awal_bank: summary.saldo_awal_bank, months })
}

#[tauri::command]
pub fn bank_statement_set(
	state: State<'_, AppState>,
	year: i32,
	fund: Option<i64>,
	month: u32,
	saldo: Option<i64>,
	keterangan: Option<String>,
) -> AppResult<()> {
	let year = validate_year(year)?;
	let (month, _) = validate_range(month, month)?;
	let ket = keterangan.as_deref().map(str::trim).filter(|k| !k.is_empty());
	state.app_db().bank_statement_set(year, fund_opt(fund).unwrap_or(0), month, saldo, ket)
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterKas {
	pub year: i32,
	pub month: u32,
	pub saved: Option<CashRegister>,
	/// Total uang fisik dari pecahan tersimpan.
	pub total_fisik: i64,
	pub saldo_tunai: i64,
	pub saldo_bank: i64,
	/// Penerimaan sejak 1 Januari s.d. akhir bulan.
	pub penerimaan_sd: i64,
	/// Pengeluaran sejak 1 Januari s.d. akhir bulan.
	pub pengeluaran_sd: i64,
	pub saldo_buku: i64,
	pub pajak_belum_disetor: i64,
}

#[tauri::command]
pub fn register_kas_get(state: State<'_, AppState>, year: i32, month: u32, fund: Option<i64>) -> AppResult<RegisterKas> {
	let year = validate_year(year)?;
	let (month, _) = validate_range(month, month)?;
	let fund = fund_opt(fund);
	let (rows, balances) = state.with_arkas(|db| Ok((bku::kas_rows(db, year)?, bku::month_balances(db, year)?)))?;
	let empty = HashMap::new();
	let req = |kind| BookRequest { kind, year, month: Some(1), until: Some(month), fund };
	let umum = build_book(&req(BookKind::Umum), &rows, &balances, &empty);
	let pajak = build_book(&req(BookKind::Pajak), &rows, &balances, &empty);
	let saved = state.app_db().cash_register_get(year, month, fund.unwrap_or(0))?;
	Ok(RegisterKas {
		year,
		month,
		total_fisik: saved.as_ref().map(|s| s.pecahan.total()).unwrap_or(0),
		saved,
		saldo_tunai: umum.closing_tunai,
		saldo_bank: umum.closing_bank,
		penerimaan_sd: umum.opening + umum.total_penerimaan,
		pengeluaran_sd: umum.total_pengeluaran,
		saldo_buku: umum.closing,
		pajak_belum_disetor: pajak.closing,
	})
}

#[tauri::command]
pub fn register_kas_set(state: State<'_, AppState>, year: i32, month: u32, fund: Option<i64>, value: CashRegister) -> AppResult<()> {
	let year = validate_year(year)?;
	let (month, _) = validate_range(month, month)?;
	if value.pecahan.kertas.values().chain(value.pecahan.logam.values()).any(|n| *n < 0) {
		return Err(AppError::InvalidInput("jumlah lembar/keping tidak boleh negatif".into()));
	}
	state.app_db().cash_register_set(year, month, fund_opt(fund).unwrap_or(0), &value)
}

#[tauri::command]
pub fn manual_tax_list(state: State<'_, AppState>, year: i32) -> AppResult<Vec<ManualTax>> {
	state.app_db().manual_taxes(validate_year(year)?)
}

#[tauri::command]
pub fn manual_tax_save(state: State<'_, AppState>, entry: ManualTax) -> AppResult<String> {
	let mut entry = entry;
	entry.tahun = validate_year(entry.tahun)?;
	if !entry.tanggal.starts_with(&entry.tahun.to_string()) {
		return Err(AppError::InvalidInput("tanggal harus di tahun anggaran yang dipilih".into()));
	}
	state.app_db().manual_tax_save(&entry)
}

#[tauri::command]
pub fn manual_tax_delete(state: State<'_, AppState>, id: String) -> AppResult<()> {
	let id = id.trim();
	if id.is_empty() || id.len() > 64 {
		return Err(AppError::InvalidInput("id tidak valid".into()));
	}
	state.app_db().manual_tax_delete(id)
}
