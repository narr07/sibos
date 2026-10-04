//! Test keamanan akses ARKAS memakai database sintetis terenkripsi.
//! Data di sini karangan, bukan data sekolah asli.

use std::path::{Path, PathBuf};

use rusqlite::Connection;

use crate::error::AppError;
use crate::repo::arkas::{queries, ArkasDb};

const KEY: &str = "kunci-uji-sintetis";

/// Bangun database mirip ARKAS (subset tabel) dan enkripsi dengan SQLCipher.
fn build_fixture(dir: &Path) -> PathBuf {
	let path = dir.join("arkas.db");
	let conn = Connection::open(&path).unwrap();
	conn.pragma_update(None, "key", KEY).unwrap();
	conn.execute_batch(
		"CREATE TABLE ref_sumber_dana (id_ref_sumber_dana INTEGER PRIMARY KEY, nama_sumber_dana TEXT);
		 CREATE TABLE anggaran (id_anggaran INTEGER PRIMARY KEY, tahun_anggaran INTEGER,
			id_ref_sumber_dana INTEGER, is_revisi INTEGER, is_approve INTEGER, jumlah REAL, soft_delete INTEGER);
		 CREATE TABLE mst_sekolah (sekolah_id TEXT, npsn TEXT, nama TEXT, alamat_jalan TEXT);
		 CREATE TABLE sekolah_penjab (sekolah_id TEXT, tahun INTEGER, ks TEXT, nip_ks TEXT,
			bendahara TEXT, nip_bendahara TEXT);
		 INSERT INTO ref_sumber_dana VALUES (1, 'BOSP Reguler'), (2, 'BOSP Kinerja');
		 INSERT INTO anggaran VALUES
			(1, 2025, 1, 0, 1, 100000000, 0),
			(2, 2026, 1, 0, 1, 120000000, 0),
			(3, 2026, 2, 0, 1, 30000000, 0),
			(4, 2024, 1, 0, 1, 90000000, 1);
		 INSERT INTO mst_sekolah VALUES ('S-1', '12345678', 'SD Negeri Contoh', 'Jl. Contoh No. 1');
		 INSERT INTO sekolah_penjab VALUES
			('S-1', 2025, 'Lama', '1', 'Lama', '2'),
			('S-1', 2026, 'Budi Santoso', '198001012005011001', 'Siti Aminah', '198502022010012002');",
	)
	.unwrap();
	drop(conn);
	path
}

fn checksum(path: &Path) -> Vec<u8> {
	// Bandingkan isi byte penuh: lebih ketat dari hash dan tanpa dependensi tambahan.
	std::fs::read(path).unwrap()
}

#[test]
fn reads_basic_data_with_correct_key() {
	let dir = tempfile::tempdir().unwrap();
	let path = build_fixture(dir.path());
	let db = ArkasDb::open(&path, KEY).unwrap();

	assert_eq!(queries::available_years(&db).unwrap(), vec![2026, 2025]);

	let funds = queries::fund_sources(&db, 2026).unwrap();
	let names: Vec<_> = funds.iter().map(|f| f.name.as_str()).collect();
	assert_eq!(names, vec!["BOSP Kinerja", "BOSP Reguler"]);

	let school = queries::school_info(&db, None).unwrap();
	assert_eq!(school.npsn.as_deref(), Some("12345678"));
	assert_eq!(school.nama.as_deref(), Some("SD Negeri Contoh"));
	assert_eq!(school.kepala_sekolah.as_deref(), Some("Budi Santoso"));
	assert_eq!(school.nip_bendahara.as_deref(), Some("198502022010012002"));

	let schema = queries::schema(&db).unwrap();
	assert!(schema.iter().any(|t| t.name == "anggaran" && t.row_count == 4));
}

#[test]
fn wrong_key_is_reported() {
	let dir = tempfile::tempdir().unwrap();
	let path = build_fixture(dir.path());
	let err = ArkasDb::open(&path, "kunci-salah").err().unwrap();
	assert!(matches!(err, AppError::ArkasBadKey), "dapat: {err:?}");
}

#[test]
fn missing_file_is_reported() {
	let dir = tempfile::tempdir().unwrap();
	let err = ArkasDb::open(&dir.path().join("tidak-ada.db"), KEY).err().unwrap();
	assert!(matches!(err, AppError::ArkasNotFound(_)));
}

