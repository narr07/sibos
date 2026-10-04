<template>
	<Teleport to="body">
		<div v-if="job" id="print-root" class="fixed inset-0 z-[100] flex flex-col bg-elevated">
			<div class="print-toolbar flex items-center gap-2 border-b border-default bg-default px-4 py-2">
				<UIcon name="i-lucide-printer" class="size-5 text-primary" />
				<p class="font-medium">
					{{ job.title }}
				</p>
				<UBadge color="neutral" variant="subtle" class="ml-2">
					{{ kertas }} {{ job.landscape ? "landscape" : "portrait" }}
				</UBadge>
				<p class="text-xs text-muted ml-2 hidden lg:block">
					Untuk PDF, pilih printer "Microsoft Print to PDF".
				</p>
				<div class="ml-auto flex gap-2">
					<UButton icon="i-lucide-printer" @click="doPrint">
						Cetak / PDF
					</UButton>
					<UButton color="neutral" variant="outline" icon="i-lucide-x" @click="close">
						Tutup
					</UButton>
				</div>
			</div>
			<div class="print-scroll flex-1 overflow-auto py-6">
				<div class="print-area mx-auto flex flex-col items-center gap-6">
					<component :is="job.component" v-bind="job.props" />
				</div>
			</div>
		</div>
	</Teleport>
</template>

<script lang="ts" setup>
	const { job, close } = usePrint();
	const { pengaturan, load } = useDocContext();

	// Muat ulang kop, pejabat, dan data sekolah setiap kali pratinjau dibuka.
	watch(job, (j) => {
		if (j) load().catch(() => {});
	});

	const kertas = computed(() => pengaturan.value?.cetak.kertas || "A4");

	// Ukuran halaman default; tiap lembar memakai @page bernama sendiri (ukuran + margin, lihat Sheet.vue).
	const pageStyle = computed(() => {
		const [w, h] = kertas.value === "F4" ? ["215mm", "330mm"] : ["210mm", "297mm"];
		const size = job.value?.landscape ? `${h} ${w}` : `${w} ${h}`;
		// Kotak margin: judul dokumen + nomor halaman, menggantikan header/footer bawaan browser (alamat URL, tanggal).
		const boxes = "@top-left { content: \"\" } @top-center { content: \"\" } @top-right { content: \"\" } @bottom-center { content: \"\" } "
			+ `@bottom-left { content: ${teksCss(job.value?.title ?? "")}; font: 8pt Arial, sans-serif; color: #555; } `
			+ "@bottom-right { content: \"Halaman \" counter(page); font: 8pt Arial, sans-serif; color: #555; }";
		return `@page { size: ${size}; margin: 10mm; ${boxes} }`;
	});

	useHead(() => ({ style: job.value ? [{ key: "print-page", innerHTML: pageStyle.value }] : [] }));

	const doPrint = async () => {
		const current = job.value;
		window.print();
		await current?.onPrinted?.();
	};

	onKeyStroke("Escape", () => {
		if (job.value) close();
	});
</script>
