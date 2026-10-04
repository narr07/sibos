//! Template dokumen buatan pengguna (SP, Nota, Kwitansi, BA, dll.) dan profil penyedia/toko.
//! Isi template disimpan sebagai JSON (daftar blok) dan dirender di frontend.

use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::AppDb;
use crate::error::{AppError, AppResult};

pub const MIGRATION: &str = "CREATE TABLE doc_template (
		id TEXT PRIMARY KEY,
		nama TEXT NOT NULL,
		jenis TEXT NOT NULL,
		toko_match TEXT,
		data TEXT NOT NULL,
		urutan INTEGER NOT NULL DEFAULT 0,
		created_at TEXT NOT NULL DEFAULT (datetime('now')),
		updated_at TEXT NOT NULL DEFAULT (datetime('now'))
	);
	CREATE TABLE penyedia (
		nama TEXT PRIMARY KEY,
		data TEXT NOT NULL,
		updated_at TEXT NOT NULL DEFAULT (datetime('now'))
	);";

/// Batas ukuran JSON (termasuk logo base64) agar database tetap ringan.
const MAX_JSON: usize = 2 * 1024 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocTemplate {
	#[serde(default)]
	pub id: String,
	pub nama: String,
	/// sp | nota | kwitansi | ba | bukti | lainnya
	pub jenis: String,
	/// Kata kunci nama toko; template dipakai otomatis untuk toko yang namanya memuat kata ini.
	#[serde(default)]
	pub toko_match: Option<String>,
	pub data: Value,
	#[serde(default)]
	pub urutan: i64,
	#[serde(default)]
	pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Penyedia {
	pub nama: String,
	pub data: Value,
	#[serde(default)]
	pub updated_at: String,
}

fn to_json(v: &Value) -> AppResult<String> {
	let s = v.to_string();
	if s.len() > MAX_JSON {
		return Err(AppError::InvalidInput("data terlalu besar (maks. 2 MB, perkecil logo)".into()));
	}
	Ok(s)
}

const JENIS: [&str; 6] = ["sp", "nota", "kwitansi", "ba", "bukti", "lainnya"];

impl AppDb {
	pub fn doc_templates(&self) -> AppResult<Vec<DocTemplate>> {
		let mut stmt = self.conn.prepare(
			"SELECT id, nama, jenis, toko_match, data, urutan, updated_at FROM doc_template ORDER BY urutan, jenis, nama",
		)?;
		let rows = stmt
			.query_map([], |r| {
				let data: String = r.get(4)?;
				Ok(DocTemplate {
					id: r.get(0)?,
					nama: r.get(1)?,
					jenis: r.get(2)?,
					toko_match: r.get(3)?,
					data: serde_json::from_str(&data).unwrap_or(Value::Null),
					urutan: r.get(5)?,
					updated_at: r.get(6)?,
				})
			})?
			.collect::<Result<Vec<_>, _>>()?;
		Ok(rows)
	}

	pub fn doc_template_save(&self, t: &DocTemplate) -> AppResult<String> {
		let nama = t.nama.trim();
		if nama.is_empty() || nama.chars().count() > 100 {
			return Err(AppError::InvalidInput("nama template wajib diisi (maks. 100 karakter)".into()));
		}
		if !JENIS.contains(&t.jenis.as_str()) {
			return Err(AppError::InvalidInput("jenis dokumen tidak dikenal".into()));
		}
		let data = to_json(&t.data)?;
		let toko = t.toko_match.as_deref().map(str::trim).filter(|s| !s.is_empty());
		if t.id.is_empty() {
			Ok(self.conn.query_row(
				"INSERT INTO doc_template (id, nama, jenis, toko_match, data, urutan)
				 VALUES (lower(hex(randomblob(16))), ?1, ?2, ?3, ?4, ?5) RETURNING id",
				params![nama, t.jenis, toko, data, t.urutan],
				|r| r.get(0),
			)?)
		} else {
			let n = self.conn.execute(
				"UPDATE doc_template SET nama = ?2, jenis = ?3, toko_match = ?4, data = ?5, urutan = ?6, updated_at = datetime('now')
				 WHERE id = ?1",
				params![t.id, nama, t.jenis, toko, data, t.urutan],
			)?;
			if n == 0 {
				return Err(AppError::InvalidInput("template tidak ditemukan".into()));
			}
			Ok(t.id.clone())
		}
	}

	pub fn doc_template_delete(&self, id: &str) -> AppResult<()> {
		self.conn.execute("DELETE FROM doc_template WHERE id = ?1", params![id])?;
		Ok(())
	}

	pub fn penyedia_list(&self) -> AppResult<Vec<Penyedia>> {
		let mut stmt = self.conn.prepare("SELECT nama, data, updated_at FROM penyedia ORDER BY nama")?;
		let rows = stmt
			.query_map([], |r| {
				let data: String = r.get(1)?;
				Ok(Penyedia { nama: r.get(0)?, data: serde_json::from_str(&data).unwrap_or(Value::Null), updated_at: r.get(2)? })
			})?
			.collect::<Result<Vec<_>, _>>()?;
		Ok(rows)
	}

	#[cfg_attr(not(test), allow(dead_code))]
	pub fn penyedia_get(&self, nama: &str) -> AppResult<Option<Penyedia>> {
		Ok(self
			.conn
			.query_row("SELECT nama, data, updated_at FROM penyedia WHERE nama = ?1", params![nama], |r| {
				let data: String = r.get(1)?;
				Ok(Penyedia { nama: r.get(0)?, data: serde_json::from_str(&data).unwrap_or(Value::Null), updated_at: r.get(2)? })
			})
			.optional()?)
	}

	pub fn penyedia_save(&self, p: &Penyedia) -> AppResult<()> {
		let nama = p.nama.trim();
		if nama.is_empty() || nama.chars().count() > 200 {
			return Err(AppError::InvalidInput("nama penyedia tidak valid".into()));
		}
		self.conn.execute(
			"INSERT INTO penyedia (nama, data) VALUES (?1, ?2)
			 ON CONFLICT(nama) DO UPDATE SET data = excluded.data, updated_at = datetime('now')",
			params![nama, to_json(&p.data)?],
		)?;
		Ok(())
	}

	pub fn penyedia_delete(&self, nama: &str) -> AppResult<()> {
		self.conn.execute("DELETE FROM penyedia WHERE nama = ?1", params![nama])?;
		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use serde_json::json;

	#[test]
	fn template_and_penyedia_crud() {
		let db = AppDb::open_in_memory().unwrap();
		let mut t = DocTemplate {
			id: String::new(),
			nama: "Nota KPRI".into(),
			jenis: "nota".into(),
			toko_match: Some(" KPR ".into()),
			data: json!({ "blocks": [{ "type": "judul", "teks": "NOTA" }] }),
			urutan: 0,
			updated_at: String::new(),
		};
		t.id = db.doc_template_save(&t).unwrap();
		let list = db.doc_templates().unwrap();
		assert_eq!(list.len(), 1);
		assert_eq!(list[0].toko_match.as_deref(), Some("KPR"));
		assert_eq!(list[0].data["blocks"][0]["teks"], "NOTA");
		t.nama = "Nota KPRI-KPR".into();
		db.doc_template_save(&t).unwrap();
		assert_eq!(db.doc_templates().unwrap()[0].nama, "Nota KPRI-KPR");
		assert!(db.doc_template_save(&DocTemplate { jenis: "aneh".into(), ..t.clone() }).is_err());
		db.doc_template_delete(&t.id).unwrap();
		assert!(db.doc_templates().unwrap().is_empty());

		let p = Penyedia { nama: "KPR RAJAGALUH".into(), data: json!({ "penanggungJawab": "Drs. H. Ahmad Kholid, M.M" }), updated_at: String::new() };
		db.penyedia_save(&p).unwrap();
		assert_eq!(db.penyedia_get("KPR RAJAGALUH").unwrap().unwrap().data["penanggungJawab"], "Drs. H. Ahmad Kholid, M.M");
		db.penyedia_delete("KPR RAJAGALUH").unwrap();
		assert!(db.penyedia_list().unwrap().is_empty());
	}
}
