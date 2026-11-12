//! Backup & restore data Lajur (dulu SIBOS), salin database ARKAS, dan backup otomatis.
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
/// Awalan nama file backup otomatis. Yang pertama dipakai untuk file baru; sisanya nama lama (SIBOS).
const AUTO_PREFIXES: [&str; 2] = ["Lajur_AutoBackup_", "SIBOS_AutoBackup_"];

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
	let snapshot = std::env::temp_dir().join(util::unique_name("sibos-snapshot", "db"));
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
		return Err(AppError::InvalidInput("ini bukan file backup Lajur".into()));
	}
	Ok((manifest, count))
}

#[tauri::command(async)]
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

#[tauri::command(async)]
pub fn backup_inspect(path: String) -> AppResult<BackupInfo> {
	let p = PathBuf::from(path.trim());
	let (manifest, _) = read_manifest(&p)?;
	Ok(BackupInfo { path: p.display().to_string(), size: std::fs::metadata(&p)?.len(), manifest })
}

/// Pulihkan backup. Data sekarang dibackup otomatis dulu ke folder `backups`.
///
/// Urutannya "siapkan dulu, baru tukar": isi backup dibongkar dan diperiksa di lokasi sementara,
/// lalu database dan folder nota ditukar dengan cara mengganti nama. Bila salah satu langkah gagal,
/// data lama dikembalikan sehingga aplikasi tidak pernah tertinggal dalam keadaan setengah jadi.
#[tauri::command(async)]
pub fn backup_restore(state: State<'_, AppState>, path: String) -> AppResult<BackupInfo> {
	let source = PathBuf::from(path.trim());
	let (manifest, _) = read_manifest(&source)?;
	let safety = state.data_dir.join("backups").join(format!("sebelum-pulih-{}.sibos", util::timestamp()));
	create_backup(&state, &safety)?;

	let mut zip = zip::ZipArchive::new(File::open(&source)?).map_err(zip_err)?;
	let temp_db = std::env::temp_dir().join(util::unique_name("sibos-restore", "db"));
	let staging = state.data_dir.join("nota.pulih-tmp");
	let result = prepare_restore(&mut zip, &temp_db, &staging).and_then(|()| swap_restore(&state, &temp_db, &staging));
	let _ = std::fs::remove_file(&temp_db);
	for suffix in ["-wal", "-shm"] {
		let _ = std::fs::remove_file(format!("{}{suffix}", temp_db.display()));
	}
	let _ = std::fs::remove_dir_all(&staging);
	result?;

	Ok(BackupInfo { path: source.display().to_string(), size: std::fs::metadata(&source)?.len(), manifest })
}

/// Bongkar `sibos.db` ke `temp_db` (lalu pastikan bisa dibuka/dimigrasi) dan folder nota ke `staging`.
/// Data aplikasi yang sedang dipakai belum disentuh sama sekali.
fn prepare_restore(zip: &mut zip::ZipArchive<File>, temp_db: &Path, staging: &Path) -> AppResult<()> {
	{
		let mut out = File::create(temp_db)?;
		std::io::copy(&mut zip.by_name("sibos.db").map_err(zip_err)?, &mut out)?;
	}
	drop(AppDb::open(temp_db)?);

	if staging.exists() {
		std::fs::remove_dir_all(staging)?;
	}
	std::fs::create_dir_all(staging)?;
	for i in 0..zip.len() {
		let mut entry = zip.by_index(i).map_err(zip_err)?;
		let Some(rel) = entry.enclosed_name() else { continue };
		let Ok(inner) = rel.strip_prefix("nota") else { continue };
		if entry.is_dir() || inner.as_os_str().is_empty() {
			continue;
		}
		let out_path = staging.join(inner);
		if let Some(dir) = out_path.parent() {
			std::fs::create_dir_all(dir)?;
		}
		std::io::copy(&mut entry, &mut File::create(out_path)?)?;
	}
	Ok(())
}

