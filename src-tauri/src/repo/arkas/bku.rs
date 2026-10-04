//! Query baca data BKU dari ARKAS. HANYA BACA, lihat `conn.rs`.

use rusqlite::{params, OptionalExtension};

use super::conn::ArkasDb;
use crate::error::AppResult;

/// Satu baris `kas_umum` beserta data pendukungnya.
#[derive(Debug, Clone, Default)]
pub struct KasRow {
	pub id: String,
	pub parent_id: Option<String>,
	pub id_ref_bku: i64,
	pub ref_bku_name: Option<String>,
	pub no_bukti: Option<String>,
	/// Format `YYYY-MM-DD`.
	pub tanggal: String,
	pub uraian: String,
	pub kode_rekening: Option<String>,
	pub kode_kegiatan: Option<String>,
	pub saldo: i64,
	pub is_ppn: bool,
	pub is_pph_21: bool,
	pub is_pph_22: bool,
	pub is_pph_23: bool,
	pub is_pph_4: bool,
	pub is_sspd: bool,
	pub fund_id: i64,
	pub fund_name: String,
	pub id_kas_nota: Option<String>,
	pub create_date: String,
	pub volume: Option<f64>,
	pub satuan: Option<String>,
	pub harga_satuan: Option<i64>,
	/// Item RKAS (rapbs) yang dibelanjakan.
	pub id_rapbs: Option<String>,
	/// Belanja lewat SIPLah (dari kas_umum_nota.is_beli_di_siplah).
	pub is_siplah: bool,
}

/// Saldo resmi ARKAS per bulan dari `aktivasi_bku`.
#[derive(Debug, Clone, Default)]
pub struct MonthBalance {
	pub fund_id: i64,
	/// 1-12.
	pub month: u32,
	pub finished: bool,
	pub saldo_akhir_bank: Option<i64>,
	pub saldo_akhir_tunai: Option<i64>,
	pub saldo_akhir_bank_sisa: Option<i64>,
	pub saldo_akhir_tunai_sisa: Option<i64>,
}

fn flag(row: &rusqlite::Row<'_>, idx: usize) -> rusqlite::Result<bool> {
	Ok(row.get::<_, Option<i64>>(idx)?.unwrap_or(0) != 0)
}

fn opt_text(row: &rusqlite::Row<'_>, idx: usize) -> rusqlite::Result<Option<String>> {
	Ok(row
		.get::<_, Option<String>>(idx)?
		.map(|s| s.trim().to_string())
		.filter(|s| !s.is_empty()))
}

