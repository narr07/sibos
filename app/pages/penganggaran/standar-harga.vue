<template>
	<LayoutPageShell title="Cari Barang & Rekening">
		<div v-if="!connected" class="py-16 text-center text-muted">
			Hubungkan ARKAS terlebih dahulu.
		</div>
		<div v-else class="space-y-4 max-w-5xl">
			<UInput
				v-model="keyword"
				icon="i-lucide-search"
				size="lg"
				placeholder="Ketik nama barang atau kode rekening (contoh: spidol, kertas, 5.1.02.01)"
				class="w-full"
				autofocus
			/>
			<p class="text-sm text-muted">
				Katalog standar harga dari database ARKAS. {{ results.length ? `${results.length} hasil${results.length >= 200 ? " (dibatasi 200)" : ""}.` : "" }}
			</p>
			<UTable
				v-if="results.length"
				:data="results"
				:columns="columns"
				sticky
				class="max-h-[calc(100vh-16rem)] border border-default rounded-md"
				:ui="{ td: 'py-1.5 text-sm', th: 'py-2 text-xs' }"
			>
				<template #kodeRekening-cell="{ row }">
					<UButton
						v-if="row.original.kodeRekening"
						size="xs"
						color="neutral"
						variant="ghost"
						icon="i-lucide-copy"
						class="font-mono"
						@click="copy(row.original.kodeRekening)"
					>
						{{ row.original.kodeRekening }}
					</UButton>
				</template>
			</UTable>
		</div>
	</LayoutPageShell>
</template>

<script lang="ts" setup>
	import type { TableColumn } from "@nuxt/ui";

	const { connected, year } = useArkas();
	const toast = useToast();
	const { copy: clip } = useClipboard();

	const keyword = ref("");
	const results = ref<StandarHarga[]>([]);

	const search = useDebounceFn(async () => {
		if (!year.value || keyword.value.trim().length < 2) {
			results.value = [];
			return;
		}
		results.value = await api.standarHargaSearch(year.value, keyword.value).catch(() => []);
	}, 300);

	watch([keyword, year], search);

	const nominal = { th: "text-right", td: "text-right tabular whitespace-nowrap" };
	const columns: TableColumn<StandarHarga>[] = [
		{ accessorKey: "namaBarang", header: "Nama barang", meta: { class: { td: "whitespace-normal min-w-64" } } },
		{ accessorKey: "satuan", header: "Satuan" },
		{ accessorKey: "harga", header: "Harga", cell: ({ row }) => row.original.harga !== null ? angka(row.original.harga) : "", meta: { class: nominal } },
		{ accessorKey: "batasAtas", header: "Batas atas", cell: ({ row }) => row.original.batasAtas ? angka(row.original.batasAtas) : "", meta: { class: { ...nominal, td: `${nominal.td} text-muted` } } },
		{ accessorKey: "kodeRekening", header: "Kode rekening", meta: { class: { td: "whitespace-nowrap" } } },
		{ accessorKey: "tahun", header: "Tahun", meta: { class: { td: "text-muted" } } }
	];

	const copy = async (kode: string) => {
		await clip(kode);
		toast.add({ title: `Kode ${kode} disalin`, color: "success" });
	};
</script>
