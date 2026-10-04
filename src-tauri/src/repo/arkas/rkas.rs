//! Query baca RKAS dari ARKAS: anggaran aktif, item rapbs, distribusi bulanan, nama kode,
//! status anggaran, dan katalog standar harga. HANYA BACA, lihat `conn.rs`.

use std::collections::HashMap;

use rusqlite::params;
use serde::Serialize;

use super::conn::ArkasDb;
use crate::error::AppResult;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnggaranInfo {
	pub id_anggaran: String,
	pub fund_id: i64,
	pub fund_name: String,
	pub is_revisi: i64,
	pub is_approve: bool,
	pub is_pengesahan: bool,
	pub alasan_penolakan: Option<String>,
	pub jumlah: i64,
	pub tanggal_pengesahan: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RkasItem {
	pub id_rapbs: String,
	pub fund_id: i64,
	pub fund_name: String,
	pub kode_kegiatan: Option<String>,
	pub kode_rekening: Option<String>,
	pub uraian: String,
	pub satuan: Option<String>,
	pub volume: f64,
	pub harga_satuan: i64,
	pub jumlah: i64,
	/// Rencana rupiah per bulan, indeks 0 = Januari.
	pub bulan: [i64; 12],
	/// Rencana volume per bulan.
	pub volume_bulan: [f64; 12],
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StandarHarga {
	pub kode_rekening: Option<String>,
	pub nama_barang: String,
	pub satuan: Option<String>,
	pub harga: Option<i64>,
	pub batas_bawah: Option<i64>,
	pub batas_atas: Option<i64>,
	pub tahun: Option<i64>,
}

fn text(row: &rusqlite::Row<'_>, idx: usize) -> rusqlite::Result<Option<String>> {
	Ok(row.get::<_, Option<String>>(idx)?.map(|s| s.trim().to_string()).filter(|s| !s.is_empty()))
}

fn money(row: &rusqlite::Row<'_>, idx: usize) -> rusqlite::Result<Option<i64>> {
	Ok(row.get::<_, Option<f64>>(idx)?.map(|v| v.round() as i64))
}

/// Semua baris anggaran (tidak terhapus) satu tahun, urut sumber dana lalu revisi tertinggi.
pub fn anggaran_list(db: &ArkasDb, year: i32) -> AppResult<Vec<AnggaranInfo>> {
	let mut stmt = db.conn().prepare(
		"SELECT a.id_anggaran, a.id_ref_sumber_dana, sd.nama_sumber_dana, a.is_revisi, a.is_approve, a.is_pengesahan,
		        a.alasan_penolakan, CAST(a.jumlah AS INTEGER), a.tanggal_pengesahan
		 FROM anggaran a LEFT JOIN ref_sumber_dana sd ON sd.id_ref_sumber_dana = a.id_ref_sumber_dana
		 WHERE a.soft_delete = 0 AND CAST(a.tahun_anggaran AS INTEGER) = ?1
		 ORDER BY a.id_ref_sumber_dana, a.is_revisi DESC",
	)?;
	let rows = stmt
		.query_map(params![year], |r| {
			Ok(AnggaranInfo {
				id_anggaran: r.get(0)?,
				fund_id: r.get::<_, Option<i64>>(1)?.unwrap_or(0),
				fund_name: text(r, 2)?.unwrap_or_default(),
				is_revisi: r.get::<_, Option<i64>>(3)?.unwrap_or(0),
				is_approve: r.get::<_, Option<i64>>(4)?.unwrap_or(0) != 0,
				is_pengesahan: r.get::<_, Option<i64>>(5)?.unwrap_or(0) != 0,
				alasan_penolakan: text(r, 6)?,
				jumlah: r.get::<_, Option<i64>>(7)?.unwrap_or(0),
				tanggal_pengesahan: text(r, 8)?,
			})
		})?
		.collect::<Result<Vec<_>, _>>()?;
	Ok(rows)
}

/// Anggaran yang berlaku per sumber dana: disetujui dengan revisi tertinggi.
pub fn active_anggaran(list: &[AnggaranInfo]) -> Vec<&AnggaranInfo> {
	let mut out: Vec<&AnggaranInfo> = Vec::new();
	for a in list.iter().filter(|a| a.is_approve) {
		match out.iter().position(|o| o.fund_id == a.fund_id) {
			Some(i) if out[i].is_revisi >= a.is_revisi => {}
			Some(i) => out[i] = a,
			None => out.push(a),
		}
	}
	out
}

/// Item RKAS dari anggaran yang berlaku, beserta rencana bulanan.
pub fn rkas_items(db: &ArkasDb, year: i32) -> AppResult<Vec<RkasItem>> {
	let list = anggaran_list(db, year)?;
	let active = active_anggaran(&list);
	let mut items = Vec::new();
	for a in active {
		let mut stmt = db.conn().prepare(
			"SELECT r.id_rapbs,
			        (SELECT rk.id_kode FROM ref_kode rk WHERE rk.id_ref_kode = r.id_ref_kode
			         ORDER BY CASE WHEN CAST(rk.tahun AS INTEGER) = ?2 THEN 0 ELSE 1 END, rk.tahun DESC LIMIT 1),
			        r.kode_rekening, COALESCE(NULLIF(trim(r.uraian_text), ''), r.uraian), r.satuan,
			        r.volume, r.harga_satuan, r.jumlah
			 FROM rapbs r WHERE r.soft_delete = 0 AND r.id_anggaran = ?1
			 ORDER BY r.urutan, r.id_rapbs",
		)?;
		let rows = stmt.query_map(params![a.id_anggaran, year], |r| {
			Ok(RkasItem {
				id_rapbs: r.get(0)?,
				fund_id: a.fund_id,
				fund_name: a.fund_name.clone(),
				kode_kegiatan: text(r, 1)?,
				kode_rekening: text(r, 2)?,
				uraian: text(r, 3)?.unwrap_or_default(),
				satuan: text(r, 4)?,
				volume: r.get::<_, Option<f64>>(5)?.unwrap_or(0.0),
				harga_satuan: money(r, 6)?.unwrap_or(0),
				jumlah: money(r, 7)?.unwrap_or(0),
				bulan: [0; 12],
				volume_bulan: [0.0; 12],
			})
		})?;
		let start = items.len();
		for row in rows {
			items.push(row?);
		}
		let index: HashMap<String, usize> =
			items[start..].iter().enumerate().map(|(i, it)| (it.id_rapbs.clone(), start + i)).collect();

		let mut per = db.conn().prepare(
			"SELECT rp.id_rapbs, rp.id_periode, rp.volume, rp.jumlah
			 FROM rapbs_periode rp JOIN rapbs r ON r.id_rapbs = rp.id_rapbs
			 WHERE rp.soft_delete = 0 AND r.soft_delete = 0 AND r.id_anggaran = ?1 AND rp.id_periode BETWEEN 81 AND 92",
		)?;
		let periods = per.query_map(params![a.id_anggaran], |r| {
			Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?, r.get::<_, Option<f64>>(2)?, money(r, 3)?))
		})?;
		for p in periods {
			let (id, periode, volume, jumlah) = p?;
			if let Some(&i) = index.get(&id) {
				let m = (periode - 81) as usize;
				items[i].bulan[m] += jumlah.unwrap_or(0);
				items[i].volume_bulan[m] += volume.unwrap_or(0.0);
			}
		}
	}
	Ok(items)
}

