//! Export Excel untuk kertas kerja, realisasi, dan semua laporan sekaligus.

use std::path::PathBuf;

use rust_xlsxwriter::Workbook;
use tauri::State;

use super::rkas::{kertas_kerja, load_realisasi, prefixes, KertasKerja};
use crate::error::{validate_year, AppError, AppResult};
use crate::export::xlsx::{add_book_sheet, add_table_sheet, Cell, TableSheet};
use crate::pengaturan;
use crate::repo::arkas::queries::{self, SchoolInfo};
use crate::services::book::{month_name, BookKind, BookRequest};
use crate::services::realisasi::Realisasi;
use crate::state::AppState;

fn target(path: &str) -> AppResult<PathBuf> {
	let mut p = PathBuf::from(path.trim());
	if p.as_os_str().is_empty() {
		return Err(AppError::InvalidInput("lokasi file belum dipilih".into()));
	}
	if p.extension().is_none_or(|e| !e.eq_ignore_ascii_case("xlsx")) {
		p.set_extension("xlsx");
	}
	Ok(p)
}

fn context(state: &AppState, year: i32, fund: Option<i64>) -> AppResult<(SchoolInfo, String)> {
	let (school, funds) = state.with_arkas(|db| Ok((queries::school_info(db, Some(year))?, queries::fund_sources(db, year)?)))?;
	let label = fund
		.and_then(|f| funds.iter().find(|s| s.id == f).map(|s| s.name.clone()))
		.unwrap_or_else(|| "Semua sumber dana".into());
	Ok((school, label))
}

fn info(school: &SchoolInfo, fund_label: &str) -> Vec<(&'static str, String)> {
	vec![
		("Nama Sekolah", school.nama.clone().unwrap_or_default()),
		("NPSN", school.npsn.clone().unwrap_or_default()),
		("Kabupaten/Kota", school.kabupaten.clone().unwrap_or_default()),
		("Sumber Dana", fund_label.to_string()),
	]
}

pub(crate) fn kertas_kerja_sheet<'a>(kk: &KertasKerja, school: &SchoolInfo, fund_label: &str) -> TableSheet<'a> {
	let mut headers = vec![
		("Kode Kegiatan", 13.0),
		("Nama Kegiatan", 30.0),
		("Kode Rekening", 18.0),
		("Uraian", 34.0),
		("Volume", 8.0),
		("Satuan", 9.0),
		("Harga Satuan", 13.0),
		("Jumlah", 14.0),
	];
	headers.extend(["Jan", "Feb", "Mar", "Apr", "Mei", "Jun", "Jul", "Agu", "Sep", "Okt", "Nov", "Des"].map(|m| (m, 11.0)));
	let mut items: Vec<_> = kk.items.iter().collect();
	items.sort_by(|a, b| (a.kode_kegiatan.as_deref(), a.kode_rekening.as_deref()).cmp(&(b.kode_kegiatan.as_deref(), b.kode_rekening.as_deref())));
	let mut month_totals = [0i64; 12];
	let rows = items
		.iter()
		.map(|it| {
			let kode = it.kode_kegiatan.clone().unwrap_or_default();
			let mut row = vec![
				Cell::Text(kode.clone()),
				Cell::Text(kk.kode_names.get(&kode).cloned().unwrap_or_default()),
				Cell::Text(it.kode_rekening.clone().unwrap_or_default()),
				Cell::Text(it.uraian.clone()),
				Cell::Num(it.volume),
				Cell::Text(it.satuan.clone().unwrap_or_default()),
				Cell::Money(it.harga_satuan),
				Cell::Money(it.jumlah),
			];
			for (m, v) in it.bulan.iter().enumerate() {
				month_totals[m] += v;
				row.push(if *v == 0 { Cell::Empty } else { Cell::Money(*v) });
			}
			row
		})
		.collect();
	let mut totals = vec![Cell::Text("JUMLAH".into()), Cell::Empty, Cell::Empty, Cell::Empty, Cell::Empty, Cell::Empty, Cell::Empty];
	totals.push(Cell::Money(kk.items.iter().map(|i| i.jumlah).sum()));
	totals.extend(month_totals.iter().map(|v| Cell::Money(*v)));
	TableSheet {
		name: "Kertas Kerja",
		title: "KERTAS KERJA RKAS",
		subtitle: format!("TAHUN ANGGARAN {}", kk.year),
		info: info(school, fund_label),
		headers,
		rows,
		totals: Some(totals),
	}
}

