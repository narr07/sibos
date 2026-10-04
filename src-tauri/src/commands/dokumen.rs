//! Template dokumen & profil penyedia. Data ARKAS tidak disentuh.

use tauri::State;

use crate::error::{AppError, AppResult};
use crate::repo::app::dokumen::{DocTemplate, Penyedia};
use crate::state::AppState;

fn validate_id(id: &str) -> AppResult<&str> {
	let id = id.trim();
	if id.is_empty() || id.len() > 64 || !id.chars().all(|c| c.is_ascii_alphanumeric()) {
		return Err(AppError::InvalidInput("id template tidak valid".into()));
	}
	Ok(id)
}

#[tauri::command]
pub fn doc_template_list(state: State<'_, AppState>) -> AppResult<Vec<DocTemplate>> {
	state.app_db().doc_templates()
}

#[tauri::command]
pub fn doc_template_save(state: State<'_, AppState>, template: DocTemplate) -> AppResult<String> {
	if !template.id.is_empty() {
		validate_id(&template.id)?;
	}
	state.app_db().doc_template_save(&template)
}

#[tauri::command]
pub fn doc_template_delete(state: State<'_, AppState>, id: String) -> AppResult<()> {
	state.app_db().doc_template_delete(validate_id(&id)?)
}

/// Penanda format file template SIBOS (dibagikan antarsekolah).
const TEMPLATE_FILE_FORMAT: &str = "sibos-template";

#[derive(serde::Serialize, serde::Deserialize)]
struct TemplateFile {
	format: String,
	versi: u32,
	templates: Vec<DocTemplate>,
}

/// Simpan template terpilih (atau semua bila `ids` kosong) ke file JSON.
#[tauri::command]
pub fn doc_template_export(state: State<'_, AppState>, ids: Vec<String>, path: String) -> AppResult<usize> {
	let all = state.app_db().doc_templates()?;
	let templates: Vec<DocTemplate> = all
		.into_iter()
		.filter(|t| ids.is_empty() || ids.contains(&t.id))
		.map(|t| DocTemplate { id: String::new(), updated_at: String::new(), ..t })
		.collect();
	if templates.is_empty() {
		return Err(AppError::InvalidInput("tidak ada template untuk diekspor".into()));
	}
	let file = TemplateFile { format: TEMPLATE_FILE_FORMAT.into(), versi: 1, templates };
	let json = serde_json::to_string_pretty(&file).map_err(|e| AppError::InvalidInput(e.to_string()))?;
	std::fs::write(&path, json)?;
	Ok(file.templates.len())
}

/// Tambahkan template dari file JSON hasil export. Template lama tidak ditimpa;
/// nama yang sudah ada diberi akhiran "(impor)".
#[tauri::command]
pub fn doc_template_import(state: State<'_, AppState>, path: String) -> AppResult<usize> {
	let text = std::fs::read_to_string(&path)?;
	let file: TemplateFile = serde_json::from_str(&text)
		.map_err(|_| AppError::InvalidInput("file bukan template SIBOS yang valid".into()))?;
	if file.format != TEMPLATE_FILE_FORMAT {
		return Err(AppError::InvalidInput("file bukan template SIBOS".into()));
	}
	let db = state.app_db();
	let existing: Vec<String> = db.doc_templates()?.into_iter().map(|t| t.nama).collect();
	let mut count = 0;
	for t in file.templates {
		let nama = if existing.contains(&t.nama) { format!("{} (impor)", t.nama) } else { t.nama.clone() };
		db.doc_template_save(&DocTemplate { id: String::new(), nama, ..t })?;
		count += 1;
	}
	Ok(count)
}

#[tauri::command]
pub fn penyedia_list(state: State<'_, AppState>) -> AppResult<Vec<Penyedia>> {
	state.app_db().penyedia_list()
}

#[tauri::command]
pub fn penyedia_save(state: State<'_, AppState>, penyedia: Penyedia) -> AppResult<()> {
	state.app_db().penyedia_save(&penyedia)
}

#[tauri::command]
pub fn penyedia_delete(state: State<'_, AppState>, nama: String) -> AppResult<()> {
	state.app_db().penyedia_delete(nama.trim())
}
