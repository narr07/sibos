//! Realisasi belanja: rencana RKAS dibandingkan belanja BKU per item dan per bulan.

use std::collections::HashMap;

use serde::Serialize;

use crate::repo::arkas::bku::KasRow;
use crate::repo::arkas::rkas::RkasItem;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RealisasiItem {
	pub id_rapbs: String,
	pub fund_name: String,
	pub kode_kegiatan: Option<String>,
	pub nama_kegiatan: Option<String>,
	pub kode_rekening: Option<String>,
	pub nama_rekening: Option<String>,
	pub uraian: String,
	pub satuan: Option<String>,
	pub volume: f64,
	pub harga_satuan: i64,
	pub pagu: i64,
	pub rencana: [i64; 12],
	pub realisasi: [i64; 12],
	pub total_realisasi: i64,
	/// Rencana s.d. bulan acuan.
	pub rencana_sd: i64,
	/// Realisasi s.d. bulan acuan.
	pub realisasi_sd: i64,
	pub sisa: i64,
	pub persen: f64,
	/// lunas | sebagian | belum | melampaui | belum_jatuh_tempo
	pub status: String,
	/// Rencana yang sudah jatuh tempo (s.d. bulan acuan) tapi belum dibelanjakan.
	pub tertunda: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OutsideItem {
	pub id: String,
	pub tanggal: String,
	pub uraian: String,
	pub kode_rekening: Option<String>,
	pub nominal: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Realisasi {
	pub year: i32,
	/// Bulan acuan untuk status jatuh tempo (1-12).
	pub upto: u32,
	pub fund: Option<i64>,
	pub items: Vec<RealisasiItem>,
	pub total_pagu: i64,
	pub total_realisasi: i64,
	pub rencana_bulan: [i64; 12],
	pub realisasi_bulan: [i64; 12],
	pub persen: f64,
	pub total_tertunda: i64,
	/// Belanja BKU yang tidak terhubung ke item RKAS aktif.
	pub luar_rkas: Vec<OutsideItem>,
}

fn month_of(tanggal: &str) -> usize {
	tanggal.get(5..7).and_then(|m| m.parse::<usize>().ok()).unwrap_or(1).clamp(1, 12)
}

fn persen(a: i64, b: i64) -> f64 {
	if b == 0 {
		0.0
	} else {
		((a as f64 / b as f64) * 1000.0).round() / 10.0
	}
}

pub struct RealisasiInput<'a> {
	pub items: &'a [RkasItem],
	pub rows: &'a [KasRow],
	pub kode_names: &'a HashMap<String, String>,
	pub rekening_names: &'a HashMap<String, String>,
}

pub fn build_realisasi(year: i32, upto: u32, fund: Option<i64>, input: &RealisasiInput<'_>) -> Realisasi {
	let upto = upto.clamp(1, 12);
	let mut spent: HashMap<&str, [i64; 12]> = HashMap::new();
	let mut luar_rkas = Vec::new();
	let known: std::collections::HashSet<&str> = input.items.iter().map(|i| i.id_rapbs.as_str()).collect();

	for r in input.rows.iter().filter(|r| matches!(r.id_ref_bku, 4 | 15 | 24 | 35) && fund.is_none_or(|f| f == r.fund_id)) {
		match r.id_rapbs.as_deref().filter(|id| known.contains(id)) {
			Some(id) => spent.entry(id).or_insert([0; 12])[month_of(&r.tanggal) - 1] += r.saldo,
			None => luar_rkas.push(OutsideItem {
				id: r.id.clone(),
				tanggal: r.tanggal.clone(),
				uraian: r.uraian.clone(),
				kode_rekening: r.kode_rekening.clone(),
				nominal: r.saldo,
			}),
		}
	}

	let mut items = Vec::new();
	let (mut rencana_bulan, mut realisasi_bulan) = ([0i64; 12], [0i64; 12]);
	for it in input.items.iter().filter(|i| fund.is_none_or(|f| f == i.fund_id)) {
		let realisasi = spent.get(it.id_rapbs.as_str()).copied().unwrap_or([0; 12]);
		let total_realisasi: i64 = realisasi.iter().sum();
		let rencana_sd: i64 = it.bulan[..upto as usize].iter().sum();
		let realisasi_sd: i64 = realisasi[..upto as usize].iter().sum();
		for m in 0..12 {
			rencana_bulan[m] += it.bulan[m];
			realisasi_bulan[m] += realisasi[m];
		}
		let status = if total_realisasi > it.jumlah {
			"melampaui"
		} else if it.jumlah > 0 && total_realisasi == it.jumlah {
			"lunas"
		} else if total_realisasi > 0 {
			"sebagian"
		} else if rencana_sd == 0 {
			"belum_jatuh_tempo"
		} else {
			"belum"
		};
		items.push(RealisasiItem {
			id_rapbs: it.id_rapbs.clone(),
			fund_name: it.fund_name.clone(),
			kode_kegiatan: it.kode_kegiatan.clone(),
			nama_kegiatan: it.kode_kegiatan.as_ref().and_then(|k| input.kode_names.get(k).cloned()),
			kode_rekening: it.kode_rekening.clone(),
			nama_rekening: it.kode_rekening.as_ref().and_then(|k| input.rekening_names.get(k).cloned()),
			uraian: it.uraian.clone(),
			satuan: it.satuan.clone(),
			volume: it.volume,
			harga_satuan: it.harga_satuan,
			pagu: it.jumlah,
			rencana: it.bulan,
			realisasi,
			total_realisasi,
			rencana_sd,
			realisasi_sd,
			sisa: it.jumlah - total_realisasi,
			persen: persen(total_realisasi, it.jumlah),
			status: status.into(),
			tertunda: (rencana_sd - realisasi_sd).max(0),
		});
	}

	let total_pagu: i64 = items.iter().map(|i| i.pagu).sum();
	let total_realisasi: i64 = items.iter().map(|i| i.total_realisasi).sum::<i64>() + luar_rkas.iter().map(|o| o.nominal).sum::<i64>();
	let total_tertunda = items.iter().map(|i| i.tertunda).sum();
	for o in &luar_rkas {
		realisasi_bulan[month_of(&o.tanggal) - 1] += o.nominal;
	}
	Realisasi {
		year,
		upto,
		fund,
		persen: persen(total_realisasi, total_pagu),
		items,
		total_pagu,
		total_realisasi,
		rencana_bulan,
		realisasi_bulan,
		total_tertunda,
		luar_rkas,
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	fn item(id: &str, jumlah: i64, bulan: [i64; 12]) -> RkasItem {
		RkasItem {
			id_rapbs: id.into(),
			fund_id: 1,
			fund_name: "BOS Reguler".into(),
			kode_kegiatan: Some("05.02.08.".into()),
			kode_rekening: Some("5.1.02.01.01.0024".into()),
			uraian: format!("item {id}"),
			satuan: Some("paket".into()),
			volume: 1.0,
			harga_satuan: jumlah,
			jumlah,
			bulan,
			volume_bulan: [0.0; 12],
		}
	}

	fn spend(id: &str, rapbs: Option<&str>, tanggal: &str, saldo: i64) -> KasRow {
		KasRow {
			id: id.into(),
			id_ref_bku: 15,
			tanggal: tanggal.into(),
			saldo,
			fund_id: 1,
			id_rapbs: rapbs.map(String::from),
			..Default::default()
		}
	}

	#[test]
	fn statuses_and_totals() {
		let mut b1 = [0; 12];
		b1[0] = 1_000;
		let mut b2 = [0; 12];
		b2[1] = 500;
		let mut b3 = [0; 12];
		b3[8] = 700;
		let mut b4 = [0; 12];
		b4[2] = 300;
		let items = vec![item("i1", 1_000, b1), item("i2", 500, b2), item("i3", 700, b3), item("i4", 300, b4)];
		let rows = vec![
			spend("k1", Some("i1"), "2026-01-10", 1_000),
			spend("k2", Some("i2"), "2026-02-10", 200),
			spend("k3", Some("i4"), "2026-03-10", 400),
			spend("k4", None, "2026-03-11", 50),
		];
		let input = RealisasiInput { items: &items, rows: &rows, kode_names: &HashMap::new(), rekening_names: &HashMap::new() };
		let r = build_realisasi(2026, 3, None, &input);
		let status: Vec<_> = r.items.iter().map(|i| i.status.as_str()).collect();
		assert_eq!(status, vec!["lunas", "sebagian", "belum_jatuh_tempo", "melampaui"]);
		assert_eq!(r.items[1].tertunda, 300);
		assert_eq!(r.total_pagu, 2_500);
		assert_eq!(r.total_realisasi, 1_650);
		assert_eq!(r.luar_rkas.len(), 1);
		assert_eq!(r.realisasi_bulan[2], 450);
		assert_eq!(r.persen, 66.0);
	}
}