/// Lembar "RKAS Tahunan" bertingkat: program → kegiatan → sub kegiatan → item,
/// dengan warna per tingkat dan baris TOTAL BELANJA.
fn add_rkas_tahunan_sheet(
	wb: &mut Workbook,
	kk: &KertasKerja,
	school: &SchoolInfo,
	fund_label: &str,
	judul: &str,
	triwulan: bool,
) -> AppResult<()> {
	use rust_xlsxwriter::{Format, FormatAlign, FormatBorder};
	use std::collections::HashMap;

	let ws = wb.add_worksheet();
	ws.set_name("RKAS")?;
	ws.set_landscape();
	ws.set_paper_size(9);
	ws.set_margins(0.4, 0.4, 0.5, 0.5, 0.3, 0.3);
	ws.set_print_fit_to_pages(1, 0);
	// Kolom tambahan Triwulan I-IV setelah Total untuk format triwulan.
	let last: u16 = if triwulan { 12 } else { 8 };
	for (c, w) in [5.0, 14.0, 35.0, 18.0, 34.0, 8.0, 10.0, 15.0, 18.0, 15.0, 15.0, 15.0, 15.0].iter().enumerate().take(last as usize + 1) {
		ws.set_column_width(c as u16, *w)?;
	}
	let tw_of = |bulan: &[i64; 12]| -> [i64; 4] { std::array::from_fn(|k| bulan[k * 3..k * 3 + 3].iter().sum()) };

	let base = Format::new().set_border(FormatBorder::Thin).set_align(FormatAlign::Top).set_text_wrap().set_font_size(10);
	let money = base.clone().set_num_format("#,##0");
	let f_title = Format::new().set_bold().set_font_size(14).set_align(FormatAlign::Center);
	let f_center = Format::new().set_align(FormatAlign::Center);
	let f_head = Format::new()
		.set_bold()
		.set_font_size(10)
		.set_font_color("#FFFFFF")
		.set_background_color("#2563EB")
		.set_border(FormatBorder::Thin)
		.set_align(FormatAlign::Center)
		.set_align(FormatAlign::VerticalCenter)
		.set_text_wrap();
	// Warna latar per tingkat kode kegiatan (0 = program).
	let level_color = ["#FCE4EC", "#D4EDDA", "#E8F5E9"];
	let row_fmt = |level: Option<usize>| -> (Format, Format) {
		match level {
			Some(l) => {
				let color = level_color[l.min(2)];
				(base.clone().set_bold().set_background_color(color), money.clone().set_bold().set_background_color(color))
			}
			None => (base.clone(), money.clone()),
		}
	};

	ws.merge_range(0, 0, 0, last, &format!("RKAS - {fund_label}"), &f_title)?;
	ws.merge_range(1, 0, 1, last, &format!("Tahun Anggaran: {}", kk.year), &f_center)?;
	let sekolah = format!(
		"Sekolah: {} (NPSN: {})",
		school.nama.clone().unwrap_or_default(),
		school.npsn.clone().unwrap_or_default()
	);
	ws.merge_range(2, 0, 2, last, &sekolah, &f_center)?;
	ws.merge_range(3, 0, 3, last, &format!("Format: Rincian RKAS ({judul})"), &f_center)?;

	let headers = ["No", "Kode Kegiatan", "Nama Kegiatan", "Kode Rekening", "Uraian", "Vol", "Satuan", "Harga Satuan", "Total"];
	let tw_headers = ["Triwulan I", "Triwulan II", "Triwulan III", "Triwulan IV"];
	for (c, h) in headers.iter().chain(tw_headers.iter().take(if triwulan { 4 } else { 0 })).enumerate() {
		ws.write_string_with_format(5, c as u16, *h, &f_head)?;
	}
	ws.set_row_height(5, 25)?;
	ws.set_repeat_rows(5, 5)?;

	let mut items: Vec<_> = kk.items.iter().collect();
	items.sort_by(|a, b| (a.kode_kegiatan.as_deref(), a.kode_rekening.as_deref()).cmp(&(b.kode_kegiatan.as_deref(), b.kode_rekening.as_deref())));
	let mut totals: HashMap<String, i64> = HashMap::new();
	let mut tw_totals: HashMap<String, [i64; 4]> = HashMap::new();
	for it in &items {
		let tw = tw_of(&it.bulan);
		for p in it.kode_kegiatan.as_deref().map(prefixes).unwrap_or_default() {
			*totals.entry(p.clone()).or_default() += it.jumlah;
			let t = tw_totals.entry(p).or_default();
			(0..4).for_each(|k| t[k] += tw[k]);
		}
	}
	let name_of = |kode: &str, level: usize| {
		kk.kode_names.get(kode).cloned().unwrap_or_else(|| {
			if level == 1 { format!("Kegiatan {}", kode.trim_end_matches('.')) } else { String::new() }
		})
	};

	let mut r: u32 = 6;
	let mut no = 0;
	let mut seen = std::collections::HashSet::new();
	for it in &items {
		let codes = it.kode_kegiatan.as_deref().map(prefixes).unwrap_or_default();
		for (level, code) in codes.iter().enumerate() {
			if !seen.insert(code.clone()) {
				continue;
			}
			no += 1;
			let (t, m) = row_fmt(Some(level));
			ws.write_number_with_format(r, 0, no as f64, &t)?;
			ws.write_string_with_format(r, 1, code, &t)?;
			ws.write_string_with_format(r, 2, name_of(code, level), &t)?;
			for c in 3..8 {
				ws.write_blank(r, c, &t)?;
			}
			ws.write_number_with_format(r, 8, totals.get(code).copied().unwrap_or(0) as f64, &m)?;
			if triwulan {
				for (k, v) in tw_totals.get(code).copied().unwrap_or_default().iter().enumerate() {
					ws.write_number_with_format(r, 9 + k as u16, *v as f64, &m)?;
				}
			}
			r += 1;
		}
		no += 1;
		let (t, m) = row_fmt(None);
		let kode = it.kode_kegiatan.clone().unwrap_or_default();
		ws.write_number_with_format(r, 0, no as f64, &t)?;
		ws.write_string_with_format(r, 1, &kode, &t)?;
		ws.write_string_with_format(r, 2, codes.last().map(|c| name_of(c, codes.len() - 1)).unwrap_or_default(), &t)?;
		ws.write_string_with_format(r, 3, it.kode_rekening.as_deref().unwrap_or_default(), &t)?;
		ws.write_string_with_format(r, 4, &it.uraian, &t)?;
		ws.write_number_with_format(r, 5, it.volume, &t)?;
		ws.write_string_with_format(r, 6, it.satuan.as_deref().unwrap_or_default(), &t)?;
		ws.write_number_with_format(r, 7, it.harga_satuan as f64, &m)?;
		ws.write_number_with_format(r, 8, it.jumlah as f64, &m)?;
		if triwulan {
			for (k, v) in tw_of(&it.bulan).iter().enumerate() {
				ws.write_number_with_format(r, 9 + k as u16, *v as f64, &m)?;
			}
		}
		r += 1;
	}
	let total_fmt = base.clone().set_bold().set_font_size(11).set_background_color("#D1D5DB");
	ws.merge_range(r, 0, r, 7, "TOTAL BELANJA", &total_fmt)?;
	let total_money = total_fmt.clone().set_num_format("#,##0");
	ws.write_number_with_format(r, 8, items.iter().map(|i| i.jumlah).sum::<i64>() as f64, &total_money)?;
	if triwulan {
		for k in 0..4 {
			let v: i64 = items.iter().map(|i| tw_of(&i.bulan)[k]).sum();
			ws.write_number_with_format(r, 9 + k as u16, v as f64, &total_money)?;
		}
	}
	Ok(())
}

