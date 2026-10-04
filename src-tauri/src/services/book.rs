//! Penyusun Buku Kas Umum dan buku pembantu (bank, tunai, pajak) dari baris ARKAS.
//!
//! Saldo dihitung dari transaksi sepanjang tahun. Baris "Saldo Awal" ARKAS (kode 8/9/28/29)
//! dipakai sebagai saldo awal bila belum ada saldo untuk akun itu, dan sebagai titik cek
//! di bulan-bulan berikutnya. Saldo akhir tiap bulan dibandingkan dengan `aktivasi_bku`.

use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use super::classify::{classify, Account, Class, Kind, Pool};
use super::tax::jenis_pajak;
use crate::repo::arkas::bku::{KasRow, MonthBalance};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BookKind {
	Umum,
	Bank,
	Tunai,
	Pajak,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BookLine {
	pub id: String,
	pub parent_id: Option<String>,
	pub tanggal: String,
	pub kode_kegiatan: Option<String>,
	pub kode_rekening: Option<String>,
	pub no_bukti: Option<String>,
	pub uraian: String,
	pub uraian_asli: String,
	pub overridden: bool,
	pub jenis: Option<String>,
	pub jenis_pajak: Option<String>,
	pub fund_name: String,
	pub penerimaan: i64,
	pub pengeluaran: i64,
	pub saldo: i64,
	pub is_tax: bool,
	pub id_kas_nota: Option<String>,
	/// Belanja (atau pajaknya) lewat SIPLah.
	pub siplah: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MonthCheck {
	pub month: u32,
	pub fund_id: i64,
	pub fund_name: String,
	pub computed_bank: i64,
	pub computed_tunai: i64,
	pub arkas_bank: i64,
	pub arkas_tunai: i64,
	pub ok: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Book {
	pub kind: BookKind,
	pub year: i32,
	pub month: Option<u32>,
	pub fund: Option<i64>,
	pub opening: i64,
	pub total_penerimaan: i64,
	pub total_pengeluaran: i64,
	pub closing: i64,
	pub closing_bank: i64,
	pub closing_tunai: i64,
	pub lines: Vec<BookLine>,
	pub checks: Vec<MonthCheck>,
	pub warnings: Vec<String>,
}

type BalanceKey = (i64, Pool, Account);

/// Hasil pemrosesan satu baris sepanjang tahun.
struct Effect {
	idx: usize,
	month: u32,
	class: Class,
	/// Baris saldo awal yang dipakai sebagai saldo awal (bukan titik cek).
	initial_opening: bool,
	bank: i64,
	tunai: i64,
	tax: i64,
}

fn month_of(tanggal: &str) -> u32 {
	tanggal.get(5..7).and_then(|m| m.parse().ok()).unwrap_or(0)
}

fn rupiah(n: i64) -> String {
	let digits = n.unsigned_abs().to_string();
	let mut out = String::new();
	for (i, c) in digits.chars().enumerate() {
		if i > 0 && (digits.len() - i).is_multiple_of(3) {
			out.push('.');
		}
		out.push(c);
	}
	if n < 0 {
		format!("-Rp {out}")
	} else {
		format!("Rp {out}")
	}
}

const BULAN: [&str; 12] = [
	"Januari", "Februari", "Maret", "April", "Mei", "Juni", "Juli", "Agustus", "September", "Oktober", "November",
	"Desember",
];

pub fn month_name(month: u32) -> &'static str {
	BULAN.get(month.saturating_sub(1) as usize).copied().unwrap_or("")
}

/// Urutan baris: tanggal, saldo awal dulu, lalu transaksi; baris pajak langsung setelah induknya.
fn sort_rows(rows: &[KasRow], classes: &[Class]) -> Vec<usize> {
	let index: HashMap<&str, usize> = rows.iter().enumerate().map(|(i, r)| (r.id.as_str(), i)).collect();
	let key = |i: usize| {
		let r = &rows[i];
		let c = classes[i];
		if c.kind == Kind::Opening {
			return (r.tanggal.clone(), 0u8, r.create_date.clone(), r.id.clone(), 0u8);
		}
		if matches!(c.kind, Kind::TaxIn | Kind::TaxOut) {
			if let Some(p) = r.parent_id.as_deref().and_then(|p| index.get(p)) {
				let parent = &rows[*p];
				let order = if c.kind == Kind::TaxIn { 1 } else { 2 };
				return (r.tanggal.clone(), 1, parent.create_date.clone(), parent.id.clone(), order);
			}
		}
		(r.tanggal.clone(), 1, r.create_date.clone(), r.id.clone(), 0)
	};
	let mut order: Vec<usize> = (0..rows.len()).collect();
	order.sort_by_cached_key(|&i| key(i));
	order
}

/// Saldo per (sumber dana, pool, akun) di akhir setiap bulan 1-12.
type Snapshots = HashMap<u32, HashMap<BalanceKey, i64>>;

struct YearLedger {
	effects: Vec<Effect>,
	snapshots: Snapshots,
	/// (bulan, pesan) untuk baris saldo awal ARKAS yang tidak cocok dengan hitungan.
	checkpoint_warnings: Vec<(u32, String)>,
}

fn run_year(rows: &[KasRow]) -> YearLedger {
	let base: Vec<Class> = rows.iter().map(|r| classify(r.id_ref_bku)).collect();
	let index: HashMap<&str, usize> = rows.iter().enumerate().map(|(i, r)| (r.id.as_str(), i)).collect();
	let classes: Vec<Class> = rows
		.iter()
		.enumerate()
		.map(|(i, r)| {
			let parent = r.parent_id.as_deref().and_then(|p| index.get(p)).map(|&p| base[p]);
			base[i].with_tax_account(parent)
		})
		.collect();

	let mut balances: HashMap<BalanceKey, i64> = HashMap::new();
	let mut initialized: HashSet<BalanceKey> = HashSet::new();
	let mut snapshots: Snapshots = HashMap::new();
	let mut effects = Vec::with_capacity(rows.len());
	let mut checkpoint_warnings = Vec::new();
	let mut current_month = 1u32;

	for idx in sort_rows(rows, &classes) {
		let r = &rows[idx];
		let c = classes[idx];
		let month = month_of(&r.tanggal);
		while current_month < month && current_month <= 12 {
			snapshots.insert(current_month, balances.clone());
			current_month += 1;
		}

		let mut effect = Effect { idx, month, class: c, initial_opening: false, bank: 0, tunai: 0, tax: 0 };

		if let (Kind::Opening, Some(account)) = (c.kind, c.opening_account) {
			let key = (r.fund_id, c.pool, account);
			if initialized.insert(key) {
				*balances.entry(key).or_default() += r.saldo;
				effect.initial_opening = true;
				match account {
					Account::Bank => effect.bank = r.saldo,
					Account::Tunai => effect.tunai = r.saldo,
				}
			} else {
				let computed = balances.get(&key).copied().unwrap_or(0);
				if computed != r.saldo {
					let akun = if account == Account::Bank { "bank" } else { "tunai" };
					checkpoint_warnings.push((
						month,
						format!(
							"Saldo awal {akun} {} ({}) di ARKAS {}, hitungan SIBOS {}",
							r.tanggal,
							r.fund_name,
							rupiah(r.saldo),
							rupiah(computed)
						),
					));
				}
			}
		} else {
			effect.bank = i64::from(c.bank) * r.saldo;
			effect.tunai = i64::from(c.tunai) * r.saldo;
			effect.tax = match c.kind {
				Kind::TaxIn => r.saldo,
				Kind::TaxOut => -r.saldo,
				_ => 0,
			};
			for (account, delta) in [(Account::Bank, effect.bank), (Account::Tunai, effect.tunai)] {
				let key = (r.fund_id, c.pool, account);
				initialized.insert(key);
				if delta != 0 {
					*balances.entry(key).or_default() += delta;
				}
			}
		}
		effects.push(effect);
	}
	while current_month <= 12 {
		snapshots.insert(current_month, balances.clone());
		current_month += 1;
	}
	YearLedger { effects, snapshots, checkpoint_warnings }
}

/// Perubahan saldo buku `kind` akibat satu baris, dan nilai kolom (penerimaan, pengeluaran).
fn columns(kind: BookKind, e: &Effect, saldo: i64) -> Option<(i64, i64)> {
	let c = e.class;
	match kind {
		BookKind::Umum => match c.kind {
			Kind::Receipt | Kind::TaxIn => Some((saldo, 0)),
			Kind::Expense | Kind::TaxOut => Some((0, saldo)),
			Kind::Transfer => Some((saldo, saldo)),
			Kind::Opening | Kind::Unknown => None,
		},
		BookKind::Bank | BookKind::Tunai => {
			let (sign, delta) = if kind == BookKind::Bank { (c.bank, e.bank) } else { (c.tunai, e.tunai) };
			if c.kind == Kind::Opening || sign == 0 {
				None
			} else if delta >= 0 && sign > 0 {
				Some((delta, 0))
			} else {
				Some((0, -delta))
			}
		}
		BookKind::Pajak => match c.kind {
			Kind::TaxIn => Some((saldo, 0)),
			Kind::TaxOut => Some((0, saldo)),
			_ => None,
		},
	}
}

/// Pengaruh baris terhadap saldo buku `kind` (termasuk baris saldo awal yang dipakai).
fn balance_delta(kind: BookKind, e: &Effect) -> i64 {
	match kind {
		BookKind::Umum => e.bank + e.tunai,
		BookKind::Bank => e.bank,
		BookKind::Tunai => e.tunai,
		BookKind::Pajak => e.tax,
	}
}

pub struct BookRequest {
	pub kind: BookKind,
	pub year: i32,
	/// Bulan awal (atau satu-satunya bulan). `None` = setahun.
	pub month: Option<u32>,
	/// Bulan akhir untuk rentang (mis. semester). `None` = sama dengan `month`.
	pub until: Option<u32>,
	pub fund: Option<i64>,
}

impl BookRequest {
	pub fn range(&self) -> (u32, u32) {
		match (self.month, self.until) {
			(Some(m), Some(u)) => (m.min(u), m.max(u)),
			(Some(m), None) => (m, m),
			_ => (1, 12),
		}
	}
}

/// Entri pajak manual (diisi bendahara di SIBOS), hanya masuk Buku Pembantu Pajak.
#[derive(Debug, Clone)]
pub struct ManualTaxLine {
	pub id: String,
	pub tanggal: String,
	pub no_bukti: Option<String>,
	pub uraian: String,
	pub jenis_pajak: String,
	pub pungut: bool,
	pub nominal: i64,
	pub fund_id: i64,
	pub fund_name: String,
}

pub fn build_book(
	req: &BookRequest,
	all_rows: &[KasRow],
	balances: &[MonthBalance],
	uraian_overrides: &HashMap<String, String>,
) -> Book {
	build_book_with_manual(req, all_rows, balances, uraian_overrides, &[])
}

pub fn build_book_with_manual(
	req: &BookRequest,
	all_rows: &[KasRow],
	balances: &[MonthBalance],
	uraian_overrides: &HashMap<String, String>,
	manual: &[ManualTaxLine],
) -> Book {
	let rows: Vec<KasRow> = all_rows
		.iter()
		.filter(|r| req.fund.is_none_or(|f| r.fund_id == f))
		.cloned()
		.collect();
	let ledger = run_year(&rows);
	let index: HashMap<&str, usize> = rows.iter().enumerate().map(|(i, r)| (r.id.as_str(), i)).collect();

	let (start, end) = req.range();

	let mut opening = 0i64;
	let mut lines = Vec::new();
	let mut warnings = Vec::new();
	let mut running = 0i64;
	let mut started = false;

	// Saldo awal = semua perubahan sebelum periode + baris saldo awal yang dipakai di dalam periode.
	for e in &ledger.effects {
		if e.month < start || (e.month <= end && e.initial_opening) {
			opening += balance_delta(req.kind, e);
		}
	}

	for e in ledger.effects.iter().filter(|e| e.month >= start && e.month <= end && !e.initial_opening) {
		let r = &rows[e.idx];
		if e.class.kind == Kind::Unknown && req.kind == BookKind::Umum {
			warnings.push(format!("Kode transaksi {} belum dikenal: {} ({})", r.id_ref_bku, r.uraian, r.tanggal));
			continue;
		}
		let Some((penerimaan, pengeluaran)) = columns(req.kind, e, r.saldo) else {
			continue;
		};
		if !started {
			running = opening;
			started = true;
		}
		running += penerimaan - pengeluaran;

		let parent = r.parent_id.as_deref().and_then(|p| index.get(p)).map(|&p| &rows[p]);
		let is_tax = matches!(e.class.kind, Kind::TaxIn | Kind::TaxOut);
		let override_text = uraian_overrides.get(&r.id).filter(|s| !s.trim().is_empty());
		lines.push(BookLine {
			id: r.id.clone(),
			parent_id: r.parent_id.clone(),
			tanggal: r.tanggal.clone(),
			kode_kegiatan: r.kode_kegiatan.clone().or_else(|| parent.and_then(|p| p.kode_kegiatan.clone())),
			kode_rekening: r.kode_rekening.clone().or_else(|| parent.and_then(|p| p.kode_rekening.clone())),
			no_bukti: r.no_bukti.clone().or_else(|| parent.and_then(|p| p.no_bukti.clone())),
			uraian: override_text.cloned().unwrap_or_else(|| r.uraian.clone()),
			uraian_asli: r.uraian.clone(),
			overridden: override_text.is_some(),
			jenis: r.ref_bku_name.clone(),
			jenis_pajak: if is_tax { jenis_pajak(r).map(String::from) } else { None },
			fund_name: r.fund_name.clone(),
			penerimaan,
			pengeluaran,
			saldo: running,
			is_tax,
			id_kas_nota: r.id_kas_nota.clone().or_else(|| parent.and_then(|p| p.id_kas_nota.clone())),
			siplah: r.is_siplah || parent.is_some_and(|p| p.is_siplah),
		});
	}

	if req.kind == BookKind::Pajak && !manual.is_empty() {
		merge_manual_tax(&mut lines, &mut opening, manual, req.fund, start, end);
	}

	let total_penerimaan: i64 = lines.iter().map(|l| l.penerimaan).sum();
	let total_pengeluaran: i64 = lines.iter().map(|l| l.pengeluaran).sum();
	let closing = opening + total_penerimaan - total_pengeluaran;

	let end_snapshot = ledger.snapshots.get(&end).cloned().unwrap_or_default();
	let sum_account = |account: Account| -> i64 {
		end_snapshot.iter().filter(|((_, _, a), _)| *a == account).map(|(_, v)| *v).sum()
	};

	warnings.extend(
		ledger
			.checkpoint_warnings
			.iter()
			.filter(|(m, _)| *m >= start && *m <= end)
			.map(|(_, w)| w.clone()),
	);

	let fund_names: HashMap<i64, String> = rows.iter().map(|r| (r.fund_id, r.fund_name.clone())).collect();
	let checks = month_checks(&ledger.snapshots, balances, req.fund, start, end, &fund_names);

	Book {
		kind: req.kind,
		year: req.year,
		month: req.month,
		fund: req.fund,
		opening,
		total_penerimaan,
		total_pengeluaran,
		closing,
		closing_bank: sum_account(Account::Bank),
		closing_tunai: sum_account(Account::Tunai),
		lines,
		checks,
		warnings,
	}
}

/// Sisipkan entri pajak manual ke Buku Pembantu Pajak dan hitung ulang saldo berjalan.
fn merge_manual_tax(
	lines: &mut Vec<BookLine>,
	opening: &mut i64,
	manual: &[ManualTaxLine],
	fund: Option<i64>,
	start: u32,
	end: u32,
) {
	for m in manual.iter().filter(|m| fund.is_none_or(|f| f == m.fund_id || m.fund_id == 0)) {
		let month = month_of(&m.tanggal);
		let delta = if m.pungut { m.nominal } else { -m.nominal };
		if month < start {
			*opening += delta;
			continue;
		}
		if month > end {
			continue;
		}
		let line = BookLine {
			id: format!("manual:{}", m.id),
			parent_id: None,
			tanggal: m.tanggal.clone(),
			kode_kegiatan: None,
			kode_rekening: None,
			no_bukti: m.no_bukti.clone(),
			uraian: m.uraian.clone(),
			uraian_asli: m.uraian.clone(),
			overridden: false,
			jenis: Some("Pajak manual".into()),
			jenis_pajak: Some(m.jenis_pajak.clone()),
			fund_name: m.fund_name.clone(),
			penerimaan: if m.pungut { m.nominal } else { 0 },
			pengeluaran: if m.pungut { 0 } else { m.nominal },
			saldo: 0,
			is_tax: true,
			id_kas_nota: None,
			siplah: false,
		};
		// Setelah baris ARKAS bertanggal sama.
		let pos = lines.iter().position(|l| l.tanggal > m.tanggal).unwrap_or(lines.len());
		lines.insert(pos, line);
	}
	let mut running = *opening;
	for l in lines.iter_mut() {
		running += l.penerimaan - l.pengeluaran;
		l.saldo = running;
	}
}

fn month_checks(
	snapshots: &Snapshots,
	balances: &[MonthBalance],
	fund: Option<i64>,
	start: u32,
	end: u32,
	fund_names: &HashMap<i64, String>,
) -> Vec<MonthCheck> {
	let mut out = Vec::new();
	for b in balances {
		if b.month < start || b.month > end || !b.finished || fund.is_some_and(|f| f != b.fund_id) {
			continue;
		}
		// Sumber dana tanpa transaksi sama sekali tidak perlu dicek.
		let Some(name) = fund_names.get(&b.fund_id) else { continue };
		let (Some(bank), Some(tunai)) = (b.saldo_akhir_bank, b.saldo_akhir_tunai) else { continue };
		let snap = snapshots.get(&b.month);
		let get = |pool: Pool, account: Account| -> i64 {
			snap.and_then(|s| s.get(&(b.fund_id, pool, account))).copied().unwrap_or(0)
		};
		let bank_sisa = b.saldo_akhir_bank_sisa.unwrap_or(0);
		let tunai_sisa = b.saldo_akhir_tunai_sisa.unwrap_or(0);
		let ok = get(Pool::Reguler, Account::Bank) == bank
			&& get(Pool::Reguler, Account::Tunai) == tunai
			&& get(Pool::Sisa, Account::Bank) == bank_sisa
			&& get(Pool::Sisa, Account::Tunai) == tunai_sisa;
		out.push(MonthCheck {
			month: b.month,
			fund_id: b.fund_id,
			fund_name: name.clone(),
			computed_bank: get(Pool::Reguler, Account::Bank) + get(Pool::Sisa, Account::Bank),
			computed_tunai: get(Pool::Reguler, Account::Tunai) + get(Pool::Sisa, Account::Tunai),
			arkas_bank: bank + bank_sisa,
			arkas_tunai: tunai + tunai_sisa,
			ok,
		});
	}
	out.sort_by_key(|c| (c.month, c.fund_id));
	out
}

#[cfg(test)]
mod tests {
	use super::*;

	fn row(id: &str, tanggal: &str, ref_bku: i64, saldo: i64) -> KasRow {
		KasRow {
			id: id.into(),
			id_ref_bku: ref_bku,
			tanggal: tanggal.into(),
			uraian: format!("uraian {id}"),
			saldo,
			fund_id: 1,
			fund_name: "BOS Reguler".into(),
			create_date: format!("{tanggal} 08:00:00"),
			..Default::default()
		}
	}

	fn tax(id: &str, parent: &str, tanggal: &str, ref_bku: i64, saldo: i64) -> KasRow {
		KasRow { parent_id: Some(parent.into()), is_ppn: true, ..row(id, tanggal, ref_bku, saldo) }
	}

	/// Januari: terima 50 jt, belanja BNU 1 jt dengan PPN 99.099. Februari: belanja 500 rb.
	fn sample() -> Vec<KasRow> {
		vec![
			row("o1", "2026-01-01", 8, 0),
			row("o2", "2026-01-01", 9, 0),
			row("t1", "2026-01-10", 2, 50_000_000),
			KasRow { no_bukti: Some("BNU01".into()), kode_kegiatan: Some("05.02.08.".into()), ..row("b1", "2026-01-15", 15, 1_000_000) },
			tax("p1", "b1", "2026-01-15", 10, 99_099),
			tax("p2", "b1", "2026-01-15", 11, 99_099),
			row("o3", "2026-02-01", 8, 49_000_000),
			row("o4", "2026-02-01", 9, 0),
			row("b2", "2026-02-05", 15, 500_000),
		]
	}

	fn req(kind: BookKind, month: Option<u32>) -> BookRequest {
		BookRequest { kind, year: 2026, month, until: None, fund: None }
	}

	#[test]
	fn bku_umum_january() {
		let book = build_book(&req(BookKind::Umum, Some(1)), &sample(), &[], &HashMap::new());
		assert_eq!(book.opening, 0);
		assert_eq!(book.total_penerimaan, 50_099_099);
		assert_eq!(book.total_pengeluaran, 1_099_099);
		assert_eq!(book.closing, 49_000_000);
		assert_eq!(book.closing_bank, 49_000_000);
		assert_eq!(book.lines.len(), 4, "saldo awal tidak tampil sebagai baris");
		// Baris pajak mengikuti induk dan mewarisi no bukti & kode kegiatan.
		assert_eq!(book.lines[1].id, "b1");
		assert_eq!(book.lines[2].no_bukti.as_deref(), Some("BNU01"));
		assert_eq!(book.lines[2].kode_kegiatan.as_deref(), Some("05.02.08."));
		assert_eq!(book.lines[2].jenis_pajak.as_deref(), Some("PPN"));
		assert_eq!(book.lines.last().unwrap().saldo, 49_000_000);
		assert!(book.warnings.is_empty(), "{:?}", book.warnings);
	}

	#[test]
	fn february_opening_comes_from_january() {
		let book = build_book(&req(BookKind::Umum, Some(2)), &sample(), &[], &HashMap::new());
		assert_eq!(book.opening, 49_000_000);
		assert_eq!(book.closing, 48_500_000);
		assert_eq!(book.lines.len(), 1);
	}

	#[test]
	fn whole_year() {
		let book = build_book(&req(BookKind::Umum, None), &sample(), &[], &HashMap::new());
		assert_eq!(book.opening, 0);
		assert_eq!(book.closing, 48_500_000);
		assert_eq!(book.lines.len(), 5);
	}

	#[test]
	fn bank_and_tunai_books() {
		let mut rows = sample();
		rows.push(row("tt", "2026-02-10", 3, 2_000_000)); // tarik tunai
		rows.push(row("k1", "2026-02-12", 4, 300_000)); // belanja tunai
		rows.push(tax("k1p", "k1", "2026-02-12", 10, 15_000));
		rows.push(tax("k1s", "k1", "2026-02-12", 11, 15_000));

		let bank = build_book(&req(BookKind::Bank, Some(2)), &rows, &[], &HashMap::new());
		assert_eq!(bank.opening, 49_000_000);
		assert_eq!(bank.total_pengeluaran, 2_500_000);
		assert_eq!(bank.closing, 46_500_000);

		let tunai = build_book(&req(BookKind::Tunai, Some(2)), &rows, &[], &HashMap::new());
		assert_eq!(tunai.opening, 0);
		assert_eq!(tunai.total_penerimaan, 2_015_000);
		assert_eq!(tunai.total_pengeluaran, 315_000);
		assert_eq!(tunai.closing, 1_700_000);

		let umum = build_book(&req(BookKind::Umum, Some(2)), &rows, &[], &HashMap::new());
		assert_eq!(umum.closing, bank.closing + tunai.closing);
	}

	#[test]
	fn tax_book() {
		let book = build_book(&req(BookKind::Pajak, Some(1)), &sample(), &[], &HashMap::new());
		assert_eq!(book.lines.len(), 2);
		assert_eq!(book.total_penerimaan, 99_099);
		assert_eq!(book.total_pengeluaran, 99_099);
		assert_eq!(book.closing, 0);
	}

	#[test]
	fn checkpoint_mismatch_warns() {
		let mut rows = sample();
		rows[6].saldo = 48_000_000; // saldo awal Februari di ARKAS tidak cocok
		let book = build_book(&req(BookKind::Umum, Some(2)), &rows, &[], &HashMap::new());
		assert_eq!(book.warnings.len(), 1);
		assert!(book.warnings[0].contains("Rp 48.000.000"), "{}", book.warnings[0]);
		assert!(book.warnings[0].contains("Rp 49.000.000"));
	}

	#[test]
	fn compares_with_aktivasi_bku() {
		let balances = vec![
			MonthBalance { fund_id: 1, month: 1, finished: true, saldo_akhir_bank: Some(49_000_000), saldo_akhir_tunai: Some(0), ..Default::default() },
			MonthBalance { fund_id: 1, month: 2, finished: true, saldo_akhir_bank: Some(1), saldo_akhir_tunai: Some(0), ..Default::default() },
			MonthBalance { fund_id: 1, month: 3, finished: false, ..Default::default() },
		];
		let book = build_book(&req(BookKind::Umum, None), &sample(), &balances, &HashMap::new());
		assert_eq!(book.checks.len(), 2, "bulan belum ditutup tidak dicek");
		assert!(book.checks[0].ok);
		assert!(!book.checks[1].ok);
		assert_eq!(book.checks[1].computed_bank, 48_500_000);
	}

	#[test]
	fn fund_filter_and_override() {
		let mut rows = sample();
		rows.push(KasRow { fund_id: 2, fund_name: "Lainnya".into(), ..row("x1", "2026-01-20", 2, 7_000) });
		let mut overrides = HashMap::new();
		overrides.insert("b1".to_string(), "Belanja buku perpustakaan".to_string());

		let book = build_book(&BookRequest { fund: Some(1), ..req(BookKind::Umum, Some(1)) }, &rows, &[], &overrides);
		assert!(book.lines.iter().all(|l| l.fund_name == "BOS Reguler"));
		let b1 = book.lines.iter().find(|l| l.id == "b1").unwrap();
		assert!(b1.overridden);
		assert_eq!(b1.uraian, "Belanja buku perpustakaan");
		assert_eq!(b1.uraian_asli, "uraian b1");

		let all = build_book(&req(BookKind::Umum, Some(1)), &rows, &[], &HashMap::new());
		assert_eq!(all.total_penerimaan, 50_099_099 + 7_000);
	}

	fn manual_line(id: &str, tanggal: &str, pungut: bool) -> ManualTaxLine {
		ManualTaxLine {
			id: id.into(),
			tanggal: tanggal.into(),
			no_bukti: None,
			uraian: format!("pajak manual {id}"),
			jenis_pajak: "PPh 21".into(),
			pungut,
			nominal: 30_000,
			fund_id: 1,
			fund_name: "BOS Reguler".into(),
		}
	}

	#[test]
	fn manual_tax_in_tax_book() {
		let manual = vec![manual_line("m1", "2026-01-05", true), manual_line("m2", "2026-02-03", false)];
		let jan = build_book_with_manual(&req(BookKind::Pajak, Some(1)), &sample(), &[], &HashMap::new(), &manual);
		assert_eq!(jan.lines.len(), 3);
		assert_eq!(jan.lines[0].id, "manual:m1", "diurutkan menurut tanggal");
		assert_eq!(jan.closing, 30_000);
		let feb = build_book_with_manual(&req(BookKind::Pajak, Some(2)), &sample(), &[], &HashMap::new(), &manual);
		assert_eq!(feb.opening, 30_000);
		assert_eq!(feb.closing, 0);
		// Pajak manual tidak memengaruhi BKU umum.
		let umum = build_book_with_manual(&req(BookKind::Umum, Some(1)), &sample(), &[], &HashMap::new(), &manual);
		assert_eq!(umum.closing, 49_000_000);
	}

	#[test]
	fn semester_range() {
		let r = BookRequest { kind: BookKind::Umum, year: 2026, month: Some(1), until: Some(6), fund: None };
		assert_eq!(r.range(), (1, 6));
		let book = build_book(&r, &sample(), &[], &HashMap::new());
		assert_eq!(book.closing, 48_500_000);
	}

	#[test]
	fn formats_rupiah() {
		assert_eq!(rupiah(0), "Rp 0");
		assert_eq!(rupiah(1_234_567), "Rp 1.234.567");
		assert_eq!(rupiah(-5_000), "-Rp 5.000");
	}
}
