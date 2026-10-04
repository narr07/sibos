//! Data nota/kwitansi milik SIBOS: gabungan bukti, status cetak, kustomisasi cetak, foto nota.

use std::collections::HashMap;

use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};

use super::AppDb;
use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Merge {
	pub id: String,
	pub no_bukti: String,
	pub uraian: String,
	pub tanggal: Option<String>,
	pub items: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct PrintOverride {
	pub no_nota: Option<String>,
	pub tanggal_nota: Option<String>,
	pub tanggal_bayar: Option<String>,
	pub uraian: Option<String>,
	pub keperluan: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NotaFile {
	pub id: String,
	pub ref_id: String,
	pub file_name: String,
	pub mime: String,
	pub created_at: String,
}

fn clean(v: Option<String>) -> Option<String> {
	v.map(|s| s.trim().to_string()).filter(|s| !s.is_empty())
}

impl AppDb {
	pub fn merges(&self, year: i32) -> AppResult<Vec<Merge>> {
		let mut stmt = self.conn.prepare(
			"SELECT m.id, m.no_bukti, m.uraian, m.tanggal, i.id_kas_umum
			 FROM bku_merge m LEFT JOIN bku_merge_item i ON i.merge_id = m.id
			 WHERE m.tahun = ?1 ORDER BY m.created_at, m.id",
		)?;
		let mut out: Vec<Merge> = Vec::new();
		let rows = stmt.query_map(params![year], |r| {
			Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, String>(2)?, r.get::<_, Option<String>>(3)?, r.get::<_, Option<String>>(4)?))
		})?;
		for row in rows {
			let (id, no_bukti, uraian, tanggal, item) = row?;
			if out.last().is_none_or(|m| m.id != id) {
				out.push(Merge { id, no_bukti, uraian, tanggal, items: Vec::new() });
			}
			if let Some(item) = item {
				out.last_mut().expect("baru ditambahkan").items.push(item);
			}
		}
		Ok(out)
	}

	/// Simpan gabungan baru. Baris yang sudah tergabung di gabungan lain akan ditolak.
	pub fn merge_create(&mut self, year: i32, no_bukti: &str, uraian: &str, tanggal: Option<&str>, items: &[String]) -> AppResult<String> {
		if items.len() < 2 {
			return Err(AppError::InvalidInput("pilih minimal dua transaksi untuk digabung".into()));
		}
		let tx = self.conn.transaction()?;
		let id: String = tx.query_row(
			"INSERT INTO bku_merge (id, tahun, no_bukti, uraian, tanggal)
			 VALUES (lower(hex(randomblob(16))), ?1, ?2, ?3, ?4) RETURNING id",
			params![year, no_bukti, uraian, tanggal],
			|r| r.get(0),
		)?;
		for item in items {
			tx.execute("INSERT INTO bku_merge_item (merge_id, id_kas_umum) VALUES (?1, ?2)", params![id, item])
				.map_err(|e| match e.sqlite_error_code() {
					Some(rusqlite::ErrorCode::ConstraintViolation) => {
						AppError::InvalidInput("ada transaksi yang sudah masuk gabungan lain".into())
					}
					_ => AppError::Sqlite(e),
				})?;
		}
		tx.commit()?;
		Ok(id)
	}

	pub fn merge_update(&self, id: &str, no_bukti: &str, uraian: &str, tanggal: Option<&str>) -> AppResult<()> {
		self.conn.execute(
			"UPDATE bku_merge SET no_bukti = ?2, uraian = ?3, tanggal = ?4, updated_at = datetime('now') WHERE id = ?1",
			params![id, no_bukti, uraian, tanggal],
		)?;
		Ok(())
	}

	pub fn merge_delete(&self, id: &str) -> AppResult<()> {
		self.conn.execute("DELETE FROM bku_merge WHERE id = ?1", params![id])?;
		Ok(())
	}

	/// Status cetak: kunci (kind, ref_id) -> waktu cetak.
	pub fn print_statuses(&self) -> AppResult<HashMap<(String, String), String>> {
		let mut stmt = self.conn.prepare("SELECT kind, ref_id, printed_at FROM print_status")?;
		let rows = stmt
			.query_map([], |r| Ok(((r.get::<_, String>(0)?, r.get::<_, String>(1)?), r.get::<_, String>(2)?)))?
			.collect::<Result<HashMap<_, _>, _>>()?;
		Ok(rows)
	}

	pub fn print_status_set(&self, kind: &str, ref_ids: &[String], printed: bool) -> AppResult<()> {
		for ref_id in ref_ids {
			if printed {
				self.conn.execute(
					"INSERT INTO print_status (kind, ref_id) VALUES (?1, ?2)
					 ON CONFLICT(kind, ref_id) DO UPDATE SET printed_at = datetime('now')",
					params![kind, ref_id],
				)?;
			} else {
				self.conn.execute("DELETE FROM print_status WHERE kind = ?1 AND ref_id = ?2", params![kind, ref_id])?;
			}
		}
		Ok(())
	}

	pub fn print_overrides(&self) -> AppResult<HashMap<String, PrintOverride>> {
		let mut stmt = self.conn.prepare("SELECT ref_id, tanggal_nota, tanggal_bayar, uraian, keperluan, no_nota FROM print_override")?;
		let rows = stmt
			.query_map([], |r| {
				Ok((
					r.get::<_, String>(0)?,
					PrintOverride {
						tanggal_nota: r.get(1)?,
						tanggal_bayar: r.get(2)?,
						uraian: r.get(3)?,
						keperluan: r.get(4)?,
						no_nota: r.get(5)?,
					},
				))
			})?
			.collect::<Result<HashMap<_, _>, _>>()?;
		Ok(rows)
	}

	pub fn print_override_set(&self, ref_id: &str, value: &PrintOverride) -> AppResult<()> {
		let v = PrintOverride {
			no_nota: clean(value.no_nota.clone()),
			tanggal_nota: clean(value.tanggal_nota.clone()),
			tanggal_bayar: clean(value.tanggal_bayar.clone()),
			uraian: clean(value.uraian.clone()),
			keperluan: clean(value.keperluan.clone()),
		};
		if v.no_nota.is_none() && v.tanggal_nota.is_none() && v.tanggal_bayar.is_none() && v.uraian.is_none() && v.keperluan.is_none() {
			self.conn.execute("DELETE FROM print_override WHERE ref_id = ?1", params![ref_id])?;
			return Ok(());
		}
		self.conn.execute(
			"INSERT INTO print_override (ref_id, tanggal_nota, tanggal_bayar, uraian, keperluan, no_nota) VALUES (?1, ?2, ?3, ?4, ?5, ?6)
			 ON CONFLICT(ref_id) DO UPDATE SET tanggal_nota = excluded.tanggal_nota, tanggal_bayar = excluded.tanggal_bayar,
			 uraian = excluded.uraian, keperluan = excluded.keperluan, no_nota = excluded.no_nota, updated_at = datetime('now')",
			params![ref_id, v.tanggal_nota, v.tanggal_bayar, v.uraian, v.keperluan, v.no_nota],
		)?;
		Ok(())
	}

	pub fn nota_files(&self, year: i32) -> AppResult<Vec<NotaFile>> {
		let mut stmt = self.conn.prepare(
			"SELECT id, ref_id, file_name, mime, created_at FROM nota_file WHERE tahun = ?1 ORDER BY created_at",
		)?;
		let rows = stmt
			.query_map(params![year], |r| {
				Ok(NotaFile { id: r.get(0)?, ref_id: r.get(1)?, file_name: r.get(2)?, mime: r.get(3)?, created_at: r.get(4)? })
			})?
			.collect::<Result<Vec<_>, _>>()?;
		Ok(rows)
	}

	pub fn nota_file_get(&self, id: &str) -> AppResult<Option<(String, String, i32)>> {
		Ok(self
			.conn
			.query_row("SELECT file_name, mime, tahun FROM nota_file WHERE id = ?1", params![id], |r| {
				Ok((r.get(0)?, r.get(1)?, r.get(2)?))
			})
			.optional()?)
	}

	pub fn nota_file_add(&self, year: i32, ref_id: &str, file_name: &str, mime: &str, sha256: &str) -> AppResult<String> {
		Ok(self.conn.query_row(
			"INSERT INTO nota_file (id, tahun, ref_id, file_name, mime, sha256)
			 VALUES (lower(hex(randomblob(16))), ?1, ?2, ?3, ?4, ?5) RETURNING id",
			params![year, ref_id, file_name, mime, sha256],
			|r| r.get(0),
		)?)
	}

	pub fn nota_file_delete(&self, id: &str) -> AppResult<()> {
		self.conn.execute("DELETE FROM nota_file WHERE id = ?1", params![id])?;
		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn merge_lifecycle() {
		let mut db = AppDb::open_in_memory().unwrap();
		let items = vec!["a".to_string(), "b".to_string()];
		let id = db.merge_create(2026, "BNU01-G", "Belanja gabungan", None, &items).unwrap();
		let merges = db.merges(2026).unwrap();
		assert_eq!(merges.len(), 1);
		assert_eq!(merges[0].items, items);
		// Baris yang sama tidak boleh masuk gabungan lain.
		assert!(db.merge_create(2026, "X", "Y", None, &["a".into(), "c".into()]).is_err());
		assert_eq!(db.merges(2026).unwrap().len(), 1, "gabungan gagal tidak tersimpan");
		db.merge_update(&id, "BNU01-G2", "Revisi", Some("2026-01-02")).unwrap();
		assert_eq!(db.merges(2026).unwrap()[0].no_bukti, "BNU01-G2");
		db.merge_delete(&id).unwrap();
		assert!(db.merges(2026).unwrap().is_empty());
		assert!(db.merge_create(2026, "Z", "Z", None, &["a".into()]).is_err(), "minimal dua transaksi");
	}

	#[test]
	fn print_status_and_override() {
		let db = AppDb::open_in_memory().unwrap();
		db.print_status_set("a2", &["n1".into(), "n2".into()], true).unwrap();
		assert_eq!(db.print_statuses().unwrap().len(), 2);
		db.print_status_set("a2", &["n1".into()], false).unwrap();
		assert_eq!(db.print_statuses().unwrap().len(), 1);

		let ov = PrintOverride { uraian: Some(" Belanja ATK ".into()), ..Default::default() };
		db.print_override_set("n1", &ov).unwrap();
		assert_eq!(db.print_overrides().unwrap()["n1"].uraian.as_deref(), Some("Belanja ATK"));
		db.print_override_set("n1", &PrintOverride::default()).unwrap();
		assert!(db.print_overrides().unwrap().is_empty(), "isian kosong menghapus kustomisasi");
	}
}
