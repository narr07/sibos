//! Database milik SIBOS (`sibos.db`). Semua data buatan pengguna disimpan di sini,
//! tidak pernah di database ARKAS.

use std::collections::HashMap;
use std::path::Path;

use rusqlite::{params, Connection, OptionalExtension};
use serde_json::Value;

use crate::error::AppResult;

pub mod dokumen;
pub mod laporan;
pub mod nota;
pub mod rkas_draft;

/// Migrasi berurutan. Indeks + 1 = nilai `user_version` setelah migrasi dijalankan.
/// Jangan ubah migrasi yang sudah dirilis; tambahkan migrasi baru di akhir.
const MIGRATIONS: &[&str] = &[
	// 1: pengaturan umum (key-value JSON)
	"CREATE TABLE settings (
		key TEXT PRIMARY KEY,
		value TEXT NOT NULL,
		updated_at TEXT NOT NULL DEFAULT (datetime('now'))
	);",
	// 2: uraian tampilan yang diubah pengguna (tidak mengubah ARKAS)
	"CREATE TABLE uraian_override (
		id_kas_umum TEXT PRIMARY KEY,
		tahun INTEGER NOT NULL,
		uraian TEXT NOT NULL,
		created_at TEXT NOT NULL DEFAULT (datetime('now')),
		updated_at TEXT NOT NULL DEFAULT (datetime('now'))
	);
	CREATE INDEX idx_uraian_override_tahun ON uraian_override (tahun);",
	// 3: nota/kwitansi, laporan, dan pajak manual
	"CREATE TABLE bku_merge (
		id TEXT PRIMARY KEY,
		tahun INTEGER NOT NULL,
		no_bukti TEXT NOT NULL,
		uraian TEXT NOT NULL,
		tanggal TEXT,
		created_at TEXT NOT NULL DEFAULT (datetime('now')),
		updated_at TEXT NOT NULL DEFAULT (datetime('now'))
	);
	CREATE TABLE bku_merge_item (
		merge_id TEXT NOT NULL REFERENCES bku_merge (id) ON DELETE CASCADE,
		id_kas_umum TEXT NOT NULL,
		PRIMARY KEY (merge_id, id_kas_umum)
	);
	CREATE UNIQUE INDEX ux_bku_merge_item_kas ON bku_merge_item (id_kas_umum);
	CREATE TABLE print_status (
		kind TEXT NOT NULL,
		ref_id TEXT NOT NULL,
		printed_at TEXT NOT NULL DEFAULT (datetime('now')),
		PRIMARY KEY (kind, ref_id)
	);
	CREATE TABLE print_override (
		ref_id TEXT PRIMARY KEY,
		tanggal_nota TEXT,
		tanggal_bayar TEXT,
		uraian TEXT,
		keperluan TEXT,
		updated_at TEXT NOT NULL DEFAULT (datetime('now'))
	);
	CREATE TABLE nota_file (
		id TEXT PRIMARY KEY,
		tahun INTEGER NOT NULL,
		ref_id TEXT NOT NULL,
		file_name TEXT NOT NULL,
		mime TEXT NOT NULL,
		sha256 TEXT,
		created_at TEXT NOT NULL DEFAULT (datetime('now'))
	);
	CREATE INDEX idx_nota_file_ref ON nota_file (ref_id);
	CREATE TABLE bank_statement (
		tahun INTEGER NOT NULL,
		sumber_dana INTEGER NOT NULL DEFAULT 0,
		bulan INTEGER NOT NULL,
		saldo INTEGER NOT NULL,
		keterangan TEXT,
		updated_at TEXT NOT NULL DEFAULT (datetime('now')),
		PRIMARY KEY (tahun, sumber_dana, bulan)
	);
	CREATE TABLE cash_register (
		tahun INTEGER NOT NULL,
		bulan INTEGER NOT NULL,
		sumber_dana INTEGER NOT NULL DEFAULT 0,
		pecahan TEXT NOT NULL,
		catatan TEXT,
		updated_at TEXT NOT NULL DEFAULT (datetime('now')),
		PRIMARY KEY (tahun, bulan, sumber_dana)
	);
	CREATE TABLE manual_tax (
		id TEXT PRIMARY KEY,
		tahun INTEGER NOT NULL,
		sumber_dana INTEGER NOT NULL DEFAULT 0,
		tanggal TEXT NOT NULL,
		no_bukti TEXT,
		uraian TEXT NOT NULL,
		jenis_pajak TEXT NOT NULL,
		arah TEXT NOT NULL CHECK (arah IN ('pungut', 'setor')),
		nominal INTEGER NOT NULL CHECK (nominal > 0),
		keterangan TEXT,
		created_at TEXT NOT NULL DEFAULT (datetime('now')),
		updated_at TEXT NOT NULL DEFAULT (datetime('now'))
	);
	CREATE INDEX idx_manual_tax_tahun ON manual_tax (tahun);",
	// 4: draft RKAS lokal
	rkas_draft::MIGRATION,
	// 5: pajak manual per nota (kunci kelompok nota di kwitansi)
	"ALTER TABLE manual_tax ADD COLUMN ref_id TEXT;
	CREATE INDEX idx_manual_tax_ref ON manual_tax (ref_id);",
	// 6: nomor nota buatan sendiri untuk belanja non-SIPLah
	"ALTER TABLE print_override ADD COLUMN no_nota TEXT;",
	// 7: template dokumen & profil penyedia
	dokumen::MIGRATION,
];