/// Semua baris kas umum (tidak terhapus) untuk satu tahun anggaran.
pub fn kas_rows(db: &ArkasDb, year: i32) -> AppResult<Vec<KasRow>> {
	let mut stmt = db.conn().prepare(
		"SELECT k.id_kas_umum, k.parent_id_kas_umum, k.id_ref_bku, rb.bku, k.no_bukti,
		        substr(k.tanggal_transaksi, 1, 10), k.uraian, k.kode_rekening,
		        (SELECT rk.id_kode FROM ref_kode rk WHERE rk.id_ref_kode = r.id_ref_kode
		         ORDER BY CASE WHEN CAST(rk.tahun AS INTEGER) = ?1 THEN 0 ELSE 1 END, rk.tahun DESC LIMIT 1),
		        CAST(k.saldo AS INTEGER), k.is_ppn, k.is_pph_21, k.is_pph_22, k.is_pph_23, k.is_pph_4, k.is_sspd,
		        a.id_ref_sumber_dana, sd.nama_sumber_dana, k.id_kas_nota, k.create_date,
		        k.volume, COALESCE(rp.satuan, r.satuan), COALESCE(rp.harga_satuan, r.harga_satuan), rp.id_rapbs,
		        (SELECT n.is_beli_di_siplah FROM kas_umum_nota n WHERE n.id_kas_nota = k.id_kas_nota AND n.soft_delete = 0 LIMIT 1)
		 FROM kas_umum k
		 JOIN anggaran a ON a.id_anggaran = k.id_anggaran
		 LEFT JOIN ref_sumber_dana sd ON sd.id_ref_sumber_dana = a.id_ref_sumber_dana
		 LEFT JOIN ref_bku rb ON rb.id_ref_bku = k.id_ref_bku
		 LEFT JOIN rapbs_periode rp ON rp.id_rapbs_periode = k.id_rapbs_periode
		 LEFT JOIN rapbs r ON r.id_rapbs = rp.id_rapbs
		 WHERE k.soft_delete = 0 AND CAST(a.tahun_anggaran AS INTEGER) = ?1",
	)?;
	let rows = stmt
		.query_map(params![year], |row| {
			Ok(KasRow {
				id: row.get(0)?,
				parent_id: opt_text(row, 1)?,
				id_ref_bku: row.get::<_, Option<i64>>(2)?.unwrap_or(0),
				ref_bku_name: opt_text(row, 3)?,
				no_bukti: opt_text(row, 4)?,
				tanggal: row.get::<_, Option<String>>(5)?.unwrap_or_default(),
				uraian: row.get::<_, Option<String>>(6)?.unwrap_or_default().trim().to_string(),
				kode_rekening: opt_text(row, 7)?,
				kode_kegiatan: opt_text(row, 8)?,
				saldo: row.get::<_, Option<i64>>(9)?.unwrap_or(0),
				is_ppn: flag(row, 10)?,
				is_pph_21: flag(row, 11)?,
				is_pph_22: flag(row, 12)?,
				is_pph_23: flag(row, 13)?,
				is_pph_4: flag(row, 14)?,
				is_sspd: flag(row, 15)?,
				fund_id: row.get::<_, Option<i64>>(16)?.unwrap_or(0),
				fund_name: row.get::<_, Option<String>>(17)?.unwrap_or_default().trim().to_string(),
				id_kas_nota: opt_text(row, 18)?,
				create_date: row.get::<_, Option<String>>(19)?.unwrap_or_default(),
				volume: row.get::<_, Option<f64>>(20).ok().flatten(),
				satuan: opt_text(row, 21).ok().flatten(),
				harga_satuan: row.get::<_, Option<f64>>(22).ok().flatten().map(|v| v.round() as i64),
				id_rapbs: opt_text(row, 23).ok().flatten(),
				is_siplah: flag(row, 24).unwrap_or(false),
			})
		})?
		.collect::<Result<Vec<_>, _>>()?;
	// Pengaman: satu id_kas_umum hanya boleh muncul sekali walau ada JOIN yang ganda.
	let mut seen = std::collections::HashSet::new();
	let rows = rows.into_iter().filter(|r: &KasRow| seen.insert(r.id.clone())).collect();
	Ok(rows)
}

/// Saldo akhir resmi per bulan per sumber dana. Bila ada beberapa baris untuk bulan yang sama
/// (satu per revisi anggaran), dipakai baris dari anggaran disetujui dengan revisi tertinggi.
pub fn month_balances(db: &ArkasDb, year: i32) -> AppResult<Vec<MonthBalance>> {
	let mut stmt = db.conn().prepare(
		"SELECT a.id_ref_sumber_dana, ab.id_periode, ab.tanggal_finish,
		        ab.saldo_akhir_bank, ab.saldo_akhir_tunai, ab.saldo_akhir_bank_sisa, ab.saldo_akhir_tunai_sisa
		 FROM aktivasi_bku ab
		 JOIN anggaran a ON a.id_anggaran = ab.id_anggaran
		 WHERE ab.soft_delete = 0 AND a.soft_delete = 0 AND CAST(a.tahun_anggaran AS INTEGER) = ?1
		   AND ab.id_periode BETWEEN 81 AND 92
		 ORDER BY a.id_ref_sumber_dana, ab.id_periode, a.is_approve DESC, a.is_revisi DESC",
	)?;
	let mut out: Vec<MonthBalance> = Vec::new();
	let rows = stmt.query_map(params![year], |row| {
		let periode: i64 = row.get(1)?;
		let finish: Option<String> = row.get(2)?;
		Ok(MonthBalance {
			fund_id: row.get::<_, Option<i64>>(0)?.unwrap_or(0),
			month: (periode - 80) as u32,
			finished: finish.is_some_and(|f| !f.trim().is_empty()),
			saldo_akhir_bank: row.get(3)?,
			saldo_akhir_tunai: row.get(4)?,
			saldo_akhir_bank_sisa: row.get(5)?,
			saldo_akhir_tunai_sisa: row.get(6)?,
		})
	})?;
	for row in rows {
		let row = row?;
		// Ambil baris pertama per (sumber dana, bulan) sesuai urutan ORDER BY.
		if !out.iter().any(|b| b.fund_id == row.fund_id && b.month == row.month) {
			out.push(row);
		}
	}
	Ok(out)
}

