//! Toko dan produk SIPLah untuk pencocokan harga barang belanja.
//! Data toko dan produk discrape dari SIPLah (Toko Ladang dsb.) dan disimpan di sibos.db.

use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};

use super::AppDb;
use crate::error::AppResult;

pub const MIGRATION: &str = "CREATE TABLE siplah_toko (
	id INTEGER PRIMARY KEY,
	nama TEXT NOT NULL,
	slug TEXT,
	url TEXT NOT NULL,
	tipe TEXT,
	slogan TEXT,
	alamat TEXT,
	provinsi TEXT,
	kabupaten TEXT,
	kecamatan TEXT,
	kodepos TEXT,
	gambar TEXT,
	total_produk INTEGER NOT NULL DEFAULT 0,
	terakhir_sync TEXT,
	created_at TEXT NOT NULL DEFAULT (datetime('now')),
	updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE TABLE siplah_produk (
	id INTEGER PRIMARY KEY,
	toko_id INTEGER NOT NULL REFERENCES siplah_toko (id) ON DELETE CASCADE,
	nama TEXT NOT NULL,
	harga INTEGER NOT NULL,
	stok INTEGER NOT NULL DEFAULT 0,
	terjual INTEGER NOT NULL DEFAULT 0,
	gambar TEXT,
	kena_pajak INTEGER NOT NULL DEFAULT 1,
	created_at TEXT NOT NULL DEFAULT (datetime('now')),
	updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX idx_siplah_produk_toko ON siplah_produk (toko_id);
CREATE INDEX idx_siplah_produk_nama ON siplah_produk (nama);";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SiplahToko {
	pub id: i64,
	pub nama: String,
	pub slug: Option<String>,
	pub url: String,
	pub tipe: Option<String>,
	pub slogan: Option<String>,
	pub alamat: Option<String>,
	pub provinsi: Option<String>,
	pub kabupaten: Option<String>,
	pub kecamatan: Option<String>,
	pub kodepos: Option<String>,
	pub gambar: Option<String>,
	#[serde(default)]
	pub total_produk: i64,
	pub terakhir_sync: Option<String>,
	#[serde(default)]
	pub created_at: String,
	#[serde(default)]
	pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SiplahProduk {
	pub id: i64,
	pub toko_id: i64,
	pub nama: String,
	pub harga: i64,
	#[serde(default)]
	pub stok: i64,
	#[serde(default)]
	pub terjual: i64,
	pub gambar: Option<String>,
	#[serde(default = "default_taxed")]
	pub kena_pajak: bool,
}

fn default_taxed() -> bool {
	true
}

impl AppDb {
	pub fn siplah_toko_list(&self) -> AppResult<Vec<SiplahToko>> {
		let mut stmt = self.conn.prepare(
			"SELECT id, nama, slug, url, tipe, slogan, alamat, provinsi, kabupaten, kecamatan, kodepos,
			        REPLACE(gambar, 'worker-siplah.tokoladang.co.id', 'siplah-oss.tokoladang.co.id'),
			        total_produk, terakhir_sync, created_at, updated_at
			 FROM siplah_toko
			 ORDER BY nama COLLATE NOCASE ASC",
		)?;
		let rows = stmt
			.query_map([], |r| {
				Ok(SiplahToko {
					id: r.get(0)?,
					nama: r.get(1)?,
					slug: r.get(2)?,
					url: r.get(3)?,
					tipe: r.get(4)?,
					slogan: r.get(5)?,
					alamat: r.get(6)?,
					provinsi: r.get(7)?,
					kabupaten: r.get(8)?,
					kecamatan: r.get(9)?,
					kodepos: r.get(10)?,
					gambar: r.get(11)?,
					total_produk: r.get(12)?,
					terakhir_sync: r.get(13)?,
					created_at: r.get(14)?,
					updated_at: r.get(15)?,
				})
			})?
			.collect::<Result<Vec<_>, _>>()?;
		Ok(rows)
	}

	pub fn siplah_toko_get(&self, id: i64) -> AppResult<Option<SiplahToko>> {
		let mut stmt = self.conn.prepare(
			"SELECT id, nama, slug, url, tipe, slogan, alamat, provinsi, kabupaten, kecamatan, kodepos,
			        REPLACE(gambar, 'worker-siplah.tokoladang.co.id', 'siplah-oss.tokoladang.co.id'),
			        total_produk, terakhir_sync, created_at, updated_at
			 FROM siplah_toko WHERE id = ?1",
		)?;
		let toko = stmt
			.query_row(params![id], |r| {
				Ok(SiplahToko {
					id: r.get(0)?,
					nama: r.get(1)?,
					slug: r.get(2)?,
					url: r.get(3)?,
					tipe: r.get(4)?,
					slogan: r.get(5)?,
					alamat: r.get(6)?,
					provinsi: r.get(7)?,
					kabupaten: r.get(8)?,
					kecamatan: r.get(9)?,
					kodepos: r.get(10)?,
					gambar: r.get(11)?,
					total_produk: r.get(12)?,
					terakhir_sync: r.get(13)?,
					created_at: r.get(14)?,
					updated_at: r.get(15)?,
				})
			})
			.optional()?;
		Ok(toko)
	}

	pub fn siplah_toko_save(&self, t: &SiplahToko) -> AppResult<()> {
		self.conn.execute(
			"INSERT INTO siplah_toko (id, nama, slug, url, tipe, slogan, alamat, provinsi, kabupaten, kecamatan, kodepos, gambar, total_produk, terakhir_sync, updated_at)
			 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, datetime('now'))
			 ON CONFLICT(id) DO UPDATE SET
			   nama = excluded.nama,
			   slug = excluded.slug,
			   url = excluded.url,
			   tipe = excluded.tipe,
			   slogan = excluded.slogan,
			   alamat = excluded.alamat,
			   provinsi = excluded.provinsi,
			   kabupaten = excluded.kabupaten,
			   kecamatan = excluded.kecamatan,
			   kodepos = excluded.kodepos,
			   gambar = excluded.gambar,
			   total_produk = CASE WHEN excluded.total_produk > 0 THEN excluded.total_produk ELSE siplah_toko.total_produk END,
			   terakhir_sync = COALESCE(excluded.terakhir_sync, siplah_toko.terakhir_sync),
			   updated_at = datetime('now')",
			params![
				t.id,
				t.nama,
				t.slug,
				t.url,
				t.tipe,
				t.slogan,
				t.alamat,
				t.provinsi,
				t.kabupaten,
				t.kecamatan,
				t.kodepos,
				t.gambar,
				t.total_produk,
				t.terakhir_sync,
			],
		)?;
		Ok(())
	}

	pub fn siplah_toko_delete(&self, id: i64) -> AppResult<()> {
		self.conn.execute("DELETE FROM siplah_toko WHERE id = ?1", params![id])?;
		Ok(())
	}

	pub fn siplah_produk_list(&self, toko_id: i64, keyword: Option<&str>) -> AppResult<Vec<SiplahProduk>> {
		let (sql, params_vec): (String, Vec<Box<dyn rusqlite::ToSql>>) = match keyword.map(str::trim).filter(|k| !k.is_empty()) {
			Some(kw) => (
				"SELECT id, toko_id, nama, harga, stok, terjual,
				        REPLACE(gambar, 'worker-siplah.tokoladang.co.id', 'siplah-oss.tokoladang.co.id'), kena_pajak
				 FROM siplah_produk
				 WHERE toko_id = ?1 AND nama LIKE ?2
				 ORDER BY nama COLLATE NOCASE ASC".into(),
				vec![Box::new(toko_id), Box::new(format!("%{kw}%"))],
			),
			None => (
				"SELECT id, toko_id, nama, harga, stok, terjual,
				        REPLACE(gambar, 'worker-siplah.tokoladang.co.id', 'siplah-oss.tokoladang.co.id'), kena_pajak
				 FROM siplah_produk
				 WHERE toko_id = ?1
				 ORDER BY nama COLLATE NOCASE ASC".into(),
				vec![Box::new(toko_id)],
			),
		};

		let mut stmt = self.conn.prepare(&sql)?;
		let param_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|b| b.as_ref()).collect();
		let rows = stmt
			.query_map(param_refs.as_slice(), |r| {
				Ok(SiplahProduk {
					id: r.get(0)?,
					toko_id: r.get(1)?,
					nama: r.get(2)?,
					harga: r.get(3)?,
					stok: r.get(4)?,
					terjual: r.get(5)?,
					gambar: r.get(6)?,
					kena_pajak: r.get::<_, i64>(7)? != 0,
				})
			})?
			.collect::<Result<Vec<_>, _>>()?;
		Ok(rows)
	}

	pub fn siplah_produk_save_batch(&mut self, toko_id: i64, produk_list: &[SiplahProduk]) -> AppResult<i64> {
		let tx = self.conn.transaction()?;
		tx.execute("DELETE FROM siplah_produk WHERE toko_id = ?1", params![toko_id])?;
		{
			let mut stmt = tx.prepare_cached(
				"INSERT INTO siplah_produk (id, toko_id, nama, harga, stok, terjual, gambar, kena_pajak, created_at, updated_at)
				 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, datetime('now'), datetime('now'))",
			)?;
			for p in produk_list {
				stmt.execute(params![
					p.id,
					toko_id,
					p.nama,
					p.harga,
					p.stok,
					p.terjual,
					p.gambar,
					if p.kena_pajak { 1 } else { 0 },
				])?;
			}
		}
		let count = produk_list.len() as i64;
		tx.execute(
			"UPDATE siplah_toko
			 SET total_produk = ?2, terakhir_sync = datetime('now'), updated_at = datetime('now')
			 WHERE id = ?1",
			params![toko_id, count],
		)?;
		tx.commit()?;
		Ok(count)
	}
}
