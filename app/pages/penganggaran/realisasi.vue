<template>
	<LayoutPageShell title="Realisasi Belanja">
		<template #actions>
			<UButton color="neutral" variant="outline" icon="i-lucide-file-spreadsheet" :loading="busy" :disabled="!data" @click="exportXlsx">
				Export Excel
			</UButton>
		</template>
		<div v-if="!connected" class="py-16 text-center text-muted">
			Hubungkan ARKAS terlebih dahulu.
		</div>
		<div v-else class="space-y-4">
			<div class="flex flex-wrap items-center gap-2">
				<USelect v-model="upto" :items="uptoItems" icon="i-lucide-calendar-check" class="w-52" aria-label="Acuan s.d. bulan" />
				<USelect v-model="status" :items="statusItems" class="w-52" />
				<UInput v-model="search" icon="i-lucide-search" placeholder="Cari item..." class="w-64" />
			</div>
			<UAlert v-if="error" color="error" variant="subtle" icon="i-lucide-triangle-alert" :title="error" />

			<div v-if="data" class="grid grid-cols-2 lg:grid-cols-4 gap-3">
				<UCard :ui="{ body: 'p-3 sm:p-3' }">
					<p class="text-xs text-muted">
						Pagu
					</p>
					<p class="text-lg font-semibold tabular">
						{{ rupiah(data.totalPagu) }}
					</p>
				</UCard>
				<UCard :ui="{ body: 'p-3 sm:p-3' }">
					<p class="text-xs text-muted">
						Realisasi
					</p>
					<p class="text-lg font-semibold tabular">
						{{ rupiah(data.totalRealisasi) }}
					</p>
					<UProgress :model-value="Math.min(100, data.persen)" size="sm" class="mt-1" />
				</UCard>
				<UCard :ui="{ body: 'p-3 sm:p-3' }">
					<p class="text-xs text-muted">
						Sisa anggaran
					</p>
					<p class="text-lg font-semibold tabular">
						{{ rupiah(data.totalPagu - data.totalRealisasi) }}
					</p>
				</UCard>
				<UCard :ui="{ body: 'p-3 sm:p-3' }">
					<p class="text-xs text-muted">
						Jatuh tempo belum dibelanjakan
					</p>
					<p class="text-lg font-semibold tabular" :class="data.totalTertunda ? 'text-warning' : ''">
						{{ rupiah(data.totalTertunda) }}
					</p>
				</UCard>
			</div>

			<UTable
				:data="rows"
				:columns="columns"
				sticky
				empty="Tidak ada item."
				:meta="{ class: { tr: (row) => row.original.luar ? 'bg-warning/5' : '' } }"
				class="max-h-[calc(100vh-24rem)] border border-default rounded-md"
				:ui="{ td: 'py-1.5 text-sm align-top', th: 'py-2 text-xs' }"
			>
				<template #kegiatan-cell="{ row }">
					<template v-if="row.original.item">
						<p class="font-mono text-xs">
							{{ row.original.item.kodeKegiatan }}
						</p>
						<p class="text-xs text-muted whitespace-normal">
							{{ row.original.item.namaKegiatan }}
						</p>
					</template>
					<span v-else class="text-xs text-warning">Di luar RKAS aktif</span>
				</template>
				<template #uraian-cell="{ row }">
					<p class="whitespace-normal">
						{{ row.original.uraian }}
					</p>
					<p v-if="row.original.item" class="text-xs text-muted font-mono">
						{{ row.original.item.kodeRekening }}
					</p>
				</template>
				<template #status-cell="{ row }">
					<template v-if="row.original.item">
						<UBadge :color="statusColor[row.original.item.status]" variant="subtle" size="sm">
							{{ statusLabel[row.original.item.status] }}
						</UBadge>
						<p v-if="row.original.item.tertunda" class="text-xs text-warning tabular mt-0.5">
							tertunda {{ angka(row.original.item.tertunda) }}
						</p>
					</template>
					<span v-else-if="row.original.luar" class="text-xs tabular">{{ tanggalId(row.original.luar.tanggal) }}</span>
				</template>
			</UTable>
		</div>
	</LayoutPageShell>
