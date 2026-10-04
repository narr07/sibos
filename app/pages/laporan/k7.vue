<template>
	<LayoutPageShell title="Laporan K7 / K7a" :picker="false">
		<template #actions>
			<UButton icon="i-lucide-printer" :disabled="!rows.length" @click="print">
				Cetak K7
			</UButton>
		</template>
		<div v-if="!connected" class="py-16 text-center text-muted">
			Hubungkan ARKAS terlebih dahulu.
		</div>
		<div v-else class="space-y-4">
			<div class="flex flex-wrap items-center gap-2">
				<USelect :model-value="year ?? undefined" :items="yearItems" class="w-32" @update:model-value="(v) => v && setYear(Number(v))" />
				<LaporanPeriodPicker v-model="range" :modes="['triwulan', 'semester', 'bulan', 'tahun']" initial="semester" />
			</div>
			<UAlert v-if="error" color="error" variant="subtle" icon="i-lucide-triangle-alert" :title="error" />
			<p class="text-sm text-muted">
				Rekapitulasi realisasi per standar (program) untuk setiap sumber dana · {{ periodLabel }}
			</p>

			<UTable
				:data="lines"
				:columns="columns"
				:meta="{ class: { tr: (row) => row.original.bold ? 'font-semibold' : '' } }"
				class="border border-default rounded-md"
				:ui="{ td: 'py-1.5 text-sm', th: 'py-2 text-xs' }"
			/>
		</div>
	</LayoutPageShell>
</template>

<script lang="ts" setup>
	import type { TableColumn } from "@nuxt/ui";
	import K7 from "~/components/Print/Laporan/K7.vue";

	const { connected, year, years, setYear } = useArkas();
	const { open } = usePrint();

	const range = ref({ start: 1, end: 6 });
	const rows = ref<{ fund: FundSource, s: PeriodSummary }[]>([]);
	const error = ref("");

	const yearItems = computed(() => years.value.map((y) => ({ label: `TA ${y}`, value: y })));
	const periodLabel = computed(() => (year.value ? labelPeriode(year.value, range.value.start, range.value.end) : ""));

	const load = async () => {
		if (!connected.value || !year.value) return;
		error.value = "";
		try {
			const data = await api.periodSummaryByFund(year.value, range.value.start, range.value.end);
			rows.value = data.map(([fund, s]) => ({ fund, s }));
		} catch (err) {
			error.value = errorMessage(err);
		}
	};
	watch([range, year, connected], load, { immediate: true, deep: true });

	const lines = computed(() => {
		const programs = new Map<string, string>();
		for (const r of rows.value) {
			for (const p of r.s.belanjaProgram) {
				if (!programs.get(p.kode)) programs.set(p.kode, p.nama);
			}
		}
		const per = (fn: (s: PeriodSummary) => number) => rows.value.map((r) => fn(r.s));
		return [
			{ label: "Saldo periode sebelumnya", values: per((s) => s.saldoAwalBank + s.saldoAwalTunai), bold: true },
			{ label: "Penerimaan periode ini", values: per((s) => s.totalPenerimaan), bold: true },
			...[...programs.entries()].sort(([a], [b]) => a.localeCompare(b)).map(([kode, nama]) => ({
				label: `${kode} ${nama}`,
				values: per((s) => s.belanjaProgram.find((p) => p.kode === kode)?.total ?? 0),
				bold: false
			})),
			{ label: "Pajak bunga, pengembalian, dan lain-lain", values: per((s) => s.pajakBunga + s.pengembalian), bold: false },
			{ label: "Jumlah pengeluaran", values: per((s) => s.totalBelanja + s.pajakBunga + s.pengembalian), bold: true },
			{ label: "Saldo akhir periode", values: per((s) => s.saldoAkhirBank + s.saldoAkhirTunai), bold: true }
		];
	});

	const nominal = { th: "text-right", td: "text-right tabular" };
	type Line = (typeof lines.value)[number];
	const columns = computed<TableColumn<Line>[]>(() => [
		{ accessorKey: "label", header: "Uraian / Standar", meta: { class: { td: "whitespace-normal" } } },
		...rows.value.map((r, i) => ({ id: `f${i}`, header: r.fund.name, cell: ({ row }: { row: { original: Line } }) => angka(row.original.values[i] ?? 0), meta: { class: nominal } })),
		{ id: "jumlah", header: "Jumlah", cell: ({ row }) => angka(row.original.values.reduce((a, b) => a + b, 0)), meta: { class: nominal } }
	]);

	const print = () => {
		if (!year.value) return;
		open({ title: "Laporan K7", component: K7, landscape: true, props: { funds: rows.value, periodLabel: periodLabel.value, year: year.value, end: range.value.end } });
	};
</script>
