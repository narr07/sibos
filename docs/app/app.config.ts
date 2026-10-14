export default defineAppConfig({
	seo: {
		title: "SIBOS",
		description: "Pendamping ARKAS untuk SPJ BOSP: buku kas, laporan, kwitansi, dan dokumen SPJ langsung dari database ARKAS."
	},
	header: {
		title: "SIBOS",
		logo: {
			light: "/logo.png",
			dark: "/logo.png",
			alt: "SIBOS"
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
		// Sama dengan warna aplikasi SIBOS
		colors: {
			primary: "emerald",
			neutral: "slate"
		}
	}
});