/// Tukar database dan folder nota dengan hasil `prepare_restore`. File lama diganti nama dulu
/// (bukan dihapus) supaya bisa dikembalikan bila penukaran gagal.
fn swap_restore(state: &AppState, temp_db: &Path, staging: &Path) -> AppResult<()> {
	let db_path = state.db_path();
	let nota_dir = state.nota_dir();
	let old_nota = state.data_dir.join("nota.lama-tmp");
	let suffixes = ["", "-wal", "-shm"];
	let with_suffix = |base: &Path, suffix: &str| PathBuf::from(format!("{}{suffix}", base.display()));
	let old_db = |suffix: &str| PathBuf::from(format!("{}.lama{suffix}", db_path.display()));

	let mut guard = state.app_db();
	// Tutup koneksi ke sibos.db agar file bisa diganti nama di Windows.
	*guard = AppDb::open_in_memory()?;

	let swap = || -> AppResult<()> {
		for suffix in suffixes {
			let _ = std::fs::remove_file(old_db(suffix));
			let current = with_suffix(&db_path, suffix);
			if current.exists() {
				std::fs::rename(&current, old_db(suffix))?;
			}
		}
		std::fs::copy(temp_db, &db_path)?;
		if old_nota.exists() {
			std::fs::remove_dir_all(&old_nota)?;
		}
		if nota_dir.exists() {
			std::fs::rename(&nota_dir, &old_nota)?;
		}
		std::fs::rename(staging, &nota_dir)?;
		Ok(())
	};

	match swap().and_then(|()| AppDb::open(&db_path)) {
		Ok(db) => {
			*guard = db;
			for suffix in suffixes {
				let _ = std::fs::remove_file(old_db(suffix));
			}
			let _ = std::fs::remove_dir_all(&old_nota);
			Ok(())
		}
		Err(e) => {
			// Kembalikan keadaan semula.
			for suffix in suffixes {
				if old_db(suffix).exists() {
					let _ = std::fs::remove_file(with_suffix(&db_path, suffix));
					let _ = std::fs::rename(old_db(suffix), with_suffix(&db_path, suffix));
				}
			}
			if old_nota.exists() {
				let _ = std::fs::remove_dir_all(&nota_dir);
				let _ = std::fs::rename(&old_nota, &nota_dir);
			}
			*guard = AppDb::open(&db_path)?;
			Err(e)
		}
	}
}

/// Salin file database ARKAS (hanya dibaca) ke lokasi pilihan pengguna.
#[tauri::command(async)]
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
	arkas::conn::copy_consistent(&source, &target)?;
	merge_wal(&target, crate::secret::load_key().ok().flatten().as_ref().map(|k| k.as_str()));
	Ok(target.display().to_string())
}

/// Gabungkan file `-wal` ke dalam SALINAN (bukan file ARKAS asli) supaya backup jadi satu file utuh.
/// Butuh kunci ARKAS; tanpa kunci, file `-wal` dibiarkan di samping salinan dan keduanya tetap lengkap.
fn merge_wal(copy: &Path, key: Option<&str>) {
	let wal = arkas::conn::wal_path(copy);
	if !wal.is_file() {
		return;
	}
	let Some(key) = key else { return };
	let merged = (|| -> rusqlite::Result<bool> {
		let conn = rusqlite::Connection::open(copy)?;
		conn.pragma_update(None, "key", key)?;
		// Baris hasil: (busy, halaman di log, halaman yang sudah dipindah). busy = 0 berarti tuntas.
		let busy: i64 = conn.query_row("PRAGMA wal_checkpoint(TRUNCATE)", [], |row| row.get(0))?;
		Ok(busy == 0)
	})();
	// SQLite biasanya menghapus -wal sendiri saat koneksi ditutup. Sisa file hanya dihapus bila
	// checkpoint tuntas dan isinya sudah kosong; selain itu dibiarkan agar tidak ada data yang hilang.
	let empty = std::fs::metadata(&wal).map(|m| m.len() == 0).unwrap_or(true);
	if merged == Ok(true) && empty {
		let _ = std::fs::remove_file(&wal);
		let _ = std::fs::remove_file(format!("{}-shm", copy.display()));
	}
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

#[tauri::command(async)]
pub fn auto_backup_get(state: State<'_, AppState>) -> AppResult<AutoBackup> {
	auto_config(&state)
}

#[tauri::command(async)]
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
	let target = folder.join(format!("{}{}.sibos", AUTO_PREFIXES[0], util::timestamp()));
	create_backup(state, &target)?;

	// Urut berdasarkan cap waktu (bukan nama utuh) agar backup lama berawalan SIBOS_ ikut dirotasi.
	let mut files: Vec<(String, PathBuf)> = std::fs::read_dir(&folder)?
		.filter_map(|e| e.ok().map(|e| e.path()))
		.filter_map(|p| {
			let name = p.file_name()?.to_str()?;
			let stamp = AUTO_PREFIXES.iter().find_map(|pre| name.strip_prefix(pre))?.strip_suffix(".sibos")?.to_string();
			Some((stamp, p))
		})
		.collect();
	files.sort();
	while files.len() > cfg.keep as usize {
		let (_, old) = files.remove(0);
		let _ = std::fs::remove_file(old);
	}
	cfg.last = Some(util::datetime_text());
	state.app_db().setting_set(AUTO_KEY, &serde_json::to_value(&cfg).unwrap_or(Value::Null))?;
	Ok(Some(target.display().to_string()))
}

#[cfg(test)]
mod tests {
	use super::*;
	use serde_json::json;