pub struct AppDb {
	conn: Connection,
}

impl AppDb {
	pub fn open(path: &Path) -> AppResult<Self> {
		if let Some(dir) = path.parent() {
			std::fs::create_dir_all(dir)?;
		}
		let conn = Connection::open(path)?;
		Self::init(conn)
	}

	pub fn open_in_memory() -> AppResult<Self> {
		Self::init(Connection::open_in_memory()?)
	}

	fn init(conn: Connection) -> AppResult<Self> {
		conn.pragma_update(None, "journal_mode", "WAL")?;
		conn.pragma_update(None, "foreign_keys", true)?;
		let mut db = Self { conn };
		db.migrate()?;
		Ok(db)
	}

	fn migrate(&mut self) -> AppResult<()> {
		let current: usize = self.conn.query_row("PRAGMA user_version", [], |row| row.get::<_, i64>(0))? as usize;
		for (i, sql) in MIGRATIONS.iter().enumerate().skip(current) {
			let tx = self.conn.transaction()?;
			tx.execute_batch(sql)?;
			tx.pragma_update(None, "user_version", (i + 1) as i64)?;
			tx.commit()?;
		}
		Ok(())
	}

	/// Salinan konsisten sibos.db ke file baru (untuk backup).
	pub fn vacuum_into(&self, target: &Path) -> AppResult<()> {
		self.conn.execute("VACUUM INTO ?1", params![target.to_string_lossy()])?;
		Ok(())
	}

	pub fn schema_version(&self) -> AppResult<i64> {
		Ok(self.conn.query_row("PRAGMA user_version", [], |row| row.get(0))?)
	}

	pub fn setting_get(&self, key: &str) -> AppResult<Option<Value>> {
		let raw: Option<String> = self
			.conn
			.query_row("SELECT value FROM settings WHERE key = ?1", params![key], |row| row.get(0))
			.optional()?;
		Ok(raw.and_then(|s| serde_json::from_str(&s).ok()))
	}

	pub fn setting_set(&self, key: &str, value: &Value) -> AppResult<()> {
		self.conn.execute(
			"INSERT INTO settings (key, value, updated_at) VALUES (?1, ?2, datetime('now'))
			 ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
			params![key, value.to_string()],
		)?;
		Ok(())
	}

	pub fn uraian_overrides(&self, year: i32) -> AppResult<HashMap<String, String>> {
		let mut stmt = self.conn.prepare("SELECT id_kas_umum, uraian FROM uraian_override WHERE tahun = ?1")?;
		let rows = stmt
			.query_map(params![year], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)))?
			.collect::<Result<HashMap<_, _>, _>>()?;
		Ok(rows)
	}

	pub fn uraian_override_set(&self, year: i32, id_kas_umum: &str, uraian: &str) -> AppResult<()> {
		self.conn.execute(
			"INSERT INTO uraian_override (id_kas_umum, tahun, uraian) VALUES (?1, ?2, ?3)
			 ON CONFLICT(id_kas_umum) DO UPDATE SET uraian = excluded.uraian, tahun = excluded.tahun,
			 updated_at = datetime('now')",
			params![id_kas_umum, year, uraian],
		)?;
		Ok(())
	}

	pub fn uraian_override_delete(&self, id_kas_umum: &str) -> AppResult<()> {
		self.conn.execute("DELETE FROM uraian_override WHERE id_kas_umum = ?1", params![id_kas_umum])?;
		Ok(())
	}

	#[cfg_attr(not(test), allow(dead_code))]
	pub fn setting_remove(&self, key: &str) -> AppResult<()> {
		self.conn.execute("DELETE FROM settings WHERE key = ?1", params![key])?;
		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use serde_json::json;

	#[test]
	fn migrates_to_latest_version() {
		let db = AppDb::open_in_memory().unwrap();
		assert_eq!(db.schema_version().unwrap(), MIGRATIONS.len() as i64);
	}

	#[test]
	fn uraian_override_roundtrip() {
		let db = AppDb::open_in_memory().unwrap();
		db.uraian_override_set(2026, "abc", "Uraian baru").unwrap();
		db.uraian_override_set(2026, "abc", "Uraian revisi").unwrap();
		db.uraian_override_set(2025, "lama", "Tahun lalu").unwrap();
		let map = db.uraian_overrides(2026).unwrap();
		assert_eq!(map.len(), 1);
		assert_eq!(map["abc"], "Uraian revisi");
		db.uraian_override_delete("abc").unwrap();
		assert!(db.uraian_overrides(2026).unwrap().is_empty());
	}

	#[test]
	fn settings_roundtrip() {
		let db = AppDb::open_in_memory().unwrap();
		assert_eq!(db.setting_get("x").unwrap(), None);
		db.setting_set("x", &json!({ "a": 1 })).unwrap();
		db.setting_set("x", &json!({ "a": 2 })).unwrap();
		assert_eq!(db.setting_get("x").unwrap(), Some(json!({ "a": 2 })));
		db.setting_remove("x").unwrap();
		assert_eq!(db.setting_get("x").unwrap(), None);
	}
}
