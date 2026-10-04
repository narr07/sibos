<template>
	<template v-for="item in pages" :key="item.key">
		<PrintDocRender :template="item.template.data" :g="item.g" :penyedia="item.penyedia" :urut="item.urut" />
	</template>
</template>

<script lang="ts" setup>
	/** Cetak beberapa dokumen untuk beberapa nota: per nota, dokumen berurutan sesuai pilihan. */
	const props = defineProps<{
		groups: NotaGroup[]
		/** Per jenis dokumen yang dipilih, urut sesuai cetak. */
		jenis: JenisDokumen[]
		templates: DocTemplate[]
		penyedia: Penyedia[]
		/** Nomor urut tiap nota dalam tahun (kunci = key nota). */
		urutan: Record<string, number>
	}>();

	const findPenyedia = (nama: string | null | undefined) => {
		if (!nama) return null;
		const n = nama.trim().toLowerCase();
		return props.penyedia.find((p) => p.nama.trim().toLowerCase() === n)?.data ?? null;
	};

	const pages = computed(() => props.groups.flatMap((g) => props.jenis.flatMap((j) => {
		const template = templateUntuk(props.templates, j, g.nota?.namaToko);
		if (!template) return [];
		return [{ key: `${g.key}:${j}`, g, template, penyedia: findPenyedia(g.nota?.namaToko), urut: props.urutan[g.key] ?? 1 }];
	})));
</script>
