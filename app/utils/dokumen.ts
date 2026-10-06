// Mesin template dokumen (SP, Nota, Kwitansi, BA, dll.).
// Template = daftar blok; teks boleh berisi isian {nama_isian} yang diisi otomatis dari BKU,
// pengaturan, dan data penyedia.

export type Kertas = "A4" | "F4" | "A5";
export type Rata = "left" | "center" | "right" | "justify";
export type JenisDokumen = "sp" | "nota" | "kwitansi" | "ba" | "bukti" | "lainnya";

export const JENIS_DOKUMEN: { value: JenisDokumen, label: string }[] = [
	{ value: "sp", label: "Surat Pesanan" },
	{ value: "nota", label: "Nota" },
	{ value: "kwitansi", label: "Kwitansi" },
	{ value: "ba", label: "BA Serah Terima" },
	{ value: "bukti", label: "Bukti Pengeluaran" },
	{ value: "lainnya", label: "Lainnya" }
];

export interface KolomTabel {
	kunci: "no" | "nama" | "volume" | "satuan" | "volume_satuan" | "harga" | "jumlah" | "dipesan" | "diterima" | "rusak" | "sesuai" | "kode_rekening"
	judul: string
	lebar: number
}

export interface Penandatangan2 {
	atas: string
	jabatan: string
	nama: string
	nip: string
}

export type Blok
	= | { id: string, type: "kop", sumber: "sekolah" | "toko" | "teks", baris: string, logo: "sekolah" | "toko" | "custom" | "none", logoData: string, garis: boolean }
		| { id: string, type: "judul", teks: string, ukuran: number, garisBawah: boolean, rata: Rata }
		| { id: string, type: "teks", isi: string, ukuran: number, rata: Rata, tebal: boolean }
		| { id: string, type: "isian", baris: { label: string, nilai: string }[], lebarLabel: number, indent: number }
		| { id: string, type: "tabel", kolom: KolomTabel[], minBaris: number, total: boolean, labelTotal: string, bersihkanNama: boolean, ukuran: number }
		| { id: string, type: "terbilang", label: string }
		| { id: string, type: "kotakTotal", label: string }
		| { id: string, type: "ttd", kolom: Penandatangan2[], tinggi: number, rata: "rata" | "kanan" | "tengah" }
		| { id: string, type: "spasi", tinggi: number }
		| { id: string, type: "garis" }
		| {
			id: string
			type: "kwitansi"
			/** Panel kiri berisi kop toko yang ditulis tegak. */
			panelToko: boolean
			motto: string
			/** Warna sorotan untuk terbilang & jumlah. */
			warna: string
			/** Visum kepala sekolah & bendahara: di bawah kotak, di dalam kotak, atau tidak ada. */
			visum: "bawah" | "dalam" | "tidak"
			tinggiTtd: number
			/** Tinggi kotak kwitansi (mm). */
			tinggiKotak?: number
			/** Jarak kotak ke visum di bawahnya (mm), ruang untuk cap. */
			jarakVisum?: number
			/** Lebar seluruh kwitansi (mm, kotak + visum); 0 = selebar halaman. */
			lebarKotak?: number
			/** Lebar panel kop toko (mm); 0 = menyesuaikan isi. */
			lebarPanel?: number
		};

export type JenisBlok = Blok["type"];

/** Omit yang bekerja per anggota union (Omit biasa menggabungkan semua tipe blok). */
type TanpaId<T> = T extends unknown ? Omit<T, "id"> : never;
export type BlokBaru = TanpaId<Blok>;

// ---------------------------------------------------------------- kanvas

/** Elemen pada template kanvas; posisi & ukuran dalam mm dari pojok kiri atas kertas. */
export interface ElemenKanvas {
	id: string
	type: "teks" | "gambar" | "garis" | "kotak" | "tabel"
	x: number
	y: number
	w: number
	h: number
	/** teks: isi, boleh berisi {isian}. */
	teks?: string
	/** Ukuran huruf (pt). */
	ukuran?: number
	tebal?: boolean
	miring?: boolean
	rata?: "left" | "center" | "right"
	/** Perataan tegak di dalam kotak. */
	tegak?: "top" | "middle" | "bottom"
	font?: "serif" | "sans"
	warna?: string
	/** gambar: data URL. */
	src?: string
	/** garis/kotak: tebal garis (pt). */
	tebalGaris?: number
	/** tabel: kolom (lebar mm), tinggi baris (mm), garis sel. */
	kolom?: { kunci: KolomTabel["kunci"], lebar: number }[]
	tinggiBaris?: number
	garisSel?: boolean
	bersihkanNama?: boolean
}

/** Gambar latar (foto/scan kwitansi kosong). Rasio dijaga dari lebar. */
export interface LatarKanvas {
	src: string
	x: number
	y: number
	w: number
	/** Tinggi / lebar gambar asli. */
	rasio: number
	opasitas: number
	/** false = latar hanya tampil di layar, tidak ikut dicetak (untuk kertas yang sudah bercetak). */
	cetak: boolean
}

