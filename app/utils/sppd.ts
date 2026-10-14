// Generator SPPD (perjalanan dinas) dari belanja transport di BKU: Surat Tugas, SPD, Kuitansi,
// Rincian Biaya, Daftar Pengeluaran Riil, dan Laporan Pelaksanaan Tugas.

/** Biaya perjalanan dinas per orang. */
export interface BiayaSppd {
	harian: number
	transport: number
	penginapan: number
	representatif: number
	sewa: number
	tol: number
}

/** Urutan baris di Rincian Perjalanan Dinas; `riil` = ikut di Daftar Pengeluaran Riil. */
export const BIAYA_SPPD: { key: keyof BiayaSppd, label: string, riil: boolean }[] = [
	{ key: "harian", label: "Uang Harian", riil: false },
	{ key: "transport", label: "Biaya Transport", riil: true },
	{ key: "penginapan", label: "Biaya Penginapan", riil: true },
	{ key: "representatif", label: "Uang Representatif", riil: false },
	{ key: "sewa", label: "Sewa Kendaraan dalam Kota", riil: true },
	{ key: "tol", label: "E-Tol", riil: true }
];

export const totalBiaya = (b: BiayaSppd, hanyaRiil = false) =>
	BIAYA_SPPD.reduce((t, x) => (hanyaRiil && !x.riil ? t : t + (Number(b[x.key]) || 0)), 0);

export type JenisDokumenSppd = "tugas" | "spd" | "kuitansi" | "rincian" | "riil" | "laporan";

export const DOKUMEN_SPPD: { value: JenisDokumenSppd, label: string }[] = [
	{ value: "tugas", label: "Surat Tugas" },
	{ value: "spd", label: "SPD (depan & belakang)" },
	{ value: "kuitansi", label: "Kuitansi" },
	{ value: "rincian", label: "Rincian Biaya & SPPD Rampung" },
	{ value: "riil", label: "Daftar Pengeluaran Riil" },
	{ value: "laporan", label: "Laporan Pelaksanaan Tugas" }
];

export interface SppdData {
	/** Maksud perjalanan dinas, mis. "Mengikuti Rapat K3S". */
	maksud: string
	tujuan: string
	berangkatDari: string
	/** ISO */
	tanggalBerangkat: string
	tanggalKembali: string
	/** "07:00" */
	jamBerangkat: string
	angkutan: string
	/** "dalam daerah" atau "luar daerah". */
	daerah: string
	// Surat tugas
	nomorSuratTugas: string
	tanggalSurat: string
	dasar: string
	// SPD
	nomorSpd: string
	tingkatBiaya: string
	instansi: string
	kodeRekening: string
	// Kuitansi
	nomorKuitansi: string
	tanggalBayar: string
	sumberDana: string
	/** Id guru yang melaksanakan perjalanan dinas. */
	pelaksana: string[]
	/** Biaya per orang (sama untuk semua pelaksana). */
	biaya: BiayaSppd
	/** Rangkaian acara untuk laporan, satu per baris. */
	acara: string
}

/** SPPD tersimpan per tahun anggaran: kunci = key nota di BKU. */
export const settingSppd = (year: number) => `sppd.${year}`;

const KATA_TRANSPORT = /transport|perjalanan dinas|sppd|\bspd\b|uang harian|akomodasi|penginapan/i;

/** Belanja yang kemungkinan transport / perjalanan dinas, dilihat dari uraian dan nama rekening. */
export const isTransport = (g: NotaGroup, rekeningNames: Record<string, string>) =>
	[g.uraian, ...g.items.flatMap((i) => [i.uraian, i.kodeRekening ? rekeningNames[i.kodeRekening] : ""])]
		.some((v) => !!v && KATA_TRANSPORT.test(v));

const AWALAN_BELANJA = /^(?:(?:belanja|biaya|bayar|pembayaran|uang|honor)\s+)*(?:(?:transport(?:asi)?|perjalanan dinas|konsumsi|makan(?:an)?(?: dan)? minum(?:an)?|snack)\s+)*/i;

/** "Biaya transport pengawas TKA" -> "Pengawas TKA". */
export const maksudDariUraian = (uraian: string) => {
	const sisa = uraian.trim().replace(AWALAN_BELANJA, "").trim() || uraian.trim();
	return sisa.charAt(0).toUpperCase() + sisa.slice(1);
};

/** Lama perjalanan dalam hari, tanggal berangkat dan kembali ikut dihitung. */
export const lamaHari = (berangkat: string, kembali: string) => {
	const a = Date.parse(berangkat.slice(0, 10));
	const b = Date.parse(kembali.slice(0, 10));
	return Number.isNaN(a) || Number.isNaN(b) ? 1 : Math.max(1, Math.round((b - a) / 86400000) + 1);
};

/** "1 (satu) hari". */
export const lamaTeks = (d: SppdData) => {
	const n = lamaHari(d.tanggalBerangkat, d.tanggalKembali);
	return `${n} (${terbilang(n)}) hari`;
};

/** "12 Mei 2026" atau "12 s.d. 13 Mei 2026" bila lebih dari sehari. */
export const rentangTanggal = (d: SppdData) =>
	d.tanggalKembali && d.tanggalKembali !== d.tanggalBerangkat
		? `${tanggalPanjang(d.tanggalBerangkat)} s.d. ${tanggalPanjang(d.tanggalKembali)}`
		: tanggalPanjang(d.tanggalBerangkat);

/** Jabatan pelaksana di dokumen SPPD. */
export const jabatanSppd = (g: Guru) => g.tugas.trim() || "Guru";

/** Isian awal SPPD dari satu belanja di BKU. */
export const sppdBawaan = (g: NotaGroup, ctx: { sekolah?: string | null }): SppdData => {
	const tanggal = g.tanggal.slice(0, 10);
	const item = g.items[0];
	const sekolah = namaSekolahRapi(ctx.sekolah);
	return {
		maksud: maksudDariUraian(g.uraian),
		tujuan: "",
		berangkatDari: sekolah,
		tanggalBerangkat: tanggal,
		tanggalKembali: tanggal,
		jamBerangkat: "07:00",
		angkutan: "Kendaraan umum",
		daerah: "dalam daerah",
		nomorSuratTugas: "",
		tanggalSurat: geserTanggal(tanggal, -1),
		dasar: "",
		nomorSpd: "",
		tingkatBiaya: "",
		instansi: sekolah,
		kodeRekening: item?.kodeRekening ?? "",
		nomorKuitansi: "",
		tanggalBayar: tanggal,
		sumberDana: g.fundName,
		pelaksana: [],
		// Bawaan: harga satuan di BKU dianggap transport per orang.
		biaya: { harian: 0, transport: item?.hargaSatuan || g.total, penginapan: 0, representatif: 0, sewa: 0, tol: 0 },
		acara: ["Pembukaan", "Sambutan", "Acara inti", "Penutup"].join("\n")
	};
};
