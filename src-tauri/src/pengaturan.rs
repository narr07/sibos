//! Pengaturan dokumen: pejabat penandatangan, kop surat, dan nomor Berita Acara.
//! Nilai bawaan diambil dari ARKAS; yang diisi pengguna disimpan di `sibos.db` dan
//! menimpa nilai bawaan per kolom. Data ARKAS tidak diubah.

use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};
use crate::repo::app::AppDb;
use crate::repo::arkas::queries::SchoolInfo;

pub const SETTING_KEY: &str = "pengaturan";

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct Pejabat {
	pub kepala_sekolah: String,
	pub nip_kepala_sekolah: String,
	pub bendahara: String,
	pub nip_bendahara: String,
	pub pemegang_barang: String,
	pub nip_pemegang_barang: String,
	pub petugas_rekon: String,
	pub nip_petugas_rekon: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct Kop {
	pub pemerintah: String,
	pub dinas: String,
	/// Nama sekolah di kop, mis. "SEKOLAH DASAR NEGERI TEJA II".
	pub nama_sekolah: String,
	pub alamat: String,
	pub telepon: String,
	pub email: String,
	pub laman: String,
	/// Baris alamat lengkap di kop (baris ke-4).
	pub baris_alamat: String,
	/// Baris kontak di kop (baris ke-5), mis. NPSN, e-mail, website.
	pub baris_kontak: String,
	/// Logo kiri (data URL), mis. logo pemerintah daerah.
	pub logo_kiri: String,
	/// Logo kanan (data URL), mis. logo sekolah.
	pub logo_kanan: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct BeritaAcara {
	pub nomor: String,
	pub tanggal_surat: String,
	pub nomor_sk: String,
	pub tanggal_sk: String,
	pub tempat: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct Cetak {
	/// Kota di atas tanda tangan, mis. "Majalengka".
	pub kota: String,
	/// "A4" atau "F4".
	pub kertas: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct Pengaturan {
	pub pejabat: Pejabat,
	pub kop: Kop,
	pub ba: BeritaAcara,
	pub cetak: Cetak,
}

/// Hasil untuk halaman Pengaturan: yang tersimpan, bawaan dari ARKAS, dan gabungannya.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PengaturanView {
	pub tersimpan: Pengaturan,
	pub bawaan: Pengaturan,
	pub efektif: Pengaturan,
}

fn s(v: &Option<String>) -> String {
	v.clone().unwrap_or_default()
}

/// "Kab. Majalengka" -> "Majalengka", "Kota Bandung" -> "Bandung".
pub fn kota_dari_kabupaten(kab: &str) -> String {
	let k = kab.trim();
	for prefix in ["Kabupaten ", "Kab. ", "Kab ", "Kota "] {
		if let Some(rest) = k.strip_prefix(prefix) {
			return rest.trim().to_string();
		}
	}
	k.to_string()
}

pub fn bawaan(school: &SchoolInfo) -> Pengaturan {
	let kab = s(&school.kabupaten);
	let pemerintah = if kab.is_empty() {
		String::new()
	} else if kab.to_lowercase().starts_with("kota") {
		format!("PEMERINTAH {}", kab.to_uppercase())
	} else {
		format!("PEMERINTAH KABUPATEN {}", kota_dari_kabupaten(&kab).to_uppercase())
	};
	Pengaturan {
		pejabat: Pejabat {
			kepala_sekolah: s(&school.kepala_sekolah),
			nip_kepala_sekolah: s(&school.nip_kepala_sekolah),
			bendahara: s(&school.bendahara),
			nip_bendahara: s(&school.nip_bendahara),
			..Default::default()
		},
		kop: Kop {
			pemerintah,
			dinas: if kab.is_empty() { String::new() } else { "DINAS PENDIDIKAN".into() },
			nama_sekolah: s(&school.nama).to_uppercase(),
			alamat: s(&school.alamat),
			telepon: s(&school.telepon),
			baris_alamat: {
				let parts: Vec<String> = [s(&school.alamat), s(&school.kecamatan), kab.clone()]
					.into_iter()
					.filter(|p| !p.trim().is_empty())
					.collect();
				if parts.is_empty() { String::new() } else { format!("Alamat {}", parts.join(", ")) }
			},
			baris_kontak: if s(&school.npsn).is_empty() { String::new() } else { format!("NPSN. {}", s(&school.npsn)) },
			..Default::default()
		},
		ba: BeritaAcara { tempat: s(&school.nama), ..Default::default() },
		cetak: Cetak { kota: kota_dari_kabupaten(&kab), kertas: "A4".into() },
	}
}

fn pick(saved: &str, default: &str) -> String {
	if saved.trim().is_empty() {
		default.to_string()
	} else {
		saved.trim().to_string()
	}
}

/// Gabungkan per kolom: nilai tersimpan menang bila tidak kosong.
pub fn gabung(saved: &Pengaturan, default: &Pengaturan) -> Pengaturan {
	let (sp, dp) = (&saved.pejabat, &default.pejabat);
	let (sk, dk) = (&saved.kop, &default.kop);
	let (sb, db) = (&saved.ba, &default.ba);
	let (sc, dc) = (&saved.cetak, &default.cetak);
	Pengaturan {
		pejabat: Pejabat {
			kepala_sekolah: pick(&sp.kepala_sekolah, &dp.kepala_sekolah),
			nip_kepala_sekolah: pick(&sp.nip_kepala_sekolah, &dp.nip_kepala_sekolah),
			bendahara: pick(&sp.bendahara, &dp.bendahara),
			nip_bendahara: pick(&sp.nip_bendahara, &dp.nip_bendahara),
			pemegang_barang: pick(&sp.pemegang_barang, &dp.pemegang_barang),
			nip_pemegang_barang: pick(&sp.nip_pemegang_barang, &dp.nip_pemegang_barang),
			petugas_rekon: pick(&sp.petugas_rekon, &dp.petugas_rekon),
			nip_petugas_rekon: pick(&sp.nip_petugas_rekon, &dp.nip_petugas_rekon),
		},
		kop: Kop {
			pemerintah: pick(&sk.pemerintah, &dk.pemerintah),
			dinas: pick(&sk.dinas, &dk.dinas),
			nama_sekolah: pick(&sk.nama_sekolah, &dk.nama_sekolah),
			alamat: pick(&sk.alamat, &dk.alamat),
			telepon: pick(&sk.telepon, &dk.telepon),
			email: pick(&sk.email, &dk.email),
			laman: pick(&sk.laman, &dk.laman),
			baris_alamat: pick(&sk.baris_alamat, &dk.baris_alamat),
			baris_kontak: pick(&sk.baris_kontak, &dk.baris_kontak),
			logo_kiri: pick(&sk.logo_kiri, &dk.logo_kiri),
			logo_kanan: pick(&sk.logo_kanan, &dk.logo_kanan),
		},
		ba: BeritaAcara {
			nomor: pick(&sb.nomor, &db.nomor),
			tanggal_surat: pick(&sb.tanggal_surat, &db.tanggal_surat),
			nomor_sk: pick(&sb.nomor_sk, &db.nomor_sk),
			tanggal_sk: pick(&sb.tanggal_sk, &db.tanggal_sk),
			tempat: pick(&sb.tempat, &db.tempat),
		},
		cetak: Cetak { kota: pick(&sc.kota, &dc.kota), kertas: pick(&sc.kertas, &dc.kertas) },
	}
}

pub fn load(app_db: &AppDb) -> AppResult<Pengaturan> {
	Ok(app_db
		.setting_get(SETTING_KEY)?
		.and_then(|v| serde_json::from_value(v).ok())
		.unwrap_or_default())
}

fn digits_only(label: &str, nip: &str) -> AppResult<()> {
	let compact: String = nip.chars().filter(|c| !c.is_whitespace()).collect();
	if compact.chars().all(|c| c.is_ascii_digit()) {
		Ok(())
	} else {
		Err(AppError::InvalidInput(format!("{label} hanya boleh berisi angka")))
	}
}

pub fn validate(p: &Pengaturan) -> AppResult<()> {
	let j = &p.pejabat;
	digits_only("NIP Kepala Sekolah", &j.nip_kepala_sekolah)?;
	digits_only("NIP Bendahara", &j.nip_bendahara)?;
	digits_only("NIP Pemegang Barang", &j.nip_pemegang_barang)?;
	digits_only("NIP Petugas Rekonsiliasi", &j.nip_petugas_rekon)?;
	if !p.cetak.kertas.is_empty() && !["A4", "F4"].contains(&p.cetak.kertas.as_str()) {
		return Err(AppError::InvalidInput("ukuran kertas harus A4 atau F4".into()));
	}
	Ok(())
}

#[cfg(test)]
mod tests {
	use super::*;

	fn school() -> SchoolInfo {
		SchoolInfo {
			nama: Some("SD Negeri Contoh".into()),
			kepala_sekolah: Some("Budi".into()),
			nip_kepala_sekolah: Some("1980".into()),
			kabupaten: Some("Kab. Majalengka".into()),
			..Default::default()
		}
	}

	#[test]
	fn defaults_from_arkas() {
		let d = bawaan(&school());
		assert_eq!(d.pejabat.kepala_sekolah, "Budi");
		assert_eq!(d.kop.pemerintah, "PEMERINTAH KABUPATEN MAJALENGKA");
		assert_eq!(d.cetak.kota, "Majalengka");
		assert_eq!(kota_dari_kabupaten("Kota Bandung"), "Bandung");
	}

	#[test]
	fn saved_values_override_per_field() {
		let mut saved = Pengaturan::default();
		saved.pejabat.bendahara = "Siti".into();
		saved.pejabat.kepala_sekolah = "  ".into();
		let merged = gabung(&saved, &bawaan(&school()));
		assert_eq!(merged.pejabat.bendahara, "Siti");
		assert_eq!(merged.pejabat.kepala_sekolah, "Budi", "isian kosong memakai bawaan");
	}

	#[test]
	fn nip_must_be_digits() {
		let mut p = Pengaturan::default();
		p.pejabat.nip_bendahara = "1985 0202".into();
		assert!(validate(&p).is_ok());
		p.pejabat.nip_bendahara = "1985-abc".into();
		assert!(validate(&p).is_err());
	}
}
