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
		versi: "0.3.0",
		tanggal: "2026-10-29",
		judul: "Katalog Toko SIPLah & Generator Standing Instruction (SI)",
		ringkasan: "Fitur belanja komparasi katalog toko SIPLah otomatis dan Generator Standing Instruction (SI) Bank untuk pemindahbukuan rekening.",
		perubahan: [
			{ jenis: "baru", teks: "Katalog Toko SIPLah: Simpan toko mitra SIPLah (Toko Ladang) hanya dengan link toko, otomatis sinkronisasi katalog dan foto produk" },
			{ jenis: "baru", teks: "Komparasi Belanja SIPLah: Halaman Belanja dengan tampilan kartu komparasi berdampingan (RKAS vs Produk Toko SIPLah) untuk melihat estimasi dan selisih harga" },
			{ jenis: "baru", teks: "Pencocokan Harga Cerdas: Algoritma tokenisasi pencocokan nama produk fleksibel dengan toleransi perbedaan penamaan kata kunci" },
			{ jenis: "baru", teks: "Filter Belanja Realisasi: Menampilkan hanya barang yang belum dibelanjakan berdasarkan realisasi transaksi BKU ARKAS" },
			{ jenis: "baru", teks: "Generator Standing Instruction (SI): Pembuat surat instruksi pemindahbukuan / transfer bank resmi ke Bank Kas Daerah / BJB" },
			{ jenis: "baru", teks: "Pilihan Belanja dari RKAS Bulanan: Ambil rencana belanja bulanan langsung dari kertas kerja ARKAS dengan filter item yang belum dibelanjakan" },
			{ jenis: "baru", teks: "Dukungan Rekanan Non-SIPLah: Input manual rekening penyedia mamin, fotokopi, atau rekanan lokal lainnya beserta potong pajak" },
			{ jenis: "perbaikan", teks: "Pemuatan gambar produk toko SIPLah Toko Ladang menggunakan host Object Storage (OSS) resmi" }
		]
	},
	{
		versi: "0.2.0",
		tanggal: "2026-10-14",
		judul: "Generator dokumen SPJ",
		ringkasan: "Menu Generator baru untuk membuat bukti SPJ konsumsi rapat dan perjalanan dinas langsung dari belanja di BKU ARKAS.",
		perubahan: [
			{ jenis: "baru", teks: "Generator Konsumsi Rapat: Surat Undangan, Daftar Hadir, Berita Acara, dan Notulen dari belanja konsumsi di BKU" },
			{ jenis: "baru", teks: "Judul rapat diambil dari uraian belanja, bisa diganti atau dipilih dari saran; isi notulen menyesuaikan judul" },
			{ jenis: "baru", teks: "Notulis rapat bergilir otomatis dan bisa dipilih dari daftar guru" },
			{ jenis: "baru", teks: "Generator SPPD: Surat Tugas, SPD, Kuitansi, Rincian Biaya & SPPD Rampung, Daftar Pengeluaran Riil, dan Laporan Pelaksanaan Tugas" },
			{ jenis: "baru", teks: "Belanja yang tercatat di rekening lain bisa dicari dan dipilih sendiri di generator" },
			{ jenis: "baru", teks: "Pengaturan: daftar guru dan tenaga kependidikan (NIP, pangkat/golongan, tugas, wali kelas/mata pelajaran)" },
			{ jenis: "ubah", teks: "Dokumen generator memakai huruf maksimal 12pt, termasuk kop surat" }
		]
	},
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
