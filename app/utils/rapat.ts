// Generator dokumen rapat (undangan, daftar hadir, berita acara, notulen) sebagai bukti SPJ konsumsi rapat.

/** Guru / tenaga kependidikan untuk daftar hadir (diisi di Pengaturan, disimpan di SIBOS). */
export interface Guru {
	id: string
	nama: string
	/** NIP atau NUPTK; boleh kosong. */
	nip: string
	/** Tugas, mis. "Guru Kelas", "Guru Mapel", "Operator". */
	tugas: string
	/** Wali kelas berapa atau mengajar mata pelajaran apa, mis. "Wali Kelas 4", "PJOK". */
	kelasMapel: string
	/** Pangkat/golongan untuk SPPD, mis. "Penata Muda / III a"; boleh kosong. */
	pangkat?: string
}

export const SETTING_GURU = "guru";
/** Dokumen rapat tersimpan per tahun anggaran: kunci = key nota di BKU. */
export const settingRapat = (year: number) => `rapat.${year}`;

export const guruKosong = (): Guru => ({ id: crypto.randomUUID(), nama: "", nip: "", tugas: "", kelasMapel: "", pangkat: "" });

/** Kolom "Jabatan / Peran" di daftar hadir: "Guru Kelas / Wali Kelas 4". */
export const peranGuru = (g: Guru) => [g.tugas, g.kelasMapel].map((v) => v.trim()).filter(Boolean).join(" / ");

const samaNama = (a?: string | null, b?: string | null) => !!a?.trim() && a.trim().toLowerCase() === (b ?? "").trim().toLowerCase();

/** Kolom "Jabatan" di daftar hadir rapat: semua "Guru", kecuali notulis rapat itu dan kepala sekolah. */
export const jabatanHadir = (g: Guru, notulis: string, kepalaSekolah?: string | null) => {
	if (samaNama(g.nama, notulis)) return "Notulis";
	return samaNama(g.nama, kepalaSekolah) ? "Kepala Sekolah" : "Guru";
};

/** Notulis bawaan bergilir (bukan kepala sekolah), supaya tidak guru yang sama terus. */
export const notulisBergilir = (guru: Guru[], urutan: number, kepalaSekolah?: string | null) => {
	const calon = guru.filter((g) => !samaNama(g.nama, kepalaSekolah));
	return calon.length ? calon[urutan % calon.length]! : null;
};

const SINGKATAN_SEKOLAH = new Set(["SD", "SDN", "SDIT", "SMP", "SMPN", "SMPIT", "SMA", "SMAN", "SMK", "SMKN", "SLB", "MI", "MIN", "MA", "MAN", "TK", "PAUD", "PKBM", "SKB"]);
const SPASI_KATA = /\s+/;
const ANGKA_ROMAWI = /^(?=[IVX])X{0,3}(?:IX|IV|V?I{0,3})$/;

/** "SD NEGERI TEJA II" -> "SD Negeri Teja II" (singkatan dan angka Romawi tetap huruf besar). */
export const namaSekolahRapi = (nama?: string | null) => (nama ?? "").trim().split(SPASI_KATA).filter(Boolean).map((kata) => {
	const besar = kata.toUpperCase();
	if (besar === "MTS") return "MTs";
	if (SINGKATAN_SEKOLAH.has(besar) || ANGKA_ROMAWI.test(besar)) return besar;
	return besar.charAt(0) + besar.slice(1).toLowerCase();
}).join(" ");

/** Jabatan penanda tangan: "Kepala SD Negeri Teja II". */
export const jabatanKepala = (nama?: string | null) => `Kepala ${namaSekolahRapi(nama) || "Sekolah"}`;

export type JenisDokumenRapat = "undangan" | "hadir" | "ba" | "notulen";

export const DOKUMEN_RAPAT: { value: JenisDokumenRapat, label: string }[] = [
	{ value: "undangan", label: "Surat Undangan" },
	{ value: "hadir", label: "Daftar Hadir" },
	{ value: "ba", label: "Berita Acara" },
	{ value: "notulen", label: "Notulen" }
];

