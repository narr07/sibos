//! Draft RKAS lokal (Penganggaran RKAS). Disimpan di sibos.db, tidak pernah dikirim ke ARKAS.

use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};

use super::AppDb;
use crate::error::{AppError, AppResult};

pub const MIGRATION: &str = "CREATE TABLE rkas_draft (
		id TEXT PRIMARY KEY,
		tahun INTEGER NOT NULL,
		sumber_dana INTEGER NOT NULL,
		nama TEXT NOT NULL,
		jenis TEXT NOT NULL CHECK (jenis IN ('awal', 'perubahan', 'pergeseran')),
		dari TEXT,
		created_at TEXT NOT NULL DEFAULT (datetime('now')),
		updated_at TEXT NOT NULL DEFAULT (datetime('now'))
	);
	CREATE TABLE rkas_draft_item (
		id TEXT PRIMARY KEY,
		draft_id TEXT NOT NULL REFERENCES rkas_draft (id) ON DELETE CASCADE,
		urutan INTEGER NOT NULL DEFAULT 0,
		kode_kegiatan TEXT,
		kode_rekening TEXT,
		uraian TEXT NOT NULL,
		satuan TEXT,
		harga_satuan INTEGER NOT NULL DEFAULT 0,
		volume_bulan TEXT NOT NULL,
		sumber_id_rapbs TEXT
	);
	CREATE INDEX idx_rkas_draft_item_draft ON rkas_draft_item (draft_id);";

#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Draft {
	pub id: String,
	pub tahun: i32,
	pub sumber_dana: i64,
	pub nama: String,
	pub jenis: String,
	pub dari: Option<String>,
	pub total: i64,
	pub item_count: i64,
	pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DraftItem {
	#[serde(default)]
	pub id: String,
	#[serde(default)]
	pub urutan: i64,
	pub kode_kegiatan: Option<String>,
	pub kode_rekening: Option<String>,
	pub uraian: String,
	pub satuan: Option<String>,
	pub harga_satuan: i64,
	pub volume_bulan: [f64; 12],
	#[serde(default)]
	pub sumber_id_rapbs: Option<String>,
}

impl DraftItem {
	pub fn volume(&self) -> f64 {
		self.volume_bulan.iter().sum()
	}

	pub fn jumlah(&self) -> i64 {
		(self.volume() * self.harga_satuan as f64).round() as i64
	}

	pub fn validate(&self) -> AppResult<()> {
		if self.uraian.trim().is_empty() {
			return Err(AppError::InvalidInput("uraian item wajib diisi".into()));
		}
		if self.harga_satuan <= 0 {
			return Err(AppError::InvalidInput("harga satuan harus lebih dari 0".into()));
		}
		if self.volume_bulan.iter().any(|v| *v < 0.0 || !v.is_finite()) {
			return Err(AppError::InvalidInput("volume tidak boleh negatif".into()));
		}
		Ok(())
	}
}

fn total_sql() -> &'static str {
	"SELECT d.id, d.tahun, d.sumber_dana, d.nama, d.jenis, d.dari, d.updated_at,
	        (SELECT count(*) FROM rkas_draft_item i WHERE i.draft_id = d.id),
	        (SELECT group_concat(i.harga_satuan || '|' || i.volume_bulan, ';') FROM rkas_draft_item i WHERE i.draft_id = d.id)
	 FROM rkas_draft d"
}

fn parse_total(concat: Option<String>) -> i64 {
	concat
		.unwrap_or_default()
		.split(';')
		.filter_map(|part| {
			let (harga, vol) = part.split_once('|')?;
			let harga: f64 = harga.parse().ok()?;
			let vols: Vec<f64> = serde_json::from_str(vol).ok()?;
			Some((harga * vols.iter().sum::<f64>()).round() as i64)
		})
		.sum()
}

