//! Klasifikasi baris `kas_umum` berdasarkan tabel resmi ARKAS `ref_bku`.
//!
//! | id | ref_bku                | id | ref_bku                     |
//! |----|------------------------|----|-----------------------------|
//! | 1  | Saldo Awal             | 9  | Saldo Awal Tunai            |
//! | 2  | Terima Dana BOS        | 10 | Pajak Belanja Terima        |
//! | 3  | Tarik Tunai            | 11 | Pajak Belanja Setor         |
//! | 4  | Kas Keluar (BPU)       | 12 | Pergeseran Tunai            |
//! | 5  | Setor Tunai            | 13 | Pergeseran Setor            |
//! | 6  | Bunga Bank             | 14 | Pengembalian Dana BOS       |
//! | 7  | Pajak Bunga            | 15 | Kas Keluar Non Tunai (BNU)  |
//! | 8  | Saldo Awal Bank        |    |                             |
//!
//! Kode 23-35 adalah versi "Sisa" (dana tahun lalu) dari kode 3-15 (id - 20).

use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Pool {
	Reguler,
	Sisa,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Account {
	Bank,
	Tunai,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
	/// Saldo awal bulan/tahun: bukan transaksi, dipakai sebagai saldo awal atau titik cek.
	Opening,
	Receipt,
	Expense,
	/// Perpindahan uang antara bank dan tunai (tidak mengubah total).
	Transfer,
	/// Pajak dipungut (masuk), akun mengikuti transaksi induk.
	TaxIn,
	/// Pajak disetor (keluar), akun mengikuti transaksi induk.
	TaxOut,
	Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Class {
	pub kind: Kind,
	pub pool: Pool,
	/// Arah perubahan saldo bank: +1 masuk, -1 keluar, 0 tidak berubah.
	pub bank: i8,
	/// Arah perubahan saldo tunai.
	pub tunai: i8,
	/// Akun untuk baris saldo awal.
	pub opening_account: Option<Account>,
}

impl Class {
	const fn new(kind: Kind, pool: Pool, bank: i8, tunai: i8) -> Self {
		Self { kind, pool, bank, tunai, opening_account: None }
	}

	const fn opening(pool: Pool, account: Account) -> Self {
		Self { kind: Kind::Opening, pool, bank: 0, tunai: 0, opening_account: Some(account) }
	}

	/// Pemetaan pajak ke akun transaksi induknya (bank untuk BNU, tunai untuk BPU).
	pub fn with_tax_account(mut self, parent: Option<Class>) -> Self {
		let sign = match self.kind {
			Kind::TaxIn => 1,
			Kind::TaxOut => -1,
			_ => return self,
		};
		let tunai_parent = parent.is_some_and(|p| p.tunai < 0 && p.bank == 0);
		if tunai_parent {
			self.tunai = sign;
		} else {
			self.bank = sign;
		}
		self
	}
}

pub fn classify(id_ref_bku: i64) -> Class {
	let (base, pool) = if (23..=35).contains(&id_ref_bku) {
		(id_ref_bku - 20, Pool::Sisa)
	} else {
		(id_ref_bku, Pool::Reguler)
	};
	match base {
		1 | 8 => Class::opening(pool, Account::Bank),
		9 => Class::opening(pool, Account::Tunai),
		2 | 6 => Class::new(Kind::Receipt, pool, 1, 0),
		3 => Class::new(Kind::Transfer, pool, -1, 1),
		4 => Class::new(Kind::Expense, pool, 0, -1),
		5 => Class::new(Kind::Transfer, pool, 1, -1),
		7 | 14 | 15 => Class::new(Kind::Expense, pool, -1, 0),
		10 => Class::new(Kind::TaxIn, pool, 0, 0),
		11 => Class::new(Kind::TaxOut, pool, 0, 0),
		// Pergeseran: belum ada contoh data; diasumsikan sama arah dengan tarik/setor tunai.
		12 => Class::new(Kind::Transfer, pool, -1, 1),
		13 => Class::new(Kind::Transfer, pool, 1, -1),
		_ => Class::new(Kind::Unknown, pool, 0, 0),
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn opening_rows() {
		assert_eq!(classify(8).opening_account, Some(Account::Bank));
		assert_eq!(classify(9).opening_account, Some(Account::Tunai));
		assert_eq!(classify(28).pool, Pool::Sisa);
		assert_eq!(classify(29).opening_account, Some(Account::Tunai));
	}

	#[test]
	fn receipts_and_expenses() {
		let terima = classify(2);
		assert_eq!((terima.kind, terima.bank, terima.tunai), (Kind::Receipt, 1, 0));
		let bpu = classify(4);
		assert_eq!((bpu.kind, bpu.bank, bpu.tunai), (Kind::Expense, 0, -1));
		let bnu = classify(15);
		assert_eq!((bnu.kind, bnu.bank, bnu.tunai), (Kind::Expense, -1, 0));
		let bnu_sisa = classify(35);
		assert_eq!((bnu_sisa.kind, bnu_sisa.pool, bnu_sisa.bank), (Kind::Expense, Pool::Sisa, -1));
		assert_eq!(classify(14).kind, Kind::Expense);
		assert_eq!(classify(6).bank, 1);
		assert_eq!(classify(7).bank, -1);
	}

	#[test]
	fn transfers_move_between_accounts() {
		let tarik = classify(3);
		assert_eq!((tarik.kind, tarik.bank, tarik.tunai), (Kind::Transfer, -1, 1));
		let setor = classify(5);
		assert_eq!((setor.kind, setor.bank, setor.tunai), (Kind::Transfer, 1, -1));
	}

	#[test]
	fn tax_follows_parent_account() {
		let in_bank = classify(10).with_tax_account(Some(classify(15)));
		assert_eq!((in_bank.bank, in_bank.tunai), (1, 0));
		let out_bank = classify(11).with_tax_account(Some(classify(15)));
		assert_eq!((out_bank.bank, out_bank.tunai), (-1, 0));
		let in_cash = classify(10).with_tax_account(Some(classify(4)));
		assert_eq!((in_cash.bank, in_cash.tunai), (0, 1));
		let orphan = classify(11).with_tax_account(None);
		assert_eq!((orphan.bank, orphan.tunai), (-1, 0));
	}

	#[test]
	fn unknown_code() {
		assert_eq!(classify(99).kind, Kind::Unknown);
	}
}
