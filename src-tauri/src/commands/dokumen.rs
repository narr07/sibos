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
