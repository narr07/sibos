<template>
	<LayoutPageShell title="Kertas Kerja (RKAS)">
		<template #actions>
			<UButton color="neutral" variant="outline" icon="i-lucide-file-spreadsheet" :loading="busy" :disabled="!data" @click="exportXlsx">
				{{ search.trim() ? "Excel hasil pencarian" : "Export Excel" }}
			</UButton>
			<UButton icon="i-lucide-printer" :disabled="!data || !filteredItems.length" @click="printPdf">
				{{ search.trim() ? "PDF hasil pencarian" : "Cetak PDF" }}
			</UButton>
		</template>

		<UEmpty
			v-if="!connected"
			icon="i-lucide-plug-zap"
			title="ARKAS belum terhubung"
			description="Hubungkan ARKAS untuk melihat kertas kerja."
			:actions="[{ label: 'Hubungkan', icon: 'i-lucide-plug', to: '/setup' }]"
			class="py-16"
		/>

		<div v-else class="flex min-h-0 flex-1 flex-col gap-4">
			<div class="flex flex-wrap items-center gap-2">
				<USelect v-model="format" :items="formatItems" class="w-64" aria-label="Format RKAS" />
				<USelect v-if="format === 'bulanan'" v-model="bulan" :items="bulanItems" class="w-40" aria-label="Bulan" />
				<UInput
					v-model="search"
					icon="i-lucide-search"
					placeholder="Cari kegiatan, rekening, atau barang..."
					aria-label="Cari kegiatan, rekening, atau barang"
					class="w-full sm:w-80"
				/>
				<USwitch v-if="format === 'tahunan'" v-model="showMonths" label="Tampilkan per bulan" />
				<UFieldGroup>
					<UButton color="neutral" variant="outline" size="sm" icon="i-lucide-chevrons-up-down" @click="expanded = true">
						Buka semua
					</UButton>
					<UButton color="neutral" variant="outline" size="sm" icon="i-lucide-chevrons-down-up" @click="expanded = {}">
						Tutup semua
					</UButton>
				</UFieldGroup>
				<span class="ml-auto text-sm text-muted">{{ filteredItems.length }} item · {{ rupiah(total) }}</span>
			</div>
			<UAlert v-if="error" color="error" variant="subtle" icon="i-lucide-triangle-alert" :title="error" />

			<div v-if="data" class="flex flex-wrap gap-2">
				<UBadge v-for="a in activeAnggaran" :key="a.idAnggaran" color="neutral" variant="subtle">
					{{ a.fundName }}: {{ rupiah(a.jumlah) }} · revisi {{ a.isRevisi }}
				</UBadge>
			</div>

			<UTable
				v-model:expanded="expanded"
				:data="tree"
				:columns="columns"
				:get-sub-rows="(row) => row.children"
				:column-pinning="perBulan || perTriwulan ? { left: ['kode', 'label'] } : undefined"
				:loading="loading"
				sticky
				empty="Tidak ada item RKAS."
				:meta="{ class: { tr: (row) => row.original.item ? '' : row.depth === 0 ? 'bg-elevated/70 font-semibold' : 'font-medium' } }"
				class="min-h-60 flex-1 border border-default rounded-md"
				:ui="{ td: 'py-1.5 text-sm', th: 'py-2 text-xs', tfoot: 'bg-elevated font-semibold' }"
			/>
		</div>
	</LayoutPageShell>
</template>