export interface RapatData {
	/** Nama kegiatan lengkap, mis. "Rapat Penyusunan RKAS". */
	namaKegiatan: string
	agenda: string
	/** Tanggal pelaksanaan (ISO). */
	tanggal: string
	/** "09:00" */
	waktuMulai: string
	waktuSelesai: string
	zona: string
	tempat: string
	// Undangan
	nomorSurat: string
	lampiran: string
	tanggalSurat: string
	kepada: string
	// Daftar hadir
	/** Id guru yang hadir. */
	peserta: string[]
	/** Baris kosong tambahan untuk peserta di luar daftar guru (komite, tamu). */
	barisKosong: number
	// Berita acara & notulen
	nomorBa: string
	pimpinan: string
	jabatanPimpinan: string
	nipPimpinan: string
	notulis: string
	nipNotulis: string
	arahan: string
	pembahasan: string
	/** Kesepakatan / keputusan rapat, satu poin per baris. */
	keputusan: string
}

export const HARI = ["Minggu", "Senin", "Selasa", "Rabu", "Kamis", "Jumat", "Sabtu"];

const keDate = (iso: string) => {
	const [y, m, d] = iso.slice(0, 10).split("-").map(Number);
	return y && m && d ? new Date(y, m - 1, d) : null;
};

const keIso = (d: Date) =>
	`${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}-${String(d.getDate()).padStart(2, "0")}`;

/** "2026-01-15" -> "Kamis". */
export const hariDari = (iso: string) => {
	const d = keDate(iso);
	return d ? HARI[d.getDay()] ?? "" : "";
};

/** Geser tanggal sekian hari; bila jatuh di hari Minggu dimundurkan ke Sabtu. */
export const geserTanggal = (iso: string, hari: number) => {
	const d = keDate(iso);
	if (!d) return iso;
	d.setDate(d.getDate() + hari);
	if (d.getDay() === 0) d.setDate(d.getDate() - 1);
	return keIso(d);
};

/** Bagian tanggal untuk kalimat berita acara: hari, tanggal/tahun terbilang, bulan. */
export const tanggalTerbilang = (iso: string) => {
	const d = keDate(iso);
	if (!d) return null;
	return { hari: HARI[d.getDay()] ?? "", tanggal: terbilang(d.getDate()), bulan: BULAN[d.getMonth()] ?? "", tahun: terbilang(d.getFullYear()) };
};

/** "09:00" -> "09.00". */
export const jam = (v: string) => v.replace(":", ".");

/** Teks multi-baris -> daftar poin tanpa baris kosong. */
export const barisTeks = (v: string) => v.split("\n").map((l) => l.trim()).filter(Boolean);

const KATA_KONSUMSI = /konsumsi|makan|minum|snack|snek|kudapan|nasi\s?(?:kotak|box|dus)|rapat/i;

/** Belanja yang kemungkinan konsumsi rapat, dilihat dari uraian dan nama rekening. */
export const isKonsumsi = (g: NotaGroup, rekeningNames: Record<string, string>) =>
	[g.uraian, ...g.items.flatMap((i) => [i.uraian, i.kodeRekening ? rekeningNames[i.kodeRekening] : ""])]
		.some((v) => !!v && KATA_KONSUMSI.test(v));

/** Ringkasan item belanja: "25 kotak Nasi kotak". */
export const ringkasItem = (i: NotaItem) => [i.volume ? angka(i.volume) : "", i.satuan ?? "", i.uraian].filter(Boolean).join(" ");

const AWALAN_RAPAT = /^rapat\b/i;

type IsiTopik = Pick<RapatData, "arahan" | "pembahasan"> & { keputusan: string[] };

