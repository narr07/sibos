/** Riwayat versi SIBOS (terbaru di atas). Tampil di halaman Changelog. */
export interface VersiRilis {
	versi: string
	tanggal: string
	judul: string
	ringkasan: string
	perubahan: { jenis: "baru" | "perbaikan" | "ubah", teks: string }[]
}

export const CHANGELOG: VersiRilis[] = [
	{
		versi: "0.1.0",
		tanggal: "2026-10-05",
		judul: "Rilis pertama SIBOS",
		ringkasan: "Pendamping ARKAS untuk menyusun SPJ BOSP: buku kas, laporan, dokumen belanja, dan RKAS dalam satu aplikasi ringan.",
		perubahan: [
			{ jenis: "baru", teks: "Buku Kas Umum, Buku Pembantu Tunai, Bank, dan Pajak langsung dari database ARKAS (hanya baca)" },
			{ jenis: "baru", teks: "Kertas Kerja RKAS format Tahunan, Triwulan, Bulanan, dan Lembar Kertas Kerja, export Excel & PDF" },
			{ jenis: "baru", teks: "Realisasi belanja, Cari Barang & Rekening, dan penyusun draft RKAS (Perubahan/Pergeseran)" },
			{ jenis: "baru", teks: "Cetak Kwitansi A2, Bukti Pengeluaran, Nota Toko, dan dokumen dari template (SP, Kwitansi, BA Serah Terima)" },
			{ jenis: "baru", teks: "Template kanvas dari foto/scan kwitansi kosong, plus export/import template" },
			{ jenis: "baru", teks: "Laporan BA Rekonsiliasi, Rekonsiliasi Bank, SPTJM, K7/K7a, Register Penutupan Kas & Berita Acara" },
			{ jenis: "baru", teks: "Pajak dicatat manual per nota dan otomatis masuk Buku Pembantu Pajak" },
			{ jenis: "baru", teks: "Kop surat, pejabat penandatangan, dan ketua komite otomatis dari ARKAS, tetap bisa diubah" },
			{ jenis: "baru", teks: "Backup & restore data SIBOS, backup otomatis, dan salin arkas.db" },
			{ jenis: "baru", teks: "Tema warna, mode terang/gelap, dan notifikasi pembaruan otomatis" }
		]
	}
];
