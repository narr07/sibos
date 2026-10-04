use std::path::PathBuf;

use serde::Serialize;
use serde_json::json;
use tauri::State;

use crate::error::{validate_year, AppError, AppResult};
use crate::repo::arkas::{self, queries, ArkasDb};
use crate::secret;
use crate::state::AppState;

const SETTING_ARKAS_PATH: &str = "arkas.path";

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionStatus {
	/// connected | not_found | no_key | bad_key | busy | error | disconnected
	pub state: String,
	pub path: Option<String>,
	pub default_path: Option<String>,
	pub using_snapshot: bool,
	pub has_stored_key: bool,
	pub message: Option<String>,
	pub school: Option<queries::SchoolInfo>,
}

fn state_from_error(err: &AppError) -> &'static str {
	match err {
		AppError::ArkasNotFound(_) => "not_found",
		AppError::ArkasNoKey => "no_key",
		AppError::ArkasBadKey => "bad_key",
		AppError::ArkasBusy => "busy",
		_ => "error",
	}
}

fn resolve_path(state: &AppState, path: Option<String>) -> AppResult<Option<PathBuf>> {
	if let Some(p) = path.filter(|p| !p.trim().is_empty()) {
		return Ok(Some(PathBuf::from(p.trim())));
	}
	let saved = state.app_db().setting_get(SETTING_ARKAS_PATH)?;
	if let Some(p) = saved.as_ref().and_then(|v| v.as_str()) {
		return Ok(Some(PathBuf::from(p)));
	}
	Ok(arkas::default_path())
}

fn build_status(state: &AppState, path: Option<&PathBuf>, err: Option<&AppError>) -> ConnectionStatus {
	let guard = state.arkas_guard();
	let default_path = arkas::default_path().map(|p| p.display().to_string());
	let has_stored_key = secret::load_key().ok().flatten().is_some();

	match (guard.as_ref(), err) {
		(Some(db), None) => ConnectionStatus {
			state: "connected".into(),
			path: Some(db.source_path.display().to_string()),
			default_path,
			using_snapshot: db.snapshot_path.is_some(),
			has_stored_key,
			message: None,
			school: queries::school_info(db, None).ok(),
		},
		(_, Some(e)) => ConnectionStatus {
			state: state_from_error(e).into(),
			path: path.map(|p| p.display().to_string()),
			default_path,
			using_snapshot: false,
			has_stored_key,
			message: Some(e.to_string()),
			school: None,
		},
		(None, None) => ConnectionStatus {
			state: "disconnected".into(),
			path: path.map(|p| p.display().to_string()),
			default_path,
			using_snapshot: false,
			has_stored_key,
			message: None,
			school: None,
		},
	}
}

#[tauri::command]
pub fn arkas_status(state: State<'_, AppState>) -> AppResult<ConnectionStatus> {
	let path = resolve_path(&state, None)?;
	Ok(build_status(&state, path.as_ref(), None))
}

/// Hubungkan ke ARKAS. Path dan kunci opsional: bila kosong dipakai yang tersimpan.
/// Selalu mengembalikan status (kegagalan dilaporkan lewat field `state`).
#[tauri::command]
pub fn arkas_connect(
	state: State<'_, AppState>,
	path: Option<String>,
	key: Option<String>,
	remember_key: Option<bool>,
) -> AppResult<ConnectionStatus> {
	let path = resolve_path(&state, path)?;
	let Some(path_buf) = path.clone() else {
		let err = AppError::ArkasNotFound("lokasi APPDATA tidak diketahui".into());
		return Ok(build_status(&state, None, Some(&err)));
	};

	let given_key = key.filter(|k| !k.is_empty());
	let key = match &given_key {
		Some(k) => Some(k.clone()),
		None => secret::load_key()?,
	};

	let result = match key.as_deref() {
		None => Err(AppError::ArkasNoKey),
		Some(k) => ArkasDb::open(&path_buf, k),
	};

	match result {
		Ok(db) => {
			*state.arkas_guard() = Some(db);
			state
				.app_db()
				.setting_set(SETTING_ARKAS_PATH, &json!(path_buf.display().to_string()))?;
			if let (Some(k), Some(true)) = (&given_key, remember_key) {
				secret::save_key(k)?;
			}
			Ok(build_status(&state, Some(&path_buf), None))
		}
		Err(e) => {
			*state.arkas_guard() = None;
			Ok(build_status(&state, Some(&path_buf), Some(&e)))
		}
	}
}

#[tauri::command]
pub fn arkas_disconnect(state: State<'_, AppState>, forget_key: Option<bool>) -> AppResult<ConnectionStatus> {
	*state.arkas_guard() = None;
	if forget_key == Some(true) {
		secret::clear_key()?;
	}
	let path = resolve_path(&state, None)?;
	Ok(build_status(&state, path.as_ref(), None))
}

#[tauri::command]
pub fn available_years(state: State<'_, AppState>) -> AppResult<Vec<i32>> {
	state.with_arkas(queries::available_years)
}

#[tauri::command]
pub fn fund_sources(state: State<'_, AppState>, year: i32) -> AppResult<Vec<queries::FundSource>> {
	let year = validate_year(year)?;
	state.with_arkas(|db| queries::fund_sources(db, year))
}

#[tauri::command]
pub fn school_info(state: State<'_, AppState>, year: Option<i32>) -> AppResult<queries::SchoolInfo> {
	let year = year.map(validate_year).transpose()?;
	state.with_arkas(|db| queries::school_info(db, year))
}

#[tauri::command]
pub fn arkas_schema(state: State<'_, AppState>) -> AppResult<Vec<queries::TableInfo>> {
	state.with_arkas(queries::schema)
}
