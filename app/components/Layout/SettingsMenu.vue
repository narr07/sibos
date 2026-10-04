<template>
	<UDropdownMenu
		:items="items"
		:content="{ align: 'center', collisionPadding: 12 }"
		:ui="{ content: collapsed ? 'w-52' : 'w-(--reka-dropdown-menu-trigger-width)' }"
	>
		<UButton
			icon="i-lucide-settings"
			color="neutral"
			variant="ghost"
			block
			:square="collapsed"
			:label="collapsed ? undefined : 'Pengaturan'"
			:trailing-icon="collapsed ? undefined : 'i-lucide-chevrons-up-down'"
			class="data-[state=open]:bg-elevated"
			:ui="{ trailingIcon: 'text-dimmed' }"
		/>

		<template #chip-leading="{ item }">
			<div class="inline-flex items-center justify-center shrink-0 size-5">
				<span class="rounded-full ring ring-bg size-2" :style="{ backgroundColor: (item as ChipItem).hex }" />
			</div>
		</template>
	</UDropdownMenu>
</template>

<script lang="ts" setup>
	import type { DropdownMenuItem } from "@nuxt/ui";

	type ChipItem = DropdownMenuItem & { hex?: string };

	defineProps<{ collapsed?: boolean }>();

	const appConfig = useAppConfig();
	const colorMode = useColorMode();

	const setColor = (kind: "primary" | "neutral", color: string) => {
		appConfig.ui.colors[kind] = color;
		try {
			localStorage.setItem(THEME_KEYS[kind], color);
		} catch {
			// Pilihan hanya tidak tersimpan untuk sesi berikutnya.
		}
	};

	const hexOf = (kind: "primary" | "neutral") =>
		themeColors[kind].find((c) => c.name === appConfig.ui.colors[kind])?.hex ?? "#737373";

	const colorMenu = (kind: "primary" | "neutral", label: string, align: "center" | "end"): ChipItem => ({
		label,
		slot: "chip",
		hex: hexOf(kind),
		content: { align, collisionPadding: 16 },
		children: themeColors[kind].map((color) => ({
			label: color.name,
			hex: color.hex,
			slot: "chip",
			type: "checkbox",
			checked: appConfig.ui.colors[kind] === color.name,
			onSelect: (e: Event) => {
				e.preventDefault();
				setColor(kind, color.name);
			}
		}))
	});

	const modeItem = (label: string, icon: string, value: "light" | "dark" | "system"): DropdownMenuItem => ({
		label,
		icon,
		type: "checkbox",
		checked: colorMode.preference === value,
		onSelect: (e: Event) => {
			e.preventDefault();
			colorMode.preference = value;
		}
	});

	/** Halaman grup "Lainnya" tampil di menu ini, bukan di sidebar. */
	const pages = menuGroups.find((g) => g.title === "Lainnya")?.pages ?? [];

	const items = computed<DropdownMenuItem[][]>(() => [
		pages.map((p) => ({ label: p.label, icon: p.icon, to: p.to })),
		[
			{
				label: "Tema",
				icon: "i-lucide-palette",
				children: [colorMenu("primary", "Warna utama", "center"), colorMenu("neutral", "Warna netral", "end")]
			},
			{
				label: "Tampilan",
				icon: "i-lucide-sun-moon",
				children: [
					modeItem("Terang", "i-lucide-sun", "light"),
					modeItem("Gelap", "i-lucide-moon", "dark"),
					modeItem("Ikuti sistem", "i-lucide-monitor", "system")
				]
			}
		]
	]);
</script>