#[test]
fn writes_are_rejected_and_file_is_untouched() {
	let dir = tempfile::tempdir().unwrap();
	let path = build_fixture(dir.path());
	let before = checksum(&path);

	{
		let db = ArkasDb::open(&path, KEY).unwrap();
		// Jalankan semua query baca.
		queries::available_years(&db).unwrap();
		queries::fund_sources(&db, 2026).unwrap();
		queries::school_info(&db, None).unwrap();
		queries::schema(&db).unwrap();

		// Percobaan menulis lewat koneksi yang sama harus ditolak SQLite.
		let attempts = [
			"UPDATE anggaran SET jumlah = 0",
			"DELETE FROM anggaran",
			"INSERT INTO ref_sumber_dana VALUES (9, 'X')",
			"CREATE TABLE x (a)",
			"DROP TABLE anggaran",
		];
		for sql in attempts {
			assert!(db.conn().execute_batch(sql).is_err(), "pernyataan tulis lolos: {sql}");
		}
		assert!(db.conn().pragma_update(None, "user_version", 99).is_err());
	}

	assert_eq!(checksum(&path), before, "file ARKAS berubah");
	assert!(!path.with_extension("db-wal").exists(), "file -wal terbentuk di samping ARKAS");
}

/// Uji ke database ARKAS asli di komputer ini (dijalankan manual, tidak di CI):
/// `SIBOS_ARKAS_KEY=... cargo test real_arkas -- --ignored --nocapture`
#[test]
#[ignore]
fn real_arkas_readonly_smoke() {
	let key = std::env::var("SIBOS_ARKAS_KEY").expect("set SIBOS_ARKAS_KEY");
	let path = crate::repo::arkas::default_path().expect("APPDATA tidak ada");
	let wal_existed = path.with_extension("db-wal").exists();
	let before = checksum(&path);

	{
		let db = ArkasDb::open(&path, &key).unwrap_or_else(|e| panic!("gagal membuka: {e}"));
		println!("snapshot dipakai: {}", db.snapshot_path.is_some());
		let schema = queries::schema(&db).unwrap();
		println!("jumlah tabel: {}", schema.len());
		for name in ["kas_umum", "kas_umum_nota", "anggaran", "rapbs", "rapbs_periode", "ref_sumber_dana"] {
			if let Some(t) = schema.iter().find(|t| t.name == name) {
				println!("  {name}: {} baris", t.row_count);
			}
		}
		let years = queries::available_years(&db).unwrap();
		println!("tahun anggaran: {years:?}");
		if let Some(y) = years.first() {
			let funds: Vec<_> = queries::fund_sources(&db, *y).unwrap().into_iter().map(|f| f.name).collect();
			println!("sumber dana {y}: {funds:?}");
		}
		let school = queries::school_info(&db, None).unwrap();
		println!("sekolah terbaca: {}", school.nama.is_some());
	}

	assert_eq!(checksum(&path), before, "file ARKAS berubah");
	assert_eq!(path.with_extension("db-wal").exists(), wal_existed, "file -wal berubah");
	println!("file arkas.db tidak berubah: OK");
}

/// Simpan kunci ARKAS ke Windows Credential Manager (dijalankan manual):
/// `SIBOS_ARKAS_KEY=... cargo test save_real_arkas_key -- --ignored --nocapture`
/// Kunci tidak dicetak; hanya memverifikasi bisa dibaca kembali dan membuka DB.
#[test]
#[ignore]
fn save_real_arkas_key() {
	let key = std::env::var("SIBOS_ARKAS_KEY").expect("set SIBOS_ARKAS_KEY");
	crate::secret::save_key(&key).expect("gagal menyimpan ke Credential Manager");

	// Baca langsung dari Credential Manager (bukan dari env) untuk memastikan benar tersimpan.
	let back = keyring::Entry::new("sibos", "arkas-db-key").unwrap().get_password().unwrap();
	assert_eq!(back.len(), key.len(), "kunci tersimpan tidak sama");

	let path = crate::repo::arkas::default_path().unwrap();
	let db = ArkasDb::open(&path, &back).expect("kunci tersimpan tidak bisa membuka ARKAS");
	let years = queries::available_years(&db).unwrap();
	println!("kunci tersimpan di Credential Manager (service=sibos, account=arkas-db-key)");
	println!("verifikasi: ARKAS terbuka, {} tahun anggaran terbaca", years.len());
}

