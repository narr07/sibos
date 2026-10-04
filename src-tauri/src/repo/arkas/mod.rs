//! Akses database ARKAS. Seluruh modul ini HANYA BACA.

pub mod bku;
pub mod conn;
pub mod queries;
pub mod rkas;

pub use conn::{default_path, ArkasDb};