/// Nama kode rekening induk belanja (tidak tersimpan di ARKAS).
fn nama_rekening_induk(kode: &str) -> &str {
	match kode {
		"5" => "BELANJA",
		"5.1" => "BELANJA OPERASI",
		"5.1.01" => "BELANJA PEGAWAI",
		"5.1.02" => "BELANJA BARANG DAN JASA",
		"5.1.02.01" => "BELANJA BARANG",
		"5.1.02.02" => "BELANJA JASA",
		"5.1.02.03" => "BELANJA PEMELIHARAAN",
		"5.1.02.04" => "BELANJA PERJALANAN DINAS",
		"5.1.02.05" => "BELANJA UANG DAN/ATAU JASA UNTUK DIBERIKAN KEPADA PIHAK KETIGA/PIHAK LAIN/MASYARAKAT",
		"5.2" => "BELANJA MODAL",
		"5.2.01" => "BELANJA MODAL TANAH",
		"5.2.02" => "BELANJA MODAL PERALATAN DAN MESIN",
		"5.2.03" => "BELANJA MODAL GEDUNG DAN BANGUNAN",
		"5.2.04" => "BELANJA MODAL JALAN, IRIGASI, DAN JARINGAN",
		"5.2.05" => "BELANJA MODAL ASET TETAP LAINNYA",
		"5.2.06" => "BELANJA MODAL ASET LAINNYA",
		_ => kode,
	}
}