/// Jelajahi struktur ARKAS asli untuk memverifikasi skema (dijalankan manual, kunci dari Credential Manager).
/// `cargo test real_arkas_explore -- --ignored --nocapture`
#[test]
#[ignore]
fn real_arkas_explore() {
	let key = crate::secret::load_key().unwrap().expect("kunci belum tersimpan");
	let path = crate::repo::arkas::default_path().unwrap();
	let before = checksum(&path);
	{
		let db = ArkasDb::open(&path, &key).unwrap();
		let c = db.conn();
		let schema = queries::schema(&db).unwrap();
		println!("== TABEL ({}):", schema.len());
		for t in &schema {
			println!("{} [{}]: {}", t.name, t.row_count, t.columns.join(", "));
		}
		let print_query = |title: &str, sql: &str| {
			println!("\n== {title}");
			let mut stmt = match c.prepare(sql) {
				Ok(s) => s,
				Err(e) => {
					println!("(gagal: {e})");
					return;
				}
			};
			let n = stmt.column_count();
			let names: Vec<String> = stmt.column_names().iter().map(|s| s.to_string()).collect();
			println!("{}", names.join(" | "));
			let mut rows = stmt.query([]).unwrap();
			while let Some(row) = rows.next().unwrap() {
				let vals: Vec<String> = (0..n)
					.map(|i| match row.get_ref(i).unwrap() {
						rusqlite::types::ValueRef::Null => "NULL".into(),
						rusqlite::types::ValueRef::Integer(v) => v.to_string(),
						rusqlite::types::ValueRef::Real(v) => v.to_string(),
						rusqlite::types::ValueRef::Text(t) => String::from_utf8_lossy(t).chars().take(70).collect(),
						rusqlite::types::ValueRef::Blob(_) => "<blob>".into(),
					})
					.collect();
				println!("{}", vals.join(" | "));
			}
		};
		for t in schema.iter().filter(|t| t.name.starts_with("ref_bku") || t.name == "ref_bku" || (t.name.starts_with("ref_") && t.row_count < 60)) {
			print_query(&format!("ISI {}", t.name), &format!("SELECT * FROM \"{}\" LIMIT 60", t.name));
		}
		print_query(
			"id_ref_bku: jumlah, contoh no_bukti & uraian",
			"SELECT id_ref_bku, count(*) n, sum(saldo) total, min(no_bukti) contoh_bukti, min(uraian) contoh_uraian
			 FROM kas_umum WHERE soft_delete = 0 GROUP BY id_ref_bku ORDER BY id_ref_bku",
		);
		print_query(
			"Kode kegiatan lewat rapbs_periode -> rapbs -> ref_kode (5 contoh)",
			"SELECT k.no_bukti, k.kode_rekening, rk.id_kode, rk.parent_kode, rk.id_level_kode, substr(rk.uraian_kode,1,40) keg, substr(r.uraian,1,30) item
			 FROM kas_umum k JOIN rapbs_periode rp ON rp.id_rapbs_periode = k.id_rapbs_periode
			 JOIN rapbs r ON r.id_rapbs = rp.id_rapbs LEFT JOIN ref_kode rk ON rk.id_ref_kode = r.id_ref_kode
			 WHERE k.soft_delete = 0 AND k.id_ref_bku = 15 ORDER BY k.tanggal_transaksi DESC LIMIT 5",
		);
		print_query(
			"Anggaran per tahun & sumber dana",
			"SELECT a.tahun_anggaran, a.id_ref_sumber_dana, sd.nama_sumber_dana, a.is_revisi, a.is_approve, a.is_aktif, a.jumlah, a.soft_delete,
			 (SELECT count(*) FROM kas_umum k WHERE k.id_anggaran = a.id_anggaran AND k.soft_delete = 0) n_kas
			 FROM anggaran a LEFT JOIN ref_sumber_dana sd ON sd.id_ref_sumber_dana = a.id_ref_sumber_dana ORDER BY 1, 2, 4",
		);
		print_query(
			"Baris saldo awal (8/9/28/29) tahun terakhir",
			"SELECT k.tanggal_transaksi, k.id_ref_bku, k.saldo, a.id_ref_sumber_dana, substr(k.uraian,1,45) uraian
			 FROM kas_umum k JOIN anggaran a ON a.id_anggaran = k.id_anggaran
			 WHERE k.soft_delete = 0 AND k.id_ref_bku IN (1,8,9,28,29) AND a.tahun_anggaran = (SELECT max(tahun_anggaran) FROM anggaran WHERE soft_delete=0)
			 ORDER BY k.tanggal_transaksi, k.id_ref_bku",
		);
		print_query(
			"aktivasi_bku tahun terakhir",
			"SELECT ab.id_periode, a.id_ref_sumber_dana, ab.tanggal_aktivasi, ab.tanggal_finish, ab.saldo_awal_bank, ab.saldo_awal_tunai, ab.saldo_akhir_bank, ab.saldo_akhir_tunai, ab.status_pengiriman, ab.soft_delete
			 FROM aktivasi_bku ab JOIN anggaran a ON a.id_anggaran = ab.id_anggaran
			 WHERE a.tahun_anggaran = (SELECT max(tahun_anggaran) FROM anggaran WHERE soft_delete=0) ORDER BY a.id_ref_sumber_dana, ab.id_periode",
		);
		print_query(
			"Format tanggal & create_date contoh",
			"SELECT tanggal_transaksi, create_date, typeof(saldo), typeof(id_kas_umum) FROM kas_umum WHERE soft_delete=0 LIMIT 3",
		);
		print_query(
			"Pajak: pasangan terima/setor per bulan tahun terakhir",
			"SELECT strftime('%m', k.tanggal_transaksi) bln, k.id_ref_bku, count(*) n, sum(k.saldo) total,
			 sum(k.is_ppn) ppn, sum(k.is_pph_21) p21, sum(k.is_pph_22) p22, sum(k.is_pph_23) p23, sum(k.is_pph_4) p4, sum(k.is_sspd) sspd
			 FROM kas_umum k JOIN anggaran a ON a.id_anggaran = k.id_anggaran
			 WHERE k.soft_delete = 0 AND k.id_ref_bku IN (10,11) AND a.tahun_anggaran = (SELECT max(tahun_anggaran) FROM anggaran WHERE soft_delete=0)
			 GROUP BY 1,2 ORDER BY 1,2",
		);
		print_query(
			"Rantai wilayah sekolah",
			"WITH RECURSIVE w(kode, nama, parent, level, depth) AS (
			   SELECT kode_wilayah, nama, mst_kode_wilayah, id_level_wilayah, 0 FROM mst_wilayah
			   WHERE kode_wilayah = (SELECT kode_wilayah FROM mst_sekolah LIMIT 1)
			   UNION ALL
			   SELECT m.kode_wilayah, m.nama, m.mst_kode_wilayah, m.id_level_wilayah, w.depth + 1
			   FROM mst_wilayah m JOIN w ON m.kode_wilayah = w.parent WHERE w.depth < 6)
			 SELECT '[' || kode || ']', nama, '[' || parent || ']', level, typeof(level) FROM w",
		);
		print_query(
			"Awalan no_bukti",
			"SELECT substr(no_bukti,1,3) awalan, id_ref_bku, count(*) n FROM kas_umum WHERE soft_delete = 0
			 GROUP BY 1,2 ORDER BY 1,2",
		);
		print_query(
			"Contoh 15 baris kas_umum terbaru",
			"SELECT tanggal_transaksi, id_ref_bku, no_bukti, kode_rekening, saldo, parent_id_kas_umum, is_ppn, is_pph_21, is_pph_23, is_pph_4, is_sspd, substr(uraian,1,50) uraian
			 FROM kas_umum WHERE soft_delete = 0 ORDER BY tanggal_transaksi DESC LIMIT 15",
		);
	}
	assert_eq!(checksum(&path), before, "file ARKAS berubah");
	println!("\nfile arkas.db tidak berubah: OK");
}

