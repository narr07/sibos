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
				<UTable :data="summary.belanjaRekening" :columns="rekeningColumns" :ui="{ td: 'py-1.5 text-sm', th: 'py-2 text-xs' }" />
			</UCard>
		</div>
	</LayoutPageShell>
</template>

<script lang="ts" setup>
	import type { TableColumn } from "@nuxt/ui";
	import BaRekon from "~/components/Print/Laporan/BaRekon.vue";

	const { connected, range, summary, error, fundLabel, periodLabel } = usePeriodSummary();
	const { open } = usePrint();

	const rekeningColumns: TableColumn<Total>[] = [
		{ accessorKey: "kode", header: "Kode rekening", meta: { class: { td: "font-mono text-xs w-44" } } },
		{ accessorKey: "nama", header: "Uraian", meta: { class: { td: "whitespace-normal" } } },
		{ accessorKey: "total", header: "Jumlah", cell: ({ row }) => angka(row.original.total), meta: { class: { th: "text-right", td: "text-right tabular" } } }
	];

	const print = () => {
		if (!summary.value) return;
		open({ title: "BA Rekonsiliasi", component: BaRekon, props: { s: summary.value, fundLabel: fundLabel.value, periodLabel: periodLabel.value } });
	};
</script>