export interface Kanvas {
	latar: LatarKanvas | null
	elemen: ElemenKanvas[]
}

export interface TemplateData {
	/** "blok" (bawaan) = blok tersusun; "kanvas" = elemen bebas di atas kertas/gambar latar. */
	mode?: "blok" | "kanvas"
	kanvas?: Kanvas
	kertas: Kertas
	orientasi: "portrait" | "landscape"
	/** Geser tanggal dokumen dari tanggal nota (mis. -3 untuk SP). */
	geserHari: number
	/** Format nomor, mis. "{urut}/SP/{bulan_romawi}/{tahun}". */
	nomorFormat: string
	ukuranHuruf: number
	margin: number
	/** Margin atas khusus (mm), mis. ruang jilid. Kosong = sama dengan margin. */
	marginAtas?: number
	/** Jadwal pekerjaan acak (tetap per transaksi) dihitung mundur dari tanggal BKU. */
	jadwal?: { mulaiMin: number, mulaiMax: number, selesaiMax: number }
	blocks: Blok[]
}

export const JADWAL_BAWAAN = { mulaiMin: 5, mulaiMax: 10, selesaiMax: 3 };

export interface DocTemplate {
	id: string
	nama: string
	jenis: JenisDokumen
	tokoMatch: string | null
	data: TemplateData
	urutan: number
	updatedAt: string
}

export interface PenyediaData {
	penanggungJawab: string
	jabatan: string
	alamat: string
	kota: string
	telp: string
	npwp: string
	/** Baris kop toko, satu baris per baris teks. */
	kop: string
	layanan: string
	logo: string
}

export interface Penyedia {
	nama: string
	data: PenyediaData
	updatedAt: string
}

export const penyediaKosong = (): PenyediaData => ({
	penanggungJawab: "",
	jabatan: "",
	alamat: "",
	kota: "",
	telp: "",
	npwp: "",
	kop: "",
	layanan: "",
	logo: ""
});

const uid = () => Math.random().toString(36).slice(2, 10);

export const KOLOM_LABEL: Record<KolomTabel["kunci"], string> = {
	no: "No",
	nama: "Nama barang",
	volume: "Volume",
	satuan: "Satuan",
	volume_satuan: "Banyaknya (volume + satuan)",
	harga: "Harga satuan",
	jumlah: "Jumlah",
	dipesan: "Jumlah dipesan",
	diterima: "Jumlah diterima",
	rusak: "Jumlah rusak",
	sesuai: "Jumlah sesuai",
	kode_rekening: "Kode rekening"
};

export const BLOK_LABEL: Record<JenisBlok, string> = {
	kop: "Kop",
	judul: "Judul",
	teks: "Paragraf",
	isian: "Daftar isian (Label : Nilai)",
	tabel: "Tabel barang",
	terbilang: "Terbilang",
	kotakTotal: "Kotak jumlah",
	ttd: "Tanda tangan",
	spasi: "Spasi",
	garis: "Garis",
	kwitansi: "Kwitansi (kotak lengkap + visum)"
};

export const blokBaru = (type: JenisBlok): Blok => {
	const id = uid();
	switch (type) {
		case "kop": return { id, type, sumber: "sekolah", baris: "", logo: "none", logoData: "", garis: true };
		case "judul": return { id, type, teks: "JUDUL DOKUMEN", ukuran: 13, garisBawah: true, rata: "center" };
		case "teks": return { id, type, isi: "Tulis teks di sini. Gunakan isian seperti {nama_toko}.", ukuran: 0, rata: "left", tebal: false };
		case "isian": return { id, type, baris: [{ label: "Nama", nilai: "{nama_toko}" }], lebarLabel: 40, indent: 0 };
		case "tabel": return {
			id,
			type,
			kolom: [
				{ kunci: "no", judul: "No", lebar: 6 },
				{ kunci: "nama", judul: "Nama Barang", lebar: 0 },
				{ kunci: "volume", judul: "Volume", lebar: 10 },
				{ kunci: "satuan", judul: "Satuan", lebar: 10 },
				{ kunci: "harga", judul: "Harga Satuan", lebar: 16 },
				{ kunci: "jumlah", judul: "Jumlah", lebar: 17 }
			],
			minBaris: 0,
			total: true,
			labelTotal: "JUMLAH",
			bersihkanNama: true,
			ukuran: 10
		};
		case "terbilang": return { id, type, label: "Terbilang" };
		case "kotakTotal": return { id, type, label: "Jumlah Rp" };
		case "ttd": return { id, type, kolom: [{ atas: "{kota}, {tanggal_dokumen}", jabatan: "Bendahara", nama: "{bendahara}", nip: "{nip_bendahara}" }], tinggi: 22, rata: "rata" };
		case "spasi": return { id, type, tinggi: 5 };
		case "garis": return { id, type };
		case "kwitansi": return { id, type, panelToko: true, motto: "Melayani Belanja TUNAI & Non TUNAI", warna: "#f6d9e0", visum: "bawah", tinggiTtd: 14, tinggiKotak: 75, lebarPanel: 0, lebarKotak: 245, jarakVisum: 10 };
	}
};

