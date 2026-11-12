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

fn open_readonly_with_compat(path: &Path, key: &str, compat: Option<i32>) -> Result<Connection, rusqlite::Error> {
	let conn = Connection::open_with_flags(
		path,
		OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
	)?;
	// Kunci SQLCipher harus jadi perintah pertama setelah open.
	conn.pragma_update(None, "key", key)?;
	if let Some(c) = compat {
		conn.pragma_update(None, "cipher_compatibility", c)?;
	}
	conn.pragma_update(None, "query_only", true)?;
	conn.busy_timeout(Duration::from_secs(3))?;
	// Membaca sqlite_master memaksa dekripsi halaman pertama: kunci salah langsung ketahuan.
	conn.query_row("SELECT count(*) FROM sqlite_master", [], |row| row.get::<_, i64>(0))?;
	Ok(conn)
}

fn open_readonly(path: &Path, key: &str) -> Result<Connection, rusqlite::Error> {
	match open_readonly_with_compat(path, key, None) {
		Ok(conn) => Ok(conn),
		Err(err) if err.sqlite_error_code() == Some(ErrorCode::NotADatabase) => {
			open_readonly_with_compat(path, key, Some(3)).or(Err(err))
		}
		Err(err) => Err(err),
	}
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
	/// Nama salinan unik: saat koneksi dimuat ulang, `Drop` koneksi lama tidak boleh
	/// menghapus salinan milik koneksi baru.
	fn copy_to_temp(path: &Path) -> AppResult<PathBuf> {
		let dir = std::env::temp_dir().join("sibos");
		std::fs::create_dir_all(&dir)?;
		remove_stale_snapshots(&dir);
		let target = dir.join(crate::util::unique_name("arkas-snapshot", "db"));
		copy_consistent(path, &target)?;
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
			let _ = std::fs::remove_file(wal_path(snapshot));
		}
	}
}

/// File WAL SQLite selalu `<nama file db>-wal` (mis. `arkas.db-wal`).
pub fn wal_path(db: &Path) -> PathBuf {
	let mut name = db.as_os_str().to_owned();
	name.push("-wal");
	PathBuf::from(name)
}

/// Ukuran + waktu ubah file (None bila tidak ada), untuk mendeteksi file yang berubah saat disalin.
fn stamp(path: &Path) -> Option<(u64, std::time::SystemTime)> {
	let meta = std::fs::metadata(path).ok()?;
	Some((meta.len(), meta.modified().ok()?))
}

/// Salin database ARKAS beserta file `-wal`-nya sebagai satu kesatuan. Bila ARKAS sedang menulis
/// (ukuran/waktu ubah berubah selama penyalinan), salinan diulang. File asli hanya dibaca.
pub fn copy_consistent(source: &Path, target: &Path) -> AppResult<()> {
	let source_wal = wal_path(source);
	let target_wal = wal_path(target);
	for attempt in 0..5 {
		if attempt > 0 {
			std::thread::sleep(Duration::from_millis(400));
		}
		let before = (stamp(source), stamp(&source_wal));
		std::fs::copy(source, target)?;
		if source_wal.is_file() {
			std::fs::copy(&source_wal, &target_wal)?;
		} else {
			// Jangan biarkan WAL lama di tujuan menempel ke salinan baru.
			let _ = std::fs::remove_file(&target_wal);
		}
		if (stamp(source), stamp(&source_wal)) == before {
			return Ok(());
		}
	}
	Err(AppError::ArkasBusy)
}

/// Hapus salinan sementara sisa sesi sebelumnya (mis. aplikasi ditutup paksa).
/// Salinan yang masih dibuka koneksi aktif gagal dihapus di Windows, dan itu memang diharapkan.
fn remove_stale_snapshots(dir: &Path) {
	let Ok(entries) = std::fs::read_dir(dir) else { return };
	for entry in entries.flatten() {
		let name = entry.file_name();
		if name.to_string_lossy().starts_with("arkas-snapshot") {
			let _ = std::fs::remove_file(entry.path());
		}
	}
}
