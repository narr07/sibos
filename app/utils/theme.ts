/** Pilihan warna tema. Nilai hex dipakai untuk swatch karena Nuxt UI hanya membuat variabel CSS untuk warna yang aktif. */
export const PRIMARY_COLORS: { name: string, hex: string }[] = [
	{ name: "red", hex: "#ef4444" },
	{ name: "orange", hex: "#f97316" },
	{ name: "amber", hex: "#f59e0b" },
	{ name: "yellow", hex: "#eab308" },
	{ name: "lime", hex: "#84cc16" },
	{ name: "green", hex: "#22c55e" },
	{ name: "emerald", hex: "#10b981" },
	{ name: "teal", hex: "#14b8a6" },
	{ name: "cyan", hex: "#06b6d4" },
	{ name: "sky", hex: "#0ea5e9" },
	{ name: "blue", hex: "#3b82f6" },
	{ name: "indigo", hex: "#6366f1" },
	{ name: "violet", hex: "#8b5cf6" },
	{ name: "purple", hex: "#a855f7" },
	{ name: "fuchsia", hex: "#d946ef" },
	{ name: "pink", hex: "#ec4899" },
	{ name: "rose", hex: "#f43f5e" }
];

export const NEUTRAL_COLORS: { name: string, hex: string }[] = [
	{ name: "slate", hex: "#64748b" },
	{ name: "gray", hex: "#6b7280" },
	{ name: "zinc", hex: "#71717a" },
	{ name: "neutral", hex: "#737373" },
	{ name: "stone", hex: "#78716c" },
	{ name: "taupe", hex: "oklch(54.7% 0.021 43.1)" },
	{ name: "mauve", hex: "oklch(54.2% 0.034 322.5)" },
	{ name: "mist", hex: "oklch(56% 0.021 213.5)" },
	{ name: "olive", hex: "oklch(58% 0.031 107.3)" }
];

export const THEME_KEYS = { primary: "sibos-theme-primary", neutral: "sibos-theme-neutral" } as const;

export const themeColors = { primary: PRIMARY_COLORS, neutral: NEUTRAL_COLORS };
