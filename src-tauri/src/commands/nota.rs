use std::collections::HashMap;
use std::io::Cursor;
use std::path::{Path, PathBuf};

use base64::Engine;
use tauri::State;

use crate::error::{validate_year, AppError, AppResult};
use crate::repo::app::nota::{NotaFile, PrintOverride};
use crate::repo::arkas::bku;
use crate::services::book::month_name;
use crate::services::nota::{group_notas, NotaGroup, NotaInput};
use crate::state::AppState;

/// Batas ukuran file sumber yang diterima (15 MB).
const MAX_UPLOAD: u64 = 15 * 1024 * 1024;
/// Sisi terpanjang foto setelah dikompres.
const MAX_SIDE: u32 = 1600;

fn validate_ref(id: &str) -> AppResult<&str> {
	let id = id.trim();
	let ok = !id.is_empty()
		&& id.len() <= 160
		&& id.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, ':' | '-' | '_' | '.' | '/'));
	if ok {
		Ok(id)
	} else {
		Err(AppError::InvalidInput("referensi nota tidak valid".into()))
	}
}

#[tauri::command]
pub fn nota_list(state: State<'_, AppState>, year: i32, month: Option<u32>, fund: Option<i64>) -> AppResult<Vec<NotaGroup>> {
	let year = validate_year(year)?;
	let (rows, notas) = state.with_arkas(|db| Ok((bku::kas_rows(db, year)?, bku::nota_infos(db, year)?)))?;
	let db = state.app_db();
	let merges = db.merges(year)?;
	let statuses = db.print_statuses()?;
	let overrides = db.print_overrides()?;
	let uraian_overrides = db.uraian_overrides(year)?;
	let mut manual_taxes: HashMap<String, Vec<crate::services::nota::NotaTax>> = HashMap::new();
	for m in db.manual_taxes(year)?.into_iter().filter(|m| m.arah == "pungut") {
		let Some(ref_id) = m.ref_id.clone() else { continue };
		manual_taxes.entry(ref_id).or_default().push(crate::services::nota::NotaTax {
			id: m.id,
			jenis: Some(m.jenis_pajak),
			uraian: m.uraian,
			nominal: m.nominal,
			tanggal_setor: None,
		});
	}
	// Tanggal setor dari entri "setor" milik nota yang sama dan jenis yang sama.
	for m in db.manual_taxes(year)?.into_iter().filter(|m| m.arah == "setor") {
		let Some(list) = m.ref_id.as_ref().and_then(|r| manual_taxes.get_mut(r)) else { continue };
		if let Some(t) = list.iter_mut().find(|t| t.jenis.as_deref() == Some(m.jenis_pajak.as_str()) && t.tanggal_setor.is_none()) {
			t.tanggal_setor = Some(m.tanggal);
		}
	}
	let mut file_counts: HashMap<String, usize> = HashMap::new();
	for f in db.nota_files(year)? {
		*file_counts.entry(f.ref_id).or_default() += 1;
	}
	drop(db);
	let input = NotaInput {
		rows: &rows,
		notas: &notas,
		merges: &merges,
		statuses: &statuses,
		overrides: &overrides,
		file_counts: &file_counts,
		uraian_overrides: &uraian_overrides,
		manual_taxes: &manual_taxes,
	};
	Ok(group_notas(&input, month, fund.filter(|f| *f != 0)))
}

fn clean_text(label: &str, text: &str, max: usize) -> AppResult<String> {
	let t = text.trim();
	if t.is_empty() {
		return Err(AppError::InvalidInput(format!("{label} wajib diisi")));
	}
	if t.chars().count() > max {
		return Err(AppError::InvalidInput(format!("{label} maksimal {max} karakter")));
	}
	Ok(t.to_string())
}