/// Lembar Kertas Kerja Unit Kerja: rincian per kode rekening induk + rencana per triwulan.
fn add_lembar_kertas_kerja_sheet(wb: &mut Workbook, kk: &KertasKerja, school: &SchoolInfo, pemerintah: &str) -> AppResult<()> {
	use rust_xlsxwriter::{Format, FormatAlign, FormatBorder};
	use std::collections::BTreeMap;

	let ws = wb.add_worksheet();
	ws.set_name("Lembar Kertas Kerja")?;
	ws.set_paper_size(9);
	ws.set_margins(0.6, 0.6, 0.6, 0.6, 0.3, 0.3);
	ws.set_print_fit_to_pages(1, 0);
	for (c, w) in [14.0, 30.0, 15.0, 15.0, 15.0, 15.0, 17.0].iter().enumerate() {
		ws.set_column_width(c as u16, *w)?;
	}
	let center_bold = Format::new().set_bold().set_align(FormatAlign::Center);
	let cell = Format::new().set_border(FormatBorder::Thin).set_align(FormatAlign::Top).set_text_wrap();
	let bold = cell.clone().set_bold();
	let money = cell.clone().set_num_format("#,##0");
	let money_bold = money.clone().set_bold();
	let head = bold.clone().set_align(FormatAlign::Center);

	let mut r: u32 = 0;
	for (i, t) in ["LEMBAR KERTAS KERJA", "UNIT KERJA", pemerintah, &format!("TAHUN ANGGARAN {}", kk.year)].iter().enumerate() {
		ws.merge_range(r, 0, r, 6, t, &if i == 0 { center_bold.clone().set_font_size(13) } else { center_bold.clone() })?;
		r += 1;
	}
	r += 1;
	ws.write_string(r, 0, "Urusan Pemerintahan")?;
	ws.write_string(r, 2, ": 1.01 - PENDIDIKAN")?;
	r += 1;
	ws.write_string(r, 0, "Organisasi")?;
	ws.write_string(
		r,
		2,
		format!(": {} - {}", school.npsn.clone().unwrap_or_default(), school.nama.clone().unwrap_or_default()),
	)?;
	r += 2;

	// Rincian per kode rekening induk (5, 5.1, 5.1.02, 5.1.02.01, ...).
	let mut jumlah: BTreeMap<String, i64> = BTreeMap::new();
	for k in ["5", "5.1", "5.1.02", "5.1.02.01", "5.1.02.02", "5.1.02.03", "5.1.02.04", "5.2"] {
		jumlah.insert(k.into(), 0);
	}
	for it in &kk.items {
		let parts: Vec<&str> = it.kode_rekening.as_deref().unwrap_or("").split('.').filter(|p| !p.is_empty()).collect();
		for n in 1..=parts.len().min(4) {
			*jumlah.entry(parts[..n].join(".")).or_default() += it.jumlah;
		}
	}
	let mut kode: Vec<&String> = jumlah.keys().filter(|k| k.starts_with('5')).collect();
	kode.sort_by_key(|k| k.split('.').map(|p| p.parse::<u32>().unwrap_or(0)).collect::<Vec<_>>());

	ws.merge_range(r, 0, r, 6, "Rincian Anggaran Pendapatan dan Belanja Unit Kerja", &center_bold)?;
	r += 1;
	ws.write_string_with_format(r, 0, "Kode Rekening", &head)?;
	ws.merge_range(r, 1, r, 5, "Uraian", &head)?;
	ws.write_string_with_format(r, 6, "Jumlah (Rp)", &head)?;
	r += 1;
	ws.write_blank(r, 0, &bold)?;
	ws.merge_range(r, 1, r, 5, "JUMLAH PENDAPATAN", &bold)?;
	ws.write_blank(r, 6, &bold)?;
	r += 1;
	for k in kode {
		let level = k.split('.').count();
		let (t, m) = if k == "5" { (&bold, &money_bold) } else { (&cell, &money) };
		let indent = "    ".repeat(level.saturating_sub(2));
		ws.write_string_with_format(r, 0, k, t)?;
		ws.merge_range(r, 1, r, 5, &format!("{indent}{}", nama_rekening_induk(k)), t)?;
		ws.write_number_with_format(r, 6, jumlah[k] as f64, m)?;
		r += 1;
	}
	let total: i64 = kk.items.iter().map(|i| i.jumlah).sum();
	ws.write_blank(r, 0, &bold)?;
	ws.merge_range(r, 1, r, 5, "Jumlah BELANJA", &bold)?;
	ws.write_number_with_format(r, 6, total as f64, &money_bold)?;
	r += 2;

	// Rencana per triwulan.
	ws.merge_range(r, 0, r, 6, "Rencana Pelaksanaan Anggaran Unit Kerja per Triwulan", &center_bold)?;
	r += 1;
	for (c, h) in ["No", "Uraian", "TW I", "TW II", "TW III", "TW IV", "Jumlah"].iter().enumerate() {
		ws.write_string_with_format(r, c as u16, *h, &head)?;
	}
	r += 1;
	let tw = |modal: Option<bool>| -> [i64; 4] {
		std::array::from_fn(|k| {
			kk.items
				.iter()
				.filter(|i| modal.is_none_or(|m| i.kode_rekening.as_deref().unwrap_or("").starts_with("5.2") == m))
				.map(|i| i.bulan[k * 3..k * 3 + 3].iter().sum::<i64>())
				.sum()
		})
	};
	for (no, uraian, v) in [("1", "Pendapatan", tw(None)), ("2.1", "Belanja Operasi", tw(Some(false))), ("2.2", "Belanja Modal", tw(Some(true)))] {
		ws.write_string_with_format(r, 0, no, &cell)?;
		ws.write_string_with_format(r, 1, uraian, &cell)?;
		for (k, x) in v.iter().enumerate() {
			ws.write_number_with_format(r, 2 + k as u16, *x as f64, &money)?;
		}
		ws.write_number_with_format(r, 6, v.iter().sum::<i64>() as f64, &money_bold)?;
		r += 1;
	}
	Ok(())
}

