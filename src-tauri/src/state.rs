use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::{Mutex, MutexGuard};

use crate::error::{AppError, AppResult};
use crate::repo::app::AppDb;
use crate::repo::arkas::ArkasDb;

pub struct AppState {
	pub arkas: Mutex<Option<ArkasDb>>,
	pub app_db: Mutex<AppDb>,
	/// Folder data aplikasi (%APPDATA%\id.sibos.app).
	pub data_dir: PathBuf,
	/// Diset oleh `siplah_sync_cancel`; diperiksa sinkronisasi SIPLah di tiap halaman.
	pub siplah_cancel: AtomicBool,
}

impl AppState {
	pub fn new(app_db: AppDb, data_dir: PathBuf) -> Self {
		Self { arkas: Mutex::new(None), app_db: Mutex::new(app_db), data_dir, siplah_cancel: AtomicBool::new(false) }
	}

	pub fn nota_dir(&self) -> PathBuf {
		self.data_dir.join("nota")
	}

	pub fn db_path(&self) -> PathBuf {
		if self.data_dir.join("silajur.db").exists() || !self.data_dir.join("sibos.db").exists() { self.data_dir.join("silajur.db") } else { self.data_dir.join("sibos.db") }
	}

	pub fn app_db(&self) -> MutexGuard<'_, AppDb> {
		self.app_db.lock().unwrap_or_else(|e| e.into_inner())
	}

	pub fn arkas_guard(&self) -> MutexGuard<'_, Option<ArkasDb>> {
		self.arkas.lock().unwrap_or_else(|e| e.into_inner())
	}

	/// Jalankan fungsi dengan koneksi ARKAS yang aktif.
	pub fn with_arkas<T>(&self, f: impl FnOnce(&ArkasDb) -> AppResult<T>) -> AppResult<T> {
		let guard = self.arkas_guard();
		let db = guard.as_ref().ok_or(AppError::ArkasNotConnected)?;
		f(db)
	}
}
