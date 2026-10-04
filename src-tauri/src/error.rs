use serde::{Serialize, Serializer};

/// Error yang dikirim ke frontend sebagai `{ code, message }`.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
	#[error("Database ARKAS belum terhubung")]
	ArkasNotConnected,
	#[error("File database ARKAS tidak ditemukan: {0}")]
	ArkasNotFound(String),
	#[error("Kunci database ARKAS belum diisi")]
	ArkasNoKey,
	#[error("Kunci database ARKAS salah atau file bukan database ARKAS")]
	ArkasBadKey,
	#[error("Database ARKAS sedang dipakai, coba lagi sebentar")]
	ArkasBusy,
	#[error("Input tidak valid: {0}")]
	InvalidInput(String),
	#[error("Gagal mengakses penyimpanan kunci: {0}")]
	Secret(String),
	#[error("Kesalahan database: {0}")]
	Sqlite(#[from] rusqlite::Error),
	#[error("Kesalahan file: {0}")]
	Io(#[from] std::io::Error),
	#[error("Gagal membuat file Excel: {0}")]
	Xlsx(#[from] rust_xlsxwriter::XlsxError),
}

impl AppError {
	pub fn code(&self) -> &'static str {
		match self {
			AppError::ArkasNotConnected => "arkas_not_connected",
			AppError::ArkasNotFound(_) => "arkas_not_found",
			AppError::ArkasNoKey => "arkas_no_key",
			AppError::ArkasBadKey => "arkas_bad_key",
			AppError::ArkasBusy => "arkas_busy",
			AppError::InvalidInput(_) => "invalid_input",
			AppError::Secret(_) => "secret",
			AppError::Sqlite(_) => "sqlite",
			AppError::Io(_) => "io",
			AppError::Xlsx(_) => "xlsx",
		}
	}
}

impl Serialize for AppError {
	fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		use serde::ser::SerializeStruct;
		let mut s = serializer.serialize_struct("AppError", 2)?;
		s.serialize_field("code", self.code())?;
		s.serialize_field("message", &self.to_string())?;
		s.end()
	}
}

pub type AppResult<T> = Result<T, AppError>;

/// Validasi tahun anggaran (sama dengan batas yang dipakai di seluruh aplikasi).
pub fn validate_year(year: i32) -> AppResult<i32> {
	if (2020..=2100).contains(&year) {
		Ok(year)
	} else {
		Err(AppError::InvalidInput(format!("tahun {year} di luar rentang 2020-2100")))
	}
}