fn map_draft(r: &rusqlite::Row<'_>) -> rusqlite::Result<Draft> {
	Ok(Draft {
		id: r.get(0)?,
		tahun: r.get(1)?,
		sumber_dana: r.get(2)?,
		nama: r.get(3)?,
		jenis: r.get(4)?,
		dari: r.get(5)?,
		updated_at: r.get(6)?,
		item_count: r.get(7)?,
		total: parse_total(r.get(8)?),
	})
}

impl AppDb {
	pub fn drafts(&self, year: i32) -> AppResult<Vec<Draft>> {
		let mut stmt = self.conn.prepare(&format!("{} WHERE d.tahun = ?1 ORDER BY d.created_at", total_sql()))?;
		let rows = stmt.query_map(params![year], map_draft)?.collect::<Result<Vec<_>, _>>()?;
		Ok(rows)
	}

	pub fn draft_get(&self, id: &str) -> AppResult<Option<Draft>> {
		Ok(self.conn.query_row(&format!("{} WHERE d.id = ?1", total_sql()), params![id], map_draft).optional()?)
	}

	pub fn draft_items(&self, draft_id: &str) -> AppResult<Vec<DraftItem>> {
		let mut stmt = self.conn.prepare(
			"SELECT id, urutan, kode_kegiatan, kode_rekening, uraian, satuan, harga_satuan, volume_bulan, sumber_id_rapbs
			 FROM rkas_draft_item WHERE draft_id = ?1 ORDER BY kode_kegiatan, kode_rekening, urutan, id",
		)?;
		let rows = stmt
			.query_map(params![draft_id], |r| {
				let vol: String = r.get(7)?;
				Ok(DraftItem {
					id: r.get(0)?,
					urutan: r.get(1)?,
					kode_kegiatan: r.get(2)?,
					kode_rekening: r.get(3)?,
					uraian: r.get(4)?,
					satuan: r.get(5)?,
					harga_satuan: r.get(6)?,
					volume_bulan: serde_json::from_str(&vol).unwrap_or([0.0; 12]),
					sumber_id_rapbs: r.get(8)?,
				})
			})?
			.collect::<Result<Vec<_>, _>>()?;
		Ok(rows)
	}