/** Ukuran kertas (mm), tegak. */
export const UKURAN_KERTAS: Record<Kertas, [number, number]> = { A4: [210, 297], F4: [210, 330], A5: [148, 210] };

export const ukuranHalaman = (data: TemplateData): [number, number] => {
	const [w, h] = UKURAN_KERTAS[data.kertas] ?? UKURAN_KERTAS.A4;
	return data.orientasi === "landscape" ? [h, w] : [w, h];
};

export const ELEMEN_LABEL: Record<ElemenKanvas["type"], string> = {
	teks: "Teks",
	gambar: "Gambar",
	garis: "Garis",
	kotak: "Kotak",
	tabel: "Tabel barang"
};

export const elemenBaru = (type: ElemenKanvas["type"], x = 20, y = 20, teks?: string): ElemenKanvas => {
	const id = uid();
	switch (type) {
		case "teks": return { id, type, x, y, w: 60, h: 8, teks: teks ?? "Teks baru", ukuran: 11, tebal: false, miring: false, rata: "left", tegak: "middle", font: "serif", warna: "#000000" };
		case "gambar": return { id, type, x, y, w: 25, h: 25, src: "" };
		case "garis": return { id, type, x, y, w: 60, h: 0, tebalGaris: 1, warna: "#000000" };
		case "kotak": return { id, type, x, y, w: 60, h: 30, tebalGaris: 1, warna: "#000000" };
		case "tabel": return {
			id,
			type,
			x,
			y,
			w: 170,
			h: 60,
			ukuran: 10,
			font: "serif",
			tinggiBaris: 6,
			garisSel: false,
			bersihkanNama: true,
			kolom: [
				{ kunci: "no", lebar: 10 },
				{ kunci: "nama", lebar: 80 },
				{ kunci: "volume_satuan", lebar: 25 },
				{ kunci: "harga", lebar: 27 },
				{ kunci: "jumlah", lebar: 28 }
			]
		};
	}
};

export const kanvasKosong = (): Kanvas => ({ latar: null, elemen: [] });

// ---------------------------------------------------------------- isian

export interface DocContext {
	g: NotaGroup
	sekolah: SchoolInfo | null
	pengaturan: Pengaturan | null
	penyedia: PenyediaData | null
	/** Nomor urut nota dalam tahun (1, 2, ...). */
	urut: number
	template: TemplateData
}

const HARI = ["Minggu", "Senin", "Selasa", "Rabu", "Kamis", "Jumat", "Sabtu"];
const ROMAWI = ["I", "II", "III", "IV", "V", "VI", "VII", "VIII", "IX", "X", "XI", "XII"];

const toDate = (iso: string) => {
	const [y, m, d] = iso.slice(0, 10).split("-").map(Number);
	return new Date(y ?? 1970, (m ?? 1) - 1, d ?? 1);
};
const toIso = (d: Date) => `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}-${String(d.getDate()).padStart(2, "0")}`;
export const geserTanggal = (iso: string, hari: number) => {
	const d = toDate(iso);
	d.setDate(d.getDate() + hari);
	return toIso(d);
};

/** Angka semu-acak yang selalu sama untuk teks yang sama (FNV-1a). */
const hashTeks = (text: string) => {
	let h = 0x811C9DC5;
	for (let i = 0; i < text.length; i++) {
		h ^= text.charCodeAt(i);
		h = Math.imul(h, 0x01000193) >>> 0;
	}
	return h >>> 0;
};

/**
 * Jadwal pekerjaan untuk SP/BA: mulai = tanggal BKU mundur acak `mulaiMin`..`mulaiMax` hari,
 * selesai = tanggal BKU mundur acak 0..`selesaiMax` hari. Acak tetapi tetap untuk transaksi yang sama.
 */
export const jadwalPekerjaan = (tanggalBku: string, kunci: string, jadwal = JADWAL_BAWAAN) => {
	const h = hashTeks(kunci);
	const lo = Math.max(0, Math.min(jadwal.mulaiMin, jadwal.mulaiMax));
	const hi = Math.max(lo, jadwal.mulaiMax);
	const mulaiMundur = lo + (h % (hi - lo + 1));
	const selesaiMundur = Math.min(Math.floor(h / 97) % (Math.max(0, jadwal.selesaiMax) + 1), mulaiMundur);
	return { mulai: geserTanggal(tanggalBku, -mulaiMundur), selesai: geserTanggal(tanggalBku, -selesaiMundur) };
};

