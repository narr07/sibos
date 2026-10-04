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

			<div class="border border-default rounded-md overflow-auto max-h-[calc(100vh-24rem)]">
				<table class="w-full text-sm">
					<thead class="sticky top-0 bg-elevated text-xs text-muted z-10">
						<tr>
							<th class="p-2 text-left">
								Kegiatan / Rekening
							</th>
							<th class="p-2 text-left">
								Uraian
							</th>
							<th class="p-2 text-right">
								Pagu
							</th>
							<th class="p-2 text-right">
								Rencana s.d.
							</th>
							<th class="p-2 text-right">
								Realisasi
							</th>
							<th class="p-2 text-right">
								Sisa
							</th>
							<th class="p-2 w-36">
								Status
							</th>
						</tr>
					</thead>
					<tbody>
						<tr v-for="i in filtered" :key="i.idRapbs" class="border-t border-default align-top">
							<td class="p-2">
								<p class="font-mono text-xs">
									{{ i.kodeKegiatan }}
								</p>
								<p class="text-xs text-muted">
									{{ i.namaKegiatan }}
								</p>
							</td>
							<td class="p-2">
								{{ i.uraian }}
								<p class="text-xs text-muted font-mono">
									{{ i.kodeRekening }}
								</p>
							</td>
							<td class="p-2 text-right tabular">
								{{ angka(i.pagu) }}
							</td>
							<td class="p-2 text-right tabular">
								{{ angka(i.rencanaSd) }}
							</td>
							<td class="p-2 text-right tabular">
								{{ angka(i.totalRealisasi) }}
							</td>
							<td class="p-2 text-right tabular">
								{{ angka(i.sisa) }}
							</td>
							<td class="p-2">
								<UBadge :color="statusColor[i.status]" variant="subtle" size="sm">
									{{ statusLabel[i.status] }}
								</UBadge>
								<p v-if="i.tertunda" class="text-xs text-warning tabular mt-0.5">
									tertunda {{ angka(i.tertunda) }}
								</p>
							</td>
						</tr>
						<tr v-for="o in outside" :key="o.id" class="border-t border-default bg-warning/5">
							<td class="p-2 text-xs text-warning">
								Di luar RKAS aktif
							</td>
							<td class="p-2">
								{{ o.uraian }}
							</td>
							<td />
							<td />
							<td class="p-2 text-right tabular">
								{{ angka(o.nominal) }}
							</td>
							<td />
							<td class="p-2 text-xs tabular">
								{{ tanggalId(o.tanggal) }}
							</td>
						</tr>
					</tbody>
				</table>
			</div>
		</div>
	</LayoutPageShell>
</template>

<script lang="ts" setup>
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

	const exportXlsx = () => {
		if (!year.value) return;
		const y = year.value;
		xlsx(`Realisasi_${y}_sd_${BULAN[upto.value - 1]}`, (path) => api.exportRealisasiXlsx(y, upto.value, fundArg.value, path));
	};
</script>
