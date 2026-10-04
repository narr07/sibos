<template>
	<LayoutPageShell title="BA Rekonsiliasi">
		<template #actions>
			<UButton color="neutral" variant="outline" icon="i-lucide-settings" to="/lainnya/pengaturan">
				Nomor & penandatangan
			</UButton>
			<UButton icon="i-lucide-printer" :disabled="!summary" @click="print">
				Cetak BA
			</UButton>
		</template>
		<div v-if="!connected" class="py-16 text-center text-muted">
			Hubungkan ARKAS terlebih dahulu.
		</div>
		<div v-else class="space-y-4">
			<LaporanPeriodPicker v-model="range" initial="semester" />
			<UAlert v-if="error" color="error" variant="subtle" icon="i-lucide-triangle-alert" :title="error" />
			<p class="text-sm text-muted">
				Berita Acara Rekonsiliasi · {{ fundLabel }} · {{ periodLabel }}
			</p>
			<LaporanSummaryView v-if="summary" :s="summary" />
			<UCard v-if="summary?.belanjaRekening.length">
				<template #header>
					<p class="font-medium">
						Rincian per kode rekening
					</p>
				</template>
				<table class="w-full text-sm">
					<tbody>
						<tr v-for="r in summary.belanjaRekening" :key="r.kode" class="border-b border-default last:border-0">
							<td class="py-1.5 font-mono text-xs w-44">
								{{ r.kode }}
							</td>
							<td class="py-1.5">
								{{ r.nama }}
							</td>
							<td class="py-1.5 text-right tabular">
								{{ angka(r.total) }}
							</td>
						</tr>
					</tbody>
				</table>
			</UCard>
		</div>
	</LayoutPageShell>
</template>

<script lang="ts" setup>
	import BaRekon from "~/components/Print/Laporan/BaRekon.vue";

	const { connected, range, summary, error, fundLabel, periodLabel } = usePeriodSummary();
	const { open } = usePrint();

	const print = () => {
		if (!summary.value) return;
		open({ title: "BA Rekonsiliasi", component: BaRekon, props: { s: summary.value, fundLabel: fundLabel.value, periodLabel: periodLabel.value } });
	};
</script>