/// Bangun semua buku dari ARKAS asli dan bandingkan saldo akhir bulanan dengan aktivasi_bku.
/// `cargo test real_arkas_books -- --ignored --nocapture`
#[test]
#[ignore]
fn real_arkas_books() {
	use std::collections::HashMap;

	use crate::repo::arkas::bku;
	use crate::services::book::{build_book, BookKind, BookRequest};

	let key = crate::secret::load_key().unwrap().expect("kunci belum tersimpan");
	let path = crate::repo::arkas::default_path().unwrap();
	let before = checksum(&path);
	{
		let db = ArkasDb::open(&path, &key).unwrap();
		let school = queries::school_info(&db, None).unwrap();
		println!(
			"sekolah terbaca: nama={} kec={:?} kab={:?} prov={:?} ks={} bendahara={}",
			school.nama.is_some(),
			school.kecamatan,
			school.kabupaten,
			school.provinsi,
			school.kepala_sekolah.is_some(),
			school.bendahara.is_some()
		);
		let mut all_ok = true;
		for year in queries::available_years(&db).unwrap() {
			let rows = bku::kas_rows(&db, year).unwrap();
			let balances = bku::month_balances(&db, year).unwrap();
			let req = |kind| BookRequest { kind, year, month: None, until: None, fund: None };
			let umum = build_book(&req(BookKind::Umum), &rows, &balances, &HashMap::new());
			let bank = build_book(&req(BookKind::Bank), &rows, &balances, &HashMap::new());
			let tunai = build_book(&req(BookKind::Tunai), &rows, &balances, &HashMap::new());
			let pajak = build_book(&req(BookKind::Pajak), &rows, &balances, &HashMap::new());
			let ok = umum.checks.iter().filter(|c| c.ok).count();
			println!(
				"\n{year}: {} baris kas, {} baris BKU | terima {} keluar {} saldo akhir {} (bank {} tunai {}) | pajak saldo {}",
				rows.len(),
				umum.lines.len(),
				umum.total_penerimaan,
				umum.total_pengeluaran,
				umum.closing,
				bank.closing,
				tunai.closing,
				pajak.closing
			);
			println!("  cek vs aktivasi_bku: {ok}/{} bulan cocok", umum.checks.len());
			for c in umum.checks.iter().filter(|c| !c.ok) {
				all_ok = false;
				println!(
					"  TIDAK COCOK bulan {} {}: SIBOS bank {} tunai {} | ARKAS bank {} tunai {}",
					c.month, c.fund_name, c.computed_bank, c.computed_tunai, c.arkas_bank, c.arkas_tunai
				);
			}
			for w in &umum.warnings {
				all_ok = false;
				println!("  PERINGATAN: {w}");
			}
			assert_eq!(umum.closing, bank.closing + tunai.closing, "BKU umum = bank + tunai");
		}

		// Ringkasan, realisasi, dan nota harus konsisten dengan BKU.
		for year in queries::available_years(&db).unwrap() {
			use crate::repo::arkas::rkas;
			use crate::services::{nota, realisasi, summary};
			let rows = bku::kas_rows(&db, year).unwrap();
			let balances = bku::month_balances(&db, year).unwrap();
			let kode = rkas::kode_names(&db, year).unwrap();
			let rek = rkas::rekening_names(&db, year).unwrap();
			let input = summary::SummaryInput { rows: &rows, balances: &balances, kode_names: &kode, rekening_names: &rek };
			let s = summary::summarize(year, 1, 12, None, &input);
			let umum = build_book(&BookRequest { kind: BookKind::Umum, year, month: None, until: None, fund: None }, &rows, &balances, &HashMap::new());
			assert_eq!(
				s.total_belanja + s.pajak_disetor + s.pajak_bunga + s.pengembalian,
				umum.total_pengeluaran,
				"{year}: belanja + pajak setor + pajak bunga + pengembalian = pengeluaran BKU"
			);
			let items = rkas::rkas_items(&db, year).unwrap();
			let rin = realisasi::RealisasiInput { items: &items, rows: &rows, kode_names: &kode, rekening_names: &rek };
			let r = realisasi::build_realisasi(year, 12, None, &rin);
			assert_eq!(r.total_realisasi, s.total_belanja, "{year}: realisasi = belanja");
			let notas = bku::nota_infos(&db, year).unwrap();
			let (es, eo, ef, eu, em) = (HashMap::new(), HashMap::new(), HashMap::new(), HashMap::new(), HashMap::new());
			let nin = nota::NotaInput { rows: &rows, notas: &notas, merges: &[], statuses: &es, overrides: &eo, file_counts: &ef, uraian_overrides: &eu, manual_taxes: &em };
			let groups = nota::group_notas(&nin, None, None);
			assert_eq!(groups.iter().map(|g| g.total).sum::<i64>(), s.total_belanja, "{year}: total nota = belanja");
			let mut status: std::collections::BTreeMap<&str, usize> = std::collections::BTreeMap::new();
			for i in &r.items {
				*status.entry(i.status.as_str()).or_default() += 1;
			}
			println!(
				"{year}: pagu {} realisasi {} ({}%) | item RKAS {} {:?} | di luar RKAS {} | nota {} (dgn toko {}) | kelompok {:?}",
				r.total_pagu,
				r.total_realisasi,
				r.persen,
				r.items.len(),
				status,
				r.luar_rkas.len(),
				groups.len(),
				groups.iter().filter(|g| g.nota.as_ref().is_some_and(|n| n.nama_toko.is_some())).count(),
				s.belanja_kelompok.iter().map(|k| (k.nama.as_str(), k.total)).collect::<Vec<_>>()
			);
			println!(
				"  program: {:?}",
				s.belanja_program.iter().map(|p| format!("{} {} = {}", p.kode, p.nama.chars().take(30).collect::<String>(), p.total)).collect::<Vec<_>>()
			);
		}

		// Contoh file Excel untuk diperiksa manual.
		let year = queries::available_years(&db).unwrap()[0];
		let month = bku::last_active_month(&db, year).unwrap();
		let rows = bku::kas_rows(&db, year).unwrap();
		let balances = bku::month_balances(&db, year).unwrap();
		let book = build_book(&BookRequest { kind: BookKind::Umum, year, month, until: None, fund: None }, &rows, &balances, &HashMap::new());
		let settings = crate::pengaturan::bawaan(&school);
		let out = std::env::temp_dir().join("sibos-contoh-bku.xlsx");
		crate::export::xlsx::write_book(&out, &book, &school, &settings, "Semua sumber dana").unwrap();
		println!("\ncontoh Excel BKU {year} bulan {month:?}: {}", out.display());
		println!("hasil keseluruhan: {}", if all_ok { "SEMUA COCOK" } else { "ADA SELISIH (lihat di atas)" });
	}
	assert_eq!(checksum(&path), before, "file ARKAS berubah");
	println!("file arkas.db tidak berubah: OK");
}

