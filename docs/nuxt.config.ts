// Situs dokumentasi dan unduhan Lajur (dulu SIBOS): https://lajur.web.id
export default defineNuxtConfig({
	extends: ["docus"],
	app: {
		head: {
			htmlAttrs: { lang: "id" },
			link: [
				{ rel: "icon", type: "image/svg+xml", href: "/favicon.svg" },
				{ rel: "apple-touch-icon", href: "/apple-touch-icon.png" }
			],
			// Identitas Lajur: semua sudut siku (radius Nuxt UI = 0)
			style: [{ innerHTML: ":root{--ui-radius:0rem}" }]
		}
	},
	site: {
		url: "https://lajur.web.id",
		name: "Lajur"
	},
	// Server MCP bawaan Docus tidak dipakai: situs ini statis (nuxt generate).
	mcp: {
		enabled: false
	}
});
