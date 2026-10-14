<template>
	<PrintSppdSuratTugas v-if="jenis.includes('tugas')" :d="d" :pelaksana="pelaksana" />
	<template v-for="orang in pelaksana" :key="orang.id">
		<template v-for="j in jenis" :key="j">
			<PrintSppdSpd v-if="j === 'spd'" :d="d" :orang="orang" />
			<PrintSppdKuitansi v-else-if="j === 'kuitansi'" :d="d" :orang="orang" />
			<PrintSppdRincian v-else-if="j === 'rincian'" :d="d" :orang="orang" />
			<PrintSppdRiil v-else-if="j === 'riil'" :d="d" :orang="orang" />
			<PrintSppdLaporan v-else-if="j === 'laporan'" :d="d" :orang="orang" />
		</template>
	</template>
</template>

<script lang="ts" setup>
	/** Berkas SPPD: satu surat tugas untuk semua pelaksana, lalu dokumen per orang berurutan. */
	const props = defineProps<{
		d: SppdData
		/** Seluruh guru dari Pengaturan; pelaksana ditentukan `d.pelaksana`. */
		guru: Guru[]
		jenis: JenisDokumenSppd[]
	}>();

	const pelaksana = computed(() => props.guru.filter((g) => props.d.pelaksana.includes(g.id)));
</script>