<script lang="ts" setup>
	import type { TableColumn } from "@nuxt/ui";
	import type { ExpandedState } from "@tanstack/vue-table";
	import LembarKertasKerja from "~/components/Print/Laporan/LembarKertasKerja.vue";
	import RkasTahunan from "~/components/Print/Laporan/RkasTahunan.vue";

	interface Node {
		key: string
		kode: string
		label: string
		total: number
		months: number[]
		item?: RkasItem
		children?: Node[]
	}

	const BULAN_PENDEK = ["Jan", "Feb", "Mar", "Apr", "Mei", "Jun", "Jul", "Agu", "Sep", "Okt", "Nov", "Des"];
	const UButton = resolveComponent("UButton");

	const { connected, year, fund, funds } = useArkas();
	const { busy, xlsx } = useSaveFile();

	const data = ref<KertasKerja | null>(null);
	const error = ref("");
	const loading = ref(false);
	const search = ref("");
	const searchDebounced = refDebounced(search, 250);
	const showMonths = ref(false);
	const expanded = ref<ExpandedState>({});
	const fundArg = computed(() => (fund.value === ALL_FUNDS ? null : fund.value));

	// Token mencegah respons lama menimpa respons baru saat tahun/dana cepat diganti.
	let token = 0;
	const load = async () => {
		if (!connected.value || !year.value) return;
		const current = ++token;
		loading.value = true;
		error.value = "";
		try {
			const res = await api.kertasKerja(year.value, fundArg.value);
			if (current === token) data.value = res;
		} catch (err) {
			if (current === token) error.value = errorMessage(err);
		} finally {
			if (current === token) loading.value = false;
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

	// Format RKAS seperti format dinas: tahunan, triwulan, bulanan, atau lembar kertas kerja per triwulan.
	type Format = "tahunan" | "triwulan" | "bulanan" | "lembar";
	const ROMAWI = ["I", "II", "III", "IV"];
	const format = ref<Format>("tahunan");
	const bulan = ref(new Date().getMonth() + 1);
	const formatItems = [
		{ label: "Rincian RKAS (Tahunan)", value: "tahunan" },
		{ label: "Rincian RKAS (Triwulan)", value: "triwulan" },
		{ label: "Rincian RKAS (Bulanan)", value: "bulanan" },
		{ label: "Lembar Kertas Kerja (Triwulan)", value: "lembar" }
	];
	const bulanItems = BULAN.map((label, i) => ({ label, value: i + 1 }));
	const perBulan = computed(() => format.value === "tahunan" && showMonths.value);
	/** Triwulan & Lembar Kertas Kerja: semua item setahun dengan kolom per triwulan. */
	const perTriwulan = computed(() => format.value === "triwulan" || format.value === "lembar");

	/** Rentang bulan (1-12) untuk format yang dipilih. */
	const range = computed<[number, number]>(() => {
		if (format.value === "bulanan") return [bulan.value, bulan.value];
		return [1, 12];
	});
	const judul = computed(() => {
		if (format.value === "triwulan") return "Triwulan";
		if (format.value === "bulanan") return `Bulan ${BULAN[bulan.value - 1]}`;
		if (format.value === "lembar") return "Lembar Kertas Kerja (Triwulan)";
		return "Tahunan";
	});

	/** Item dengan jumlah & volume hanya untuk periode terpilih; item tanpa rencana di periode itu disembunyikan. */
	const periodItems = computed(() => {
		const items = data.value?.items ?? [];
		const [a, b] = range.value;
		if (a === 1 && b === 12) return items;
		const inRange = (m: number) => m >= a - 1 && m <= b - 1;
		return items
			.map((it) => ({
				...it,
				jumlah: it.bulan.reduce((s, v, m) => s + (inRange(m) ? v : 0), 0),
				volume: it.volumeBulan.reduce((s, v, m) => s + (inRange(m) ? v : 0), 0),
				bulan: it.bulan.map((v, m) => (inRange(m) ? v : 0))
			}))
			.filter((it) => it.jumlah > 0);
	});

	const filteredItems = computed(() => {
		const items = periodItems.value;
		const q = searchDebounced.value.trim().toLowerCase();
		if (!q) return items;
		const names = data.value?.kodeNames ?? {};
		return items.filter((i) => [i.uraian, i.kodeRekening, i.kodeKegiatan, i.kodeKegiatan ? names[i.kodeKegiatan] : ""]
			.some((v) => v?.toLowerCase().includes(q)));
	});

	const total = computed(() => filteredItems.value.reduce((s, i) => s + i.jumlah, 0));

	// Saat mencari, buka semua supaya hasil langsung terlihat; saat kosong, kembali terlipat.
	watch(searchDebounced, (q) => {
		expanded.value = q.trim() ? true : {};
	});

	const nominal = { th: "text-right", td: "text-right tabular whitespace-nowrap" };
	const columns = computed<TableColumn<Node>[]>(() => [
		{
			accessorKey: "kode",
			header: "Kode",
			meta: { class: { td: "whitespace-nowrap" } },
			cell: ({ row }) => h("div", { class: "flex items-center gap-1", style: { paddingLeft: `${row.depth}rem` } }, [
				row.getCanExpand()
					? h(UButton, {
						icon: row.getIsExpanded() ? "i-lucide-chevron-down" : "i-lucide-chevron-right",
						color: "neutral",
						variant: "ghost",
						size: "xs",
						"aria-label": row.getIsExpanded() ? "Tutup" : "Buka",
						onClick: () => row.toggleExpanded()
					})
					: h("span", { class: "w-6 shrink-0" }),
				h("span", { class: "font-mono text-xs" }, row.original.kode)
			]),
			footer: "JUMLAH"
		},
		{ accessorKey: "label", header: "Uraian", meta: { class: { td: "whitespace-normal min-w-64" } } },
		{ id: "volume", header: "Volume", cell: ({ row }) => row.original.item?.volume ?? "", meta: { class: nominal } },
		{ id: "satuan", header: "Satuan", cell: ({ row }) => row.original.item?.satuan ?? "" },
		{ id: "harga", header: "Harga", cell: ({ row }) => (row.original.item ? angka(row.original.item.hargaSatuan) : ""), meta: { class: nominal } },
		{
			accessorKey: "total",
			header: "Jumlah",
			cell: ({ row }) => angka(row.original.total),
			footer: () => h("div", { class: "text-right tabular" }, angka(total.value)),
			meta: { class: nominal }
		},
		...(perTriwulan.value
			? ROMAWI.map((r, k): TableColumn<Node> => {
				const tw = (months: number[]) => months.slice(k * 3, k * 3 + 3).reduce((s, v) => s + v, 0);
				return {
					id: `tw${k}`,
					header: `Triwulan ${r}`,
					cell: ({ row }) => (tw(row.original.months) ? angka(tw(row.original.months)) : ""),
					footer: () => h("div", { class: "text-right tabular" }, angka(filteredItems.value.reduce((s, it) => s + tw(it.bulan), 0))),
					meta: { class: nominal }
				};
			})
			: []),
		...(perBulan.value
			? BULAN_PENDEK.map((m, i): TableColumn<Node> => ({
				id: `m${i}`,
				header: m,
				cell: ({ row }) => (row.original.months[i] ? angka(row.original.months[i]) : ""),
				footer: () => h("div", { class: "text-right tabular text-xs" }, angka(filteredItems.value.reduce((s, it) => s + (it.bulan[i] ?? 0), 0))),
				meta: { class: { th: "text-right", td: "text-right tabular text-xs" } }
			}))
			: [])
	]);

	const prefixes = (kode: string) => {
		const parts = kode.split(".").filter(Boolean);
		return parts.map((_, n) => `${parts.slice(0, n + 1).join(".")}.`);
	};

	// Program -> komponen -> kegiatan -> item, bersarang lewat `children` untuk baris yang bisa dilipat.
	const tree = computed<Node[]>(() => {
		const names = data.value?.kodeNames ?? {};
		const rek = data.value?.rekeningNames ?? {};
		const items = [...filteredItems.value].sort((a, b) =>
			(a.kodeKegiatan ?? "").localeCompare(b.kodeKegiatan ?? "") || (a.kodeRekening ?? "").localeCompare(b.kodeRekening ?? ""));
		const roots: Node[] = [];
		const index = new Map<string, Node>();
		for (const it of items) {
			const codes = it.kodeKegiatan ? prefixes(it.kodeKegiatan) : ["-"];
			let siblings = roots;
			for (const code of codes) {
				let node = index.get(code);
				if (!node) {
					node = {
						key: `k${code}`,
						kode: code,
						label: names[code] ?? (code === "-" ? "Tanpa kegiatan" : ""),
						total: 0,
						months: Array.from<number>({ length: 12 }).fill(0),
						children: []
					};
					index.set(code, node);
					siblings.push(node);
				}
				node.total += it.jumlah;
				node.months = node.months.map((v, m) => v + (it.bulan[m] ?? 0));
				siblings = node.children!;
			}
			siblings.push({
				key: `i${it.idRapbs}`,
				kode: it.kodeRekening ?? "",
				label: `${it.uraian}${it.kodeRekening && rek[it.kodeRekening] ? ` — ${rek[it.kodeRekening]}` : ""}`,
				total: it.jumlah,
				months: it.bulan,
				item: it
			});
		}
		return roots;
	});

	const { open: openPrint } = usePrint();
	const printPdf = () => {
		if (!year.value || !data.value) return;
		const fundLabel = funds.value.find((f) => f.id === fund.value)?.name ?? "Semua sumber dana";
		if (format.value === "lembar") {
			openPrint({ title: `Lembar Kertas Kerja ${year.value}`, component: LembarKertasKerja, props: { items: filteredItems.value, year: year.value } });
			return;
		}
		openPrint({
			title: `RKAS - ${judul.value} ${year.value}`,
			landscape: true,
			component: RkasTahunan,
			props: { items: filteredItems.value, kodeNames: data.value.kodeNames, fundLabel, year: year.value, funds: funds.value, judul: judul.value, kolom: format.value === "triwulan" ? "triwulan" : "alokasi" }
		});
	};

	const exportXlsx = () => {
		if (!year.value) return;
		const y = year.value;
		const q = search.value.trim();
		const [start, end] = range.value;
		const nama = `RKAS_${y}_${fileSafe(judul.value)}${q ? `_${fileSafe(q)}` : ""}`;
		xlsx(nama, (path) => api.exportKertasKerjaXlsx(y, fundArg.value, q, start, end, judul.value, format.value === "triwulan", format.value === "lembar", path));
	};
</script>
