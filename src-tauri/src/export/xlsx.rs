//! Export buku kas ke Excel (.xlsx) dengan format cetak: judul, identitas sekolah,
//! tabel, jumlah, keterangan penutupan, dan tanda tangan.

use std::path::Path;

use rust_xlsxwriter::{Format, FormatAlign, FormatBorder, FormatUnderline, Workbook, Worksheet};

use crate::error::AppResult;
use crate::pengaturan::Pengaturan;
use crate::repo::arkas::queries::SchoolInfo;
use crate::services::book::{month_name, Book, BookKind};

fn title(kind: BookKind) -> &'static str {
	match kind {
		BookKind::Umum => "BUKU KAS UMUM",
		BookKind::Bank => "BUKU PEMBANTU BANK",
		BookKind::Tunai => "BUKU PEMBANTU KAS TUNAI",
		BookKind::Pajak => "BUKU PEMBANTU PAJAK",
	}
}

fn days_in_month(year: i32, month: u32) -> u32 {
	match month {
		1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
		4 | 6 | 9 | 11 => 30,
		2 if (year % 4 == 0 && year % 100 != 0) || year % 400 == 0 => 29,
		_ => 28,
	}
}

/// Tanggal tanda tangan: akhir bulan, atau 31 Desember untuk satu tahun.
pub fn tanggal_ttd(year: i32, month: Option<u32>) -> String {
	let m = month.unwrap_or(12);
	format!("{} {} {}", days_in_month(year, m), month_name(m), year)
}

/// `2026-09-04` -> `04-09-2026`.
fn tanggal_id(iso: &str) -> String {
	match (iso.get(0..4), iso.get(5..7), iso.get(8..10)) {
		(Some(y), Some(m), Some(d)) => format!("{d}-{m}-{y}"),
		_ => iso.to_string(),
	}
}

fn periode(book: &Book) -> String {
	match book.month {
		Some(m) => format!("BULAN {} TAHUN {}", month_name(m).to_uppercase(), book.year),
		None => format!("TAHUN ANGGARAN {}", book.year),
	}
}

pub fn write_book(
	path: &Path,
	book: &Book,
	school: &SchoolInfo,
	pengaturan: &Pengaturan,
	fund_label: &str,
) -> AppResult<()> {
	let mut wb = Workbook::new();
	add_book_sheet(&mut wb, book, school, pengaturan, fund_label)?;
	wb.save(path)?;
	Ok(())
}

fn sheet_name(kind: BookKind) -> &'static str {
	match kind {
		BookKind::Umum => "BKU",
		BookKind::Bank => "Buku Bank",
		BookKind::Tunai => "Buku Tunai",
		BookKind::Pajak => "Buku Pajak",
	}
}