/** Isi notulen per jenis rapat, dipilih dari kata kunci di judul. Yang pertama cocok dipakai. */
const TOPIK: (IsiTopik & { cocok: RegExp })[] = [
	{
		cocok: /rkas|anggaran|rencana kegiatan/i,
		arahan: "Pimpinan rapat menyampaikan pagu dana BOSP yang diterima sekolah dan menekankan agar penyusunan anggaran mengacu pada rapor pendidikan, kebutuhan prioritas sekolah, serta petunjuk teknis yang berlaku.",
		pembahasan: "Peserta menyampaikan usulan kebutuhan tiap kelas dan bidang, menentukan skala prioritas kegiatan, lalu menyesuaikannya dengan pagu anggaran yang tersedia.",
		keputusan: [
			"Rencana kegiatan dan anggaran disusun berdasarkan prioritas kebutuhan sekolah dan disesuaikan dengan pagu dana BOSP.",
			"Usulan kebutuhan yang belum tertampung dicatat untuk dipertimbangkan pada perubahan anggaran berikutnya.",
			"Tim BOSP sekolah menuangkan hasil rapat ke dalam aplikasi ARKAS."
		]
	},
	{
		cocok: /evaluasi|realisasi|laporan|pertanggungjawaban|spj|monitoring/i,
		arahan: "Pimpinan rapat menyampaikan pentingnya keterbukaan dan ketertiban administrasi dalam penggunaan dana BOSP.",
		pembahasan: "Bendahara memaparkan realisasi penerimaan dan penggunaan dana pada periode berjalan. Peserta mencermati kegiatan yang sudah dan belum terlaksana beserta kendalanya.",
		keputusan: [
			"Laporan realisasi penggunaan dana BOSP periode berjalan diterima oleh peserta rapat.",
			"Kegiatan yang belum terlaksana dijadwalkan ulang pada periode berikutnya.",
			"Bukti pengeluaran dilengkapi dan diarsipkan sesuai ketentuan."
		]
	},
	{
		cocok: /pembagian tugas|awal tahun|tahun ajaran|tahun pelajaran|program kerja|kurikulum/i,
		arahan: "Pimpinan rapat menyampaikan arah program sekolah dan mengajak seluruh guru menyiapkan pembelajaran dengan baik.",
		pembahasan: "Peserta membahas pembagian tugas mengajar dan tugas tambahan, kalender pendidikan, jadwal pelajaran, serta kesiapan perangkat pembelajaran.",
		keputusan: [
			"Pembagian tugas mengajar dan tugas tambahan ditetapkan sesuai hasil musyawarah.",
			"Setiap guru menyiapkan perangkat pembelajaran sebelum kegiatan belajar dimulai.",
			"Kalender pendidikan dan jadwal pelajaran diberlakukan sesuai kesepakatan rapat."
		]
	},
	{
		cocok: /asesmen|ujian|penilaian|sumatif|ulangan/i,
		arahan: "Pimpinan rapat menekankan agar penilaian dilaksanakan dengan jujur, tertib, dan sesuai jadwal.",
		pembahasan: "Peserta membahas jadwal pelaksanaan, pembagian tugas panitia dan pengawas, penyusunan dan penggandaan naskah soal, serta tata tertib pelaksanaan.",
		keputusan: [
			"Jadwal pelaksanaan dan susunan panitia ditetapkan sesuai hasil rapat.",
			"Guru menyerahkan naskah soal dan kisi-kisi kepada panitia sesuai tenggat yang disepakati.",
			"Hasil penilaian diolah dan dilaporkan kepada orang tua/wali murid."
		]
	},
	{
		cocok: /kenaikan kelas|kelulusan|rapor/i,
		arahan: "Pimpinan rapat meminta agar keputusan diambil secara objektif berdasarkan data nilai, kehadiran, dan sikap murid.",
		pembahasan: "Wali kelas memaparkan rekap nilai, kehadiran, dan catatan perkembangan murid. Peserta membahas murid yang memerlukan pertimbangan khusus.",
		keputusan: [
			"Hasil rapat ditetapkan berdasarkan kriteria yang berlaku di sekolah.",
			"Wali kelas menyelesaikan pengisian rapor sesuai jadwal.",
			"Hasil keputusan disampaikan kepada orang tua/wali murid saat pembagian rapor."
		]
	},
	{
		cocok: /ppdb|spmb|penerimaan (?:peserta didik|murid|siswa)/i,
		arahan: "Pimpinan rapat menekankan agar penerimaan murid baru berjalan objektif, transparan, dan sesuai ketentuan dinas pendidikan.",
		pembahasan: "Peserta membahas daya tampung, jadwal dan jalur pendaftaran, susunan panitia, serta sosialisasi kepada masyarakat.",
		keputusan: [
			"Susunan panitia dan jadwal penerimaan murid baru ditetapkan sesuai hasil rapat.",
			"Daya tampung disesuaikan dengan jumlah rombongan belajar dan ruang kelas yang tersedia.",
			"Sosialisasi dilakukan melalui papan informasi sekolah dan pertemuan dengan masyarakat."
		]
	},
	{
		cocok: /komite|orang tua|wali murid/i,
		arahan: "Pimpinan rapat menyampaikan program sekolah dan mengajak orang tua serta komite mendukung kegiatan belajar murid.",
		pembahasan: "Sekolah memaparkan program dan kebutuhan sekolah. Komite dan orang tua menyampaikan masukan serta bentuk dukungan yang dapat diberikan.",
		keputusan: [
			"Komite dan orang tua mendukung pelaksanaan program sekolah.",
			"Masukan dari komite dan orang tua ditampung sebagai bahan perbaikan layanan sekolah.",
			"Komunikasi antara sekolah dan orang tua dilanjutkan melalui wali kelas."
		]
	}
];

