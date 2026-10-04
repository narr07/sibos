//! Pajak di SIBOS diinput MANUAL oleh bendahara: tidak ada perhitungan tarif otomatis
//! dan pajak tidak ditambahkan otomatis ke nota. Modul ini hanya membaca jenis pajak
//! dari baris yang sudah dicatat (di ARKAS atau entri manual SIBOS) untuk ditampilkan.

use crate::repo::arkas::bku::KasRow;

/// Jenis pajak dari flag baris ARKAS, dengan cadangan dari teks uraian.
pub fn jenis_pajak(row: &KasRow) -> Option<&'static str> {
	let by_flag = if row.is_ppn {
		Some("PPN")
	} else if row.is_pph_21 {
		Some("PPh 21")
	} else if row.is_pph_22 {
		Some("PPh 22")
	} else if row.is_pph_23 {
		Some("PPh 23")
	} else if row.is_pph_4 {
		Some("PPh 4(2)")
	} else if row.is_sspd {
		Some("Pajak Daerah")
	} else {
		None
	};
	by_flag.or_else(|| {
		let text = row.uraian.to_lowercase();
		[
			("ppn", "PPN"),
			("pph 21", "PPh 21"),
			("pph21", "PPh 21"),
			("pph 22", "PPh 22"),
			("pph 23", "PPh 23"),
			("pph 4", "PPh 4(2)"),
			("sspd", "Pajak Daerah"),
			("pajak daerah", "Pajak Daerah"),
		]
		.iter()
		.find(|(needle, _)| text.contains(needle))
		.map(|(_, label)| *label)
	})
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn detects_from_flags_then_text() {
		let row = KasRow { is_ppn: true, ..Default::default() };
		assert_eq!(jenis_pajak(&row), Some("PPN"));
		let row = KasRow { uraian: "Setor PPh 23".into(), ..Default::default() };
		assert_eq!(jenis_pajak(&row), Some("PPh 23"));
		let row = KasRow { uraian: "Belanja ATK".into(), ..Default::default() };
		assert_eq!(jenis_pajak(&row), None);
	}
}