fn status_label(s: &str) -> &'static str {
	match s {
		"lunas" => "Lunas",
		"sebagian" => "Sebagian",
		"belum" => "Belum dibelanjakan",
		"melampaui" => "Melampaui pagu",
		_ => "Belum jatuh tempo",
	}
}

fn realisasi_sheet<'a>(r: &Realisasi, school: &SchoolInfo, fund_label: &str) -> TableSheet<'a> {
	let headers = vec![
		("Kode Kegiatan", 13.0),
		("Nama Kegiatan", 28.0),
		("Kode Rekening", 18.0),
		("Uraian", 34.0),
		("Pagu", 14.0),
		("Rencana s.d. Bulan", 14.0),
		("Realisasi", 14.0),
		("Sisa", 14.0),
		("%", 7.0),
		("Status", 16.0),
	];
	let mut rows: Vec<Vec<Cell>> = r
		.items
		.iter()
		.map(|i| {
			vec![
				Cell::Text(i.kode_kegiatan.clone().unwrap_or_default()),
				Cell::Text(i.nama_kegiatan.clone().unwrap_or_default()),
				Cell::Text(i.kode_rekening.clone().unwrap_or_default()),
				Cell::Text(i.uraian.clone()),
				Cell::Money(i.pagu),
				Cell::Money(i.rencana_sd),
				Cell::Money(i.total_realisasi),
				Cell::Money(i.sisa),
				Cell::Num(i.persen),
				Cell::Text(status_label(&i.status).into()),
			]
		})
		.collect();
	for o in &r.luar_rkas {
		rows.push(vec![
			Cell::Empty,
			Cell::Text("Di luar RKAS aktif".into()),
			Cell::Text(o.kode_rekening.clone().unwrap_or_default()),
			Cell::Text(o.uraian.clone()),
			Cell::Empty,
			Cell::Empty,
			Cell::Money(o.nominal),
			Cell::Empty,
			Cell::Empty,
			Cell::Text(o.tanggal.clone()),
		]);
	}
	let totals = vec![
		Cell::Text("JUMLAH".into()),
		Cell::Empty,
		Cell::Empty,
		Cell::Empty,
		Cell::Money(r.total_pagu),
		Cell::Money(r.items.iter().map(|i| i.rencana_sd).sum()),
		Cell::Money(r.total_realisasi),
		Cell::Money(r.total_pagu - r.total_realisasi),
		Cell::Num(r.persen),
		Cell::Empty,
	];
	TableSheet {
		name: "Realisasi",
		title: "REALISASI BELANJA",
		subtitle: format!("TAHUN ANGGARAN {} (ACUAN S.D. {})", r.year, month_name(r.upto).to_uppercase()),
		info: info(school, fund_label),
		headers,
		rows,
		totals: Some(totals),
	}
}