/// Telusuri asal baris pajak: `cargo test real_arkas_trace_tax -- --ignored --nocapture`
#[test]
#[ignore]
fn real_arkas_trace_tax() {
	let key = crate::secret::load_key().unwrap().expect("kunci belum tersimpan");
	let path = crate::repo::arkas::default_path().unwrap();
	let before = checksum(&path);
	{
		let db = ArkasDb::open(&path, &key).unwrap();
		let mut stmt = db
			.conn()
			.prepare(
				"SELECT t.tanggal_transaksi, t.id_ref_bku, rb.bku, t.uraian, t.uraian_pajak, t.saldo, t.is_ppn, t.is_pph_21, t.is_pph_23,
				        p.no_bukti, p.uraian, p.saldo, n.nama_toko, n.no_nota, n.has_ppn, n.total, t.create_date
				 FROM kas_umum t
				 LEFT JOIN ref_bku rb ON rb.id_ref_bku = t.id_ref_bku
				 LEFT JOIN kas_umum p ON p.id_kas_umum = t.parent_id_kas_umum
				 LEFT JOIN kas_umum_nota n ON n.id_kas_nota = p.id_kas_nota
				 WHERE t.soft_delete = 0 AND t.saldo IN (81759, 375057, 105546, 215541)
				 ORDER BY t.tanggal_transaksi, t.saldo, t.id_ref_bku",
			)
			.unwrap();
		let mut rows = stmt.query([]).unwrap();
		while let Some(r) = rows.next().unwrap() {
			let s = |i: usize| r.get::<_, Option<String>>(i).ok().flatten().unwrap_or_default();
			let n = |i: usize| r.get::<_, Option<i64>>(i).ok().flatten().unwrap_or(0);
			println!(
				"{} | ref {} {} | uraian pajak: '{}' / '{}' | Rp {} | ppn={} | INDUK: {} '{}' Rp {} | NOTA: {} no {} has_ppn={} total {} | dibuat {}",
				s(0), n(1), s(2), s(3), s(4), n(5), n(6), s(9), s(10), n(11), s(12), s(13), n(14), n(15), s(16)
			);
		}
	}
	assert_eq!(checksum(&path), before, "file ARKAS berubah");
	println!("file arkas.db tidak berubah: OK");
}

