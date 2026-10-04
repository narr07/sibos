mod commands;
mod error;
mod export;
mod pengaturan;
mod repo;
mod secret;
mod services;
mod state;
mod util;

#[cfg(test)]
mod tests;

use tauri::Manager;

use crate::repo::app::AppDb;
use crate::state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
	tauri::Builder::default()
		.plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
			if let Some(window) = app.get_webview_window("main") {
				let _ = window.unminimize();
				let _ = window.set_focus();
			}
		}))
		.plugin(tauri_plugin_window_state::Builder::default().build())
		.plugin(tauri_plugin_dialog::init())
		.plugin(tauri_plugin_opener::init())
		.plugin(tauri_plugin_store::Builder::new().build())
		.plugin(tauri_plugin_updater::Builder::new().build())
		.plugin(tauri_plugin_process::init())
		.setup(|app| {
			let data_dir = app.path().app_data_dir()?;
			let app_db = AppDb::open(&data_dir.join("sibos.db"))?;
			app.manage(AppState::new(app_db, data_dir));
			// Backup otomatis (bila diaktifkan) di latar belakang agar jendela tidak tertahan.
			let handle = app.handle().clone();
			std::thread::spawn(move || {
				let state = handle.state::<AppState>();
				if let Err(e) = commands::backup::run_auto_backup(&state) {
					eprintln!("backup otomatis gagal: {e}");
				}
			});
			Ok(())
		})
		.invoke_handler(tauri::generate_handler![
			commands::arkas::arkas_status,
			commands::arkas::arkas_connect,
			commands::arkas::arkas_disconnect,
			commands::arkas::available_years,
			commands::arkas::fund_sources,
			commands::arkas::school_info,
			commands::arkas::arkas_schema,
			commands::settings::settings_get,
			commands::settings::settings_set,
			commands::settings::app_info,
			commands::settings::pengaturan_get,
			commands::settings::pengaturan_set,
			commands::bku::book,
			commands::bku::last_active_month,
			commands::bku::uraian_override_set,
			commands::bku::uraian_override_delete,
			commands::bku::export_book_xlsx,
			commands::nota::nota_list,
			commands::nota::merge_create,
			commands::nota::merge_update,
			commands::nota::merge_delete,
			commands::nota::print_status_set,
			commands::nota::print_override_set,
			commands::nota::nota_tax_set,
			commands::dokumen::doc_template_list,
			commands::dokumen::doc_template_save,
			commands::dokumen::doc_template_delete,
			commands::dokumen::doc_template_export,
			commands::dokumen::doc_template_import,
			commands::dokumen::penyedia_list,
			commands::dokumen::penyedia_save,
			commands::dokumen::penyedia_delete,
			commands::nota::nota_file_list,
			commands::nota::nota_file_upload,
			commands::nota::nota_file_data,
			commands::nota::nota_file_delete,
			commands::laporan::period_summary,
			commands::laporan::period_summary_by_fund,
			commands::laporan::rekon_bank,
			commands::laporan::bank_statement_set,
			commands::laporan::register_kas_get,
			commands::laporan::register_kas_set,
			commands::laporan::manual_tax_list,
			commands::laporan::manual_tax_save,
			commands::laporan::manual_tax_delete,
			commands::rkas::kertas_kerja,
			commands::rkas::realisasi,
			commands::rkas::standar_harga_search,
			commands::rkas::dashboard,
			commands::backup::backup_create,
			commands::backup::backup_inspect,
			commands::backup::backup_restore,
			commands::backup::backup_arkas_db,
			commands::backup::auto_backup_get,
			commands::backup::auto_backup_set,
			commands::export::export_kertas_kerja_xlsx,
			commands::export::export_realisasi_xlsx,
			commands::export::export_all_xlsx,
			commands::rkas_draft::rkas_draft_list,
			commands::rkas_draft::rkas_draft_from_arkas,
			commands::rkas_draft::rkas_draft_copy,
			commands::rkas_draft::rkas_draft_detail,
			commands::rkas_draft::rkas_draft_item_save,
			commands::rkas_draft::rkas_draft_item_delete,
			commands::rkas_draft::rkas_draft_rename,
			commands::rkas_draft::rkas_draft_delete,
			commands::rkas_draft::rkas_draft_export_xlsx,
		])
		.run(tauri::generate_context!())
		.expect("gagal menjalankan aplikasi");
}