/// Gabungkan beberapa transaksi menjadi satu bukti (disimpan di sibos.db, ARKAS tidak berubah).
#[tauri::command]
pub fn merge_create(
	state: State<'_, AppState>,
	year: i32,
	no_bukti: String,
	uraian: String,
	tanggal: Option<String>,
	items: Vec<String>,
) -> AppResult<String> {
	let year = validate_year(year)?;
	let no_bukti = clean_text("Nomor bukti", &no_bukti, 60)?;
	let uraian = clean_text("Uraian", &uraian, 500)?;
	let items: Vec<String> = items.iter().map(|i| validate_ref(i).map(String::from)).collect::<Result<_, _>>()?;
	state.app_db().merge_create(year, &no_bukti, &uraian, tanggal.as_deref().filter(|t| !t.is_empty()), &items)
}

#[tauri::command]
pub fn merge_update(state: State<'_, AppState>, id: String, no_bukti: String, uraian: String, tanggal: Option<String>) -> AppResult<()> {
	let no_bukti = clean_text("Nomor bukti", &no_bukti, 60)?;
	let uraian = clean_text("Uraian", &uraian, 500)?;
	state.app_db().merge_update(validate_ref(&id)?, &no_bukti, &uraian, tanggal.as_deref().filter(|t| !t.is_empty()))
}

#[tauri::command]
pub fn merge_delete(state: State<'_, AppState>, id: String) -> AppResult<()> {
	state.app_db().merge_delete(validate_ref(&id)?)
}

#[tauri::command]
pub fn print_status_set(state: State<'_, AppState>, kind: String, refs: Vec<String>, printed: bool) -> AppResult<()> {
	if !["bukti", "a2", "nota"].contains(&kind.as_str()) {
		return Err(AppError::InvalidInput("jenis cetak tidak dikenal".into()));
	}
	let refs: Vec<String> = refs.iter().map(|r| validate_ref(r).map(String::from)).collect::<Result<_, _>>()?;
	state.app_db().print_status_set(&kind, &refs, printed)
}

#[tauri::command]
pub fn print_override_set(state: State<'_, AppState>, ref_id: String, value: PrintOverride) -> AppResult<()> {
	state.app_db().print_override_set(validate_ref(&ref_id)?, &value)
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotaTaxInput {
	pub jenis: String,
	pub nominal: i64,
	/// Tanggal setor (opsional). Bila diisi, dibuat juga baris "setor" di Buku Pembantu Pajak.
	pub tanggal_setor: Option<String>,
}

/// Simpan pajak manual satu nota. Otomatis masuk Buku Pembantu Pajak sebagai pungut (dan setor bila ada tanggalnya).
#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub fn nota_tax_set(
	state: State<'_, AppState>,
	year: i32,
	ref_id: String,
	tanggal: String,
	no_bukti: Option<String>,
	keterangan: String,
	fund: Option<i64>,
	taxes: Vec<NotaTaxInput>,
) -> AppResult<()> {
	use crate::repo::app::laporan::ManualTax;
	let year = validate_year(year)?;
	let ref_id = validate_ref(&ref_id)?;
	let mut entries = Vec::new();
	for t in taxes.iter().filter(|t| t.nominal > 0) {
		let base = ManualTax {
			id: String::new(),
			tahun: year,
			sumber_dana: fund.unwrap_or(0),
			tanggal: tanggal.clone(),
			no_bukti: no_bukti.clone().filter(|n| !n.is_empty()),
			uraian: format!("Pungut {} - {}", t.jenis.trim(), keterangan.trim()),
			jenis_pajak: t.jenis.trim().to_string(),
			arah: "pungut".into(),
			nominal: t.nominal,
			keterangan: None,
			ref_id: Some(ref_id.to_string()),
		};
		let setor = t.tanggal_setor.as_ref().filter(|s| !s.is_empty()).map(|tgl| ManualTax {
			tanggal: tgl.clone(),
			uraian: format!("Setor {} - {}", t.jenis.trim(), keterangan.trim()),
			arah: "setor".into(),
			..base.clone()
		});
		entries.push(base);
		entries.extend(setor);
	}
	state.app_db().nota_tax_replace(ref_id, &entries)
}

#[tauri::command]
pub fn nota_file_list(state: State<'_, AppState>, year: i32) -> AppResult<Vec<NotaFile>> {
	state.app_db().nota_files(validate_year(year)?)
}

