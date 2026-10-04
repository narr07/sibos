import { invoke, isTauri } from "@tauri-apps/api/core";

export type ConnectionState = "connected" | "not_found" | "no_key" | "bad_key" | "busy" | "error" | "disconnected";

export interface SchoolInfo {
	sekolahId: string | null
	npsn: string | null
	nama: string | null
	alamat: string | null
	kepalaSekolah: string | null
	nipKepalaSekolah: string | null
	bendahara: string | null
	nipBendahara: string | null
	komite: string | null
	nipKomite: string | null
	telepon: string | null
	kecamatan: string | null
	kabupaten: string | null
	provinsi: string | null
}

export interface ConnectionStatus {
	state: ConnectionState
	path: string | null
	defaultPath: string | null
	usingSnapshot: boolean
	hasStoredKey: boolean
	message: string | null
	school: SchoolInfo | null
}

export interface FundSource {
	id: number
	name: string
	kode: string | null
}

export interface TableInfo {
	name: string
	columns: string[]
	rowCount: number
}

export interface AppInfo {
	version: string
	schemaVersion: number
	dataDir: string
}

export type BookKind = "umum" | "bank" | "tunai" | "pajak";

export interface BookLine {
	id: string
	parentId: string | null
	tanggal: string
	kodeKegiatan: string | null
	kodeRekening: string | null
	noBukti: string | null
	uraian: string
	uraianAsli: string
	overridden: boolean
	jenis: string | null
	jenisPajak: string | null
	fundName: string
	penerimaan: number
	pengeluaran: number
	saldo: number
	isTax: boolean
	idKasNota: string | null
	siplah: boolean
}

export interface MonthCheck {
	month: number
	fundId: number
	fundName: string
	computedBank: number
	computedTunai: number
	arkasBank: number
	arkasTunai: number
	ok: boolean
}

export interface Book {
	kind: BookKind
	year: number
	month: number | null
	fund: number | null
	opening: number
	totalPenerimaan: number
	totalPengeluaran: number
	closing: number
	closingBank: number
	closingTunai: number
	lines: BookLine[]
	checks: MonthCheck[]
	warnings: string[]
}

export interface Pengaturan {
	pejabat: {
		kepalaSekolah: string
		nipKepalaSekolah: string
		bendahara: string
		nipBendahara: string
		pemegangBarang: string
		nipPemegangBarang: string
		petugasRekon: string
		nipPetugasRekon: string
		komite: string
		nipKomite: string
		skKepalaSekolah: string
		skBendahara: string
		tanggalSkBendahara: string
	}
	kop: {
		pemerintah: string
		dinas: string
		namaSekolah: string
		alamat: string
		telepon: string
		email: string
		laman: string
		barisAlamat: string
		barisKontak: string
		logoKiri: string
		logoKanan: string
	}
	ba: { nomor: string, tanggalSurat: string, nomorSk: string, tanggalSk: string, tempat: string }
	cetak: { kota: string, kertas: string }
}

export interface PengaturanView {
	tersimpan: Pengaturan
	bawaan: Pengaturan
	efektif: Pengaturan
}

export interface ApiError {
	code: string
	message: string
}

/** Error di sisi frontend dengan bentuk yang sama seperti error dari Rust. */
export class ClientError extends Error implements ApiError {
	constructor(public code: string, message: string) {
		super(message);
	}
}

export const inTauri = () => isTauri();

/** Pesan error yang bisa ditampilkan ke pengguna. */
export const errorMessage = (err: unknown): string => {
	if (err && typeof err === "object" && "message" in err) return String((err as ApiError).message);
	return String(err);
};

const call = <T>(cmd: string, args?: Record<string, unknown>): Promise<T> => {
	if (!isTauri()) {
		return Promise.reject(new ClientError("no_tauri", "Jalankan lewat aplikasi desktop (bun run tauri:dev)."));
	}
	return invoke<T>(cmd, args);
};