/// Pajak vs SIPLah: `cargo test real_arkas_tax_siplah -- --ignored --nocapture`
#[test]
#[ignore]
fn real_arkas_tax_siplah() {
	let key = crate::secret::load_key().unwrap().expect("kunci belum tersimpan");
	let path = crate::repo::arkas::default_path().unwrap();
	let before = checksum(&path);
	{
		let db = ArkasDb::open(&path, &key).unwrap();
		let mut stmt = db
			.conn()
			.prepare(
				"WITH nota AS (
				   SELECT n.id_kas_nota, n.nama_toko, n.is_beli_di_siplah AS siplah, n.has_ppn, CAST(n.total AS INTEGER) AS total,
				          a.tahun_anggaran AS tahun,
				          (SELECT sum(k.saldo) FROM kas_umum k WHERE k.id_kas_nota = n.id_kas_nota AND k.soft_delete = 0 AND k.id_ref_bku IN (4,15,24,35)) AS belanja,
				          (SELECT sum(t.saldo) FROM kas_umum t JOIN kas_umum p ON p.id_kas_umum = t.parent_id_kas_umum
				            WHERE p.id_kas_nota = n.id_kas_nota AND t.soft_delete = 0 AND t.id_ref_bku IN (10,30)) AS pajak_terima,
				          (SELECT sum(t.saldo) FROM kas_umum t JOIN kas_umum p ON p.id_kas_umum = t.parent_id_kas_umum
				            WHERE p.id_kas_nota = n.id_kas_nota AND t.soft_delete = 0 AND t.id_ref_bku IN (11,31)) AS pajak_setor,
				          (SELECT min(t.tanggal_transaksi) = max(t.tanggal_transaksi) FROM kas_umum t JOIN kas_umum p ON p.id_kas_umum = t.parent_id_kas_umum
				            WHERE p.id_kas_nota = n.id_kas_nota AND t.soft_delete = 0 AND t.id_ref_bku IN (10,11,30,31)) AS sama_hari
				   FROM kas_umum_nota n
				   JOIN kas_umum k0 ON k0.id_kas_nota = n.id_kas_nota AND k0.soft_delete = 0
				   JOIN anggaran a ON a.id_anggaran = k0.id_anggaran
				   WHERE n.soft_delete = 0
				   GROUP BY n.id_kas_nota)
				 SELECT tahun, siplah, (pajak_terima IS NOT NULL) AS ada_pajak, count(*) AS jumlah_nota,
				        sum(belanja), sum(COALESCE(pajak_terima,0)), sum(COALESCE(pajak_setor,0)), sum(has_ppn),
				        sum(CASE WHEN pajak_terima IS NOT NULL AND sama_hari THEN 1 ELSE 0 END)
				 FROM nota GROUP BY tahun, siplah, ada_pajak ORDER BY tahun DESC, siplah, ada_pajak",
			)
			.unwrap();
		println!("tahun | siplah | ada_pajak | nota | belanja | pajak_terima | pajak_setor | has_ppn | terima&setor_sama_hari");
		let mut rows = stmt.query([]).unwrap();
		while let Some(r) = rows.next().unwrap() {
			let v: Vec<String> = (0..9).map(|i| r.get::<_, Option<i64>>(i).ok().flatten().map(|x| x.to_string()).unwrap_or_else(|| "-".into())).collect();
			println!("{}", v.join(" | "));
		}
	}
	assert_eq!(checksum(&path), before, "file ARKAS berubah");
	println!("file arkas.db tidak berubah: OK");
}