pub fn add_book_sheet(
	wb: &mut Workbook,
	book: &Book,
	school: &SchoolInfo,
	pengaturan: &Pengaturan,
	fund_label: &str,
) -> AppResult<()> {
	let pajak = book.kind == BookKind::Pajak;
	let ws = wb.add_worksheet();
	ws.set_name(sheet_name(book.kind))?;
	ws.set_landscape();
	// 9 = A4, 14 = Folio (mendekati F4)
	ws.set_paper_size(if pengaturan.cetak.kertas == "F4" { 14 } else { 9 });
	ws.set_margins(0.4, 0.4, 0.5, 0.5, 0.3, 0.3);
	ws.set_print_fit_to_pages(1, 0);

	let f_title = Format::new().set_bold().set_font_size(13).set_align(FormatAlign::Center);
	let f_sub = Format::new().set_bold().set_align(FormatAlign::Center);
	let f_label = Format::new();
	let f_head = Format::new()
		.set_bold()
		.set_align(FormatAlign::Center)
		.set_align(FormatAlign::VerticalCenter)
		.set_text_wrap()
		.set_border(FormatBorder::Thin)
		.set_background_color("#E8F5EE");
	let f_cell = Format::new().set_border(FormatBorder::Thin).set_align(FormatAlign::Top);
	let f_wrap = f_cell.clone().set_text_wrap();
	let f_center = f_cell.clone().set_align(FormatAlign::Center);
	let f_money = f_cell.clone().set_num_format("#,##0");
	let f_total_label = f_cell.clone().set_bold().set_align(FormatAlign::Right);
	let f_total = f_money.clone().set_bold();
	let f_name = Format::new().set_bold().set_underline(FormatUnderline::Single).set_align(FormatAlign::Center);
	let f_mid = Format::new().set_align(FormatAlign::Center);

	// Kolom
	let headers: Vec<(&str, f64)> = if pajak {
		vec![
			("No", 5.0),
			("Tanggal", 12.0),
			("No. Bukti", 12.0),
			("Uraian", 48.0),
			("Jenis Pajak", 13.0),
			("Pungut", 15.0),
			("Setor", 15.0),
			("Saldo", 16.0),
		]
	} else {
		vec![
			("No", 5.0),
			("Tanggal", 12.0),
			("Kode Kegiatan", 13.0),
			("Kode Rekening", 18.0),
			("No. Bukti", 11.0),
			("Uraian", 44.0),
			("Penerimaan", 16.0),
			("Pengeluaran", 16.0),
			("Saldo", 17.0),
		]
	};
	let last_col = (headers.len() - 1) as u16;
	for (i, (_, w)) in headers.iter().enumerate() {
		ws.set_column_width(i as u16, *w)?;
	}

	// Judul & identitas
	let mut r: u32 = 0;
	ws.merge_range(r, 0, r, last_col, title(book.kind), &f_title)?;
	r += 1;
	ws.merge_range(r, 0, r, last_col, &periode(book), &f_sub)?;
	r += 2;
	let identitas = [
		("Nama Sekolah", school.nama.clone().unwrap_or_default()),
		("NPSN", school.npsn.clone().unwrap_or_default()),
		("Kecamatan", school.kecamatan.clone().unwrap_or_default()),
		("Kabupaten/Kota", school.kabupaten.clone().unwrap_or_default()),
		("Provinsi", school.provinsi.clone().unwrap_or_default()),
		("Sumber Dana", fund_label.to_string()),
	];
	for (label, value) in identitas {
		ws.write_string_with_format(r, 0, label, &f_label)?;
		ws.merge_range(r, 2, r, 5.min(last_col), &format!(": {value}"), &f_label)?;
		r += 1;
	}
	r += 1;

	// Header tabel
	let header_row = r;
	for (i, (h, _)) in headers.iter().enumerate() {
		ws.write_string_with_format(r, i as u16, *h, &f_head)?;
	}
	ws.set_row_height(r, 30)?;
	ws.set_repeat_rows(header_row, header_row)?;
	r += 1;

	let (c_pen, c_peng, c_saldo, c_uraian) = if pajak { (5u16, 6u16, 7u16, 3u16) } else { (6, 7, 8, 5) };

	// Saldo awal
	for c in 0..=last_col {
		ws.write_blank(r, c, &f_cell)?;
	}
	ws.write_string_with_format(r, c_uraian, "Saldo awal", &f_wrap)?;
	ws.write_number_with_format(r, c_saldo, book.opening as f64, &f_money)?;
	r += 1;

	for (i, line) in book.lines.iter().enumerate() {
		ws.write_number_with_format(r, 0, (i + 1) as f64, &f_center)?;
		ws.write_string_with_format(r, 1, tanggal_id(&line.tanggal), &f_center)?;
		if pajak {
			ws.write_string_with_format(r, 2, line.no_bukti.as_deref().unwrap_or(""), &f_center)?;
			ws.write_string_with_format(r, 3, &line.uraian, &f_wrap)?;
			ws.write_string_with_format(r, 4, line.jenis_pajak.as_deref().unwrap_or(""), &f_center)?;
		} else {
			ws.write_string_with_format(r, 2, line.kode_kegiatan.as_deref().unwrap_or(""), &f_center)?;
			ws.write_string_with_format(r, 3, line.kode_rekening.as_deref().unwrap_or(""), &f_center)?;
			let bukti = match (line.no_bukti.as_deref(), line.siplah) {
				(Some(b), true) => format!("{b} (SIPLah)"),
				(Some(b), false) => b.to_string(),
				(None, true) => "(SIPLah)".to_string(),
				(None, false) => String::new(),
			};
			ws.write_string_with_format(r, 4, &bukti, &f_center)?;
			ws.write_string_with_format(r, 5, &line.uraian, &f_wrap)?;
		}
		ws.write_number_with_format(r, c_pen, line.penerimaan as f64, &f_money)?;
		ws.write_number_with_format(r, c_peng, line.pengeluaran as f64, &f_money)?;
		ws.write_number_with_format(r, c_saldo, line.saldo as f64, &f_money)?;
		r += 1;
	}

	// Jumlah
	ws.merge_range(r, 0, r, c_pen - 1, "Jumlah", &f_total_label)?;
	ws.write_number_with_format(r, c_pen, book.total_penerimaan as f64, &f_total)?;
	ws.write_number_with_format(r, c_peng, book.total_pengeluaran as f64, &f_total)?;
	ws.write_number_with_format(r, c_saldo, book.closing as f64, &f_total)?;
	r += 2;

	// Keterangan penutupan (BKU umum)
	let f_money_plain = Format::new().set_num_format("\"Rp \"#,##0");
	if book.kind == BookKind::Umum {
		ws.write_string_with_format(
			r,
			1,
			format!("Pada tanggal {}, buku ditutup dengan posisi sebagai berikut:", tanggal_ttd(book.year, book.month)),
			&f_label,
		)?;
		r += 1;
		for (label, value) in [
			("Saldo buku", book.closing),
			("Terdiri dari: saldo bank", book.closing_bank),
			("saldo tunai", book.closing_tunai),
		] {
			ws.write_string_with_format(r, 2, label, &f_label)?;
			ws.write_number_with_format(r, 4, value as f64, &f_money_plain)?;
			r += 1;
		}
		r += 1;
	}

	// Tanda tangan
	let p = &pengaturan.pejabat;
	let (left_a, left_b) = (1u16, 3u16);
	let (right_a, right_b) = (c_peng.saturating_sub(1), last_col);
	let kota = pengaturan.cetak.kota.trim();
	let tempat_tanggal = if kota.is_empty() {
		tanggal_ttd(book.year, book.month)
	} else {
		format!("{kota}, {}", tanggal_ttd(book.year, book.month))
	};
	ws.merge_range(r, right_a, r, right_b, &tempat_tanggal, &f_mid)?;
	r += 1;
	ws.merge_range(r, left_a, r, left_b, "Menyetujui,", &f_mid)?;
	r += 1;
	ws.merge_range(r, left_a, r, left_b, "Kepala Sekolah", &f_mid)?;
	ws.merge_range(r, right_a, r, right_b, "Bendahara", &f_mid)?;
	r += 4;
	ws.merge_range(r, left_a, r, left_b, &p.kepala_sekolah, &f_name)?;
	ws.merge_range(r, right_a, r, right_b, &p.bendahara, &f_name)?;
	r += 1;
	ws.merge_range(r, left_a, r, left_b, &format!("NIP. {}", p.nip_kepala_sekolah), &f_mid)?;
	ws.merge_range(r, right_a, r, right_b, &format!("NIP. {}", p.nip_bendahara), &f_mid)?;
	Ok(())
}