#[tauri::command]
#[allow(clippy::too_many_arguments)] // parameter command Tauri dikirim satu per satu dari frontend
pub fn export_kertas_kerja_xlsx(
	state: State<'_, AppState>,
	year: i32,
	fund: Option<i64>,
	search: Option<String>,
	start: Option<u32>,
	end: Option<u32>,
	judul: Option<String>,
	triwulan: Option<bool>,
	lembar: Option<bool>,
	path: String,
) -> AppResult<String> {
	let year = validate_year(year)?;
	let fund = fund.filter(|f| *f != 0);
	let path = target(&path)?;
	let mut kk = kertas_kerja(state.clone(), year, fund)?;
	let (school, label) = context(&state, year, fund)?;
	// Periode (triwulan/bulan): jumlah & volume hanya untuk bulan terpilih, item kosong dibuang.
	let (a, b) = (start.unwrap_or(1).clamp(1, 12) as usize, end.unwrap_or(12).clamp(1, 12) as usize);
	if (a, b) != (1, 12) {
		for it in &mut kk.items {
			it.jumlah = it.bulan[a - 1..b].iter().sum();
			it.volume = it.volume_bulan[a - 1..b].iter().sum();
		}
		kk.items.retain(|i| i.jumlah > 0);
	}
	// Sama dengan pencarian di layar: hanya item yang cocok yang diekspor.
	let q = search.map(|s| s.trim().to_lowercase()).filter(|s| !s.is_empty());
	if let Some(q) = &q {
		let names = kk.kode_names.clone();
		kk.items.retain(|i| {
			let nama = i.kode_kegiatan.as_ref().and_then(|k| names.get(k));
			[Some(&i.uraian), i.kode_rekening.as_ref(), i.kode_kegiatan.as_ref(), nama]
				.into_iter()
				.flatten()
				.any(|v| v.to_lowercase().contains(q.as_str()))
		});
	}
	let mut wb = Workbook::new();
	if lembar.unwrap_or(false) {
		let p = pengaturan::gabung(&pengaturan::load(&state.app_db())?, &pengaturan::bawaan(&school));
		add_lembar_kertas_kerja_sheet(&mut wb, &kk, &school, &p.kop.pemerintah)?;
	} else {
		add_rkas_tahunan_sheet(&mut wb, &kk, &school, &label, judul.as_deref().unwrap_or("Tahunan"), triwulan.unwrap_or(false))?;
	}
	wb.save(&path)?;
	Ok(path.display().to_string())
}

