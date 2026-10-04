// Tipe data dari command Rust (lihat src-tauri/src). Nama field camelCase sesuai serde.

// ---- Nota / kwitansi ----
export interface NotaInfo {
	idKasNota: string
	noBukti: string | null
	noNota: string | null
	tanggalNota: string | null
	namaToko: string | null
	alamatToko: string | null
	noTelp: string | null
	npwp: string | null
	isBadanUsaha: boolean
	total: number | null
	isSiplah: boolean
}

export interface NotaItem {
	id: string
	tanggal: string
	noBukti: string | null
	uraian: string
	kodeRekening: string | null
	kodeKegiatan: string | null
	volume: number | null
	satuan: string | null
	hargaSatuan: number | null
	nominal: number
}

export interface NotaTax {
	id: string
	jenis: string | null
	uraian: string
	nominal: number
	tanggalSetor: string | null
}

export interface PrintOverride {
	noNota?: string | null
	tanggalNota?: string | null
	tanggalBayar?: string | null
	uraian?: string | null
	keperluan?: string | null
}

export interface NotaGroup {
	key: string
	kind: "nota" | "bukti" | "merge"
	tanggal: string
	noBukti: string
	uraian: string
	fundName: string
	nota: NotaInfo | null
	items: NotaItem[]
	/** Pajak input manual (dicetak). */
	taxes: NotaTax[]
	/** Pajak tercatat di ARKAS, mis. PPN SIPLah (informasi saja). */
	arkasTaxes: NotaTax[]
	isSiplah: boolean
	total: number
	totalPajak: number
	diterima: number
	mergeId: string | null
	printedBukti: string | null
	printedA2: string | null
	printedNota: string | null
	override: PrintOverride | null
	fileCount: number
}

export interface NotaFile {
	id: string
	refId: string
	fileName: string
	mime: string
	createdAt: string
}

// ---- Laporan ----
export interface Total {
	kode: string
	nama: string
	total: number
}

export interface MonthFlow {
	month: number
	penerimaan: number
	belanja: number
	saldoAkhirBank: number
	saldoAkhirTunai: number
}

export interface ReceiptLine {
	tanggal: string
	uraian: string
	jenis: string
	fundName: string
	nominal: number
}

export interface PeriodSummary {
	year: number
	start: number
	end: number
	fund: number | null
	saldoAwalBank: number
	saldoAwalTunai: number
	saldoAkhirBank: number
	saldoAkhirTunai: number
	penerimaan: ReceiptLine[]
	penerimaanDana: number
	bunga: number
	totalPenerimaan: number
	belanjaKelompok: Total[]
	belanjaRekening: Total[]
	belanjaProgram: Total[]
	belanjaOperasi: number
	belanjaModal: number
	totalBelanja: number
	pajakBunga: number
	pengembalian: number
	pajakDipungut: number
	pajakDisetor: number
	perBulan: MonthFlow[]
	warnings: string[]
}

export interface RekonBankMonth {
	month: number
	saldoBank: number
	saldoTunai: number
	penerimaan: number
	belanja: number
	rekeningKoran: number | null
	selisih: number | null
	keterangan: string | null
}

export interface RekonBank {
	year: number
	fund: number | null
	saldoAwalBank: number
	months: RekonBankMonth[]
}

export interface Pecahan {
	kertas: Record<string, number>
	logam: Record<string, number>
}

export interface CashRegister {
	pecahan: Pecahan
	catatan: string | null
}

export interface RegisterKas {
	year: number
	month: number
	saved: CashRegister | null
	totalFisik: number
	saldoTunai: number
	saldoBank: number
	penerimaanSd: number
	pengeluaranSd: number
	saldoBuku: number
	pajakBelumDisetor: number
}

export interface ManualTax {
	id: string
	tahun: number
	sumberDana: number
	tanggal: string
	noBukti: string | null
	uraian: string
	jenisPajak: string
	arah: "pungut" | "setor"
	nominal: number
	keterangan: string | null
	refId?: string | null
}

