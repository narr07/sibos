use serde::Serialize;
use serde_json::Value;
use tauri::{AppHandle, Manager, State};

use crate::error::{validate_year, AppError, AppResult};
use crate::pengaturan::{self, Pengaturan, PengaturanView};
use crate::repo::arkas::queries;
use crate::state::AppState;

fn validate_key(key: &str) -> AppResult<&str> {
	let ok = !key.is_empty()
		&& key.len() <= 100
		&& key.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'));
	if ok {
		Ok(key)
	} else {
		Err(AppError::InvalidInput(format!("nama pengaturan tidak valid: {key}")))
	}
}

#[tauri::command]
pub fn settings_get(state: State<'_, AppState>, key: String) -> AppResult<Option<Value>> {
	state.app_db().setting_get(validate_key(&key)?)
}

#[tauri::command]
pub fn settings_set(state: State<'_, AppState>, key: String, value: Value) -> AppResult<()> {
	state.app_db().setting_set(validate_key(&key)?, &value)
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
	pub version: String,
	pub schema_version: i64,
	pub data_dir: String,
}

#[tauri::command]
pub fn app_info(app: AppHandle, state: State<'_, AppState>) -> AppResult<AppInfo> {
	let data_dir = app.path().app_data_dir().map(|p| p.display().to_string()).unwrap_or_default();
	Ok(AppInfo {
		version: app.package_info().version.to_string(),
		schema_version: state.app_db().schema_version()?,
		data_dir,
	})
}

/// Pengaturan dokumen: yang tersimpan, bawaan dari ARKAS, dan gabungannya.
#[tauri::command]
pub fn pengaturan_get(state: State<'_, AppState>, year: Option<i32>) -> AppResult<PengaturanView> {
	let year = year.map(validate_year).transpose()?;
	let school = state.with_arkas(|db| queries::school_info(db, year)).unwrap_or_default();
	let tersimpan = pengaturan::load(&state.app_db())?;
	let bawaan = pengaturan::bawaan(&school);
	let efektif = pengaturan::gabung(&tersimpan, &bawaan);
	Ok(PengaturanView { tersimpan, bawaan, efektif })
}

#[tauri::command]
pub fn pengaturan_set(state: State<'_, AppState>, value: Pengaturan) -> AppResult<()> {
	pengaturan::validate(&value)?;
	let json = serde_json::to_value(&value).map_err(|e| AppError::InvalidInput(e.to_string()))?;
	state.app_db().setting_set(pengaturan::SETTING_KEY, &json)
}
