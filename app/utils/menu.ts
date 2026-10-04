import type { NavigationMenuItem } from "@nuxt/ui";

export interface MenuPage {
	label: string
	icon: string
	to: string
	/** Fase rilis saat menu ini dikerjakan. */
	phase: number
	description: string
	redesign?: boolean
}

export interface MenuGroup {
	title: string
	pages: MenuPage[]
}

/** Susunan menu aplikasi, dikelompokkan per bagian. */
export const menuGroups: MenuGroup[] = [
	{
		title: "",
		pages: [
			{ label: "Dashboard", icon: "i-lucide-layout-dashboard", to: "/", phase: 4, description: "Ringkasan keuangan, status RKAS dan BKU, grafik, dan peringatan." }
		]
	},
	{
		title: "Penganggaran",
		pages: [
			{ label: "Kertas Kerja (RKAS)", icon: "i-lucide-file-spreadsheet", to: "/penganggaran/kertas-kerja", phase: 4, description: "RKAS per program, kegiatan, rekening, dan item dengan distribusi bulanan." },
			{ label: "Penganggaran RKAS", icon: "i-lucide-pencil-ruler", to: "/penganggaran/rkas", phase: 5, description: "Penyusun draft RKAS lokal: versi Awal, Perubahan, Pergeseran." },
			{ label: "Cari Barang & Rekening", icon: "i-lucide-search", to: "/penganggaran/standar-harga", phase: 5, description: "Cari katalog standar harga ARKAS dan salin kode rekening." },
			{ label: "Realisasi Belanja", icon: "i-lucide-chart-column", to: "/penganggaran/realisasi", phase: 4, description: "Rencana RKAS dibandingkan realisasi BKU per item dan per bulan." }
		]
	},
	{
		title: "Penatausahaan",
		pages: [
			{ label: "Buku Kas Umum", icon: "i-lucide-book-open", to: "/penatausahaan/bku", phase: 1, description: "Transaksi BKU per bulan dan sumber dana dengan saldo berjalan." },
			{ label: "Buku Pembantu Tunai", icon: "i-lucide-wallet", to: "/penatausahaan/tunai", phase: 1, description: "BKU difilter mutasi tunai." },
			{ label: "Buku Pembantu Bank", icon: "i-lucide-landmark", to: "/penatausahaan/bank", phase: 1, description: "BKU difilter mutasi bank." },
			{ label: "Buku Pembantu Pajak", icon: "i-lucide-receipt", to: "/penatausahaan/pajak", phase: 1, description: "Pungutan dan setoran pajak per jenis.", redesign: true },
			{ label: "Cetak Kwitansi A2", icon: "i-lucide-printer", to: "/penatausahaan/kwitansi", phase: 2, description: "Kelompok nota, gabungan bukti, foto nota, cetak Bukti Pengeluaran dan Kwitansi A2.", redesign: true }
		]
	},
	{
		title: "Laporan",
		pages: [
			{ label: "BA Rekonsiliasi", icon: "i-lucide-file-check", to: "/laporan/ba-rekon", phase: 3, description: "Berita Acara Rekonsiliasi BOSP format dinas." },
			{ label: "Rekonsiliasi Bank", icon: "i-lucide-scale", to: "/laporan/rekon-bank", phase: 3, description: "Saldo bank BKU dibandingkan rekening koran per bulan." },
			{ label: "Cetak SPTJM", icon: "i-lucide-stamp", to: "/laporan/sptjm", phase: 3, description: "Surat Pernyataan Tanggung Jawab Mutlak per semester atau bulan." },
			{ label: "Laporan K7 / K7a", icon: "i-lucide-table", to: "/laporan/k7", phase: 5, description: "Rekap realisasi per standar nasional pendidikan." },
			{ label: "Register Kas", icon: "i-lucide-coins", to: "/laporan/register-kas", phase: 3, description: "Hitung fisik uang, BA Pemeriksaan Kas, dan tutup kas." }
		]
	},
	{
		title: "Lainnya",
		pages: [
			{ label: "Backup & Restore", icon: "i-lucide-database-backup", to: "/lainnya/backup", phase: 5, description: "Backup data SIBOS, restore, salin arkas.db, export semua laporan." },
			{ label: "Template Dokumen", icon: "i-lucide-layout-template", to: "/lainnya/template", phase: 6, description: "Buat dan atur format SP, Nota, Kwitansi, BA Serah Terima." },
			{ label: "Data Penyedia", icon: "i-lucide-store", to: "/lainnya/penyedia", phase: 6, description: "Penanggung jawab, kop, dan logo toko untuk dokumen." },
			{ label: "Pengaturan", icon: "i-lucide-settings", to: "/lainnya/pengaturan", phase: 1, description: "Pejabat penandatangan, kop surat, nomor BA." },
			{ label: "Changelog", icon: "i-lucide-history", to: "/lainnya/changelog", phase: 0, description: "Riwayat versi dan perubahan aplikasi." },
			{ label: "Tentang Aplikasi", icon: "i-lucide-info", to: "/lainnya/tentang", phase: 0, description: "Fitur, pembuat, versi, dan pembaruan aplikasi." }
		]
	}
];

export const allPages = menuGroups.flatMap((g) => g.pages);

export const findPage = (path: string) => allPages.find((p) => p.to === path);

/** Item untuk UNavigationMenu: satu array per grup. Grup "Lainnya" ada di menu Pengaturan (LayoutSettingsMenu). */
export const navigationItems = (): NavigationMenuItem[][] =>
	menuGroups.filter((group) => group.title !== "Lainnya").map((group) => [
		...(group.title ? [{ label: group.title, type: "label" as const }] : []),
		...group.pages.map((p) => ({ label: p.label, icon: p.icon, to: p.to }))
	]);