/// Nama program/komponen/kegiatan per kode (mis. "05.02.08." -> "Pengadaan ..."), prioritas tahun anggaran.
pub fn kode_names(db: &ArkasDb, year: i32) -> AppResult<HashMap<String, String>> {
	let mut stmt = db.conn().prepare(
		"SELECT id_kode, uraian_kode FROM ref_kode
		 WHERE id_kode IS NOT NULL
		 ORDER BY CASE WHEN CAST(tahun AS INTEGER) = ?1 THEN 0 ELSE 1 END DESC, tahun",
	)?;
	let mut map = HashMap::new();
	// Urutan: tahun lain dulu lalu tahun berjalan, sehingga tahun berjalan menimpa.
	for row in stmt.query_map(params![year], |r| Ok((text(r, 0)?, text(r, 1)?)))? {
		if let (Some(k), Some(v)) = row? {
			map.insert(k, v);
		}
	}
	Ok(map)
}

/// Nama rekening per kode rekening, prioritas tahun anggaran.
pub fn rekening_names(db: &ArkasDb, year: i32) -> AppResult<HashMap<String, String>> {
	let mut stmt = db.conn().prepare(
		"SELECT kode_rekening, rekening FROM ref_rekening
		 ORDER BY CASE WHEN CAST(tahun AS INTEGER) = ?1 THEN 0 ELSE 1 END DESC, tahun",
	)?;
	let mut map = HashMap::new();
	for row in stmt.query_map(params![year], |r| Ok((text(r, 0)?, text(r, 1)?)))? {
		if let (Some(k), Some(v)) = row? {
			map.insert(k, v);
		}
	}
	Ok(map)
}

/// Cari katalog standar harga (`ref_acuan_barang`) berdasarkan nama barang atau kode rekening.
pub fn standar_harga_search(db: &ArkasDb, year: i32, keyword: &str, limit: u32) -> AppResult<Vec<StandarHarga>> {
	let pattern = format!("%{}%", keyword.trim());
	let mut stmt = db.conn().prepare(
		"SELECT kode_rekening, nama_barang, satuan, harga_barang, batas_bawah, batas_atas, CAST(tahun AS INTEGER)
		 FROM ref_acuan_barang
		 WHERE (nama_barang LIKE ?1 OR kode_rekening LIKE ?1) AND expired_date IS NULL
		 ORDER BY CASE WHEN CAST(tahun AS INTEGER) = ?2 THEN 0 ELSE 1 END, tahun DESC, nama_barang
		 LIMIT ?3",
	)?;
	let rows = stmt
		.query_map(params![pattern, year, limit], |r| {
			Ok(StandarHarga {
				kode_rekening: text(r, 0)?,
				nama_barang: text(r, 1)?.unwrap_or_default(),
				satuan: text(r, 2)?,
				harga: money(r, 3)?,
				batas_bawah: money(r, 4)?,
				batas_atas: money(r, 5)?,
				tahun: r.get(6)?,
			})
		})?
		.collect::<Result<Vec<_>, _>>()?;
	Ok(rows)
}

#[cfg(test)]
mod tests {
	use super::*;

	fn ang(fund: i64, rev: i64, approve: bool) -> AnggaranInfo {
		AnggaranInfo {
			id_anggaran: format!("{fund}-{rev}"),
			fund_id: fund,
			fund_name: String::new(),
			is_revisi: rev,
			is_approve: approve,
			is_pengesahan: approve,
			alasan_penolakan: None,
			jumlah: 0,
			tanggal_pengesahan: None,
		}
	}

	#[test]
	fn active_is_highest_approved_revision() {
		let list = vec![ang(1, 103, false), ang(1, 102, true), ang(1, 100, true), ang(5, 0, true)];
		let active = active_anggaran(&list);
		assert_eq!(active.len(), 2);
		assert_eq!(active[0].id_anggaran, "1-102");
		assert_eq!(active[1].id_anggaran, "5-0");
	}
}
