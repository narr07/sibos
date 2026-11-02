//! Toko dan produk SIPLah untuk pencocokan harga barang belanja.
//! Data toko dan produk diambil dari situs mitra SIPLah (Toko Ladang, Gramedia, Intan, Temprina,
//! Masmedia; lihat `crate::siplah`) dan disimpan di sibos.db.

use rusqlite::{params, OptionalExtension, Row};
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

/// Banyak platform SIPLah. Toko lama (sebelum migrasi) semuanya Toko Ladang: `kode` = ID merchant-nya.
/// Toko platform lain memakai ID internal negatif (lihat `crate::siplah::stable_id`).
pub const MIGRATION_PLATFORM: &str = "ALTER TABLE siplah_toko ADD COLUMN platform TEXT NOT NULL DEFAULT 'tokoladang';
ALTER TABLE siplah_toko ADD COLUMN kode TEXT;
UPDATE siplah_toko SET kode = CAST(id AS TEXT);
CREATE UNIQUE INDEX idx_siplah_toko_platform_kode ON siplah_toko (platform, kode);
ALTER TABLE siplah_produk ADD COLUMN harga_max INTEGER;
ALTER TABLE siplah_produk ADD COLUMN url TEXT;";

fn default_platform() -> String {
	"tokoladang".into()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SiplahToko {
	pub id: i64,
	/// Kode platform: "tokoladang", "gramedia", "intan", "temprina", "masmedia".
	#[serde(default = "default_platform")]
	pub platform: String,
	/// ID atau slug toko di situs asal.
	#[serde(default)]
	pub kode: Option<String>,
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
	/// Harga terendah (atau satu-satunya harga bila tidak berzona).
	pub harga: i64,
	/// Harga zona tertinggi; None bila produk hanya punya satu harga.
	#[serde(default)]
	pub harga_max: Option<i64>,
	#[serde(default)]
	pub stok: i64,
	#[serde(default)]
	pub terjual: i64,
	pub gambar: Option<String>,
	#[serde(default = "default_taxed")]
	pub kena_pajak: bool,
	/// Halaman produk di situs asal.
	#[serde(default)]
	pub url: Option<String>,
}

impl SiplahProduk {
	/// Harga untuk perbandingan dan perkiraan: harga zona tertinggi agar anggaran tidak kurang.
	pub fn harga_banding(&self) -> i64 {
		self.harga_max.unwrap_or(self.harga).max(self.harga)
	}
}

fn default_taxed() -> bool {
	true
}

const TOKO_COLUMNS: &str = "id, platform, kode, nama, slug, url, tipe, slogan, alamat, provinsi, kabupaten, kecamatan, kodepos,
	REPLACE(gambar, 'worker-siplah.tokoladang.co.id', 'siplah-oss.tokoladang.co.id'),
	total_produk, terakhir_sync, created_at, updated_at";

fn toko_from_row(r: &Row) -> rusqlite::Result<SiplahToko> {
	Ok(SiplahToko {
		id: r.get(0)?,
		platform: r.get(1)?,
		kode: r.get(2)?,
		nama: r.get(3)?,
		slug: r.get(4)?,
		url: r.get(5)?,
		tipe: r.get(6)?,
		slogan: r.get(7)?,
		alamat: r.get(8)?,
		provinsi: r.get(9)?,
		kabupaten: r.get(10)?,
		kecamatan: r.get(11)?,
		kodepos: r.get(12)?,
		gambar: r.get(13)?,
		total_produk: r.get(14)?,
		terakhir_sync: r.get(15)?,
		created_at: r.get(16)?,
		updated_at: r.get(17)?,
	})
}

impl AppDb {
	pub fn siplah_toko_list(&self) -> AppResult<Vec<SiplahToko>> {
		let mut stmt = self.conn.prepare(&format!("SELECT {TOKO_COLUMNS} FROM siplah_toko ORDER BY nama COLLATE NOCASE ASC"))?;
		let rows = stmt.query_map([], toko_from_row)?.collect::<Result<Vec<_>, _>>()?;
		Ok(rows)
	}

	pub fn siplah_toko_get(&self, id: i64) -> AppResult<Option<SiplahToko>> {
		let mut stmt = self.conn.prepare(&format!("SELECT {TOKO_COLUMNS} FROM siplah_toko WHERE id = ?1"))?;
		Ok(stmt.query_row(params![id], toko_from_row).optional()?)
	}

	pub fn siplah_toko_save(&self, t: &SiplahToko) -> AppResult<()> {
		self.conn.execute(
			"INSERT INTO siplah_toko (id, platform, kode, nama, slug, url, tipe, slogan, alamat, provinsi, kabupaten, kecamatan, kodepos, gambar, total_produk, terakhir_sync, updated_at)
			 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, datetime('now'))
			 ON CONFLICT(id) DO UPDATE SET
			   platform = excluded.platform,
			   kode = excluded.kode,
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
				t.platform,
				t.kode,
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
		let base = "SELECT id, toko_id, nama, harga, harga_max, stok, terjual,
		        REPLACE(gambar, 'worker-siplah.tokoladang.co.id', 'siplah-oss.tokoladang.co.id'), kena_pajak, url
		 FROM siplah_produk WHERE toko_id = ?1";
		let keyword = keyword.map(str::trim).filter(|k| !k.is_empty());
		let sql = match keyword {
			Some(_) => format!("{base} AND nama LIKE ?2 ORDER BY nama COLLATE NOCASE ASC"),
			None => format!("{base} ORDER BY nama COLLATE NOCASE ASC"),
		};
		let mut stmt = self.conn.prepare(&sql)?;
		let map = |r: &Row| {
			Ok(SiplahProduk {
				id: r.get(0)?,
				toko_id: r.get(1)?,
				nama: r.get(2)?,
				harga: r.get(3)?,
				harga_max: r.get(4)?,
				stok: r.get(5)?,
				terjual: r.get(6)?,
				gambar: r.get(7)?,
				kena_pajak: r.get::<_, i64>(8)? != 0,
				url: r.get(9)?,
			})
		};
		let rows = match keyword {
			Some(kw) => stmt.query_map(params![toko_id, format!("%{kw}%")], map)?.collect::<Result<Vec<_>, _>>()?,
			None => stmt.query_map(params![toko_id], map)?.collect::<Result<Vec<_>, _>>()?,
		};
		Ok(rows)
	}

	pub fn siplah_produk_save_batch(&mut self, toko_id: i64, produk_list: &[SiplahProduk]) -> AppResult<i64> {
		let tx = self.conn.transaction()?;
		tx.execute("DELETE FROM siplah_produk WHERE toko_id = ?1", params![toko_id])?;
		{
			// OR REPLACE: produk yang tampil di dua halaman (urutan berubah saat diambil) cukup disimpan sekali.
			let mut stmt = tx.prepare_cached(
				"INSERT OR REPLACE INTO siplah_produk (id, toko_id, nama, harga, harga_max, stok, terjual, gambar, kena_pajak, url, created_at, updated_at)
				 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, datetime('now'), datetime('now'))",
			)?;
			for p in produk_list {
				stmt.execute(params![
					p.id,
					toko_id,
					p.nama,
					p.harga,
					p.harga_max,
					p.stok,
					p.terjual,
					p.gambar,
					if p.kena_pajak { 1 } else { 0 },
					p.url,
				])?;
			}
		}
		let count: i64 = tx.query_row("SELECT count(*) FROM siplah_produk WHERE toko_id = ?1", params![toko_id], |r| r.get(0))?;
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

#[cfg(test)]
mod tests {
	use super::*;

	fn toko(id: i64, platform: &str, kode: &str) -> SiplahToko {
		SiplahToko {
			id,
			platform: platform.into(),
			kode: Some(kode.into()),
			nama: "Toko Uji".into(),
			slug: None,
			url: "https://contoh".into(),
			tipe: None,
			slogan: None,
			alamat: None,
			provinsi: None,
			kabupaten: None,
			kecamatan: None,
			kodepos: None,
			gambar: None,
			total_produk: 0,
			terakhir_sync: None,
			created_at: String::new(),
			updated_at: String::new(),
		}
	}

	fn produk(id: i64, toko_id: i64, harga: i64, harga_max: Option<i64>) -> SiplahProduk {
		SiplahProduk { id, toko_id, nama: format!("Produk {id}"), harga, harga_max, stok: 1, terjual: 0, gambar: None, kena_pajak: true, url: Some(format!("https://contoh/p/{id}")) }
	}

	#[test]
	fn platform_toko_and_zone_prices_roundtrip() {
		let mut db = AppDb::open_in_memory().unwrap();
		db.siplah_toko_save(&toko(-42, "gramedia", "abc123")).unwrap();
		let t = db.siplah_toko_get(-42).unwrap().unwrap();
		assert_eq!(t.platform, "gramedia");
		assert_eq!(t.kode.as_deref(), Some("abc123"));

		// Produk ganda (tampil di dua halaman) disimpan sekali; total mengikuti isi tabel.
		let list = [produk(-1, -42, 86_900, Some(181_100)), produk(-2, -42, 5_000, None), produk(-2, -42, 5_000, None)];
		assert_eq!(db.siplah_produk_save_batch(-42, &list).unwrap(), 2);
		let saved = db.siplah_produk_list(-42, None).unwrap();
		let zona = saved.iter().find(|p| p.id == -1).unwrap();
		assert_eq!((zona.harga, zona.harga_max), (86_900, Some(181_100)));
		assert_eq!(zona.harga_banding(), 181_100);
		assert_eq!(zona.url.as_deref(), Some("https://contoh/p/-1"));
		assert_eq!(saved.iter().find(|p| p.id == -2).unwrap().harga_banding(), 5_000);
		assert_eq!(db.siplah_produk_list(-42, Some("produk -2")).unwrap().len(), 1);
	}

	#[test]
	fn same_store_twice_is_rejected_by_platform_kode() {
		let db = AppDb::open_in_memory().unwrap();
		db.siplah_toko_save(&toko(-1, "intan", "toko-a")).unwrap();
		assert!(db.siplah_toko_save(&toko(-2, "intan", "toko-a")).is_err());
		// Kode sama di platform lain boleh.
		db.siplah_toko_save(&toko(-3, "temprina", "toko-a")).unwrap();
	}
}
