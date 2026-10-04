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
			<div v-if="results.length" class="border border-default rounded-md overflow-auto max-h-[calc(100vh-16rem)]">
				<table class="w-full text-sm">
					<thead class="sticky top-0 bg-elevated text-xs text-muted">
						<tr>
							<th class="p-2 text-left">
								Nama barang
							</th>
							<th class="p-2 text-left">
								Satuan
							</th>
							<th class="p-2 text-right">
								Harga
							</th>
							<th class="p-2 text-right">
								Batas atas
							</th>
							<th class="p-2 text-left">
								Kode rekening
							</th>
							<th class="p-2 text-left">
								Tahun
							</th>
						</tr>
					</thead>
					<tbody>
						<tr v-for="(r, i) in results" :key="i" class="border-t border-default">
							<td class="p-2">
								{{ r.namaBarang }}
							</td>
							<td class="p-2">
								{{ r.satuan }}
							</td>
							<td class="p-2 text-right tabular">
								{{ r.harga !== null ? angka(r.harga) : "" }}
							</td>
							<td class="p-2 text-right tabular text-muted">
								{{ r.batasAtas ? angka(r.batasAtas) : "" }}
							</td>
							<td class="p-2 whitespace-nowrap">
								<UButton v-if="r.kodeRekening" size="xs" color="neutral" variant="ghost" icon="i-lucide-copy" class="font-mono" @click="copy(r.kodeRekening)">
									{{ r.kodeRekening }}
								</UButton>
							</td>
							<td class="p-2 text-muted">
								{{ r.tahun }}
							</td>
						</tr>
					</tbody>
				</table>
			</div>
		</div>
	</LayoutPageShell>
</template>

<script lang="ts" setup>
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

	const copy = async (kode: string) => {
		await clip(kode);
		toast.add({ title: `Kode ${kode} disalin`, color: "success" });
	};
</script>
