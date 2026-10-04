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
		/** Margin dalam mm (minimal 10 mm saat dicetak). */
		margin?: number
		/** Margin atas khusus dalam mm (mis. ruang jilid). */
		marginTop?: number
		/** Tanpa margin minimum: posisi isi diukur dari tepi kertas (template kanvas). */
		exact?: boolean
	}>();
	const { pengaturan } = useDocContext();

	const SIZES = { A4: [210, 297], F4: [215, 330], A5: [148, 210] } as const;
	/**
	 * Kotak margin halaman: judul dokumen di kiri bawah, nomor halaman di kanan bawah. Dengan kotak
	 * margin terisi, header/footer bawaan browser (alamat, tanggal) tidak ikut dicetak.
	 */
	const { job } = usePrint();
	const marginBoxes = computed(() => [
		"@top-left { content: \"\" }",
		"@top-center { content: \"\" }",
		"@top-right { content: \"\" }",
		`@bottom-left { content: ${teksCss(job.value?.title ?? "")}; font: 8pt Arial, sans-serif; color: #555; }`,
		"@bottom-center { content: \"\" }",
		"@bottom-right { content: \"Halaman \" counter(page); font: 8pt Arial, sans-serif; color: #555; }"
	].join(" "));
	/** Margin cetak minimum (mm) agar isi tidak terpotong printer. */
	const MIN_MARGIN = 10;

	const layout = computed(() => {
		const paper = props.paper ?? (pengaturan.value?.cetak.kertas === "F4" ? "F4" : "A4");
		const [w, h] = SIZES[paper];
		const [width, height] = props.landscape ? [h, w] : [w, h];
		const min = props.exact ? 0 : MIN_MARGIN;
		// Bawaan 1,97 cm di semua sisi (sama dengan dokumen Word sekolah).
		const side = Math.max(min, props.margin ?? 19.7);
		const top = Math.max(min, props.marginTop ?? side);
		const bottom = side;
		return { paper, width, height, side, top, bottom };
	});

	/**
	 * Halaman cetak bernama per ukuran kertas + margin. Margin dipasang di @page (bukan padding),
	 * sehingga lembar lanjutan juga bermargin dan lembar tegak/mendatar tidak saling mengecilkan.
	 */
	const pageName = computed(() => {
		const l = layout.value;
		const mm = (v: number) => Math.round(v * 10);
		return `${l.paper.toLowerCase()}-${props.landscape ? "l" : "p"}-${mm(l.top)}-${mm(l.side)}-${mm(l.bottom)}`;
	});

	useHead(() => {
		const l = layout.value;
		return {
			style: [{
				key: `page-${pageName.value}`,
				innerHTML: `@page ${pageName.value} { size: ${l.width}mm ${l.height}mm; margin: ${l.top}mm ${l.side}mm ${l.bottom}mm ${l.side}mm; ${marginBoxes.value} }`
			}]
		};
	});

	const style = computed(() => {
		const l = layout.value;
		return {
			width: `${l.width}mm`,
			minHeight: `${l.height}mm`,
			padding: `${l.top}mm ${l.side}mm ${l.bottom}mm ${l.side}mm`,
			page: pageName.value,
			"--page-h": `${l.height}mm`
		};
	});
</script>