#[tauri::command]
pub fn export_realisasi_xlsx(state: State<'_, AppState>, year: i32, upto: Option<u32>, fund: Option<i64>, path: String) -> AppResult<String> {
	let year = validate_year(year)?;
	let fund = fund.filter(|f| *f != 0);
	let path = target(&path)?;
	let r = load_realisasi(&state, year, upto.unwrap_or(12).clamp(1, 12), fund)?;
	let (school, label) = context(&state, year, fund)?;
	let mut wb = Workbook::new();
	add_table_sheet(&mut wb, &realisasi_sheet(&r, &school, &label))?;
	wb.save(&path)?;
	Ok(path.display().to_string())
}

/// Semua laporan satu tahun dalam satu file Excel: BKU, buku pembantu, realisasi, kertas kerja.
#[tauri::command]
pub fn export_all_xlsx(state: State<'_, AppState>, year: i32, fund: Option<i64>, path: String) -> AppResult<String> {
	let year = validate_year(year)?;
	let fund = fund.filter(|f| *f != 0);
	let path = target(&path)?;
	let (school, label) = context(&state, year, fund)?;
	let settings = pengaturan::gabung(&pengaturan::load(&state.app_db())?, &pengaturan::bawaan(&school));

	let mut wb = Workbook::new();
	for kind in [BookKind::Umum, BookKind::Bank, BookKind::Tunai, BookKind::Pajak] {
		let book = super::bku::load_book(&state, &BookRequest { kind, year, month: None, until: None, fund })?;
		add_book_sheet(&mut wb, &book, &school, &settings, &label)?;
	}
	let r = load_realisasi(&state, year, 12, fund)?;
	add_table_sheet(&mut wb, &realisasi_sheet(&r, &school, &label))?;
	let kk = kertas_kerja(state.clone(), year, fund)?;
	add_table_sheet(&mut wb, &kertas_kerja_sheet(&kk, &school, &label))?;
	wb.save(&path)?;
	Ok(path.display().to_string())
}
