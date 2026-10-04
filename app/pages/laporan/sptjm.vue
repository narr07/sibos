<template>
	<LayoutPageShell title="Cetak SPTJM">
		<template #actions>
			<UButton icon="i-lucide-printer" :disabled="!summary" @click="print">
				Cetak SPTJM
			</UButton>
		</template>
		<div v-if="!connected" class="py-16 text-center text-muted">
			Hubungkan ARKAS terlebih dahulu.
		</div>
		<div v-else class="space-y-4">
			<LaporanPeriodPicker v-model="range" :modes="['semester', 'bulan', 'triwulan', 'tahun']" initial="semester" />
			<UAlert v-if="error" color="error" variant="subtle" icon="i-lucide-triangle-alert" :title="error" />
			<p class="text-sm text-muted">
				Surat Pernyataan Tanggung Jawab Mutlak · {{ fundLabel }} · {{ periodLabel }}
			</p>
			<LaporanSummaryView v-if="summary" :s="summary" />
		</div>
	</LayoutPageShell>
</template>

<script lang="ts" setup>
	import Sptjm from "~/components/Print/Laporan/Sptjm.vue";

	const { connected, range, summary, error, fundLabel, periodLabel } = usePeriodSummary();
	const { open } = usePrint();

	const print = () => {
		if (!summary.value) return;
		open({ title: "SPTJM", component: Sptjm, props: { s: summary.value, fundLabel: fundLabel.value, periodLabel: periodLabel.value } });
	};
</script>