/** "Senin tanggal empat bulan September tahun dua ribu dua puluh enam". */
export const kalimatTanggal = (iso: string) => {
	const d = toDate(iso);
	return `${HARI[d.getDay()]} tanggal ${terbilang(d.getDate())} bulan ${BULAN[d.getMonth()]} tahun ${terbilang(d.getFullYear())}`;
};

const jenisBelanja = (kode: string | null | undefined) => {
	if (!kode) return "Belanja Barang/Jasa";
	if (kode.startsWith("5.2")) return "Belanja Modal";
	if (kode.startsWith("5.1.02.02")) return "Belanja Jasa";
	if (kode.startsWith("5.1.02.03")) return "Belanja Pemeliharaan";
	if (kode.startsWith("5.1.02.04")) return "Belanja Perjalanan Dinas";
	return "Belanja Barang";
};

const VAR_RE = /\{([a-z_]+)\}/g;

export const fillVars = (text: string, vars: Record<string, string>) =>
	text.replace(VAR_RE, (m, k: string) => (k in vars ? vars[k] ?? "" : m));

/** Semua isian yang tersedia untuk teks template, beserta nilainya untuk satu nota. */
export const buildVars = (ctx: DocContext): Record<string, string> => {
	const { g, sekolah, pengaturan, penyedia, template } = ctx;
	const p = pengaturan?.pejabat;
	const tanggalNota = g.override?.tanggalNota || g.nota?.tanggalNota || g.tanggal;
	const tanggalBayar = g.override?.tanggalBayar || g.tanggal;
	const tanggalDok = geserTanggal(tanggalNota, template.geserHari || 0);
	const dok = toDate(tanggalDok);
	const jadwal = jadwalPekerjaan(g.tanggal, g.key, { ...JADWAL_BAWAAN, ...template.jadwal });
	const kota = pengaturan?.cetak.kota || "";
	const alamatSekolah = [pengaturan?.kop.alamat || sekolah?.alamat, sekolah?.kecamatan, sekolah?.kabupaten].filter(Boolean).join(", ");
	const vars: Record<string, string> = {
		nama_sekolah: sekolah?.nama ?? "",
		npsn: sekolah?.npsn ?? "",
		alamat_sekolah: alamatSekolah,
		kecamatan: sekolah?.kecamatan ?? "",
		kabupaten: sekolah?.kabupaten ?? "",
		provinsi: sekolah?.provinsi ?? "",
		kota,
		pemerintah: pengaturan?.kop.pemerintah ?? "",
		dinas: pengaturan?.kop.dinas ?? "",
		kepala_sekolah: p?.kepalaSekolah ?? "",
		nip_kepala_sekolah: p?.nipKepalaSekolah ?? "",
		bendahara: p?.bendahara ?? "",
		nip_bendahara: p?.nipBendahara ?? "",
		pemegang_barang: p?.pemegangBarang ?? "",
		nip_pemegang_barang: p?.nipPemegangBarang ?? "",
		nama_toko: g.nota?.namaToko ?? "",
		alamat_toko: penyedia?.alamat || g.nota?.alamatToko || "",
		kota_toko: penyedia?.kota || kota,
		telp_toko: penyedia?.telp || g.nota?.noTelp || "",
		npwp_toko: penyedia?.npwp || g.nota?.npwp || "",
		penanggung_jawab: penyedia?.penanggungJawab || g.nota?.namaToko || "",
		jabatan_penanggung_jawab: penyedia?.jabatan || "Pemilik",
		layanan_toko: penyedia?.layanan ?? "",
		no_bukti: g.noBukti,
		no_nota: g.override?.noNota || g.nota?.noNota || "",
		urut: String(ctx.urut).padStart(3, "0"),
		tanggal_nota: tanggalPanjang(tanggalNota),
		tanggal_bayar: tanggalPanjang(tanggalBayar),
		tanggal_dokumen: tanggalPanjang(tanggalDok),
		hari_dokumen: HARI[dok.getDay()] ?? "",
		kalimat_tanggal: kalimatTanggal(tanggalDok),
		tanggal_bku: tanggalPanjang(g.tanggal),
		tanggal_mulai: tanggalPanjang(jadwal.mulai),
		tanggal_selesai: tanggalPanjang(jadwal.selesai),
		kalimat_tanggal_mulai: kalimatTanggal(jadwal.mulai),
		kalimat_tanggal_selesai: kalimatTanggal(jadwal.selesai),
		bulan: BULAN[dok.getMonth()] ?? "",
		bulan_romawi: ROMAWI[dok.getMonth()] ?? "",
		tahun: String(dok.getFullYear()),
		semester: dok.getMonth() < 6 ? "I" : "II",
		total: angka(g.total),
		terbilang: terbilangRupiah(g.total),
		pajak: g.totalPajak ? angka(g.totalPajak) : "-",
		dibayar: angka(g.diterima),
		untuk_pembayaran: g.override?.keperluan || g.override?.uraian || g.uraian,
		uraian: g.uraian,
		sumber_dana: g.fundName,
		jenis_belanja: jenisBelanja(g.items[0]?.kodeRekening),
		jumlah_item: String(g.items.length)
	};
	vars.nomor = fillVars(template.nomorFormat || "{no_bukti}", vars);
	return vars;
};

