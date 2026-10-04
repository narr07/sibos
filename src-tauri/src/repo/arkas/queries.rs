//! Query baca ke database ARKAS untuk data dasar: tahun, sumber dana, sekolah, skema.
//! HANYA BACA, lihat `conn.rs`.

use std::collections::BTreeMap;

use rusqlite::types::ValueRef;
use rusqlite::{params, OptionalExtension};
use serde::Serialize;

use super::conn::ArkasDb;
use crate::error::AppResult;

#[derive(Debug, Serialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FundSource {
	pub id: i64,
	pub name: String,
}

#[derive(Debug, Serialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct SchoolInfo {
	pub sekolah_id: Option<String>,
	pub npsn: Option<String>,
	pub nama: Option<String>,
	pub alamat: Option<String>,
	pub kepala_sekolah: Option<String>,
	pub nip_kepala_sekolah: Option<String>,
	pub bendahara: Option<String>,
	pub nip_bendahara: Option<String>,
	pub telepon: Option<String>,
	pub kecamatan: Option<String>,
	pub kabupaten: Option<String>,
	pub provinsi: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TableInfo {
	pub name: String,
	pub columns: Vec<String>,
	pub row_count: i64,
}

fn value_to_string(value: ValueRef<'_>) -> Option<String> {
	match value {
		ValueRef::Null => None,
		ValueRef::Integer(i) => Some(i.to_string()),
		ValueRef::Real(f) => Some(f.to_string()),
		ValueRef::Text(t) => Some(String::from_utf8_lossy(t).trim().to_string()),
		ValueRef::Blob(_) => None,
	}
}

/// Baca satu baris sebagai peta kolom -> teks. Kolom tabel sekolah ARKAS belum
/// terdokumentasi lengkap, jadi nilai diambil dari beberapa kandidat nama kolom.
fn first_row_map(db: &ArkasDb, sql: &str, param: Option<&str>) -> AppResult<Option<BTreeMap<String, String>>> {
	let mut stmt = db.conn().prepare(sql)?;
	let names: Vec<String> = stmt.column_names().iter().map(|s| s.to_lowercase()).collect();
	let mapper = |row: &rusqlite::Row<'_>| {
		let mut map = BTreeMap::new();
		for (i, name) in names.iter().enumerate() {
			if let Some(v) = value_to_string(row.get_ref(i)?) {
				if !v.is_empty() {
					map.insert(name.clone(), v);
				}
			}
		}
		Ok(map)
	};
	let row = match param {
		Some(p) => stmt.query_row(params![p], mapper).optional()?,
		None => stmt.query_row([], mapper).optional()?,
	};
	Ok(row)
}

fn pick(map: &BTreeMap<String, String>, candidates: &[&str]) -> Option<String> {
	candidates.iter().find_map(|c| map.get(*c).cloned())
}

fn table_exists(db: &ArkasDb, name: &str) -> AppResult<bool> {
	let found: Option<String> = db
		.conn()
		.query_row(
			"SELECT name FROM sqlite_master WHERE type = 'table' AND name = ?1",
			params![name],
			|row| row.get(0),
		)
		.optional()?;
	Ok(found.is_some())
}

fn has_column(db: &ArkasDb, table: &str, column: &str) -> AppResult<bool> {
	let mut stmt = db.conn().prepare(&format!("PRAGMA table_info(\"{table}\")"))?;
	let found = stmt
		.query_map([], |row| row.get::<_, String>(1))?
		.filter_map(Result::ok)
		.any(|c| c.eq_ignore_ascii_case(column));
	Ok(found)
}

/// Daftar tahun anggaran yang punya anggaran aktif, terbaru dulu.
pub fn available_years(db: &ArkasDb) -> AppResult<Vec<i32>> {
	let mut stmt = db.conn().prepare(
		"SELECT DISTINCT CAST(tahun_anggaran AS INTEGER) AS tahun
		 FROM anggaran
		 WHERE soft_delete = 0 AND tahun_anggaran IS NOT NULL
		 ORDER BY tahun DESC",
	)?;
	let years = stmt
		.query_map([], |row| row.get::<_, i32>(0))?
		.collect::<Result<Vec<_>, _>>()?;
	Ok(years)
}

/// Sumber dana yang punya anggaran di tahun tertentu.
pub fn fund_sources(db: &ArkasDb, year: i32) -> AppResult<Vec<FundSource>> {
	let mut stmt = db.conn().prepare(
		"SELECT DISTINCT sd.id_ref_sumber_dana, sd.nama_sumber_dana
		 FROM anggaran a
		 JOIN ref_sumber_dana sd ON a.id_ref_sumber_dana = sd.id_ref_sumber_dana
		 WHERE CAST(a.tahun_anggaran AS INTEGER) = ?1 AND a.soft_delete = 0
		 ORDER BY sd.nama_sumber_dana",
	)?;
	let rows = stmt
		.query_map(params![year], |row| {
			Ok(FundSource { id: row.get(0)?, name: row.get::<_, String>(1)?.trim().to_string() })
		})?
		.collect::<Result<Vec<_>, _>>()?;
	Ok(rows)
}