/// Isi sel untuk lembar tabel umum.
pub enum Cell {
	Text(String),
	Money(i64),
	Num(f64),
	Empty,
}

pub struct TableSheet<'a> {
	pub name: &'a str,
	pub title: &'a str,
	pub subtitle: String,
	pub info: Vec<(&'a str, String)>,
	pub headers: Vec<(&'a str, f64)>,
	pub rows: Vec<Vec<Cell>>,
	pub totals: Option<Vec<Cell>>,
}

fn write_cell(ws: &mut Worksheet, r: u32, c: u16, cell: &Cell, text: &Format, money: &Format, num: &Format) -> AppResult<()> {
	match cell {
		Cell::Text(s) => ws.write_string_with_format(r, c, s, text)?,
		Cell::Money(v) => ws.write_number_with_format(r, c, *v as f64, money)?,
		Cell::Num(v) => ws.write_number_with_format(r, c, *v, num)?,
		Cell::Empty => ws.write_blank(r, c, text)?,
	};
	Ok(())
}

/// Lembar tabel umum (kertas kerja, realisasi, dll.) dengan judul, info, header, isi, dan jumlah.
pub fn add_table_sheet(wb: &mut Workbook, sheet: &TableSheet<'_>) -> AppResult<()> {
	let ws = wb.add_worksheet();
	ws.set_name(sheet.name)?;
	ws.set_landscape();
	ws.set_paper_size(9);
	ws.set_margins(0.4, 0.4, 0.5, 0.5, 0.3, 0.3);
	ws.set_print_fit_to_pages(1, 0);

	let last_col = (sheet.headers.len().max(1) - 1) as u16;
	for (i, (_, w)) in sheet.headers.iter().enumerate() {
		ws.set_column_width(i as u16, *w)?;
	}
	let f_title = Format::new().set_bold().set_font_size(13).set_align(FormatAlign::Center);
	let f_sub = Format::new().set_bold().set_align(FormatAlign::Center);
	let f_head = Format::new()
		.set_bold()
		.set_align(FormatAlign::Center)
		.set_align(FormatAlign::VerticalCenter)
		.set_text_wrap()
		.set_border(FormatBorder::Thin)
		.set_background_color("#E8F5EE");
	let f_text = Format::new().set_border(FormatBorder::Thin).set_align(FormatAlign::Top).set_text_wrap();
	let f_money = Format::new().set_border(FormatBorder::Thin).set_align(FormatAlign::Top).set_num_format("#,##0");
	let f_num = Format::new().set_border(FormatBorder::Thin).set_align(FormatAlign::Top).set_num_format("#,##0.##");
	let f_bold_text = f_text.clone().set_bold();
	let f_bold_money = f_money.clone().set_bold();
	let f_bold_num = f_num.clone().set_bold();

	let mut r: u32 = 0;
	ws.merge_range(r, 0, r, last_col, sheet.title, &f_title)?;
	r += 1;
	if !sheet.subtitle.is_empty() {
		ws.merge_range(r, 0, r, last_col, &sheet.subtitle, &f_sub)?;
		r += 1;
	}
	r += 1;
	for (label, value) in &sheet.info {
		ws.write_string(r, 0, *label)?;
		ws.write_string(r, 2, format!(": {value}"))?;
		r += 1;
	}
	if !sheet.info.is_empty() {
		r += 1;
	}
	let header_row = r;
	for (i, (h, _)) in sheet.headers.iter().enumerate() {
		ws.write_string_with_format(r, i as u16, *h, &f_head)?;
	}
	ws.set_repeat_rows(header_row, header_row)?;
	r += 1;
	for row in &sheet.rows {
		for (c, cell) in row.iter().enumerate() {
			write_cell(ws, r, c as u16, cell, &f_text, &f_money, &f_num)?;
		}
		r += 1;
	}
	if let Some(totals) = &sheet.totals {
		for (c, cell) in totals.iter().enumerate() {
			write_cell(ws, r, c as u16, cell, &f_bold_text, &f_bold_money, &f_bold_num)?;
		}
	}
	Ok(())
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn signature_dates() {
		assert_eq!(tanggal_ttd(2026, Some(2)), "28 Februari 2026");
		assert_eq!(tanggal_ttd(2024, Some(2)), "29 Februari 2024");
		assert_eq!(tanggal_ttd(2026, None), "31 Desember 2026");
		assert_eq!(tanggal_id("2026-09-04"), "04-09-2026");
	}
}