/** Daftar isian untuk bantuan di editor (nama + contoh keterangan). */
export const DAFTAR_ISIAN: { kunci: string, ket: string }[] = [
	{ kunci: "nama_sekolah", ket: "Nama sekolah" },
	{ kunci: "alamat_sekolah", ket: "Alamat sekolah" },
	{ kunci: "npsn", ket: "NPSN" },
	{ kunci: "kota", ket: "Kota tanda tangan (Pengaturan)" },
	{ kunci: "kepala_sekolah", ket: "Kepala sekolah" },
	{ kunci: "nip_kepala_sekolah", ket: "NIP kepala sekolah" },
	{ kunci: "bendahara", ket: "Bendahara" },
	{ kunci: "nip_bendahara", ket: "NIP bendahara" },
	{ kunci: "pemegang_barang", ket: "Pemegang barang" },
	{ kunci: "nama_toko", ket: "Nama toko (ARKAS)" },
	{ kunci: "alamat_toko", ket: "Alamat toko" },
	{ kunci: "kota_toko", ket: "Kota toko (Data Penyedia)" },
	{ kunci: "penanggung_jawab", ket: "Penanggung jawab toko" },
	{ kunci: "jabatan_penanggung_jawab", ket: "Jabatan penanggung jawab" },
	{ kunci: "layanan_toko", ket: "Daftar layanan toko" },
	{ kunci: "nomor", ket: "Nomor dokumen (format template)" },
	{ kunci: "urut", ket: "Nomor urut nota dalam tahun (001)" },
	{ kunci: "no_bukti", ket: "Nomor bukti BKU" },
	{ kunci: "no_nota", ket: "Nomor nota/invoice" },
	{ kunci: "tanggal_nota", ket: "Tanggal nota" },
	{ kunci: "tanggal_bayar", ket: "Tanggal bayar" },
	{ kunci: "tanggal_dokumen", ket: "Tanggal dokumen (nota ± geser hari)" },
	{ kunci: "kalimat_tanggal", ket: "Senin tanggal empat bulan ... tahun ..." },
	{ kunci: "tanggal_bku", ket: "Tanggal transaksi di BKU" },
	{ kunci: "tanggal_mulai", ket: "Mulai pekerjaan (BKU mundur acak 5-10 hari)" },
	{ kunci: "tanggal_selesai", ket: "Selesai pekerjaan (maks. 3 hari sebelum BKU)" },
	{ kunci: "kalimat_tanggal_mulai", ket: "Kalimat hari-tanggal mulai" },
	{ kunci: "kalimat_tanggal_selesai", ket: "Kalimat hari-tanggal selesai (untuk BA)" },
	{ kunci: "bulan_romawi", ket: "Bulan romawi (IX)" },
	{ kunci: "tahun", ket: "Tahun" },
	{ kunci: "semester", ket: "Semester (I/II)" },
	{ kunci: "total", ket: "Jumlah belanja" },
	{ kunci: "terbilang", ket: "Terbilang jumlah" },
	{ kunci: "pajak", ket: "Pajak (input manual)" },
	{ kunci: "dibayar", ket: "Dibayarkan ke penyedia" },
	{ kunci: "untuk_pembayaran", ket: "Untuk pembayaran / keperluan" },
	{ kunci: "jenis_belanja", ket: "Belanja Barang/Jasa/Modal" },
	{ kunci: "sumber_dana", ket: "Sumber dana" }
];

// ---------------------------------------------------------------- template awal

const tpl = (nama: string, jenis: JenisDokumen, data: Omit<TemplateData, "blocks"> & { blocks: BlokBaru[] }): Omit<DocTemplate, "id" | "updatedAt" | "urutan"> => ({
	nama,
	jenis,
	tokoMatch: null,
	data: { ...data, blocks: data.blocks.map((b) => ({ ...b, id: uid() }) as Blok) }
});

const kopSekolah: BlokBaru = { type: "kop", sumber: "sekolah", baris: "", logo: "none", logoData: "", garis: true };
const kopToko: BlokBaru = { type: "kop", sumber: "toko", baris: "", logo: "toko", logoData: "", garis: true };