/// Bulan terakhir yang punya transaksi (bukan baris saldo awal), 1-12.
pub fn last_active_month(db: &ArkasDb, year: i32) -> AppResult<Option<u32>> {
	let month: Option<String> = db
		.conn()
		.query_row(
			"SELECT max(strftime('%m', k.tanggal_transaksi))
			 FROM kas_umum k JOIN anggaran a ON a.id_anggaran = k.id_anggaran
			 WHERE k.soft_delete = 0 AND CAST(a.tahun_anggaran AS INTEGER) = ?1
			   AND k.id_ref_bku NOT IN (1, 8, 9, 28, 29)",
			params![year],
			|row| row.get(0),
		)
		.optional()?
		.flatten();
	Ok(month.and_then(|m| m.parse().ok()))
}

/// Data nota/toko dari `kas_umum_nota`.
#[derive(Debug, Clone, Default, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NotaInfo {
	pub id_kas_nota: String,
	pub no_bukti: Option<String>,
	pub no_nota: Option<String>,
	pub tanggal_nota: Option<String>,
	pub nama_toko: Option<String>,
	pub alamat_toko: Option<String>,
	pub no_telp: Option<String>,
	pub npwp: Option<String>,
	pub is_badan_usaha: bool,
	pub total: Option<i64>,
	pub is_siplah: bool,
}

/// Semua nota (tidak terhapus) yang dipakai transaksi tahun tertentu, kunci = id_kas_nota.
pub fn nota_infos(db: &ArkasDb, year: i32) -> AppResult<std::collections::HashMap<String, NotaInfo>> {
	let mut stmt = db.conn().prepare(
		"SELECT n.id_kas_nota, n.no_bukti, n.no_nota, substr(n.tanggal_nota, 1, 10), n.nama_toko, n.alamat_toko,
		        n.no_telp, n.npwp, n.is_badan_usaha, CAST(n.total AS INTEGER), n.is_beli_di_siplah
		 FROM kas_umum_nota n
		 WHERE n.soft_delete = 0 AND n.id_kas_nota IN (
		   SELECT k.id_kas_nota FROM kas_umum k JOIN anggaran a ON a.id_anggaran = k.id_anggaran
		   WHERE k.soft_delete = 0 AND k.id_kas_nota IS NOT NULL AND CAST(a.tahun_anggaran AS INTEGER) = ?1)",
	)?;
	let rows = stmt
		.query_map(params![year], |row| {
			Ok(NotaInfo {
				id_kas_nota: row.get(0)?,
				no_bukti: opt_text(row, 1)?,
				no_nota: opt_text(row, 2)?,
				tanggal_nota: opt_text(row, 3)?,
				nama_toko: opt_text(row, 4)?,
				alamat_toko: opt_text(row, 5)?,
				no_telp: opt_text(row, 6)?,
				npwp: opt_text(row, 7)?,
				is_badan_usaha: flag(row, 8)?,
				total: row.get(9)?,
				is_siplah: flag(row, 10)?,
			})
		})?
		.map(|r| r.map(|n| (n.id_kas_nota.clone(), n)))
		.collect::<Result<std::collections::HashMap<_, _>, _>>()?;
	Ok(rows)
}
