<template>
	<UAlert icon="i-lucide-receipt-text" color="neutral" variant="subtle" :title="`${g.noBukti || 'Tanpa no. bukti'} · ${tanggalPanjang(g.tanggal)} · ${rupiah(g.total)}`">
		<template #description>
			<p v-if="g.nota?.namaToko">
				{{ g.nota.namaToko }}
			</p>
			<p>{{ g.items.map(ringkasItem).join("; ") }}</p>
			<p v-if="namaKegiatan">
				Kegiatan RKAS: {{ namaKegiatan }}
			</p>
		</template>
	</UAlert>
</template>

<script lang="ts" setup>
	/** Ringkasan belanja BKU yang menjadi sumber dokumen. */
	defineProps<{ g: NotaGroup, namaKegiatan?: string }>();
</script>