/** Template awal, meniru file "KWITASNI KPR BARU.xlsx" (sheet SP BARANG, Nota, Kwitansi, BA Serah Terima). */
export const TEMPLATE_AWAL = () => [
	tpl("Surat Pesanan (SP) Barang", "sp", {
		kertas: "F4",
		orientasi: "portrait",
		geserHari: -3,
		nomorFormat: "421.2/{urut} SD/{bulan_romawi}/{tahun}",
		ukuranHuruf: 11,
		margin: 15,
		blocks: [
			kopSekolah,
			{ type: "judul", teks: "SURAT PESANAN (SP)", ukuran: 13, garisBawah: true, rata: "center" },
			{ type: "teks", isi: "Nomor : {nomor}", ukuran: 0, rata: "center", tebal: false },
			{ type: "spasi", tinggi: 3 },
			{ type: "teks", isi: "Paket pekerjaan :\n{jenis_belanja}\nPada kegiatan BOS Semester {semester} Tahun Anggaran {tahun}\n{nama_sekolah}", ukuran: 0, rata: "center", tebal: false },
			{ type: "spasi", tinggi: 3 },
			{ type: "teks", isi: "Yang bertanda tangan di bawah ini :", ukuran: 0, rata: "left", tebal: false },
			{ type: "isian", baris: [{ label: "Nama", nilai: "{kepala_sekolah}" }, { label: "NIP", nilai: "{nip_kepala_sekolah}" }, { label: "Jabatan", nilai: "Kepala Sekolah" }, { label: "Alamat", nilai: "{alamat_sekolah}" }], lebarLabel: 32, indent: 8 },
			{ type: "teks", isi: "dengan ini memerintahkan kepada penyedia barang :", ukuran: 0, rata: "left", tebal: false },
			{ type: "isian", baris: [{ label: "Nama Penyedia", nilai: "{penanggung_jawab} ({nama_toko})" }, { label: "Jabatan", nilai: "{jabatan_penanggung_jawab}" }, { label: "Alamat", nilai: "{alamat_toko}" }], lebarLabel: 32, indent: 8 },
			{ type: "teks", isi: "Selanjutnya disebut sebagai penyedia Barang / Jasa\n\nuntuk mengirimkan barang / jasa dengan mempertimbangkan ketentuan-ketentuan sebagai berikut :\n1. Rincian Barang :", ukuran: 0, rata: "left", tebal: false },
			{ type: "tabel", kolom: [{ kunci: "no", judul: "NO", lebar: 6 }, { kunci: "nama", judul: "NAMA BARANG", lebar: 0 }, { kunci: "satuan", judul: "Satuan", lebar: 11 }, { kunci: "volume", judul: "Volume", lebar: 10 }, { kunci: "harga", judul: "Harga Satuan", lebar: 15 }, { kunci: "jumlah", judul: "Jumlah", lebar: 17 }], minBaris: 0, total: true, labelTotal: "JUMLAH", bersihkanNama: true, ukuran: 10 },
			{ type: "teks", isi: "2. Tanggal mulai pekerjaan : {tanggal_mulai}\n3. Waktu penyelesaian : pekerjaan harus sudah selesai pada tanggal {tanggal_selesai}\n4. Alamat pengiriman : {nama_sekolah}, {alamat_sekolah}\n5. Ada sanksi denda apabila pihak penyedia barang terlambat dalam pengiriman, yaitu sebesar 1/1000 (satu per seribu) dari nilai pekerjaan atau bagian tertentu sebelum PPN sesuai standar ketentuan dan syarat umum pekerjaan.", ukuran: 0, rata: "left", tebal: false },
			{ type: "teks", isi: "Demikian Surat Pesanan Barang ini dibuat untuk dipergunakan sebagaimana mestinya.", ukuran: 0, rata: "left", tebal: false },
			{ type: "ttd", kolom: [{ atas: "Untuk dan atas nama\n{nama_toko}", jabatan: "Penyedia Barang", nama: "{penanggung_jawab}", nip: "" }, { atas: "{kota}, {tanggal_mulai}\nUntuk dan atas nama", jabatan: "Kepala {nama_sekolah}\nKuasa Pengguna Anggaran", nama: "{kepala_sekolah}", nip: "{nip_kepala_sekolah}" }], tinggi: 22, rata: "rata" }
		]
	}),
	tpl("Nota Toko", "nota", {
		kertas: "A5",
		orientasi: "portrait",
		geserHari: 0,
		nomorFormat: "{no_nota}",
		ukuranHuruf: 10,
		margin: 10,
		blocks: [
			kopToko,
			{ type: "teks", isi: "{layanan_toko}", ukuran: 8, rata: "left", tebal: false },
			{ type: "teks", isi: "{kota_toko}, {tanggal_nota}\nTuan / Toko : {nama_sekolah}", ukuran: 0, rata: "right", tebal: false },
			{ type: "teks", isi: "NOTA NO. : {nomor}", ukuran: 0, rata: "left", tebal: true },
			{ type: "tabel", kolom: [{ kunci: "volume_satuan", judul: "Banyaknya", lebar: 16 }, { kunci: "nama", judul: "Nama Barang", lebar: 0 }, { kunci: "harga", judul: "Harga Satuan", lebar: 18 }, { kunci: "jumlah", judul: "Jumlah", lebar: 20 }], minBaris: 8, total: true, labelTotal: "Jumlah Rp.", bersihkanNama: true, ukuran: 9 },
			{ type: "ttd", kolom: [{ atas: "", jabatan: "Tanda Terima", nama: "{bendahara}", nip: "" }, { atas: "", jabatan: "Hormat Kami", nama: "{penanggung_jawab}", nip: "" }], tinggi: 18, rata: "rata" }
		]
	}),
	tpl("Kwitansi", "kwitansi", {
		kertas: "A4",
		orientasi: "landscape",
		geserHari: 0,
		nomorFormat: "{no_bukti}",
		ukuranHuruf: 11,
		margin: 8,
		marginAtas: 30,
		blocks: [
			{ type: "kwitansi", panelToko: true, motto: "Melayani Belanja TUNAI & Non TUNAI", warna: "#f6d9e0", visum: "bawah", tinggiTtd: 14, tinggiKotak: 75, lebarPanel: 0, lebarKotak: 245, jarakVisum: 10 }
		]
	}),
	tpl("BA Serah Terima Barang", "ba", {
		kertas: "F4",
		orientasi: "portrait",
		geserHari: 0,
		nomorFormat: "{urut}/BA/{bulan_romawi}/{tahun}",
		ukuranHuruf: 11,
		margin: 15,
		blocks: [
			kopToko,
			{ type: "judul", teks: "BERITA ACARA SERAH TERIMA BARANG", ukuran: 13, garisBawah: true, rata: "center" },
			{ type: "teks", isi: "Nomor : {nomor}", ukuran: 0, rata: "center", tebal: false },
			{ type: "spasi", tinggi: 3 },
			{ type: "teks", isi: "Pada hari ini {kalimat_tanggal_selesai}, yang bertanda tangan di bawah ini :", ukuran: 0, rata: "justify", tebal: false },
			{ type: "isian", baris: [{ label: "I.  Nama", nilai: "{penanggung_jawab}" }, { label: "    Jabatan", nilai: "{jabatan_penanggung_jawab} {nama_toko}" }, { label: "    Alamat", nilai: "{alamat_toko}" }], lebarLabel: 32, indent: 0 },
			{ type: "teks", isi: "Selanjutnya disebut Pihak Kesatu (yang menyerahkan barang)", ukuran: 0, rata: "left", tebal: false },
			{ type: "isian", baris: [{ label: "II. Nama", nilai: "{bendahara}" }, { label: "    NIP", nilai: "{nip_bendahara}" }, { label: "    Jabatan", nilai: "Pemeriksa / penerima barang" }, { label: "    Alamat", nilai: "{alamat_sekolah}" }], lebarLabel: 32, indent: 0 },
			{ type: "teks", isi: "Selanjutnya disebut Pihak Kedua (yang menerima barang)\n\nPihak kesatu menyerahkan barang kepada pihak kedua dalam keadaan baik dan lengkap sesuai rincian sebagai berikut :", ukuran: 0, rata: "left", tebal: false },
			{ type: "tabel", kolom: [{ kunci: "no", judul: "NO", lebar: 6 }, { kunci: "nama", judul: "NAMA BARANG", lebar: 0 }, { kunci: "dipesan", judul: "Jumlah Dipesan", lebar: 10 }, { kunci: "harga", judul: "Harga Satuan", lebar: 13 }, { kunci: "diterima", judul: "Jumlah Diterima", lebar: 10 }, { kunci: "rusak", judul: "Jumlah Rusak", lebar: 9 }, { kunci: "sesuai", judul: "Jumlah Sesuai", lebar: 9 }, { kunci: "jumlah", judul: "Total", lebar: 15 }], minBaris: 0, total: true, labelTotal: "JUMLAH", bersihkanNama: true, ukuran: 9 },
			{ type: "teks", isi: "Demikian berita acara serah terima barang ini dibuat untuk dipergunakan sebagaimana mestinya.", ukuran: 0, rata: "left", tebal: false },
			{ type: "ttd", kolom: [{ atas: "Pihak kesatu,", jabatan: "{jabatan_penanggung_jawab} {nama_toko}", nama: "{penanggung_jawab}", nip: "" }, { atas: "Pihak kedua,", jabatan: "Pemeriksa / penerima barang", nama: "{bendahara}", nip: "{nip_bendahara}" }], tinggi: 20, rata: "rata" },
			{ type: "ttd", kolom: [{ atas: "Mengetahui,", jabatan: "Kepala {nama_sekolah}", nama: "{kepala_sekolah}", nip: "{nip_kepala_sekolah}" }], tinggi: 20, rata: "tengah" }
		]
	})
];

