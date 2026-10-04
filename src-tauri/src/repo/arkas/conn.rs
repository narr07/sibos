//! Koneksi ke database ARKAS. HANYA BACA.
//!
//! Aturan mutlak: file ARKAS tidak boleh diubah sedikit pun. Karena itu:
//! - file dibuka dengan flag read-only,
//! - `query_only` diaktifkan sehingga SQLite menolak pernyataan tulis,
//! - modul ini tidak menyediakan fungsi untuk menulis.
//!
//! Bila file tidak bisa dibuka read-only (mis. butuh file -shm), aplikasi membaca
//! salinan sementara di folder temp. File asli tetap tidak disentuh.

use std::path::{Path, PathBuf};
use std::time::Duration;

use rusqlite::{Connection, ErrorCode, OpenFlags};

use crate::error::{AppError, AppResult};

pub struct ArkasDb {
	conn: Connection,
	/// Path file ARKAS asli yang dipilih pengguna.
	pub source_path: PathBuf,
	/// Terisi bila data dibaca dari salinan sementara.
	pub snapshot_path: Option<PathBuf>,
}

/// Lokasi default database ARKAS: `%APPDATA%\Arkas\arkas.db`.
pub fn default_path() -> Option<PathBuf> {
	std::env::var_os("APPDATA").map(|appdata| Path::new(&appdata).join("Arkas").join("arkas.db"))
}

fn map_open_error(err: rusqlite::Error) -> AppError {
	match err.sqlite_error_code() {
		Some(ErrorCode::NotADatabase) => AppError::ArkasBadKey,
		Some(ErrorCode::DatabaseBusy) | Some(ErrorCode::DatabaseLocked) => AppError::ArkasBusy,
		_ => AppError::Sqlite(err),
	}
}

fn open_readonly(path: &Path, key: &str) -> Result<Connection, rusqlite::Error> {
	let conn = Connection::open_with_flags(
		path,
		OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
	)?;
	// Kunci SQLCipher harus jadi perintah pertama setelah open.
	conn.pragma_update(None, "key", key)?;
	conn.pragma_update(None, "query_only", true)?;
	conn.busy_timeout(Duration::from_secs(3))?;
	// Membaca sqlite_master memaksa dekripsi halaman pertama: kunci salah langsung ketahuan.
	conn.query_row("SELECT count(*) FROM sqlite_master", [], |row| row.get::<_, i64>(0))?;
	Ok(conn)
}

impl ArkasDb {
	pub fn open(path: &Path, key: &str) -> AppResult<Self> {
		if !path.is_file() {
			return Err(AppError::ArkasNotFound(path.display().to_string()));
		}
		if key.is_empty() {
			return Err(AppError::ArkasNoKey);
		}

		match open_readonly(path, key) {
			Ok(conn) => Ok(Self { conn, source_path: path.to_path_buf(), snapshot_path: None }),
			Err(err) if err.sqlite_error_code() == Some(ErrorCode::CannotOpen) => {
				let snapshot = Self::copy_to_temp(path)?;
				let conn = open_readonly(&snapshot, key).map_err(map_open_error)?;
				Ok(Self { conn, source_path: path.to_path_buf(), snapshot_path: Some(snapshot) })
			}
			Err(err) => Err(map_open_error(err)),
		}
	}

	/// Salin file ARKAS (dan -wal bila ada) ke folder temp, lalu baca salinannya.
	fn copy_to_temp(path: &Path) -> AppResult<PathBuf> {
		let dir = std::env::temp_dir().join("sibos");
		std::fs::create_dir_all(&dir)?;
		let target = dir.join("arkas-snapshot.db");
		std::fs::copy(path, &target)?;
		let wal = path.with_extension("db-wal");
		if wal.is_file() {
			std::fs::copy(&wal, target.with_extension("db-wal"))?;
		}
		Ok(target)
	}

	pub fn conn(&self) -> &Connection {
		&self.conn
	}
}

impl Drop for ArkasDb {
	fn drop(&mut self) {
		if let Some(snapshot) = &self.snapshot_path {
			let _ = std::fs::remove_file(snapshot);
			let _ = std::fs::remove_file(snapshot.with_extension("db-wal"));
		}
	}
}