/** Pemanggil command Rust. Nama dan bentuk data mengikuti src-tauri/src/commands. */
export const api = {
	arkasStatus: () => call<ConnectionStatus>("arkas_status"),
	arkasConnect: (args: { path?: string, key?: string, rememberKey?: boolean } = {}) =>
		call<ConnectionStatus>("arkas_connect", args),
	arkasDisconnect: (forgetKey = false) => call<ConnectionStatus>("arkas_disconnect", { forgetKey }),
	availableYears: () => call<number[]>("available_years"),
	fundSources: (year: number) => call<FundSource[]>("fund_sources", { year }),
	schoolInfo: (year?: number | null) => call<SchoolInfo>("school_info", { year: year ?? null }),
	arkasSchema: () => call<TableInfo[]>("arkas_schema"),
	settingsGet: <T = unknown>(key: string) => call<T | null>("settings_get", { key }),
	settingsSet: (key: string, value: unknown) => call<void>("settings_set", { key, value }),
	appInfo: () => call<AppInfo>("app_info"),
	pengaturanGet: (year?: number | null) => call<PengaturanView>("pengaturan_get", { year: year ?? null }),
	pengaturanSet: (value: Pengaturan) => call<void>("pengaturan_set", { value }),
	book: (args: { kind: BookKind, year: number, month?: number | null, fund?: number | null }) =>
		call<Book>("book", { month: null, fund: null, ...args }),
	lastActiveMonth: (year: number) => call<number | null>("last_active_month", { year }),
	uraianOverrideSet: (year: number, id: string, uraian: string) => call<void>("uraian_override_set", { year, id, uraian }),
	uraianOverrideDelete: (id: string) => call<void>("uraian_override_delete", { id }),
	exportBookXlsx: (args: { kind: BookKind, year: number, month?: number | null, fund?: number | null, path: string }) =>
		call<string>("export_book_xlsx", { month: null, fund: null, ...args }),

	// Nota / kwitansi
	notaList: (year: number, month: number | null, fund: number | null) => call<NotaGroup[]>("nota_list", { year, month, fund }),
	mergeCreate: (args: { year: number, noBukti: string, uraian: string, tanggal?: string | null, items: string[] }) =>
		call<string>("merge_create", { tanggal: null, ...args }),
	mergeUpdate: (args: { id: string, noBukti: string, uraian: string, tanggal?: string | null }) =>
		call<void>("merge_update", { tanggal: null, ...args }),
	mergeDelete: (id: string) => call<void>("merge_delete", { id }),
	printStatusSet: (kind: "bukti" | "a2" | "nota", refs: string[], printed: boolean) => call<void>("print_status_set", { kind, refs, printed }),
	printOverrideSet: (refId: string, value: PrintOverride) => call<void>("print_override_set", { refId, value }),
	notaTaxSet: (args: {
		year: number
		refId: string
		tanggal: string
		noBukti: string | null
		keterangan: string
		fund: number | null
		taxes: { jenis: string, nominal: number, tanggalSetor: string | null }[]
	}) => call<void>("nota_tax_set", args),
	docTemplateList: () => call<DocTemplate[]>("doc_template_list"),
	docTemplateSave: (template: Omit<DocTemplate, "updatedAt"> & { updatedAt?: string }) => call<string>("doc_template_save", { template: { updatedAt: "", ...template } }),
	docTemplateDelete: (id: string) => call<void>("doc_template_delete", { id }),
	docTemplateExport: (ids: string[], path: string) => call<number>("doc_template_export", { ids, path }),
	docTemplateImport: (path: string) => call<number>("doc_template_import", { path }),
	penyediaList: () => call<Penyedia[]>("penyedia_list"),
	penyediaSave: (penyedia: Penyedia) => call<void>("penyedia_save", { penyedia }),
	penyediaDelete: (nama: string) => call<void>("penyedia_delete", { nama }),
	notaFileList: (year: number) => call<NotaFile[]>("nota_file_list", { year }),
	notaFileUpload: (year: number, refId: string, month: number, sourcePath: string) =>
		call<string>("nota_file_upload", { year, refId, month, sourcePath }),
	notaFileData: (id: string) => call<string>("nota_file_data", { id }),
	notaFileDelete: (id: string) => call<void>("nota_file_delete", { id }),

	// Laporan
	periodSummary: (year: number, start: number, end: number, fund: number | null) =>
		call<PeriodSummary>("period_summary", { year, start, end, fund }),
	periodSummaryByFund: (year: number, start: number, end: number) =>
		call<[FundSource, PeriodSummary][]>("period_summary_by_fund", { year, start, end }),
	rekonBank: (year: number, fund: number | null) => call<RekonBank>("rekon_bank", { year, fund }),
	bankStatementSet: (year: number, fund: number | null, month: number, saldo: number | null, keterangan: string | null) =>
		call<void>("bank_statement_set", { year, fund, month, saldo, keterangan }),
	registerKasGet: (year: number, month: number, fund: number | null) => call<RegisterKas>("register_kas_get", { year, month, fund }),
	registerKasSet: (year: number, month: number, fund: number | null, value: CashRegister) =>
		call<void>("register_kas_set", { year, month, fund, value }),
	manualTaxList: (year: number) => call<ManualTax[]>("manual_tax_list", { year }),
	manualTaxSave: (entry: ManualTax) => call<string>("manual_tax_save", { entry }),
	manualTaxDelete: (id: string) => call<void>("manual_tax_delete", { id }),

	// RKAS
	kertasKerja: (year: number, fund: number | null) => call<KertasKerja>("kertas_kerja", { year, fund }),
	realisasi: (year: number, upto: number | null, fund: number | null) => call<Realisasi>("realisasi", { year, upto, fund }),
	standarHargaSearch: (year: number, keyword: string) => call<StandarHarga[]>("standar_harga_search", { year, keyword }),
	dashboard: (year: number, fund: number | null) => call<Dashboard>("dashboard", { year, fund }),
	rkasDraftList: (year: number) => call<Draft[]>("rkas_draft_list", { year }),
	rkasDraftFromArkas: (year: number, fund: number, nama: string) => call<string>("rkas_draft_from_arkas", { year, fund, nama }),
	rkasDraftCopy: (id: string, jenis: "perubahan" | "pergeseran", nama: string) => call<string>("rkas_draft_copy", { id, jenis, nama }),
	rkasDraftDetail: (id: string) => call<DraftDetail>("rkas_draft_detail", { id }),
	rkasDraftItemSave: (draftId: string, item: DraftItem) => call<string>("rkas_draft_item_save", { draftId, item }),
	rkasDraftItemDelete: (draftId: string, itemId: string) => call<void>("rkas_draft_item_delete", { draftId, itemId }),
	rkasDraftRename: (id: string, nama: string) => call<void>("rkas_draft_rename", { id, nama }),
	rkasDraftDelete: (id: string) => call<void>("rkas_draft_delete", { id }),
	rkasDraftExportXlsx: (id: string, path: string) => call<string>("rkas_draft_export_xlsx", { id, path }),

	// Export & backup
	exportKertasKerjaXlsx: (year: number, fund: number | null, search: string, start: number, end: number, judul: string, triwulan: boolean, lembar: boolean, path: string) =>
		call<string>("export_kertas_kerja_xlsx", { year, fund, search, start, end, judul, triwulan, lembar, path }),
	exportRealisasiXlsx: (year: number, upto: number | null, fund: number | null, path: string) =>
		call<string>("export_realisasi_xlsx", { year, upto, fund, path }),
	exportAllXlsx: (year: number, fund: number | null, path: string) => call<string>("export_all_xlsx", { year, fund, path }),
	backupCreate: (path: string) => call<BackupInfo>("backup_create", { path }),
	backupInspect: (path: string) => call<BackupInfo>("backup_inspect", { path }),
	backupRestore: (path: string) => call<BackupInfo>("backup_restore", { path }),
	backupArkasDb: (path: string) => call<string>("backup_arkas_db", { path }),
	autoBackupGet: () => call<AutoBackup>("auto_backup_get"),
	autoBackupSet: (value: AutoBackup) => call<void>("auto_backup_set", { value })
};
