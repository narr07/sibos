export default defineAppConfig({
	seo: {
		title: "Lajur",
		description: "Pendamping ARKAS untuk SPJ BOSP: buku kas, laporan, kwitansi, dan dokumen SPJ langsung dari database ARKAS."
	},
	header: {
		title: "Lajur",
		// Simbol (persegi) dipakai gambar pratinjau OG; header menampilkan wordmark "L lajur".
		// Harus SVG: template OG Docus menyisipkan isi file logo sebagai markup SVG.
		logo: {
			light: "/logo-light.svg",
			dark: "/logo-dark.svg",
			alt: "Lajur",
			display: "wordmark",
			wordmark: {
				light: "/wordmark-light.svg",
				dark: "/wordmark-dark.svg"
			},
			favicon: "/favicon.svg",
			class: "h-7"
		}
	},
	socials: {
		github: "https://github.com/narr07/sibos"
	},
	github: {
		url: "https://github.com/narr07/sibos",
		branch: "main",
		rootDir: "docs"
	},
	toc: {
		title: "Di halaman ini"
	},
	docus: {
		locale: "id"
	},
	ui: {
		// Sama dengan warna aplikasi Lajur
		colors: {
			primary: "emerald",
			neutral: "slate"
		}
	}
});