/// Pasang template dokumen + profil penyedia dari file JSON ke sibos.db milik pengguna (bukan ARKAS).
/// `SIBOS_SEED_JSON=... cargo test seed_doc_templates -- --ignored --nocapture`
#[test]
#[ignore]
fn seed_doc_templates() {
	use crate::repo::app::dokumen::{DocTemplate, Penyedia};
	use crate::repo::app::AppDb;

	#[derive(serde::Deserialize)]
	struct Seed {
		templates: Vec<DocTemplate>,
		penyedia: Penyedia,
	}
	let json = std::fs::read_to_string(std::env::var("SIBOS_SEED_JSON").expect("set SIBOS_SEED_JSON")).unwrap();
	let seed: Seed = serde_json::from_str(&json).unwrap();
	let appdata = std::env::var("APPDATA").unwrap();
	let path = Path::new(&appdata).join("id.sibos.app").join("sibos.db");
	let db = AppDb::open(&path).unwrap();
	let existing = db.doc_templates().unwrap();
	for t in &seed.templates {
		// Template dengan nama sama diperbarui (id dipertahankan), yang baru ditambahkan.
		let mut t = t.clone();
		if let Some(old) = existing.iter().find(|e| e.nama == t.nama) {
			t.id = old.id.clone();
			println!("diperbarui: {} ({})", t.nama, t.jenis);
		} else {
			println!("dipasang: {} ({})", t.nama, t.jenis);
		}
		db.doc_template_save(&t).unwrap();
	}
	db.penyedia_save(&seed.penyedia).unwrap();
	println!("profil penyedia: {}", seed.penyedia.nama);
	println!("total template: {}", db.doc_templates().unwrap().len());
}

/// Isi kop surat di Pengaturan sibos.db (bukan ARKAS): teks kop + logo kiri (dibuat persegi).
/// `SIBOS_LOGO_KIRI=... cargo test seed_kop -- --ignored --nocapture`
#[test]
#[ignore]
fn seed_kop() {
	use base64::Engine;
	use crate::pengaturan::{self, Pengaturan};
	use crate::repo::app::AppDb;

	let appdata = std::env::var("APPDATA").unwrap();
	let db = AppDb::open(&Path::new(&appdata).join("id.sibos.app").join("sibos.db")).unwrap();
	let mut p: Pengaturan = pengaturan::load(&db).unwrap();
	p.kop.pemerintah = "PEMERINTAH KABUPATEN MAJALENGKA".into();
	p.kop.dinas = "DINAS PENDIDIKAN".into();
	p.kop.nama_sekolah = "SEKOLAH DASAR NEGERI TEJA II".into();
	p.kop.baris_alamat = "Alamat Blok Desa, Desa Teja Kecamatan Rajagaluh, Kabupaten Majalengka-45472".into();
	p.kop.baris_kontak = "NPSN. 20246133 E-mail: sdnteja2@gmail.com website: sdnteja2.sch.id".into();
	p.kop.email = "sdnteja2@gmail.com".into();
	p.kop.laman = "sdnteja2.sch.id".into();

	if let Ok(path) = std::env::var("SIBOS_LOGO_KIRI") {
		// Tempatkan logo di tengah kanvas persegi transparan, tanpa ditarik.
		let img = image::open(path).unwrap().to_rgba8();
		let size = 400u32;
		let scale = (size as f32 / img.width() as f32).min(size as f32 / img.height() as f32);
		let (w, h) = ((img.width() as f32 * scale) as u32, (img.height() as f32 * scale) as u32);
		let resized = image::imageops::resize(&img, w, h, image::imageops::FilterType::Lanczos3);
		let mut canvas = image::RgbaImage::new(size, size);
		image::imageops::overlay(&mut canvas, &resized, i64::from((size - w) / 2), i64::from((size - h) / 2));
		let mut png = std::io::Cursor::new(Vec::new());
		canvas.write_to(&mut png, image::ImageFormat::Png).unwrap();
		p.kop.logo_kiri = format!("data:image/png;base64,{}", base64::engine::general_purpose::STANDARD.encode(png.into_inner()));
		println!("logo kiri: {}x{} -> persegi {size}x{size}", img.width(), img.height());
	}
	db.setting_set(pengaturan::SETTING_KEY, &serde_json::to_value(&p).unwrap()).unwrap();
	println!("kop tersimpan: {} / {}", p.kop.pemerintah, p.kop.nama_sekolah);
}