/// Nama kecamatan, kabupaten/kota, dan provinsi dengan menelusuri `mst_wilayah` ke atas.
fn wilayah(db: &ArkasDb, kode: &str) -> AppResult<(Option<String>, Option<String>, Option<String>)> {
	let (mut kec, mut kab, mut prov) = (None, None, None);
	let mut current = Some(kode.trim().to_string());
	for _ in 0..5 {
		let Some(k) = current.take().filter(|k| !k.is_empty()) else { break };
		let row: Option<(Option<String>, Option<String>, Option<i64>)> = db
			.conn()
			.query_row(
				"SELECT nama, mst_kode_wilayah, id_level_wilayah FROM mst_wilayah WHERE kode_wilayah = ?1",
				params![k],
				|r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
			)
			.optional()?;
		let Some((nama, parent, level)) = row else { break };
		let nama = nama.map(|n| n.trim().to_string());
		// Level kecamatan di data ARKAS kadang tercatat 0, jadi kenali juga dari awalan "Kec.".
		let is_kec = nama.as_deref().is_some_and(|n| n.to_lowercase().starts_with("kec"));
		match level {
			Some(3) => kec = nama,
			Some(2) => kab = nama,
			Some(1) => prov = nama,
			_ if is_kec && kec.is_none() => kec = nama,
			_ => {}
		}
		current = parent.map(|p| p.trim().to_string());
	}
	Ok((kec, kab, prov))
}

/// Profil sekolah dan pejabat dari `mst_sekolah` + `sekolah_penjab`.
/// Pejabat diambil dari baris tahun `year` (atau tahun terdekat sebelumnya), bila tidak ada pakai yang terbaru.
pub fn school_info(db: &ArkasDb, year: Option<i32>) -> AppResult<SchoolInfo> {
	let Some(school) = first_row_map(db, "SELECT * FROM mst_sekolah LIMIT 1", None)? else {
		return Ok(SchoolInfo::default());
	};

	let mut info = SchoolInfo {
		sekolah_id: pick(&school, &["sekolah_id", "id_sekolah"]),
		npsn: pick(&school, &["npsn"]),
		nama: pick(&school, &["nama", "nama_sekolah"]),
		alamat: pick(&school, &["alamat_jalan", "alamat", "alamat_sekolah"]),
		telepon: pick(&school, &["telepon", "nomor_telepon"]),
		kepala_sekolah: pick(&school, &["kepsek"]),
		nip_kepala_sekolah: pick(&school, &["nip_kepsek"]),
		..Default::default()
	};

	if let Some(kode) = pick(&school, &["kode_wilayah"]) {
		if table_exists(db, "mst_wilayah")? {
			let (kec, kab, prov) = wilayah(db, &kode)?;
			info.kecamatan = kec;
			info.kabupaten = kab;
			info.provinsi = prov;
		}
	}

	if let Some(id) = info.sekolah_id.clone() {
		if table_exists(db, "sekolah_penjab")? {
			let year = year.unwrap_or(9999).to_string();
			let sql = format!(
				"SELECT * FROM sekolah_penjab WHERE sekolah_id = ?1 {}
				 ORDER BY CASE WHEN CAST(tahun AS INTEGER) <= {year} THEN 0 ELSE 1 END,
				          CASE WHEN CAST(tahun AS INTEGER) <= {year} THEN -CAST(tahun AS INTEGER) ELSE CAST(tahun AS INTEGER) END
				 LIMIT 1",
				if has_column(db, "sekolah_penjab", "soft_delete")? { "AND soft_delete = 0" } else { "" }
			);
			if let Some(penjab) = first_row_map(db, &sql, Some(&id))? {
				if let Some(ks) = pick(&penjab, &["ks", "kepala_sekolah"]) {
					info.kepala_sekolah = Some(ks);
					info.nip_kepala_sekolah = pick(&penjab, &["nip_ks", "nip_kepala_sekolah"]);
				}
				info.bendahara = pick(&penjab, &["bendahara"]);
				info.nip_bendahara = pick(&penjab, &["nip_bendahara"]);
			}
		}
	}
	Ok(info)
}

/// Daftar tabel dan kolom ARKAS, untuk memverifikasi skema saat pengembangan.
pub fn schema(db: &ArkasDb) -> AppResult<Vec<TableInfo>> {
	let mut stmt = db
		.conn()
		.prepare("SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%' ORDER BY name")?;
	let tables = stmt.query_map([], |row| row.get::<_, String>(0))?.collect::<Result<Vec<_>, _>>()?;

	let mut out = Vec::with_capacity(tables.len());
	for name in tables {
		// Nama tabel disisipkan ke SQL, jadi hanya terima karakter identifier biasa.
		if !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
			continue;
		}
		let mut cols = db.conn().prepare(&format!("PRAGMA table_info(\"{name}\")"))?;
		let columns = cols.query_map([], |row| row.get::<_, String>(1))?.collect::<Result<Vec<_>, _>>()?;
		let row_count = db
			.conn()
			.query_row(&format!("SELECT count(*) FROM \"{name}\""), [], |row| row.get(0))?;
		out.push(TableInfo { name, columns, row_count });
	}
	Ok(out)
}
