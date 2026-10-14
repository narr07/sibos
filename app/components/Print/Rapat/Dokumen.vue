<template>
	<template v-for="j in jenis" :key="j">
		<PrintRapatUndangan v-if="j === 'undangan'" :r="r" />
		<PrintRapatDaftarHadir v-else-if="j === 'hadir'" :r="r" :hadir="hadir" />
		<PrintRapatBeritaAcara v-else-if="j === 'ba'" :r="r" :jumlah-hadir="jumlahHadir" />
		<PrintRapatNotulen v-else-if="j === 'notulen'" :r="r" :jumlah-hadir="jumlahHadir" />
	</template>
</template>

<script lang="ts" setup>
	/** Satu berkas rapat: dokumen terpilih dicetak berurutan. */
	const props = defineProps<{
		r: RapatData
		/** Seluruh guru dari Pengaturan; yang hadir ditentukan `r.peserta`. */
		guru: Guru[]
		jenis: JenisDokumenRapat[]
	}>();

	const hadir = computed(() => props.guru.filter((g) => props.r.peserta.includes(g.id)));
	const jumlahHadir = computed(() => hadir.value.length + Math.max(0, props.r.barisKosong));
</script>
