<template>
	<div class="print-page bg-white text-black shadow-md" :style="style">
		<slot />
	</div>
</template>

<script lang="ts" setup>
	const props = defineProps<{
		landscape?: boolean
		/** Ukuran kertas khusus dokumen ini; default mengikuti Pengaturan. */
		paper?: "A4" | "F4" | "A5"
		/** Margin dalam mm. */
		margin?: number
		/** Margin atas khusus dalam mm (mis. ruang jilid). */
		marginTop?: number
	}>();
	const { pengaturan } = useDocContext();

	const SIZES = { A4: [210, 297], F4: [215, 330], A5: [148, 210] } as const;

	const style = computed(() => {
		const paper = props.paper ?? (pengaturan.value?.cetak.kertas === "F4" ? "F4" : "A4");
		const [w, h] = SIZES[paper];
		const [width, height] = props.landscape ? [h, w] : [w, h];
		// Bawaan 1,97 cm di semua sisi (sama dengan dokumen Word sekolah).
		const margin = props.margin ?? 19.7;
		const side = `${margin}mm`;
		const top = props.marginTop ?? margin;
		const bottom = margin;
		const pad = `${top}mm ${side} ${bottom}mm ${side}`;
		// Nama halaman (CSS @page bernama) agar satu kali cetak bisa campur ukuran kertas.
		const page = `${paper.toLowerCase()}-${props.landscape ? "l" : "p"}`;
		return { width: `${width}mm`, minHeight: `${height}mm`, padding: pad, page, "--page-h": `${height}mm` };
	});
</script>