	fn state_in(dir: &Path) -> AppState {
		AppState::new(AppDb::open(&dir.join("sibos.db")).unwrap(), dir.to_path_buf())
	}

	fn restore_from(state: &AppState, backup: &Path) -> AppResult<()> {
		let mut zip = zip::ZipArchive::new(File::open(backup)?).map_err(zip_err)?;
		let temp_db = std::env::temp_dir().join(util::unique_name("sibos-restore-uji", "db"));
		let staging = state.data_dir.join("nota.pulih-tmp");
		let result = prepare_restore(&mut zip, &temp_db, &staging).and_then(|()| swap_restore(state, &temp_db, &staging));
		let _ = std::fs::remove_file(&temp_db);
		let _ = std::fs::remove_dir_all(&staging);
		result
	}

	#[test]
	fn restore_replaces_db_and_nota_without_leftovers() {
		let dir = tempfile::tempdir().unwrap();
		let state = state_in(dir.path());
		state.app_db().setting_set("uji", &json!(1)).unwrap();
		std::fs::create_dir_all(state.nota_dir()).unwrap();
		std::fs::write(state.nota_dir().join("a.txt"), "asli").unwrap();
		let backup = dir.path().join("uji.sibos");
		create_backup(&state, &backup).unwrap();

		state.app_db().setting_set("uji", &json!(2)).unwrap();
		std::fs::write(state.nota_dir().join("a.txt"), "diubah").unwrap();
		std::fs::write(state.nota_dir().join("baru.txt"), "x").unwrap();

		restore_from(&state, &backup).unwrap();

		assert_eq!(state.app_db().setting_get("uji").unwrap(), Some(json!(1)));
		assert_eq!(std::fs::read_to_string(state.nota_dir().join("a.txt")).unwrap(), "asli");
		assert!(!state.nota_dir().join("baru.txt").exists());
		let names: Vec<String> = std::fs::read_dir(dir.path()).unwrap().map(|e| e.unwrap().file_name().to_string_lossy().into_owned()).collect();
		assert!(!names.iter().any(|n| n.contains("lama") || n.contains("pulih-tmp")), "sisa file sementara: {names:?}");
	}

	#[test]
	fn broken_backup_leaves_current_data_untouched() {
		let dir = tempfile::tempdir().unwrap();
		let state = state_in(dir.path());
		state.app_db().setting_set("uji", &json!(2)).unwrap();
		std::fs::create_dir_all(state.nota_dir()).unwrap();
		std::fs::write(state.nota_dir().join("a.txt"), "sekarang").unwrap();

		// Zip tanpa sibos.db: gagal di tahap persiapan, sebelum data apa pun ditukar.
		let broken = dir.path().join("rusak.sibos");
		{
			let mut zip = zip::ZipWriter::new(File::create(&broken).unwrap());
			zip.start_file("nota/a.txt", SimpleFileOptions::default()).unwrap();
			zip.write_all(b"dari backup").unwrap();
			zip.finish().unwrap();
		}
		assert!(restore_from(&state, &broken).is_err());

		assert_eq!(state.app_db().setting_get("uji").unwrap(), Some(json!(2)));
		assert_eq!(std::fs::read_to_string(state.nota_dir().join("a.txt")).unwrap(), "sekarang");
	}

	#[test]
	fn arkas_copy_includes_wal_and_merges_it() {
		let dir = tempfile::tempdir().unwrap();
		let source = dir.path().join("arkas.db");
		let key = "kunci-uji";
		// Koneksi dibiarkan terbuka agar perubahan masih tertahan di file -wal (seperti ARKAS yang sedang jalan).
		let writer = rusqlite::Connection::open(&source).unwrap();
		writer.pragma_update(None, "key", key).unwrap();
		writer.pragma_update(None, "journal_mode", "WAL").unwrap();
		writer.pragma_update(None, "wal_autocheckpoint", 0).unwrap();
		writer.execute_batch("CREATE TABLE t (a INTEGER); INSERT INTO t VALUES (1), (2), (3);").unwrap();
		assert!(arkas::conn::wal_path(&source).is_file());

		let target = dir.path().join("salinan.db");
		arkas::conn::copy_consistent(&source, &target).unwrap();
		assert!(arkas::conn::wal_path(&target).is_file(), "file -wal tidak ikut disalin");

		merge_wal(&target, Some(key));
		assert!(!arkas::conn::wal_path(&target).exists(), "-wal belum digabung");
		let copy = rusqlite::Connection::open(&target).unwrap();
		copy.pragma_update(None, "key", key).unwrap();
		let n: i64 = copy.query_row("SELECT count(*) FROM t", [], |r| r.get(0)).unwrap();
		assert_eq!(n, 3);
		drop(writer);
	}
}