fn unique_name() -> String {
	use std::sync::atomic::{AtomicU32, Ordering};
	static COUNTER: AtomicU32 = AtomicU32::new(0);
	let nanos = std::time::SystemTime::now()
		.duration_since(std::time::UNIX_EPOCH)
		.map(|d| d.as_nanos())
		.unwrap_or(0);
	format!("{nanos:x}{:04x}", COUNTER.fetch_add(1, Ordering::Relaxed) & 0xffff)
}

/// Kompres foto ke JPEG (sisi terpanjang maks. 1600 px, kualitas 80). PDF disalin apa adanya.
fn prepare_file(source: &Path) -> AppResult<(Vec<u8>, &'static str, &'static str)> {
	let meta = std::fs::metadata(source)?;
	if meta.len() > MAX_UPLOAD {
		return Err(AppError::InvalidInput("ukuran file maksimal 15 MB".into()));
	}
	let ext = source.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
	match ext.as_str() {
		"pdf" => Ok((std::fs::read(source)?, "pdf", "application/pdf")),
		"jpg" | "jpeg" | "png" => {
			let img = image::ImageReader::open(source)?
				.with_guessed_format()?
				.decode()
				.map_err(|e| AppError::InvalidInput(format!("gambar tidak bisa dibaca: {e}")))?;
			let img = if img.width() > MAX_SIDE || img.height() > MAX_SIDE {
				img.resize(MAX_SIDE, MAX_SIDE, image::imageops::FilterType::Triangle)
			} else {
				img
			};
			let mut out = Cursor::new(Vec::new());
			let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut out, 80);
			img.to_rgb8()
				.write_with_encoder(encoder)
				.map_err(|e| AppError::InvalidInput(format!("gagal mengompres gambar: {e}")))?;
			Ok((out.into_inner(), "jpg", "image/jpeg"))
		}
		_ => Err(AppError::InvalidInput("format file harus JPG, PNG, atau PDF".into())),
	}
}

/// Simpan foto nota ke `nota/<tahun>/<MM-Bulan>/`. File asli pengguna tidak diubah.
#[tauri::command]
pub fn nota_file_upload(
	state: State<'_, AppState>,
	year: i32,
	ref_id: String,
	month: u32,
	source_path: String,
) -> AppResult<String> {
	let year = validate_year(year)?;
	let ref_id = validate_ref(&ref_id)?;
	let month = month.clamp(1, 12);
	let (bytes, ext, mime) = prepare_file(Path::new(source_path.trim()))?;

	let folder = format!("{year}/{month:02}-{}", month_name(month));
	let relative = format!("{folder}/{}.{ext}", unique_name());
	let full = state.nota_dir().join(&relative);
	if let Some(dir) = full.parent() {
		std::fs::create_dir_all(dir)?;
	}
	std::fs::write(&full, &bytes)?;
	state.app_db().nota_file_add(year, ref_id, &relative, mime, "")
}

fn file_path(state: &AppState, relative: &str) -> AppResult<PathBuf> {
	// Cegah path keluar dari folder nota.
	if relative.contains("..") || Path::new(relative).is_absolute() {
		return Err(AppError::InvalidInput("lokasi file tidak valid".into()));
	}
	Ok(state.nota_dir().join(relative))
}

/// Isi file sebagai data URL untuk ditampilkan atau dicetak.
#[tauri::command]
pub fn nota_file_data(state: State<'_, AppState>, id: String) -> AppResult<String> {
	let (relative, mime, _) = state
		.app_db()
		.nota_file_get(validate_ref(&id)?)?
		.ok_or_else(|| AppError::InvalidInput("file nota tidak ditemukan".into()))?;
	let bytes = std::fs::read(file_path(&state, &relative)?)?;
	Ok(format!("data:{mime};base64,{}", base64::engine::general_purpose::STANDARD.encode(bytes)))
}

#[tauri::command]
pub fn nota_file_delete(state: State<'_, AppState>, id: String) -> AppResult<()> {
	let id = validate_ref(&id)?;
	let db = state.app_db();
	if let Some((relative, _, _)) = db.nota_file_get(id)? {
		let path = file_path(&state, &relative)?;
		if path.exists() {
			std::fs::remove_file(path)?;
		}
	}
	db.nota_file_delete(id)
}