	/// Buat draft baru beserta itemnya dalam satu transaksi.
	pub fn draft_create(&mut self, year: i32, fund: i64, nama: &str, jenis: &str, dari: Option<&str>, items: &[DraftItem]) -> AppResult<String> {
		if !["awal", "perubahan", "pergeseran"].contains(&jenis) {
			return Err(AppError::InvalidInput("jenis draft tidak dikenal".into()));
		}
		let tx = self.conn.transaction()?;
		let id: String = tx.query_row(
			"INSERT INTO rkas_draft (id, tahun, sumber_dana, nama, jenis, dari) VALUES (lower(hex(randomblob(16))), ?1, ?2, ?3, ?4, ?5) RETURNING id",
			params![year, fund, nama, jenis, dari],
			|r| r.get(0),
		)?;
		for (i, it) in items.iter().enumerate() {
			tx.execute(
				"INSERT INTO rkas_draft_item (id, draft_id, urutan, kode_kegiatan, kode_rekening, uraian, satuan, harga_satuan, volume_bulan, sumber_id_rapbs)
				 VALUES (lower(hex(randomblob(16))), ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
				params![
					id,
					i as i64,
					it.kode_kegiatan,
					it.kode_rekening,
					it.uraian,
					it.satuan,
					it.harga_satuan,
					serde_json::to_string(&it.volume_bulan).unwrap_or_else(|_| "[]".into()),
					it.sumber_id_rapbs
				],
			)?;
		}
		tx.commit()?;
		Ok(id)
	}

	pub fn draft_item_save(&self, draft_id: &str, item: &DraftItem) -> AppResult<String> {
		item.validate()?;
		let vol = serde_json::to_string(&item.volume_bulan).unwrap_or_else(|_| "[]".into());
		let id = if item.id.is_empty() {
			self.conn.query_row(
				"INSERT INTO rkas_draft_item (id, draft_id, urutan, kode_kegiatan, kode_rekening, uraian, satuan, harga_satuan, volume_bulan)
				 VALUES (lower(hex(randomblob(16))), ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8) RETURNING id",
				params![draft_id, item.urutan, item.kode_kegiatan, item.kode_rekening, item.uraian.trim(), item.satuan, item.harga_satuan, vol],
				|r| r.get(0),
			)?
		} else {
			let n = self.conn.execute(
				"UPDATE rkas_draft_item SET kode_kegiatan = ?3, kode_rekening = ?4, uraian = ?5, satuan = ?6, harga_satuan = ?7, volume_bulan = ?8
				 WHERE id = ?1 AND draft_id = ?2",
				params![item.id, draft_id, item.kode_kegiatan, item.kode_rekening, item.uraian.trim(), item.satuan, item.harga_satuan, vol],
			)?;
			if n == 0 {
				return Err(AppError::InvalidInput("item tidak ditemukan".into()));
			}
			item.id.clone()
		};
		self.conn.execute("UPDATE rkas_draft SET updated_at = datetime('now') WHERE id = ?1", params![draft_id])?;
		Ok(id)
	}

	pub fn draft_item_delete(&self, draft_id: &str, item_id: &str) -> AppResult<()> {
		self.conn.execute("DELETE FROM rkas_draft_item WHERE id = ?1 AND draft_id = ?2", params![item_id, draft_id])?;
		Ok(())
	}

	pub fn draft_rename(&self, id: &str, nama: &str) -> AppResult<()> {
		self.conn.execute("UPDATE rkas_draft SET nama = ?2, updated_at = datetime('now') WHERE id = ?1", params![id, nama])?;
		Ok(())
	}

	pub fn draft_delete(&self, id: &str) -> AppResult<()> {
		let children: i64 = self.conn.query_row("SELECT count(*) FROM rkas_draft WHERE dari = ?1", params![id], |r| r.get(0))?;
		if children > 0 {
			return Err(AppError::InvalidInput("hapus dulu versi turunan dari draft ini".into()));
		}
		self.conn.execute("DELETE FROM rkas_draft WHERE id = ?1", params![id])?;
		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	fn item(uraian: &str, harga: i64, jan: f64) -> DraftItem {
		let mut v = [0.0; 12];
		v[0] = jan;
		DraftItem {
			id: String::new(),
			urutan: 0,
			kode_kegiatan: Some("05.02.08.".into()),
			kode_rekening: Some("5.1.02.01.01.0024".into()),
			uraian: uraian.into(),
			satuan: Some("rim".into()),
			harga_satuan: harga,
			volume_bulan: v,
			sumber_id_rapbs: None,
		}
	}

	#[test]
	fn draft_versions_and_totals() {
		let mut db = AppDb::open_in_memory().unwrap();
		let awal = db.draft_create(2026, 1, "RKAS Awal", "awal", None, &[item("Kertas", 50_000, 10.0), item("Spidol", 5_000, 4.0)]).unwrap();
		let d = db.draft_get(&awal).unwrap().unwrap();
		assert_eq!((d.total, d.item_count), (520_000, 2));

		let items = db.draft_items(&awal).unwrap();
		let geser = db.draft_create(2026, 1, "Pergeseran 1", "pergeseran", Some(&awal), &items).unwrap();
		let mut first = db.draft_items(&geser).unwrap().remove(0);
		first.volume_bulan[0] = 9.0;
		db.draft_item_save(&geser, &first).unwrap();
		assert_eq!(db.draft_get(&geser).unwrap().unwrap().total, 470_000);
		assert_eq!(db.draft_get(&awal).unwrap().unwrap().total, 520_000, "draft asal tidak berubah");

		assert!(db.draft_delete(&awal).is_err(), "masih punya turunan");
		db.draft_delete(&geser).unwrap();
		db.draft_delete(&awal).unwrap();
		assert!(db.drafts(2026).unwrap().is_empty());
		assert!(db.draft_item_save("x", &item("", 1, 1.0)).is_err());
	}
}
