<template>
	<LayoutPageShell title="Cari Barang & Rekening">
		<UEmpty
			v-if="!connected"
			icon="i-lucide-plug-zap"
			title="ARKAS belum terhubung"
			description="Hubungkan ARKAS untuk mencari katalog standar harga."
			:actions="[{ label: 'Hubungkan', icon: 'i-lucide-plug', to: '/setup' }]"
			class="py-16"
		/>

		<div v-else class="flex min-h-0 flex-1 flex-col gap-3 max-w-6xl">
			<div>
				<UInput
					ref="input"
					v-model="keyword"
					icon="i-lucide-search"
					size="lg"
					:loading="loading"
					placeholder="Cari nama barang atau kode rekening"
					aria-label="Cari nama barang atau kode rekening"
					class="w-full"
				/>
				<p class="text-xs text-muted mt-1.5">
					Katalog standar harga dari database ARKAS. Contoh: spidol, kertas, 5.1.02.01. Klik kanan baris untuk menyalin.
				</p>
			</div>

			<UAlert v-if="error" color="error" variant="subtle" icon="i-lucide-triangle-alert" title="Pencarian gagal" :description="error" />

			<UEmpty
				v-else-if="searched && !results.length && !loading"
				icon="i-lucide-search-x"
				title="Tidak ada hasil"
				:description="`Tidak ditemukan barang atau rekening untuk '${keyword.trim()}'.`"
			/>

			<UEmpty
				v-else-if="!searched && !loading"
				icon="i-lucide-search"
				title="Mulai mencari"
				description="Ketik minimal 2 karakter untuk mencari barang atau kode rekening."
			/>

			<div v-else-if="results.length" class="flex min-h-0 flex-1 flex-col gap-3">
				<div class="flex items-center gap-2">
					<UBadge color="neutral" variant="subtle">
						{{ results.length }} hasil
					</UBadge>
					<UBadge v-if="results.length >= LIMIT" color="warning" variant="subtle" icon="i-lucide-triangle-alert">
						Dibatasi {{ LIMIT }} hasil, persempit kata kunci
					</UBadge>
				</div>
				<UContextMenu :items="contextItems" class="min-h-0 flex-1">
					<UTable
						:data="results"
						:columns="columns"
						:loading="loading"
						sticky
						class="min-h-60 flex-1 border border-default rounded-md"
						:ui="{ td: 'py-1.5 text-sm', th: 'py-2 text-xs' }"
						@contextmenu="onContextmenu"
					>
						<template #kodeRekening-cell="{ row }">
							<UButton
								v-if="row.original.kodeRekening"
								size="xs"
								color="neutral"
								variant="ghost"
								:icon="copiedKey === `kode:${row.index}` ? 'i-lucide-check' : 'i-lucide-copy'"
								class="font-mono"
								:aria-label="`Salin kode rekening ${row.original.kodeRekening}`"
								title="Salin kode rekening"
								@click="copy(row.original.kodeRekening, 'Kode rekening', `kode:${row.index}`)"
							>
								{{ row.original.kodeRekening }}
							</UButton>
						</template>
					</UTable>
				</UContextMenu>
			</div>
		</div>
	</LayoutPageShell>
</template>

<script lang="ts" setup>
	import type { ContextMenuItem, TableColumn, TableRow } from "@nuxt/ui";

	/** Batas jumlah hasil dari backend. */
	const LIMIT = 200;

	const { connected, year } = useArkas();
	const toast = useToast();
	const { copy: clip } = useClipboard();
	const input = useTemplateRef<{ inputRef?: HTMLInputElement }>("input");

	const keyword = ref("");
	const results = ref<StandarHarga[]>([]);
	const loading = ref(false);
	const error = ref("");
	const searched = ref(false);

	// Token mencegah respons lama menimpa hasil yang lebih baru saat mengetik cepat.
	let token = 0;
	const search = useDebounceFn(async () => {
		const q = keyword.value.trim();
		if (!year.value || q.length < 2) {
			token++;
			results.value = [];
			searched.value = false;
			error.value = "";
			loading.value = false;
			return;
		}
		const current = ++token;
		loading.value = true;
		error.value = "";
		try {
			const res = await api.standarHargaSearch(year.value, q);
			if (current === token) {
				results.value = res;
				searched.value = true;
			}
		} catch (err) {
			if (current === token) error.value = errorMessage(err);
		} finally {
			if (current === token) loading.value = false;
		}
	}, 300);

	watch([keyword, year], search);

	onMounted(() => nextTick(() => input.value?.inputRef?.focus()));

	const nominal = { th: "text-right", td: "text-right tabular whitespace-nowrap" };
	const columns: TableColumn<StandarHarga>[] = [
		{ accessorKey: "namaBarang", header: "Nama barang", meta: { class: { td: "whitespace-normal min-w-64" } } },
		{ accessorKey: "satuan", header: "Satuan" },
		{ accessorKey: "harga", header: "Harga", cell: ({ row }) => row.original.harga !== null ? angka(row.original.harga) : "", meta: { class: nominal } },
		{ accessorKey: "batasAtas", header: "Batas atas", cell: ({ row }) => row.original.batasAtas ? angka(row.original.batasAtas) : "", meta: { class: { ...nominal, td: `${nominal.td} text-muted` } } },
		{ accessorKey: "kodeRekening", header: "Kode rekening", meta: { class: { td: "whitespace-nowrap" } } },
		{ accessorKey: "tahun", header: "Tahun", meta: { class: { td: "text-muted" } } }
	];

	// Ikon centang sesaat pada tombol yang baru disalin.
	const copiedKey = ref<string | null>(null);
	let copiedTimer: ReturnType<typeof setTimeout> | undefined;
	const copy = async (text: string, label: string, key?: string) => {
		await clip(text);
		toast.add({ title: `${label} disalin`, description: text, color: "success" });
		if (key) {
			copiedKey.value = key;
			clearTimeout(copiedTimer);
			copiedTimer = setTimeout(() => (copiedKey.value = null), 1500);
		}
	};

	// Menu klik kanan: salin nama barang, harga, atau kode rekening dari baris yang diklik.
	const contextRow = ref<StandarHarga | null>(null);
	const onContextmenu = (_e: Event, row: TableRow<StandarHarga>) => {
		contextRow.value = row.original;
	};
	const contextItems = computed<ContextMenuItem[]>(() => {
		const r = contextRow.value;
		if (!r) return [];
		return [
			{ label: "Salin nama barang", icon: "i-lucide-type", onSelect: () => copy(r.namaBarang, "Nama barang") },
			...(r.harga !== null ? [{ label: "Salin harga", icon: "i-lucide-banknote", onSelect: () => copy(String(r.harga), "Harga") }] : []),
			...(r.kodeRekening ? [{ label: "Salin kode rekening", icon: "i-lucide-copy", onSelect: () => copy(r.kodeRekening!, "Kode rekening") }] : [])
		];
	});
</script>
