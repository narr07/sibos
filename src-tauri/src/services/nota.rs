//! Pengelompokan transaksi belanja menjadi nota/bukti untuk dicetak (Bukti Pengeluaran, Kwitansi A2).
//! Pajak hanya ditampilkan sesuai yang dicatat bendahara; tidak ada perhitungan otomatis.

use std::collections::{HashMap, HashSet};

use serde::Serialize;

use super::tax::jenis_pajak;
use crate::repo::app::nota::{Merge, PrintOverride};
use crate::repo::arkas::bku::{KasRow, NotaInfo};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NotaItem {
	pub id: String,
	pub tanggal: String,
	pub no_bukti: Option<String>,
	pub uraian: String,
	pub kode_rekening: Option<String>,
	pub kode_kegiatan: Option<String>,
	pub volume: Option<f64>,
	pub satuan: Option<String>,
	pub harga_satuan: Option<i64>,
	pub nominal: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NotaTax {
	pub id: String,
	pub jenis: Option<String>,
	pub uraian: String,
	pub nominal: i64,
	/// Tanggal setor bila sudah dicatat (hanya untuk pajak input manual).
	pub tanggal_setor: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NotaGroup {
	/// Kunci stabil: `nota:<id>`, `bukti:<no>:<tanggal>`, atau `merge:<id>`.
	pub key: String,
	pub kind: String,
	pub tanggal: String,
	pub no_bukti: String,
	pub uraian: String,
	pub fund_name: String,
	pub nota: Option<NotaInfo>,
	pub items: Vec<NotaItem>,
	/// Pajak yang diinput manual untuk nota ini (dipakai di cetakan).
	pub taxes: Vec<NotaTax>,
	/// Pajak yang tercatat di ARKAS (mis. PPN SIPLah) — hanya informasi, tidak dicetak sebagai potongan.
	pub arkas_taxes: Vec<NotaTax>,
	pub is_siplah: bool,
	pub total: i64,
	pub total_pajak: i64,
	pub diterima: i64,
	pub merge_id: Option<String>,
	pub printed_bukti: Option<String>,
	pub printed_a2: Option<String>,
	pub printed_nota: Option<String>,
	#[serde(rename = "override")]
	pub print_override: Option<PrintOverride>,
	pub file_count: usize,
}

pub struct NotaInput<'a> {
	pub rows: &'a [KasRow],
	pub notas: &'a HashMap<String, NotaInfo>,
	pub merges: &'a [Merge],
	pub statuses: &'a HashMap<(String, String), String>,
	pub overrides: &'a HashMap<String, PrintOverride>,
	pub file_counts: &'a HashMap<String, usize>,
	pub uraian_overrides: &'a HashMap<String, String>,
	/// Pajak manual per kunci nota.
	pub manual_taxes: &'a HashMap<String, Vec<NotaTax>>,
}

fn is_belanja(r: &KasRow) -> bool {
	matches!(r.id_ref_bku, 4 | 15 | 24 | 35)
}

fn is_tax_in(r: &KasRow) -> bool {
	matches!(r.id_ref_bku, 10 | 30)
}

fn month_of(tanggal: &str) -> u32 {
	tanggal.get(5..7).and_then(|m| m.parse().ok()).unwrap_or(0)
}

pub fn group_notas(input: &NotaInput<'_>, month: Option<u32>, fund: Option<i64>) -> Vec<NotaGroup> {
	let merged: HashMap<&str, &Merge> =
		input.merges.iter().flat_map(|m| m.items.iter().map(move |i| (i.as_str(), m))).collect();

	let mut order: Vec<String> = Vec::new();
	let mut groups: HashMap<String, NotaGroup> = HashMap::new();
	let mut owner: HashMap<&str, String> = HashMap::new();

	for r in input.rows.iter().filter(|r| is_belanja(r) && fund.is_none_or(|f| f == r.fund_id)) {
		let (key, kind) = if let Some(m) = merged.get(r.id.as_str()) {
			(format!("merge:{}", m.id), "merge")
		} else if let Some(n) = &r.id_kas_nota {
			(format!("nota:{n}"), "nota")
		} else {
			(format!("bukti:{}:{}", r.no_bukti.as_deref().unwrap_or("-"), r.tanggal), "bukti")
		};
		owner.insert(r.id.as_str(), key.clone());
		let group = groups.entry(key.clone()).or_insert_with(|| {
			order.push(key.clone());
			let merge = merged.get(r.id.as_str()).copied();
			NotaGroup {
				key: key.clone(),
				kind: kind.into(),
				tanggal: merge.and_then(|m| m.tanggal.clone()).unwrap_or_else(|| r.tanggal.clone()),
				no_bukti: merge.map(|m| m.no_bukti.clone()).or_else(|| r.no_bukti.clone()).unwrap_or_default(),
				uraian: merge.map(|m| m.uraian.clone()).unwrap_or_default(),
				fund_name: r.fund_name.clone(),
				nota: r.id_kas_nota.as_ref().and_then(|n| input.notas.get(n)).cloned(),
				items: Vec::new(),
				taxes: input.manual_taxes.get(&key).cloned().unwrap_or_default(),
				arkas_taxes: Vec::new(),
				is_siplah: r.id_kas_nota.as_ref().and_then(|n| input.notas.get(n)).is_some_and(|n| n.is_siplah),
				total: 0,
				total_pajak: 0,
				diterima: 0,
				merge_id: merge.map(|m| m.id.clone()),
				printed_bukti: input.statuses.get(&("bukti".into(), key.clone())).cloned(),
				printed_a2: input.statuses.get(&("a2".into(), key.clone())).cloned(),
				printed_nota: input.statuses.get(&("nota".into(), key.clone())).cloned(),
				print_override: input.overrides.get(&key).cloned(),
				file_count: input.file_counts.get(&key).copied().unwrap_or(0),
			}
		});
		if kind != "merge" && r.tanggal < group.tanggal {
			group.tanggal = r.tanggal.clone();
		}
		group.items.push(NotaItem {
			id: r.id.clone(),
			tanggal: r.tanggal.clone(),
			no_bukti: r.no_bukti.clone(),
			uraian: input.uraian_overrides.get(&r.id).cloned().unwrap_or_else(|| r.uraian.clone()),
			kode_rekening: r.kode_rekening.clone(),
			kode_kegiatan: r.kode_kegiatan.clone(),
			volume: r.volume,
			satuan: r.satuan.clone(),
			harga_satuan: r.harga_satuan,
			nominal: r.saldo,
		});
		group.total += r.saldo;
	}

	// Pajak dipungut yang tercatat di ARKAS untuk item di dalam kelompok (informasi saja).
	let mut seen_tax: HashSet<&str> = HashSet::new();
	for r in input.rows.iter().filter(|r| is_tax_in(r)) {
		let Some(key) = r.parent_id.as_deref().and_then(|p| owner.get(p)) else { continue };
		if !seen_tax.insert(r.id.as_str()) {
			continue;
		}
		if let Some(g) = groups.get_mut(key) {
			g.arkas_taxes.push(NotaTax {
				id: r.id.clone(),
				jenis: jenis_pajak(r).map(String::from),
				uraian: r.uraian.clone(),
				nominal: r.saldo,
				tanggal_setor: None,
			});
		}
	}

	let mut out: Vec<NotaGroup> = order
		.into_iter()
		.filter_map(|k| groups.remove(&k))
		.filter(|g| month.is_none_or(|m| month_of(&g.tanggal) == m))
		.map(|mut g| {
			g.total_pajak = g.taxes.iter().map(|t| t.nominal).sum();
			g.diterima = g.total - g.total_pajak;
			if g.uraian.is_empty() {
				g.uraian = match g.items.len() {
					1 => g.items[0].uraian.clone(),
					n => format!("{} dan {} item lainnya", g.items[0].uraian, n - 1),
				};
			}
			if g.no_bukti.is_empty() {
				g.no_bukti = g.nota.as_ref().and_then(|n| n.no_bukti.clone()).unwrap_or_default();
			}
			g
		})
		.collect();
	out.sort_by(|a, b| (a.tanggal.as_str(), a.no_bukti.as_str()).cmp(&(b.tanggal.as_str(), b.no_bukti.as_str())));
	out
}

#[cfg(test)]
mod tests {
	use super::*;

	fn row(id: &str, ref_bku: i64, saldo: i64, nota: Option<&str>, bukti: &str) -> KasRow {
		KasRow {
			id: id.into(),
			id_ref_bku: ref_bku,
			tanggal: "2026-09-04".into(),
			uraian: format!("uraian {id}"),
			saldo,
			fund_id: 1,
			fund_name: "BOS Reguler".into(),
			id_kas_nota: nota.map(String::from),
			no_bukti: (!bukti.is_empty()).then(|| bukti.to_string()),
			..Default::default()
		}
	}

	#[test]
	fn groups_by_nota_merge_and_tax() {
		let mut ppn = row("t1", 10, 81_759, None, "");
		ppn.parent_id = Some("a2".into());
		ppn.is_ppn = true;
		let rows = vec![
			row("a1", 15, 25_500, Some("N1"), "BNU87"),
			row("a2", 15, 260_000, Some("N1"), "BNU87"),
			row("b1", 15, 450_000, None, "BNU85"),
			row("c1", 15, 150_000, None, "BNU86"),
			row("c2", 15, 825_000, None, "BNU88"),
			ppn,
			row("in", 2, 1_000_000, None, "BBU01"),
		];
		let merges = vec![Merge {
			id: "m1".into(),
			no_bukti: "BNU86-88".into(),
			uraian: "Belanja perlengkapan".into(),
			tanggal: None,
			items: vec!["c1".into(), "c2".into()],
		}];
		let mut notas = HashMap::new();
		notas.insert("N1".to_string(), NotaInfo { id_kas_nota: "N1".into(), nama_toko: Some("Toko ATK".into()), ..Default::default() });
		let empty_s = HashMap::new();
		let empty_o = HashMap::new();
		let empty_f = HashMap::new();
		let empty_u = HashMap::new();
		let mut manual = HashMap::new();
		manual.insert(
			"nota:N1".to_string(),
			vec![NotaTax { id: "m".into(), jenis: Some("PPh 23".into()), uraian: String::new(), nominal: 5_710, tanggal_setor: None }],
		);
		let input = NotaInput {
			rows: &rows,
			notas: &notas,
			merges: &merges,
			statuses: &empty_s,
			overrides: &empty_o,
			file_counts: &empty_f,
			uraian_overrides: &empty_u,
			manual_taxes: &manual,
		};
		let groups = group_notas(&input, Some(9), None);
		assert_eq!(groups.len(), 3, "nota N1, bukti BNU85, gabungan m1");

		let n1 = groups.iter().find(|g| g.key == "nota:N1").unwrap();
		assert_eq!(n1.items.len(), 2);
		assert_eq!(n1.total, 285_500);
		assert_eq!(n1.arkas_taxes[0].nominal, 81_759, "PPN ARKAS hanya informasi");
		assert_eq!(n1.arkas_taxes[0].jenis.as_deref(), Some("PPN"));
		assert_eq!(n1.total_pajak, 5_710, "pajak cetak dari input manual");
		assert_eq!(n1.diterima, 285_500 - 5_710);
		assert_eq!(n1.nota.as_ref().unwrap().nama_toko.as_deref(), Some("Toko ATK"));

		let m = groups.iter().find(|g| g.kind == "merge").unwrap();
		assert_eq!((m.no_bukti.as_str(), m.total, m.uraian.as_str()), ("BNU86-88", 975_000, "Belanja perlengkapan"));

		assert!(group_notas(&input, Some(8), None).is_empty(), "filter bulan");
	}
}