/** Kolom yang isinya mengikuti judul rapat selama belum diubah manual. */
export const KOLOM_ISI = ["agenda", "arahan", "pembahasan", "keputusan"] as const;

/** Agenda dan isi notulen dari judul rapat: memakai topik yang cocok, atau kalimat umum. */
export const isiRapat = (namaKegiatan: string): Pick<RapatData, typeof KOLOM_ISI[number]> => {
	const topik = namaKegiatan.trim().replace(AWALAN_RAPAT, "").trim() || "kegiatan sekolah";
	const isi: IsiTopik = TOPIK.find((t) => t.cocok.test(topik)) ?? {
		arahan: `Pimpinan rapat menyampaikan maksud dan tujuan rapat ${topik} serta mengajak seluruh peserta berperan aktif dalam pembahasan.`,
		pembahasan: `Peserta membahas rencana, pembagian tugas, jadwal, dan kebutuhan yang berkaitan dengan ${topik}.`,
		keputusan: [
			`Peserta rapat menyepakati hasil pembahasan ${topik}.`,
			"Pelaksanaan kegiatan berpedoman pada petunjuk teknis pengelolaan dana BOSP yang berlaku.",
			"Hasil rapat ditindaklanjuti oleh masing-masing penanggung jawab sesuai tugasnya."
		]
	};
	return { agenda: `Pembahasan ${topik}`, arahan: isi.arahan, pembahasan: isi.pembahasan, keputusan: isi.keputusan.join("\n") };
};

/** "Penyusunan RKAS" -> "Rapat Penyusunan RKAS"; kosong bila tidak ada isinya selain kata "rapat". */
const judulRapat = (teks: string) => {
	const inti = maksudDariUraian(teks).replace(AWALAN_RAPAT, "").trim();
	return inti.length >= 4 ? `Rapat ${inti.charAt(0).toUpperCase()}${inti.slice(1)}` : "";
};

/**
 * Pilihan judul rapat untuk satu belanja, yang paling khusus lebih dulu: uraian BKU, uraian item,
 * lalu nama kegiatan RKAS (sering sama untuk banyak belanja, jadi hanya cadangan).
 */
export const saranJudul = (g: NotaGroup, namaKegiatan?: string | null) =>
	[...new Set([g.uraian, ...g.items.map((i) => i.uraian), namaKegiatan ?? ""].map(judulRapat).filter(Boolean))];

/** Isian awal dokumen rapat dari satu belanja di BKU. */
export const rapatBawaan = (
	g: NotaGroup,
	ctx: { namaKegiatan?: string | null, pejabat?: Pengaturan["pejabat"] | null, guru: Guru[] }
): RapatData => {
	const tanggal = g.tanggal.slice(0, 10);
	const namaKegiatan = saranJudul(g, ctx.namaKegiatan)[0] ?? "Rapat Dewan Guru";
	return {
		namaKegiatan,
		...isiRapat(namaKegiatan),
		tanggal,
		waktuMulai: "09:00",
		waktuSelesai: "12:00",
		zona: "WIB",
		tempat: "Ruang Guru",
		nomorSurat: "",
		lampiran: "-",
		tanggalSurat: geserTanggal(tanggal, -3),
		kepada: "Bapak/Ibu Dewan Guru dan Tenaga Kependidikan",
		peserta: ctx.guru.map((x) => x.id),
		barisKosong: 0,
		nomorBa: "",
		pimpinan: ctx.pejabat?.kepalaSekolah ?? "",
		jabatanPimpinan: "Kepala Sekolah",
		nipPimpinan: ctx.pejabat?.nipKepalaSekolah ?? "",
		notulis: "",
		nipNotulis: ""
	};
};
