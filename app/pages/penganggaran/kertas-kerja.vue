<template>
	<LayoutPageShell title="Kertas Kerja (RKAS)">
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
				<UInput v-model="search" icon="i-lucide-search" placeholder="Cari kegiatan, rekening, atau barang..." class="w-80" />
				<USwitch v-model="showMonths" label="Tampilkan per bulan" />
				<span class="ml-auto text-sm text-muted">{{ filteredItems.length }} item · {{ rupiah(total) }}</span>
			</div>
			<UAlert v-if="error" color="error" variant="subtle" icon="i-lucide-triangle-alert" :title="error" />

			<div v-if="data" class="flex flex-wrap gap-2">
				<UBadge v-for="a in activeAnggaran" :key="a.idAnggaran" color="neutral" variant="subtle">
					{{ a.fundName }}: {{ rupiah(a.jumlah) }} · revisi {{ a.isRevisi }}
				</UBadge>
			</div>

			<div class="border border-default rounded-md overflow-auto max-h-[calc(100vh-19rem)]">
				<table class="w-full text-sm">
					<thead class="sticky top-0 bg-elevated text-xs text-muted z-10">
						<tr>
							<th class="p-2 text-left">
								Kode
							</th>
							<th class="p-2 text-left">
								Uraian
							</th>
							<th class="p-2 text-right">
								Volume
							</th>
							<th class="p-2 text-left">
								Satuan
							</th>
							<th class="p-2 text-right">
								Harga
							</th>
							<th class="p-2 text-right">
								Jumlah
							</th>
							<template v-if="showMonths">
								<th v-for="m in BULAN_PENDEK" :key="m" class="p-2 text-right">
									{{ m }}
								</th>
							</template>
						</tr>
					</thead>
					<tbody>
						<template v-for="node in tree" :key="node.key">
							<tr :class="node.level === 0 ? 'bg-elevated/70 font-semibold' : node.level === 1 ? 'font-medium' : ''" class="border-t border-default">
								<td class="p-2 font-mono text-xs whitespace-nowrap" :style="{ paddingLeft: `${0.5 + node.level * 1}rem` }">
									{{ node.kode }}
								</td>
								<td class="p-2" :colspan="node.item ? 1 : 4">
									{{ node.label }}
								</td>
								<template v-if="node.item">
									<td class="p-2 text-right tabular">
										{{ node.item.volume }}
									</td>
									<td class="p-2">
										{{ node.item.satuan }}
									</td>
									<td class="p-2 text-right tabular">
										{{ angka(node.item.hargaSatuan) }}
									</td>
								</template>
								<td class="p-2 text-right tabular whitespace-nowrap">
									{{ angka(node.total) }}
								</td>
								<template v-if="showMonths">
									<td v-for="(v, i) in node.months" :key="i" class="p-2 text-right tabular text-xs">
										{{ v ? angka(v) : "" }}
									</td>
								</template>
							</tr>
						</template>
					</tbody>
				</table>
			</div>
		</div>
	</LayoutPageShell>
</template>

<script lang="ts" setup>
	interface Node {
		key: string
		level: number
		kode: string
		label: string
		total: number
		months: number[]
		item?: RkasItem
	}

	const BULAN_PENDEK = ["Jan", "Feb", "Mar", "Apr", "Mei", "Jun", "Jul", "Agu", "Sep", "Okt", "Nov", "Des"];

	const { connected, year, fund } = useArkas();
	const { busy, xlsx } = useSaveFile();

	const data = ref<KertasKerja | null>(null);
	const error = ref("");
	const search = ref("");
	const showMonths = ref(false);
	const fundArg = computed(() => (fund.value === ALL_FUNDS ? null : fund.value));

	const load = async () => {
		if (!connected.value || !year.value) return;
		error.value = "";
		try {
			data.value = await api.kertasKerja(year.value, fundArg.value);
		} catch (err) {
			error.value = errorMessage(err);
		}
	};
	watch([year, fund, connected], load, { immediate: true });

	const activeAnggaran = computed(() => {
		const best = new Map<number, AnggaranInfo>();
		for (const a of data.value?.anggaran ?? []) {
			if (!a.isApprove) continue;
			const cur = best.get(a.fundId);
			if (!cur || a.isRevisi > cur.isRevisi) best.set(a.fundId, a);
		}
		return [...best.values()].filter((a) => fundArg.value === null || a.fundId === fundArg.value);
	});

	const filteredItems = computed(() => {
		const items = data.value?.items ?? [];
		const q = search.value.trim().toLowerCase();
		if (!q) return items;
		const names = data.value?.kodeNames ?? {};
		return items.filter((i) => [i.uraian, i.kodeRekening, i.kodeKegiatan, i.kodeKegiatan ? names[i.kodeKegiatan] : ""]
			.some((v) => v?.toLowerCase().includes(q)));
	});

	const total = computed(() => filteredItems.value.reduce((s, i) => s + i.jumlah, 0));

	const prefixes = (kode: string) => {
		const parts = kode.split(".").filter(Boolean);
		return parts.map((_, n) => `${parts.slice(0, n + 1).join(".")}.`);
	};

	// Program -> komponen -> kegiatan -> item
	const tree = computed<Node[]>(() => {
		const names = data.value?.kodeNames ?? {};
		const rek = data.value?.rekeningNames ?? {};
		const items = [...filteredItems.value].sort((a, b) =>
			(a.kodeKegiatan ?? "").localeCompare(b.kodeKegiatan ?? "") || (a.kodeRekening ?? "").localeCompare(b.kodeRekening ?? ""));
		const out: Node[] = [];
		const index = new Map<string, Node>();
		for (const it of items) {
			const codes = it.kodeKegiatan ? prefixes(it.kodeKegiatan) : ["-"];
			codes.forEach((code, level) => {
				const existing = index.get(code);
				const node: Node = existing ?? {
					key: `k${code}`,
					level,
					kode: code,
					label: names[code] ?? (code === "-" ? "Tanpa kegiatan" : ""),
					total: 0,
					months: Array.from<number>({ length: 12 }).fill(0)
				};
				if (!existing) {
					index.set(code, node);
					out.push(node);
				}
				node.total += it.jumlah;
				node.months = node.months.map((v, m) => v + (it.bulan[m] ?? 0));
			});
			out.push({
				key: `i${it.idRapbs}`,
				level: codes.length,
				kode: it.kodeRekening ?? "",
				label: `${it.uraian}${it.kodeRekening && rek[it.kodeRekening] ? ` — ${rek[it.kodeRekening]}` : ""}`,
				total: it.jumlah,
				months: it.bulan,
				item: it
			});
		}
		return out;
	});

	const exportXlsx = () => {
		if (!year.value) return;
		const y = year.value;
		xlsx(`Kertas_Kerja_${y}`, (path) => api.exportKertasKerjaXlsx(y, fundArg.value, path));
	};
</script>
