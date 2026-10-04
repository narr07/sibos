//! Ringkasan keuangan satu periode (bulan, triwulan, semester, tahun) untuk laporan:
//! SPTJM, BA Rekonsiliasi, K7, Rekonsiliasi Bank, Register Kas, dan Dashboard.

use std::collections::{BTreeMap, HashMap};

use serde::Serialize;

use super::book::{build_book, BookKind, BookRequest};
use super::classify::classify;
use crate::repo::arkas::bku::{KasRow, MonthBalance};

/// Kelompok belanja menurut kode rekening (urutan tampil).
pub const KELOMPOK: [(&str, &str); 9] = [
	("5.1.02.01", "Belanja Barang"),
	("5.1.02.02", "Belanja Jasa"),
	("5.1.02.03", "Belanja Pemeliharaan"),
	("5.1.02.04", "Belanja Perjalanan Dinas"),
	("5.1.02", "Belanja Barang dan Jasa Lainnya"),
	("5.2.02", "Belanja Modal Peralatan dan Mesin"),
	("5.2.03", "Belanja Modal Gedung dan Bangunan"),
	("5.2.05", "Belanja Modal Aset Tetap Lainnya"),
	("5.2", "Belanja Modal Lainnya"),
];

pub fn kelompok_of(kode_rekening: &str) -> (&'static str, &'static str) {
	KELOMPOK
		.iter()
		.find(|(prefix, _)| kode_rekening.starts_with(prefix))
		.copied()
		.unwrap_or(("5", "Belanja Lainnya"))
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReceiptLine {
	pub tanggal: String,
	pub uraian: String,
	pub jenis: String,
	pub fund_name: String,
	pub nominal: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Total {
	pub kode: String,
	pub nama: String,
	pub total: i64,
}

#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct MonthFlow {
	pub month: u32,
	pub penerimaan: i64,
	pub belanja: i64,
	pub saldo_akhir_bank: i64,
	pub saldo_akhir_tunai: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PeriodSummary {
	pub year: i32,
	pub start: u32,
	pub end: u32,
	pub fund: Option<i64>,
	pub saldo_awal_bank: i64,
	pub saldo_awal_tunai: i64,
	pub saldo_akhir_bank: i64,
	pub saldo_akhir_tunai: i64,
	pub penerimaan: Vec<ReceiptLine>,
	/// Terima dana (kode 2).
	pub penerimaan_dana: i64,
	/// Bunga bank (kode 6/26).
	pub bunga: i64,
	pub total_penerimaan: i64,
	pub belanja_kelompok: Vec<Total>,
	pub belanja_rekening: Vec<Total>,
	/// Per program (segmen pertama kode kegiatan), untuk K7.
	pub belanja_program: Vec<Total>,
	pub belanja_operasi: i64,
	pub belanja_modal: i64,
	pub total_belanja: i64,
	pub pajak_bunga: i64,
	pub pengembalian: i64,
	pub pajak_dipungut: i64,
	pub pajak_disetor: i64,
	pub per_bulan: Vec<MonthFlow>,
	pub warnings: Vec<String>,
}

fn month_of(tanggal: &str) -> u32 {
	tanggal.get(5..7).and_then(|m| m.parse().ok()).unwrap_or(0)
}

/// Kode program dari kode kegiatan: "05.02.08." -> "05.".
pub fn program_of(kode_kegiatan: &str) -> String {
	match kode_kegiatan.split('.').next() {
		Some(first) if !first.is_empty() => format!("{first}."),
		_ => kode_kegiatan.to_string(),
	}
}

pub struct SummaryInput<'a> {
	pub rows: &'a [KasRow],
	pub balances: &'a [MonthBalance],
	pub kode_names: &'a HashMap<String, String>,
	pub rekening_names: &'a HashMap<String, String>,
}

pub fn summarize(year: i32, start: u32, end: u32, fund: Option<i64>, input: &SummaryInput<'_>) -> PeriodSummary {
	let empty = HashMap::new();
	let req = |kind, s: u32, e: u32| BookRequest { kind, year, month: Some(s), until: Some(e), fund };
	let bank = build_book(&req(BookKind::Bank, start, end), input.rows, input.balances, &empty);
	let tunai = build_book(&req(BookKind::Tunai, start, end), input.rows, input.balances, &empty);

	let mut penerimaan = Vec::new();
	let (mut penerimaan_dana, mut bunga, mut pajak_bunga, mut pengembalian) = (0, 0, 0, 0);
	let (mut pajak_dipungut, mut pajak_disetor) = (0, 0);
	let mut kelompok: BTreeMap<usize, (String, String, i64)> = BTreeMap::new();
	let mut rekening: BTreeMap<String, i64> = BTreeMap::new();
	let mut program: BTreeMap<String, i64> = BTreeMap::new();
	let (mut operasi, mut modal) = (0, 0);
	let mut flows: BTreeMap<u32, MonthFlow> = (start..=end).map(|m| (m, MonthFlow { month: m, ..Default::default() })).collect();

	for r in input.rows.iter().filter(|r| fund.is_none_or(|f| f == r.fund_id)) {
		let m = month_of(&r.tanggal);
		if m < start || m > end {
			continue;
		}
		let base = if (23..=35).contains(&r.id_ref_bku) { r.id_ref_bku - 20 } else { r.id_ref_bku };
		let flow = flows.entry(m).or_default();
		match base {
			2 => {
				penerimaan_dana += r.saldo;
				flow.penerimaan += r.saldo;
				penerimaan.push(ReceiptLine {
					tanggal: r.tanggal.clone(),
					uraian: r.uraian.clone(),
					jenis: r.ref_bku_name.clone().unwrap_or_else(|| "Terima Dana".into()),
					fund_name: r.fund_name.clone(),
					nominal: r.saldo,
				});
			}
			6 => {
				bunga += r.saldo;
				flow.penerimaan += r.saldo;
			}
			7 => pajak_bunga += r.saldo,
			14 => pengembalian += r.saldo,
			10 => pajak_dipungut += r.saldo,
			11 => pajak_disetor += r.saldo,
			4 | 15 => {
				flow.belanja += r.saldo;
				let kode = r.kode_rekening.clone().unwrap_or_default();
				let (prefix, nama) = kelompok_of(&kode);
				let order = KELOMPOK.iter().position(|(p, _)| *p == prefix).unwrap_or(KELOMPOK.len());
				kelompok.entry(order).or_insert_with(|| (prefix.into(), nama.into(), 0)).2 += r.saldo;
				*rekening.entry(kode.clone()).or_default() += r.saldo;
				if kode.starts_with("5.2") {
					modal += r.saldo;
				} else {
					operasi += r.saldo;
				}
				let prog = r.kode_kegiatan.as_deref().map(program_of).unwrap_or_else(|| "-".into());
				*program.entry(prog).or_default() += r.saldo;
			}
			_ => {
				debug_assert!(classify(r.id_ref_bku).kind != super::classify::Kind::Unknown || base == 0);
			}
		}
	}

	// Saldo akhir tiap bulan dari buku bank & tunai satu bulan.
	for (m, flow) in flows.iter_mut() {
		flow.saldo_akhir_bank = build_book(&req(BookKind::Bank, *m, *m), input.rows, input.balances, &empty).closing;
		flow.saldo_akhir_tunai = build_book(&req(BookKind::Tunai, *m, *m), input.rows, input.balances, &empty).closing;
	}

	let total_belanja = operasi + modal;
	let mut warnings = bank.warnings.clone();
	warnings.extend(tunai.warnings.iter().cloned());

	PeriodSummary {
		year,
		start,
		end,
		fund,
		saldo_awal_bank: bank.opening,
		saldo_awal_tunai: tunai.opening,
		saldo_akhir_bank: bank.closing,
		saldo_akhir_tunai: tunai.closing,
		penerimaan,
		penerimaan_dana,
		bunga,
		total_penerimaan: penerimaan_dana + bunga,
		belanja_kelompok: kelompok.into_values().map(|(kode, nama, total)| Total { kode, nama, total }).collect(),
		belanja_rekening: rekening
			.into_iter()
			.map(|(kode, total)| Total { nama: input.rekening_names.get(&kode).cloned().unwrap_or_default(), kode, total })
			.collect(),
		belanja_program: program
			.into_iter()
			.map(|(kode, total)| Total { nama: input.kode_names.get(&kode).cloned().unwrap_or_default(), kode, total })
			.collect(),
		belanja_operasi: operasi,
		belanja_modal: modal,
		total_belanja,
		pajak_bunga,
		pengembalian,
		pajak_dipungut,
		pajak_disetor,
		per_bulan: flows.into_values().collect(),
		warnings,
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	fn row(id: &str, tanggal: &str, ref_bku: i64, saldo: i64, rek: &str, keg: &str) -> KasRow {
		KasRow {
			id: id.into(),
			id_ref_bku: ref_bku,
			tanggal: tanggal.into(),
			uraian: id.into(),
			saldo,
			fund_id: 1,
			fund_name: "BOS Reguler".into(),
			create_date: tanggal.into(),
			kode_rekening: (!rek.is_empty()).then(|| rek.to_string()),
			kode_kegiatan: (!keg.is_empty()).then(|| keg.to_string()),
			..Default::default()
		}
	}

	fn rows() -> Vec<KasRow> {
		vec![
			row("o1", "2026-01-01", 8, 0, "", ""),
			row("t1", "2026-01-10", 2, 50_000_000, "4.3.1.01", ""),
			row("b1", "2026-01-15", 15, 1_000_000, "5.1.02.01.01.0024", "05.02.08."),
			row("b2", "2026-02-15", 15, 4_000_000, "5.2.05.01.01.0001", "02.01.01."),
			row("b3", "2026-04-15", 15, 500_000, "5.1.02.02.01.0042", "05.01.01."),
			row("i1", "2026-04-30", 6, 10_000, "", ""),
			row("i2", "2026-04-30", 7, 2_000, "", ""),
		]
	}

	#[test]
	fn first_semester_totals() {
		let mut names = HashMap::new();
		names.insert("05.".to_string(), "Pengembangan Sarana".to_string());
		let input = SummaryInput { rows: &rows(), balances: &[], kode_names: &names, rekening_names: &HashMap::new() };
		let s = summarize(2026, 1, 6, None, &input);
		assert_eq!(s.saldo_awal_bank, 0);
		assert_eq!(s.penerimaan_dana, 50_000_000);
		assert_eq!(s.bunga, 10_000);
		assert_eq!(s.total_belanja, 5_500_000);
		assert_eq!(s.belanja_modal, 4_000_000);
		assert_eq!(s.belanja_operasi, 1_500_000);
		assert_eq!(s.pajak_bunga, 2_000);
		assert_eq!(s.saldo_akhir_bank, 50_000_000 + 10_000 - 5_500_000 - 2_000);
		assert_eq!(s.belanja_kelompok[0].nama, "Belanja Barang");
		assert_eq!(s.belanja_kelompok.len(), 3);
		let prog05 = s.belanja_program.iter().find(|p| p.kode == "05.").unwrap();
		assert_eq!((prog05.total, prog05.nama.as_str()), (1_500_000, "Pengembangan Sarana"));
		assert_eq!(s.per_bulan.len(), 6);
		assert_eq!(s.per_bulan[1].saldo_akhir_bank, 45_000_000);
	}

	#[test]
	fn second_month_opening() {
		let input = SummaryInput { rows: &rows(), balances: &[], kode_names: &HashMap::new(), rekening_names: &HashMap::new() };
		let s = summarize(2026, 2, 2, None, &input);
		assert_eq!(s.saldo_awal_bank, 49_000_000);
		assert_eq!(s.saldo_akhir_bank, 45_000_000);
		assert_eq!(s.total_penerimaan, 0);
	}

	#[test]
	fn kelompok_mapping() {
		assert_eq!(kelompok_of("5.1.02.03.01.0001").1, "Belanja Pemeliharaan");
		assert_eq!(kelompok_of("5.2.02.10.01.0002").1, "Belanja Modal Peralatan dan Mesin");
		assert_eq!(kelompok_of("5.2.04.01").1, "Belanja Modal Lainnya");
		assert_eq!(program_of("05.02.08."), "05.");
	}
}