// ---- RKAS ----
export interface AnggaranInfo {
	idAnggaran: string
	fundId: number
	fundName: string
	isRevisi: number
	isApprove: boolean
	isPengesahan: boolean
	alasanPenolakan: string | null
	jumlah: number
	tanggalPengesahan: string | null
}

export interface RkasItem {
	idRapbs: string
	fundId: number
	fundName: string
	kodeKegiatan: string | null
	kodeRekening: string | null
	uraian: string
	satuan: string | null
	volume: number
	hargaSatuan: number
	jumlah: number
	bulan: number[]
	volumeBulan: number[]
}

export interface KertasKerja {
	year: number
	anggaran: AnggaranInfo[]
	items: RkasItem[]
	kodeNames: Record<string, string>
	rekeningNames: Record<string, string>
}

export type RealisasiStatus = "lunas" | "sebagian" | "belum" | "melampaui" | "belum_jatuh_tempo";

export interface RealisasiItem {
	idRapbs: string
	fundName: string
	kodeKegiatan: string | null
	namaKegiatan: string | null
	kodeRekening: string | null
	namaRekening: string | null
	uraian: string
	satuan: string | null
	volume: number
	hargaSatuan: number
	pagu: number
	rencana: number[]
	realisasi: number[]
	totalRealisasi: number
	rencanaSd: number
	realisasiSd: number
	sisa: number
	persen: number
	status: RealisasiStatus
	tertunda: number
}

export interface OutsideItem {
	id: string
	tanggal: string
	uraian: string
	kodeRekening: string | null
	nominal: number
}

export interface Realisasi {
	year: number
	upto: number
	fund: number | null
	items: RealisasiItem[]
	totalPagu: number
	totalRealisasi: number
	rencanaBulan: number[]
	realisasiBulan: number[]
	persen: number
	totalTertunda: number
	luarRkas: OutsideItem[]
}

export interface StandarHarga {
	kodeRekening: string | null
	namaBarang: string
	satuan: string | null
	harga: number | null
	batasBawah: number | null
	batasAtas: number | null
	tahun: number | null
}

export interface Dashboard {
	year: number
	upto: number
	summary: PeriodSummary
	pagu: number
	totalRealisasi: number
	persen: number
	totalTertunda: number
	itemBelum: number
	itemMelampaui: number
	rencanaBulan: number[]
	realisasiBulan: number[]
	anggaran: AnggaranInfo[]
	bulanDitutup: number[]
	pajakBelumDisetor: number
}

export interface Draft {
	id: string
	tahun: number
	sumberDana: number
	nama: string
	jenis: "awal" | "perubahan" | "pergeseran"
	dari: string | null
	total: number
	itemCount: number
	updatedAt: string
}

export interface DraftItem {
	id: string
	urutan: number
	kodeKegiatan: string | null
	kodeRekening: string | null
	uraian: string
	satuan: string | null
	hargaSatuan: number
	volumeBulan: number[]
	sumberIdRapbs: string | null
}

export interface DraftDetail {
	draft: Draft
	items: DraftItem[]
	parent: Draft | null
	kodeNames: Record<string, string>
	rekeningNames: Record<string, string>
	fundName: string
}

// ---- Backup ----
export interface BackupManifest {
	app: string
	format: number
	version: string
	createdAt: string
	schemaVersion: number
	npsn: string | null
	sekolah: string | null
	notaFiles: number
}

export interface BackupInfo {
	path: string
	size: number
	manifest: BackupManifest
}

export interface AutoBackup {
	enabled: boolean
	folder: string
	keep: number
	last: string | null
}

// ---- Cetak ----
export interface Penandatangan {
	jabatan: string
	nama?: string
	nip?: string
	/** Baris di atas jabatan, mis. "Mengetahui," atau "Majalengka, 30 September 2026". */
	atas?: string
}
