// Situs dokumentasi dan unduhan SIBOS: https://sibos.permadi.dev
export default defineNuxtConfig({
	extends: ["docus"],
	app: {
		head: {
			htmlAttrs: { lang: "id" },
			link: [{ rel: "icon", type: "image/png", href: "/favicon.png" }]
		}
	},
	site: {
		url: "https://sibos.permadi.dev",
		name: "SIBOS"
	},
	// Server MCP bawaan Docus tidak dipakai: situs ini statis (nuxt generate).
	mcp: {
		enabled: false
	}
});