/// Coba beberapa kandidat kunci (dipisah `|`) ke ARKAS asli dengan setelan SQLCipher v4 dan v3.
/// Hanya mencetak nomor kandidat yang berhasil, bukan kuncinya.
#[test]
#[ignore]
fn real_arkas_try_keys() {
	use rusqlite::OpenFlags;
	let candidates = std::env::var("SIBOS_KEY_CANDIDATES").expect("set SIBOS_KEY_CANDIDATES");
	let path = crate::repo::arkas::default_path().unwrap();
	let before = checksum(&path);
	let mut found = false;
	for (i, key) in candidates.split('|').enumerate() {
		for compat in [4, 3] {
			let conn = Connection::open_with_flags(&path, OpenFlags::SQLITE_OPEN_READ_ONLY).unwrap();
			conn.pragma_update(None, "key", key).unwrap();
			conn.pragma_update(None, "cipher_compatibility", compat).unwrap();
			conn.pragma_update(None, "query_only", true).unwrap();
			let ok = conn.query_row("SELECT count(*) FROM sqlite_master", [], |r| r.get::<_, i64>(0)).is_ok();
			println!("kandidat #{} (panjang {}), SQLCipher v{compat}: {}", i + 1, key.len(), if ok { "BERHASIL" } else { "gagal" });
			found |= ok;
		}
	}
	assert_eq!(checksum(&path), before, "file ARKAS berubah");
	println!("file arkas.db tidak berubah: OK");
	assert!(found, "tidak ada kandidat yang cocok");
}

/// Ambil isi semua string literal ("...") dari kode Rust, termasuk yang multi-baris.
fn string_literals(code: &str) -> Vec<String> {
	let mut out = Vec::new();
	let mut chars = code.chars().peekable();
	let mut in_str = false;
	let mut current = String::new();
	while let Some(c) = chars.next() {
		if in_str {
			match c {
				'\\' => {
					chars.next();
				}
				'"' => {
					out.push(std::mem::take(&mut current));
					in_str = false;
				}
				_ => current.push(c),
			}
		} else if c == '"' {
			in_str = true;
		} else if c == '\'' {
			// Lewati literal karakter seperti '"' agar tidak dianggap awal string.
			if chars.peek() == Some(&'\\') {
				chars.next();
			}
			chars.next();
			if chars.peek() == Some(&'\'') {
				chars.next();
			}
		} else if c == '/' && chars.peek() == Some(&'/') {
			// Komentar baris: lewati sampai akhir baris.
			for c2 in chars.by_ref() {
				if c2 == '\n' {
					break;
				}
			}
		}
	}
	out
}

fn has_word(text: &str, word: &str) -> bool {
	text.split(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
		.any(|w| w.eq_ignore_ascii_case(word))
}

/// Kode di `repo/arkas` tidak boleh berisi pernyataan tulis sama sekali.
#[test]
fn arkas_module_has_no_write_statements() {
	let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src").join("repo").join("arkas");
	// Kata kunci SQL yang menulis, dicari di dalam string literal.
	let banned_sql = [
		"INSERT", "UPDATE", "DELETE", "REPLACE", "UPSERT", "ALTER", "DROP", "CREATE", "VACUUM", "REINDEX",
		"ATTACH", "journal_mode", "user_version", "wal_checkpoint", "writable_schema",
	];
	// API rusqlite untuk menulis, dicari di kode.
	let banned_api = ["execute", "execute_batch", "transaction", "savepoint"];

	let mut hits = Vec::new();
	for entry in std::fs::read_dir(&dir).unwrap() {
		let path = entry.unwrap().path();
		let code = std::fs::read_to_string(&path).unwrap();
		for lit in string_literals(&code) {
			for word in banned_sql {
				if has_word(&lit, word) {
					hits.push(format!("{}: SQL `{word}` di \"{}\"", path.display(), lit.trim()));
				}
			}
		}
		for (n, line) in code.lines().enumerate() {
			let code_part = line.split("//").next().unwrap_or("");
			for word in banned_api {
				if has_word(code_part, word) {
					hits.push(format!("{}:{}: API `{word}`", path.display(), n + 1));
				}
			}
		}
	}
	assert!(hits.is_empty(), "pernyataan tulis ditemukan:\n{}", hits.join("\n"));
}

#[test]
fn write_scanner_catches_violations() {
	let sample = "let a = \"SELECT 1\";\nconn.query(\"update kas_umum set saldo = 0\");";
	let lits = string_literals(sample);
	assert!(lits.iter().any(|l| has_word(l, "UPDATE")));
	assert!(!lits.iter().any(|l| has_word(l, "DROP")));
}