/** Contoh profil penyedia dari file Excel (KPRI-KPR Rajagaluh). */
export const PENYEDIA_CONTOH: Penyedia = {
	nama: "KPR RAJAGALUH",
	updatedAt: "",
	data: {
		penanggungJawab: "Drs. H. Ahmad Kholid, M.M",
		jabatan: "Bendahara",
		alamat: "Jl. Raya Utara No. 60 Rajagaluhlor - Rajagaluh",
		kota: "Rajagaluh",
		telp: "(0233) 510277",
		npwp: "",
		kop: "KOPERASI PEGAWAI REPUBLIK INDONESIA\nKOPERASI PENDIDIK RAJAGALUH\n( KPRI - KPR )\nAlamat Kantor : Jl Raya Utara No. 60 Telp. (0233) 510277\n(Sebelah Utara Terminal Bus / Depan Alfa Mart Rajagaluhlor - Rajagaluh)",
		layanan: "Menerima Segala Pesanan :\n- Barang, Printer - Alkes & Obat-obatan\n- Laptop / Net Book - Material Bangunan\n- Meubelair & Furniture - Alat Olahraga\n- Pesanan Catering (Nasi kotak & snack)\n\nToko Kertas & Percetakan\n- Sedia macam-macam kertas & ATK\n- Undangan Pernikahan/Khitanan, Nota, dll.\n- Pesanan Spanduk, brosur, Yassin, dll.\n- Foto copy penggandaan & penjilidan",
		logo: ""
	}
};

