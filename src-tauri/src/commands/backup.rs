//! Backup & restore data SIBOS, salin database ARKAS, dan backup otomatis.
//! Database ARKAS hanya disalin (dibaca), tidak pernah ditulis.

use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::State;
use zip::write::SimpleFileOptions;

use crate::error::{AppError, AppResult};
use crate::repo::app::AppDb;
use crate::repo::arkas;
use crate::state::AppState;
use crate::util;

const AUTO_KEY: &str = "backup.auto";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Manifest {
	pub app: String,
	pub format: u32,
	pub version: String,
	pub created_at: String,
	pub schema_version: i64,
	pub npsn: Option<String>,
	pub sekolah: Option<String>,
	pub nota_files: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupInfo {
	pub path: String,
	pub size: u64,
	pub manifest: Manifest,
}

fn zip_err(e: zip::result::ZipError) -> AppError {
	AppError::InvalidInput(format!("file backup tidak valid: {e}"))
}

fn walk(dir: &Path, out: &mut Vec<PathBuf>) -> std::io::Result<()> {
	if !dir.exists() {
		return Ok(());
	}
	for entry in std::fs::read_dir(dir)? {
		let path = entry?.path();
		if path.is_dir() {
			walk(&path, out)?;
		} else {
			out.push(path);
		}
	}
	Ok(())
}

/// Buat file backup `.sibos` (zip): manifest.json + sibos.db + folder nota.
pub(crate) fn create_backup(state: &AppState, target: &Path) -> AppResult<BackupInfo> {
	let snapshot = std::env::temp_dir().join(format!("sibos-snapshot-{}.db", util::timestamp()));
	let _ = std::fs::remove_file(&snapshot);
	let schema_version = {
		let db = state.app_db();
		db.vacuum_into(&snapshot)?;
		db.schema_version()?
	};
	let school = state.with_arkas(|db| arkas::queries::school_info(db, None)).ok();

	let mut nota_files = Vec::new();
	walk(&state.nota_dir(), &mut nota_files)?;

	let manifest = Manifest {
		app: "SIBOS".into(),
		format: 1,
		version: env!("CARGO_PKG_VERSION").into(),
		created_at: util::datetime_text(),
		schema_version,
		npsn: school.as_ref().and_then(|s| s.npsn.clone()),
		sekolah: school.as_ref().and_then(|s| s.nama.clone()),
		nota_files: nota_files.len(),
	};

	if let Some(dir) = target.parent() {
		std::fs::create_dir_all(dir)?;
	}
	let result = (|| -> AppResult<()> {
		let mut zip = zip::ZipWriter::new(File::create(target)?);
		let opts = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
		zip.start_file("manifest.json", opts).map_err(zip_err)?;
		zip.write_all(serde_json::to_string_pretty(&manifest).unwrap_or_default().as_bytes())?;
		zip.start_file("sibos.db", opts).map_err(zip_err)?;
		std::io::copy(&mut File::open(&snapshot)?, &mut zip)?;
		let base = state.nota_dir();
		for f in &nota_files {
			let rel = f.strip_prefix(&base).unwrap_or(f).to_string_lossy().replace('\\', "/");
			zip.start_file(format!("nota/{rel}"), opts).map_err(zip_err)?;
			std::io::copy(&mut File::open(f)?, &mut zip)?;
		}
		zip.finish().map_err(zip_err)?;
		Ok(())
	})();
	let _ = std::fs::remove_file(&snapshot);
	result?;

	Ok(BackupInfo { path: target.display().to_string(), size: std::fs::metadata(target)?.len(), manifest })
}

fn read_manifest(path: &Path) -> AppResult<(Manifest, usize)> {
	let mut zip = zip::ZipArchive::new(File::open(path)?).map_err(zip_err)?;
	let count = zip.len();
	let mut text = String::new();
	zip.by_name("manifest.json").map_err(zip_err)?.read_to_string(&mut text)?;
	let manifest: Manifest = serde_json::from_str(&text).map_err(|e| AppError::InvalidInput(format!("manifest rusak: {e}")))?;
	if manifest.app != "SIBOS" {
		return Err(AppError::InvalidInput("ini bukan file backup SIBOS".into()));
	}
	Ok((manifest, count))
}

#[tauri::command]
pub fn backup_create(state: State<'_, AppState>, path: String) -> AppResult<BackupInfo> {
	let mut target = PathBuf::from(path.trim());
	if target.as_os_str().is_empty() {
		return Err(AppError::InvalidInput("lokasi backup belum dipilih".into()));
	}
	if target.extension().is_none_or(|e| !e.eq_ignore_ascii_case("sibos")) {
		target.set_extension("sibos");
	}
	create_backup(&state, &target)
}

#[tauri::command]
pub fn backup_inspect(path: String) -> AppResult<BackupInfo> {
	let p = PathBuf::from(path.trim());
	let (manifest, _) = read_manifest(&p)?;
	Ok(BackupInfo { path: p.display().to_string(), size: std::fs::metadata(&p)?.len(), manifest })
}

/// Pulihkan backup. Data sekarang dibackup otomatis dulu ke folder `backups`.
#[tauri::command]
pub fn backup_restore(state: State<'_, AppState>, path: String) -> AppResult<BackupInfo> {
	let source = PathBuf::from(path.trim());
	let (manifest, _) = read_manifest(&source)?;
	let safety = state.data_dir.join("backups").join(format!("sebelum-pulih-{}.sibos", util::timestamp()));
	create_backup(&state, &safety)?;

	let mut zip = zip::ZipArchive::new(File::open(&source)?).map_err(zip_err)?;
	let temp_db = std::env::temp_dir().join(format!("sibos-restore-{}.db", util::timestamp()));
	{
		let mut out = File::create(&temp_db)?;
		std::io::copy(&mut zip.by_name("sibos.db").map_err(zip_err)?, &mut out)?;
	}
	// Pastikan database bisa dibuka (dan dimigrasi bila versinya lebih lama).
	drop(AppDb::open(&temp_db)?);

	let db_path = state.db_path();
	{
		let mut guard = state.app_db();
		*guard = AppDb::open_in_memory()?;
		for suffix in ["", "-wal", "-shm"] {
			let p = PathBuf::from(format!("{}{suffix}", db_path.display()));
			if p.exists() {
				std::fs::remove_file(p)?;
			}
		}
		std::fs::copy(&temp_db, &db_path)?;
		*guard = AppDb::open(&db_path)?;
	}
	let _ = std::fs::remove_file(&temp_db);
	for suffix in ["-wal", "-shm"] {
		let _ = std::fs::remove_file(format!("{}{suffix}", temp_db.display()));
	}

	let nota_dir = state.nota_dir();
	if nota_dir.exists() {
		std::fs::remove_dir_all(&nota_dir)?;
	}
	for i in 0..zip.len() {
		let mut entry = zip.by_index(i).map_err(zip_err)?;
		let Some(rel) = entry.enclosed_name() else { continue };
		let Ok(inner) = rel.strip_prefix("nota") else { continue };
		if entry.is_dir() || inner.as_os_str().is_empty() {
			continue;
		}
		let out_path = nota_dir.join(inner);
		if let Some(dir) = out_path.parent() {
			std::fs::create_dir_all(dir)?;
		}
		std::io::copy(&mut entry, &mut File::create(out_path)?)?;
	}

	Ok(BackupInfo { path: source.display().to_string(), size: std::fs::metadata(&source)?.len(), manifest })
}

/// Salin file database ARKAS (hanya dibaca) ke lokasi pilihan pengguna.
#[tauri::command]
pub fn backup_arkas_db(state: State<'_, AppState>, path: String) -> AppResult<String> {
	let source = state
		.arkas_guard()
		.as_ref()
		.map(|db| db.source_path.clone())
		.or_else(arkas::default_path)
		.ok_or_else(|| AppError::ArkasNotFound("lokasi tidak diketahui".into()))?;
	if !source.is_file() {
		return Err(AppError::ArkasNotFound(source.display().to_string()));
	}
	let mut target = PathBuf::from(path.trim());
	if target.extension().is_none() {
		target.set_extension("db");
	}
	if target == source {
		return Err(AppError::InvalidInput("lokasi tujuan sama dengan file ARKAS asli".into()));
	}
	std::fs::copy(&source, &target)?;
	Ok(target.display().to_string())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AutoBackup {
	pub enabled: bool,
	pub folder: String,
	pub keep: u32,
	pub last: Option<String>,
}

impl Default for AutoBackup {
	fn default() -> Self {
		Self { enabled: false, folder: String::new(), keep: 10, last: None }
	}
}

fn auto_config(state: &AppState) -> AppResult<AutoBackup> {
	Ok(state.app_db().setting_get(AUTO_KEY)?.and_then(|v| serde_json::from_value(v).ok()).unwrap_or_default())
}

#[tauri::command]
pub fn auto_backup_get(state: State<'_, AppState>) -> AppResult<AutoBackup> {
	auto_config(&state)
}

#[tauri::command]
pub fn auto_backup_set(state: State<'_, AppState>, value: AutoBackup) -> AppResult<()> {
	if value.enabled && value.folder.trim().is_empty() {
		return Err(AppError::InvalidInput("pilih folder backup otomatis".into()));
	}
	let mut v = value;
	v.keep = v.keep.clamp(1, 100);
	state.app_db().setting_set(AUTO_KEY, &serde_json::to_value(&v).unwrap_or(Value::Null))
}

/// Dipanggil saat aplikasi dibuka. Kegagalan tidak menghentikan aplikasi.
pub fn run_auto_backup(state: &AppState) -> AppResult<Option<String>> {
	let mut cfg = auto_config(state)?;
	if !cfg.enabled || cfg.folder.trim().is_empty() {
		return Ok(None);
	}
	let folder = PathBuf::from(cfg.folder.trim());
	let target = folder.join(format!("SIBOS_AutoBackup_{}.sibos", util::timestamp()));
	create_backup(state, &target)?;

	let mut files: Vec<PathBuf> = std::fs::read_dir(&folder)?
		.filter_map(|e| e.ok().map(|e| e.path()))
		.filter(|p| p.file_name().and_then(|n| n.to_str()).is_some_and(|n| n.starts_with("SIBOS_AutoBackup_") && n.ends_with(".sibos")))
		.collect();
	files.sort();
	while files.len() > cfg.keep as usize {
		let old = files.remove(0);
		let _ = std::fs::remove_file(old);
	}
	cfg.last = Some(util::datetime_text());
	state.app_db().setting_set(AUTO_KEY, &serde_json::to_value(&cfg).unwrap_or(Value::Null))?;
	Ok(Some(target.display().to_string()))
}