</template>

<script lang="ts" setup>
	import type { TableColumn } from "@nuxt/ui";

	const { connected, year, fund } = useArkas();
	const { busy, xlsx } = useSaveFile();

	const data = ref<Realisasi | null>(null);
	const error = ref("");
	const upto = ref(new Date().getMonth() + 1);
	const status = ref<"semua" | RealisasiStatus>("semua");
	const search = ref("");

	const uptoItems = BULAN.map((label, i) => ({ label: `Acuan s.d. ${label}`, value: i + 1 }));
	const statusLabel: Record<RealisasiStatus, string> = {
		lunas: "Lunas",
		sebagian: "Sebagian",
		belum: "Belum dibelanjakan",
		melampaui: "Melampaui pagu",
		belum_jatuh_tempo: "Belum jatuh tempo"
	};
	const statusColor: Record<RealisasiStatus, "success" | "info" | "warning" | "error" | "neutral"> = {
		lunas: "success",
		sebagian: "info",
		belum: "warning",
		melampaui: "error",
		belum_jatuh_tempo: "neutral"
	};
	const statusItems = [{ label: "Semua status", value: "semua" }, ...Object.entries(statusLabel).map(([value, label]) => ({ label, value }))];

	const fundArg = computed(() => (fund.value === ALL_FUNDS ? null : fund.value));

	const load = async () => {
		if (!connected.value || !year.value) return;
		error.value = "";
		try {
			data.value = await api.realisasi(year.value, upto.value, fundArg.value);
		} catch (err) {
			error.value = errorMessage(err);
		}
	};
	watch(year, (y) => {
		if (y && y < new Date().getFullYear()) upto.value = 12;
	}, { immediate: true });
	watch([upto, year, fund, connected], load, { immediate: true });

	const filtered = computed(() => {
		const q = search.value.trim().toLowerCase();
		return (data.value?.items ?? []).filter((i) =>
			(status.value === "semua" || i.status === status.value)
			&& (!q || [i.uraian, i.kodeRekening, i.kodeKegiatan, i.namaKegiatan].some((v) => v?.toLowerCase().includes(q))));
	});
	const outside = computed(() => (status.value === "semua" && !search.value ? data.value?.luarRkas ?? [] : []));

	/** Baris tabel: item RKAS atau belanja di luar RKAS aktif. */
	interface Baris { key: string, uraian: string, realisasi: number, item?: RealisasiItem, luar?: OutsideItem }
	const rows = computed<Baris[]>(() => [
		...filtered.value.map((i) => ({ key: i.idRapbs, uraian: i.uraian, realisasi: i.totalRealisasi, item: i })),
		...outside.value.map((o) => ({ key: `luar-${o.id}`, uraian: o.uraian, realisasi: o.nominal, luar: o }))
	]);
	const nominal = { th: "text-right", td: "text-right tabular" };
	const nilai = (v: number | undefined) => (v === undefined ? "" : angka(v));
	const columns: TableColumn<Baris>[] = [
		{ id: "kegiatan", header: "Kegiatan / Rekening" },
		{ accessorKey: "uraian", header: "Uraian", meta: { class: { td: "min-w-64" } } },
		{ id: "pagu", header: "Pagu", cell: ({ row }) => nilai(row.original.item?.pagu), meta: { class: nominal } },
		{ id: "rencanaSd", header: "Rencana s.d.", cell: ({ row }) => nilai(row.original.item?.rencanaSd), meta: { class: nominal } },
		{ accessorKey: "realisasi", header: "Realisasi", cell: ({ row }) => angka(row.original.realisasi), meta: { class: nominal } },
		{ id: "sisa", header: "Sisa", cell: ({ row }) => nilai(row.original.item?.sisa), meta: { class: nominal } },
		{ id: "status", header: "Status", meta: { class: { th: "w-36" } } }
	];

	const exportXlsx = () => {
		if (!year.value) return;
		const y = year.value;
		xlsx(`Realisasi_${y}_sd_${BULAN[upto.value - 1]}`, (path) => api.exportRealisasiXlsx(y, upto.value, fundArg.value, path));
	};
</script>
