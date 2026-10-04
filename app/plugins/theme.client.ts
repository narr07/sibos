/** Terapkan warna tema tersimpan sebelum halaman tampil. */
export default defineNuxtPlugin(() => {
	const appConfig = useAppConfig();
	for (const kind of ["primary", "neutral"] as const) {
		try {
			const saved = localStorage.getItem(THEME_KEYS[kind]);
			if (saved && themeColors[kind].some((c) => c.name === saved)) appConfig.ui.colors[kind] = saved;
		} catch {
			// Penyimpanan tidak tersedia: pakai warna bawaan.
		}
	}
});
