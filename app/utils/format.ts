const NON_ALNUM = /[^a-z0-9]+/g;
const WHITESPACE = /\s+/g;
const FILE_UNSAFE = /[\\/:*?"<>|]/g;

/** Ubah teks jadi id yang aman untuk HTML, mis. "Buku Kas Umum" -> "buku-kas-umum". */
export const slugify = (text: string) => text.toLowerCase().replace(NON_ALNUM, "-");

const rupiahFormat = new Intl.NumberFormat("id-ID", { maximumFractionDigits: 0 });

/** 1234567 -> "1.234.567" (tanpa "Rp", untuk kolom tabel). */
export const angka = (n: number) => rupiahFormat.format(n);

/** 1234567 -> "Rp 1.234.567". */
export const rupiah = (n: number) => (n < 0 ? `-Rp ${angka(-n)}` : `Rp ${angka(n)}`);

export const BULAN = [
	"Januari",
	"Februari",
	"Maret",
	"April",
	"Mei",
	"Juni",
	"Juli",
	"Agustus",
	"September",
	"Oktober",
	"November",
	"Desember"
];

/** "2026-09-04" -> "04-09-2026". */
export const tanggalId = (iso: string) => {
	const [y, m, d] = iso.split("-");
	return y && m && d ? `${d}-${m}-${y}` : iso;
};

/** Teks aman untuk nama file Windows: spasi jadi "_", karakter terlarang dibuang. */
export const fileSafe = (text: string) => text.trim().replace(FILE_UNSAFE, "").replace(WHITESPACE, "_");

/** "2026-09-04" -> "4 September 2026". */
export const tanggalPanjang = (iso: string | null | undefined) => {
	if (!iso) return "";
	const [y, m, d] = iso.slice(0, 10).split("-").map(Number);
	if (!y || !m || !d) return iso;
	return `${d} ${BULAN[m - 1]} ${y}`;
};

/** Tanggal terakhir suatu bulan dalam format ISO. */
export const akhirBulan = (year: number, month: number) => {
	const day = new Date(year, month, 0).getDate();
	return `${year}-${String(month).padStart(2, "0")}-${String(day).padStart(2, "0")}`;
};

/** Label periode: "Januari", "Januari - Juni 2026", dst. */
export const labelPeriode = (year: number, start: number, end: number) =>
	start === end ? `${BULAN[start - 1]} ${year}` : `${BULAN[start - 1]} - ${BULAN[end - 1]} ${year}`;

/** Pecahan rupiah untuk hitung fisik uang (Register Kas). */
export const PECAHAN_KERTAS = [100000, 50000, 20000, 10000, 5000, 2000, 1000];
export const PECAHAN_LOGAM = [1000, 500, 200, 100];

const CSS_UNSAFE = /["\\]/g;
const SPASI = /\s+/g;
/** Teks aman untuk nilai `content` CSS (mis. judul di margin halaman cetak), sudah berkutip. */
export const teksCss = (v: string) => `"${v.replace(CSS_UNSAFE, "").replace(SPASI, " ")}"`;
