import process from "node:process";

const host = process.env.TAURI_DEV_HOST;

export default defineNuxtConfig({
	modules: [
		"@vueuse/nuxt",
		"@nuxt/ui",
		"@nuxt/eslint"
	],
	app: {
		head: {
			title: "SIBOS",
			charset: "utf-8",
			htmlAttrs: { lang: "id" },
			link: [{ rel: "icon", type: "image/png", href: "/favicon.png" }],
			meta: [
				{ name: "format-detection", content: "no" },
				{ name: "viewport", content: "width=device-width, initial-scale=1" }
			]
		}
	},
	css: [
		"@/assets/css/main.css"
	],
	ssr: false,
	dir: {
		modules: "app/modules"
	},
	imports: {
		presets: [
			{
				from: "zod",
				imports: [
					"z",
					{
						name: "infer",
						as: "zInfer",
						type: true
					}
				]
			}
		]
	},
	icon: {
		// Ikon dibundel lokal: aplikasi harus jalan offline
		provider: "none",
		clientBundle: {
			scan: true,
			// Ikon menu ditulis di file .ts (app/utils/menu.ts), pastikan ikut dibundel
			icons: [
				"lucide:book-open",
				"lucide:chart-column",
				"lucide:coins",
				"lucide:database-backup",
				"lucide:file-check",
				"lucide:file-spreadsheet",
				"lucide:history",
				"lucide:info",
				"lucide:landmark",
				"lucide:layout-dashboard",
				"lucide:layout-template",
				"lucide:pencil-ruler",
				"lucide:printer",
				"lucide:receipt",
				"lucide:scale",
				"lucide:search",
				"lucide:settings",
				"lucide:stamp",
				"lucide:store",
				"lucide:table",
				"lucide:wallet"
			]
		}
	},
	ui: {
		// Pakai font sistem, tanpa @nuxt/fonts (aplikasi harus jalan offline)
		fonts: false
	},
	vite: {
		clearScreen: false,
		envPrefix: ["VITE_", "TAURI_"],
		server: {
			strictPort: true,
			hmr: host
				? {
					protocol: "ws",
					host,
					port: 3001
				}
				: undefined,
			watch: {
				ignored: ["**/src-tauri/**"]
			}
		}
	},
	devServer: {
		host: host || "0.0.0.0"
	},
	eslint: {
		config: {
			standalone: false
		}
	},
	devtools: {
		enabled: false
	},
	experimental: {
		typedPages: true
	},
	compatibilityDate: "2026-01-01"
});