/** Template yang cocok untuk toko (kata kunci tokoMatch), lalu template umum. */
export const templateUntuk = (templates: DocTemplate[], jenis: JenisDokumen, namaToko: string | null | undefined) => {
	const toko = (namaToko ?? "").toLowerCase();
	const sejenis = templates.filter((t) => t.jenis === jenis);
	return sejenis.find((t) => t.tokoMatch && toko.includes(t.tokoMatch.toLowerCase()))
		?? sejenis.find((t) => !t.tokoMatch)
		?? null;
};

/** Nama barang tanpa awalan kategori ARKAS ("Alat Tulis Kantor-Spidol" -> "Spidol"). */
export const bersihkanNama = (uraian: string) => {
	const i = uraian.indexOf("-");
	return i > 0 && i < 40 ? uraian.slice(i + 1).trim() : uraian;
};

/** Isi satu sel tabel barang (dipakai template blok & kanvas). */
export const isiSelTabel = (k: KolomTabel["kunci"], it: NotaItem, i: number, bersih: boolean) => {
	const fmtVol = (v: number | null) => (v === null || v === undefined ? "" : v.toLocaleString("id-ID"));
	const harga = it.volume && it.volume > 0 ? Math.round(it.nominal / it.volume) : it.hargaSatuan;
	switch (k) {
		case "no": return String(i + 1);
		case "nama": return bersih ? bersihkanNama(it.uraian) : it.uraian;
		case "volume":
		case "dipesan":
		case "diterima":
		case "sesuai": return fmtVol(it.volume);
		case "satuan": return it.satuan ?? "";
		case "volume_satuan": return `${fmtVol(it.volume)} ${it.satuan ?? ""}`.trim();
		case "harga": return harga ? angka(harga) : "";
		case "jumlah": return angka(it.nominal);
		case "rusak": return "-";
		case "kode_rekening": return it.kodeRekening ?? "";
	}
	return "";
};

export const kolomAngka = (k: KolomTabel["kunci"]) => ["volume", "harga", "jumlah", "dipesan", "diterima", "rusak", "sesuai"].includes(k);

/** Nama kode rekening induk belanja (tidak tersimpan di ARKAS) untuk Lembar Kertas Kerja. */
export const NAMA_REKENING_INDUK: Record<string, string> = {
	5: "BELANJA",
	5.1: "BELANJA OPERASI",
	"5.1.01": "BELANJA PEGAWAI",
	"5.1.02": "BELANJA BARANG DAN JASA",
	"5.1.02.01": "BELANJA BARANG",
	"5.1.02.02": "BELANJA JASA",
	"5.1.02.03": "BELANJA PEMELIHARAAN",
	"5.1.02.04": "BELANJA PERJALANAN DINAS",
	"5.1.02.05": "BELANJA UANG DAN/ATAU JASA UNTUK DIBERIKAN KEPADA PIHAK KETIGA/PIHAK LAIN/MASYARAKAT",
	5.2: "BELANJA MODAL",
	"5.2.01": "BELANJA MODAL TANAH",
	"5.2.02": "BELANJA MODAL PERALATAN DAN MESIN",
	"5.2.03": "BELANJA MODAL GEDUNG DAN BANGUNAN",
	"5.2.04": "BELANJA MODAL JALAN, IRIGASI, DAN JARINGAN",
	"5.2.05": "BELANJA MODAL ASET TETAP LAINNYA",
	"5.2.06": "BELANJA MODAL ASET LAINNYA"
};

/** Baris yang selalu tampil di Lembar Kertas Kerja walau nilainya 0. */
export const REKENING_INDUK_TETAP = ["5", "5.1", "5.1.02", "5.1.02.01", "5.1.02.02", "5.1.02.03", "5.1.02.04", "5.2"];
