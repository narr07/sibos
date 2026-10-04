//! Data laporan milik SIBOS: saldo rekening koran, register kas, dan pajak manual.

use std::collections::BTreeMap;

use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};

use super::AppDb;
use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BankStatement {
	pub bulan: u32,
	pub saldo: i64,
	pub keterangan: Option<String>,
}

/// Jumlah lembar/keping per pecahan, kunci = nilai pecahan dalam rupiah.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct Pecahan {
	pub kertas: BTreeMap<String, i64>,
	pub logam: BTreeMap<String, i64>,
}

impl Pecahan {
	pub fn total(&self) -> i64 {
		self.kertas.iter().chain(self.logam.iter()).map(|(nilai, jumlah)| nilai.parse::<i64>().unwrap_or(0) * jumlah).sum()
	}
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct CashRegister {
	pub pecahan: Pecahan,
	pub catatan: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManualTax {
	#[serde(default)]
	pub id: String,
	#[serde(default)]
	pub tahun: i32,
	#[serde(default)]
	pub sumber_dana: i64,
	pub tanggal: String,
	#[serde(default)]
	pub no_bukti: Option<String>,
	pub uraian: String,
	pub jenis_pajak: String,
	/// "pungut" atau "setor".
	pub arah: String,
	pub nominal: i64,
	#[serde(default)]
	pub keterangan: Option<String>,
	/// Kunci nota bila pajak diinput dari halaman kwitansi.
	#[serde(default)]
	pub ref_id: Option<String>,
}

impl ManualTax {
	pub fn validate(&self) -> AppResult<()> {
		let date_ok = self.tanggal.len() == 10 && self.tanggal.as_bytes()[4] == b'-' && self.tanggal.as_bytes()[7] == b'-';
		if !date_ok {
			return Err(AppError::InvalidInput("tanggal harus berformat YYYY-MM-DD".into()));
		}
		if self.uraian.trim().is_empty() {
			return Err(AppError::InvalidInput("uraian wajib diisi".into()));
		}
		if self.jenis_pajak.trim().is_empty() {
			return Err(AppError::InvalidInput("jenis pajak wajib diisi".into()));
		}
		if self.arah != "pungut" && self.arah != "setor" {
			return Err(AppError::InvalidInput("arah harus pungut atau setor".into()));
		}
		if self.nominal <= 0 {
			return Err(AppError::InvalidInput("nominal harus lebih dari 0".into()));
		}
		Ok(())
	}
}

impl AppDb {
	pub fn bank_statements(&self, year: i32, fund: i64) -> AppResult<Vec<BankStatement>> {
		let mut stmt = self.conn.prepare(
			"SELECT bulan, saldo, keterangan FROM bank_statement WHERE tahun = ?1 AND sumber_dana = ?2 ORDER BY bulan",
		)?;
		let rows = stmt
			.query_map(params![year, fund], |r| Ok(BankStatement { bulan: r.get(0)?, saldo: r.get(1)?, keterangan: r.get(2)? }))?
			.collect::<Result<Vec<_>, _>>()?;
		Ok(rows)
	}

	/// Simpan saldo rekening koran. `saldo = None` menghapus isian bulan itu.
	pub fn bank_statement_set(&self, year: i32, fund: i64, month: u32, saldo: Option<i64>, keterangan: Option<&str>) -> AppResult<()> {
		match saldo {
			None => {
				self.conn.execute(
					"DELETE FROM bank_statement WHERE tahun = ?1 AND sumber_dana = ?2 AND bulan = ?3",
					params![year, fund, month],
				)?;
			}
			Some(saldo) => {
				self.conn.execute(
					"INSERT INTO bank_statement (tahun, sumber_dana, bulan, saldo, keterangan) VALUES (?1, ?2, ?3, ?4, ?5)
					 ON CONFLICT(tahun, sumber_dana, bulan) DO UPDATE SET saldo = excluded.saldo,
					 keterangan = excluded.keterangan, updated_at = datetime('now')",
					params![year, fund, month, saldo, keterangan],
				)?;
			}
		}
		Ok(())
	}

	pub fn cash_register_get(&self, year: i32, month: u32, fund: i64) -> AppResult<Option<CashRegister>> {
		let row: Option<(String, Option<String>)> = self
			.conn
			.query_row(
				"SELECT pecahan, catatan FROM cash_register WHERE tahun = ?1 AND bulan = ?2 AND sumber_dana = ?3",
				params![year, month, fund],
				|r| Ok((r.get(0)?, r.get(1)?)),
			)
			.optional()?;
		Ok(row.map(|(json, catatan)| CashRegister { pecahan: serde_json::from_str(&json).unwrap_or_default(), catatan }))
	}

	pub fn cash_register_set(&self, year: i32, month: u32, fund: i64, value: &CashRegister) -> AppResult<()> {
		let json = serde_json::to_string(&value.pecahan).map_err(|e| AppError::InvalidInput(e.to_string()))?;
		self.conn.execute(
			"INSERT INTO cash_register (tahun, bulan, sumber_dana, pecahan, catatan) VALUES (?1, ?2, ?3, ?4, ?5)
			 ON CONFLICT(tahun, bulan, sumber_dana) DO UPDATE SET pecahan = excluded.pecahan,
			 catatan = excluded.catatan, updated_at = datetime('now')",
			params![year, month, fund, json, value.catatan],
		)?;
		Ok(())
	}

	pub fn manual_taxes(&self, year: i32) -> AppResult<Vec<ManualTax>> {
		let mut stmt = self.conn.prepare(
			"SELECT id, tahun, sumber_dana, tanggal, no_bukti, uraian, jenis_pajak, arah, nominal, keterangan, ref_id
			 FROM manual_tax WHERE tahun = ?1 ORDER BY tanggal, created_at",
		)?;
		let rows = stmt
			.query_map(params![year], |r| {
				Ok(ManualTax {
					id: r.get(0)?,
					tahun: r.get(1)?,
					sumber_dana: r.get(2)?,
					tanggal: r.get(3)?,
					no_bukti: r.get(4)?,
					uraian: r.get(5)?,
					jenis_pajak: r.get(6)?,
					arah: r.get(7)?,
					nominal: r.get(8)?,
					keterangan: r.get(9)?,
					ref_id: r.get(10)?,
				})
			})?
			.collect::<Result<Vec<_>, _>>()?;
		Ok(rows)
	}

	/// Tambah (id kosong) atau ubah entri pajak manual. Mengembalikan id.
	pub fn manual_tax_save(&self, entry: &ManualTax) -> AppResult<String> {
		entry.validate()?;
		if entry.id.is_empty() {
			Ok(self.conn.query_row(
				"INSERT INTO manual_tax (id, tahun, sumber_dana, tanggal, no_bukti, uraian, jenis_pajak, arah, nominal, keterangan, ref_id)
				 VALUES (lower(hex(randomblob(16))), ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10) RETURNING id",
				params![
					entry.tahun,
					entry.sumber_dana,
					entry.tanggal,
					entry.no_bukti,
					entry.uraian.trim(),
					entry.jenis_pajak.trim(),
					entry.arah,
					entry.nominal,
					entry.keterangan,
					entry.ref_id
				],
				|r| r.get(0),
			)?)
		} else {
			let changed = self.conn.execute(
				"UPDATE manual_tax SET tahun = ?2, sumber_dana = ?3, tanggal = ?4, no_bukti = ?5, uraian = ?6,
				 jenis_pajak = ?7, arah = ?8, nominal = ?9, keterangan = ?10, updated_at = datetime('now') WHERE id = ?1",
				params![
					entry.id,
					entry.tahun,
					entry.sumber_dana,
					entry.tanggal,
					entry.no_bukti,
					entry.uraian.trim(),
					entry.jenis_pajak.trim(),
					entry.arah,
					entry.nominal,
					entry.keterangan
				],
			)?;
			if changed == 0 {
				return Err(AppError::InvalidInput("entri pajak tidak ditemukan".into()));
			}
			Ok(entry.id.clone())
		}
	}

	/// Ganti semua pajak manual milik satu nota dalam satu transaksi.
	pub fn nota_tax_replace(&mut self, ref_id: &str, entries: &[ManualTax]) -> AppResult<()> {
		for e in entries {
			e.validate()?;
		}
		let tx = self.conn.transaction()?;
		tx.execute("DELETE FROM manual_tax WHERE ref_id = ?1", params![ref_id])?;
		for e in entries {
			tx.execute(
				"INSERT INTO manual_tax (id, tahun, sumber_dana, tanggal, no_bukti, uraian, jenis_pajak, arah, nominal, keterangan, ref_id)
				 VALUES (lower(hex(randomblob(16))), ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
				params![e.tahun, e.sumber_dana, e.tanggal, e.no_bukti, e.uraian.trim(), e.jenis_pajak.trim(), e.arah, e.nominal, e.keterangan, ref_id],
			)?;
		}
		tx.commit()?;
		Ok(())
	}

	pub fn manual_tax_delete(&self, id: &str) -> AppResult<()> {
		self.conn.execute("DELETE FROM manual_tax WHERE id = ?1", params![id])?;
		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	fn tax(arah: &str, nominal: i64) -> ManualTax {
		ManualTax {
			id: String::new(),
			tahun: 2026,
			sumber_dana: 1,
			tanggal: "2026-02-10".into(),
			no_bukti: None,
			uraian: "Hutang PPh 21 Desember".into(),
			jenis_pajak: "PPh 21".into(),
			arah: arah.into(),
			nominal,
			keterangan: None,
			ref_id: None,
		}
	}

	#[test]
	fn nota_tax_replace_keeps_other_entries() {
		let mut db = AppDb::open_in_memory().unwrap();
		db.manual_tax_save(&tax("pungut", 1_000)).unwrap();
		let mut a = tax("pungut", 25_000);
		a.jenis_pajak = "PPh 23".into();
		let mut b = tax("setor", 25_000);
		b.jenis_pajak = "PPh 23".into();
		db.nota_tax_replace("nota:N1", &[a.clone(), b]).unwrap();
		assert_eq!(db.manual_taxes(2026).unwrap().len(), 3);
		db.nota_tax_replace("nota:N1", &[a]).unwrap();
		let all = db.manual_taxes(2026).unwrap();
		assert_eq!(all.len(), 2);
		assert_eq!(all.iter().filter(|t| t.ref_id.as_deref() == Some("nota:N1")).count(), 1);
		db.nota_tax_replace("nota:N1", &[]).unwrap();
		assert_eq!(db.manual_taxes(2026).unwrap().len(), 1, "entri lain tidak terhapus");
	}

	#[test]
	fn manual_tax_crud_and_validation() {
		let db = AppDb::open_in_memory().unwrap();
		let id = db.manual_tax_save(&tax("pungut", 50_000)).unwrap();
		let mut saved = db.manual_taxes(2026).unwrap().remove(0);
		assert_eq!(saved.id, id);
		saved.nominal = 60_000;
		db.manual_tax_save(&saved).unwrap();
		assert_eq!(db.manual_taxes(2026).unwrap()[0].nominal, 60_000);
		assert!(db.manual_tax_save(&tax("pungut", 0)).is_err());
		assert!(db.manual_tax_save(&tax("lainnya", 10)).is_err());
		db.manual_tax_delete(&id).unwrap();
		assert!(db.manual_taxes(2026).unwrap().is_empty());
	}

	#[test]
	fn bank_statement_and_register() {
		let db = AppDb::open_in_memory().unwrap();
		db.bank_statement_set(2026, 1, 3, Some(35_288_950), None).unwrap();
		db.bank_statement_set(2026, 1, 3, Some(35_000_000), Some("koreksi")).unwrap();
		let rows = db.bank_statements(2026, 1).unwrap();
		assert_eq!((rows.len(), rows[0].saldo), (1, 35_000_000));
		db.bank_statement_set(2026, 1, 3, None, None).unwrap();
		assert!(db.bank_statements(2026, 1).unwrap().is_empty());

		let mut reg = CashRegister::default();
		reg.pecahan.kertas.insert("100000".into(), 3);
		reg.pecahan.logam.insert("500".into(), 4);
		assert_eq!(reg.pecahan.total(), 302_000);
		db.cash_register_set(2026, 3, 1, &reg).unwrap();
		assert_eq!(db.cash_register_get(2026, 3, 1).unwrap().unwrap().pecahan, reg.pecahan);
		assert!(db.cash_register_get(2026, 4, 1).unwrap().is_none());
	}
}
