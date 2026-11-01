<template>
	<div class="space-y-4">
		<UAlert
			v-if="!tokoList.length && !loadingToko"
			icon="i-lucide-store"
			color="neutral"
			variant="subtle"
			title="Belum ada toko SIPLah tersimpan"
			description="Tambahkan toko Toko Ladang lewat menu Toko SIPLah, lalu sinkronkan katalognya."
			:actions="[{ label: 'Buka Toko SIPLah', to: '/penganggaran/siplah', color: 'neutral', variant: 'outline' }]"
		/>

		<template v-else>
			<div class="flex flex-wrap items-end gap-2">
				<UFormField label="Toko" class="w-72">
					<USelect v-model="tokoId" :items="tokoItems" :loading="loadingToko" placeholder="Pilih toko" />
				</UFormField>
				<UFormField label="Periode" class="w-44">
					<USelect v-model="bulan" :items="bulanItems" />
				</UFormField>
				<UButton icon="i-lucide-scale" :loading="loading" :disabled="!tokoId" @click="muat">
					Bandingkan
				</UButton>
				<UButton
					v-if="tokoId"
					icon="i-lucide-refresh-cw"
					color="neutral"
					variant="outline"
					:loading="syncing"
					@click="sinkronkan(tokoId)"
				>
					{{ syncing && progressLabel ? progressLabel : "Sinkronkan katalog" }}
				</UButton>
				<UButton v-if="syncing" icon="i-lucide-x" color="neutral" variant="outline" :disabled="cancelling" @click="batalkan">
					{{ cancelling ? "Membatalkan..." : "Batalkan" }}
				</UButton>
				<p v-if="tokoAktif" class="text-xs text-muted pb-2">
					{{ tokoAktif.totalProduk }} produk · sinkron {{ tokoAktif.terakhirSync ?? "belum pernah" }}
				</p>
			</div>

			<div v-if="report" class="space-y-4">
				<div class="grid grid-cols-2 lg:grid-cols-4 gap-3">
					<UCard :ui="{ body: 'p-3 sm:p-3' }">
						<p class="text-xs text-muted">
							Total draft {{ bulan ? BULAN[bulan - 1] : "setahun" }}
						</p>
						<p class="text-lg font-semibold tabular">
							{{ rupiah(report.summary.totalAnggaranRkas) }}
						</p>
						<p class="text-xs text-muted">
							{{ report.summary.totalItems }} item
						</p>
					</UCard>
					<UCard :ui="{ body: 'p-3 sm:p-3' }">
						<p class="text-xs text-muted">
							Perkiraan bila belanja di toko ini
						</p>
						<p class="text-lg font-semibold tabular">
							{{ rupiah(report.summary.totalPerkiraan) }}
						</p>
						<p class="text-xs text-muted">
							Item tanpa padanan memakai harga draft
						</p>
					</UCard>
					<UCard :ui="{ body: 'p-3 sm:p-3' }">
						<p class="text-xs text-muted">
							Selisih item yang cocok
						</p>
						<p class="text-lg font-semibold tabular" :class="report.summary.totalHemat >= 0 ? 'text-success' : 'text-error'">
							{{ report.summary.totalHemat >= 0 ? `Hemat ${rupiah(report.summary.totalHemat)}` : `Lebih ${rupiah(-report.summary.totalHemat)}` }}
						</p>
						<p class="text-xs text-muted">
							{{ report.summary.totalLebihMurah }} lebih murah · {{ report.summary.totalLebihMahal }} lebih mahal
						</p>
					</UCard>
					<UCard :ui="{ body: 'p-3 sm:p-3' }">
						<p class="text-xs text-muted">
							Item ditemukan di toko
						</p>
						<p class="text-lg font-semibold tabular">
							{{ report.summary.totalCocok }} / {{ report.summary.totalItems }}
						</p>
						<p class="text-xs text-muted">
							{{ report.summary.totalTidakCocok }} belum ada padanan
						</p>
					</UCard>
				</div>

				<div class="flex flex-wrap items-center gap-2">
					<UInput v-model="cari" icon="i-lucide-search" placeholder="Cari item atau produk..." class="w-72" />
					<USelect v-model="filterStatus" :items="statusItems" class="w-48" />
					<p class="text-xs text-muted ml-auto">
						Kecocokan dicari otomatis dari nama barang. Periksa produknya sebelum memakai harganya.
					</p>
				</div>

				<UTable
					:data="rows"
					:columns="columns"
					sticky
					empty="Tidak ada item."
					class="max-h-[calc(100vh-30rem)] border border-default"
					:ui="{ td: 'py-1.5 text-sm align-top', th: 'py-2 text-xs' }"
				>
					<template #uraian-cell="{ row }">
						<p class="whitespace-normal">
							{{ row.original.namaRkas }}
						</p>
						<p class="text-xs text-muted tabular">
							{{ formatVolume(row.original.volume) }} {{ row.original.satuan ?? "" }} × {{ angka(row.original.hargaRkas) }}
						</p>
					</template>
					<template #produk-cell="{ row }">
						<template v-if="row.original.matchedProduct">
							<ULink :to="produkUrl(row.original.matchedProduct.id)" target="_blank" class="whitespace-normal text-highlighted hover:underline">
								{{ row.original.matchedProduct.nama }}
							</ULink>
							<p class="text-xs text-muted">
								Kecocokan {{ row.original.score }}%
							</p>
						</template>
						<span v-else class="text-muted">Tidak ditemukan</span>
					</template>
					<template #selisih-cell="{ row }">
						<span v-if="row.original.matchedProduct" :class="warnaSelisih(row.original.selisih)">
							{{ row.original.selisih > 0 ? "+" : "" }}{{ angka(row.original.selisih) }}
							<span class="text-xs">({{ row.original.persenSelisih > 0 ? "+" : "" }}{{ row.original.persenSelisih.toFixed(0) }}%)</span>
						</span>
					</template>
					<template #aksi-cell="{ row }">
						<div class="flex justify-end gap-1">
							<UDropdownMenu v-if="row.original.candidates.length > 1" :items="pilihanKandidat(row.original)">
								<UButton size="xs" color="neutral" variant="ghost" icon="i-lucide-list" aria-label="Ganti produk" title="Ganti produk" />
							</UDropdownMenu>
							<UButton
								size="xs"
								color="primary"
								variant="soft"
								icon="i-lucide-check"
								:disabled="!row.original.matchedProduct || row.original.selisih === 0"
								:loading="menyimpan === row.original.sourceId"
								title="Ganti harga satuan item draft dengan harga toko (berlaku untuk semua bulan)"
								@click="pakaiHarga(row.original)"
							>
								Pakai harga
							</UButton>
						</div>
					</template>
				</UTable>
			</div>

			<p v-else-if="!loading" class="py-12 text-center text-muted">
				Pilih toko, lalu klik Bandingkan untuk melihat harga Toko Ladang dan perkiraan biaya draft ini.
			</p>
		</template>
	</div>
</template>

<script lang="ts" setup>
	import type { DropdownMenuItem, TableColumn } from "@nuxt/ui";
	import type { SiplahMatchItem, SiplahMatchReport, SiplahToko } from "~/utils/api";
	import type { Draft, DraftItem } from "~/utils/types";

	const props = defineProps<{ draft: Draft, items: DraftItem[] }>();
	const emit = defineEmits<{ applied: [] }>();
	const toast = useToast();

	const TOKO_KEY = "lajur-banding-toko";
	const tokoList = ref<SiplahToko[]>([]);
	const loadingToko = ref(false);
	const tokoId = ref<number | undefined>();
	const bulan = ref(0);
	const report = ref<SiplahMatchReport | null>(null);
	const loading = ref(false);
	const cari = ref("");
	const filterStatus = ref<"semua" | SiplahMatchItem["status"]>("semua");
	const menyimpan = ref<string | null>(null);

	const tokoItems = computed(() => tokoList.value.map((t) => ({ label: t.nama, value: t.id })));
	const tokoAktif = computed(() => tokoList.value.find((t) => t.id === tokoId.value));
	const bulanItems = [{ label: "Setahun", value: 0 }, ...BULAN.map((label, i) => ({ label, value: i + 1 }))];
	const statusItems = [
		{ label: "Semua status", value: "semua" },
		{ label: "Lebih murah", value: "lebih_murah" },
		{ label: "Harga sama", value: "sesuai" },
		{ label: "Lebih mahal", value: "lebih_mahal" },
		{ label: "Tidak ditemukan", value: "tidak_cocok" }
	];

	const produkUrl = (id: number) => `https://siplah.tokoladang.co.id/p/produk.${id}`;
	const warnaSelisih = (selisih: number) => (selisih < 0 ? "text-success" : selisih > 0 ? "text-error" : "text-muted");

	const nominal = { th: "text-right", td: "text-right tabular whitespace-nowrap" };
	const columns: TableColumn<SiplahMatchItem>[] = [
		{ id: "uraian", header: "Item draft", meta: { class: { td: "min-w-64" } } },
		{ id: "produk", header: "Produk Toko Ladang", meta: { class: { td: "min-w-64" } } },
		{ id: "hargaToko", header: "Harga toko", cell: ({ row }) => (row.original.matchedProduct ? angka(row.original.matchedProduct.harga) : "-"), meta: { class: nominal } },
		{ id: "selisih", header: "Selisih / satuan", meta: { class: nominal } },
		{ id: "perkiraan", header: "Perkiraan jumlah", cell: ({ row }) => angka(estimasiToko(row.original) ?? row.original.totalRkas), meta: { class: nominal } },
		{ id: "aksi", header: "", meta: { class: { th: "w-40", td: "whitespace-nowrap" } } }
	];

	const rows = computed(() => {
		const q = cari.value.trim().toLowerCase();
		return (report.value?.items ?? []).filter((it) => {
			if (filterStatus.value !== "semua" && it.status !== filterStatus.value) return false;
			return !q || it.namaRkas.toLowerCase().includes(q) || (it.matchedProduct?.nama.toLowerCase().includes(q) ?? false);
		});
	});

	const muatToko = async () => {
		loadingToko.value = true;
		try {
			tokoList.value = await api.siplahTokoList();
			const tersimpan = Number(localStorage.getItem(TOKO_KEY));
			if (!tokoList.value.some((t) => t.id === tokoId.value)) {
				tokoId.value = tokoList.value.find((t) => t.id === tersimpan)?.id ?? tokoList.value[0]?.id;
			}
		} catch (err) {
			toast.add({ title: "Gagal memuat toko", description: errorMessage(err), color: "error" });
		} finally {
			loadingToko.value = false;
		}
	};

	const muat = async () => {
		if (!tokoId.value) return;
		localStorage.setItem(TOKO_KEY, String(tokoId.value));
		loading.value = true;
		try {
			report.value = await api.siplahMatchPrices(props.draft.tahun, tokoId.value, "draft", bulan.value || null, null, false, props.draft.id);
		} catch (err) {
			report.value = null;
			toast.add({ title: "Gagal membandingkan", description: errorMessage(err), color: "error" });
		} finally {
			loading.value = false;
		}
	};

	const { syncing, cancelling, progressLabel, sinkronkan, batalkan } = useSiplahSync(async () => {
		await muatToko();
		await muat();
	});

	const pilihanKandidat = (it: SiplahMatchItem): DropdownMenuItem[] =>
		it.candidates.map((c) => ({
			label: `${c.produk.nama} · ${rupiah(c.produk.harga)}`,
			icon: c.produk.id === it.matchedProduct?.id ? "i-lucide-check" : undefined,
			onSelect: () => {
				pasangProduk(it, c.produk);
				it.score = c.score;
				if (report.value) report.value.summary = ringkasanSiplah(report.value.items);
			}
		}));

	/** Simpan harga toko sebagai harga satuan item draft (semua bulan ikut berubah). */
	const pakaiHarga = async (it: SiplahMatchItem) => {
		const produk = it.matchedProduct;
		const item = props.items.find((i) => i.id === it.sourceId);
		if (!produk || !item) return;
		menyimpan.value = it.sourceId;
		try {
			await api.rkasDraftItemSave(props.draft.id, { ...structuredClone(toRaw(item)), hargaSatuan: produk.harga });
			it.hargaRkas = produk.harga;
			it.totalRkas = Math.round(it.volume * produk.harga);
			pasangProduk(it, produk);
			if (report.value) report.value.summary = ringkasanSiplah(report.value.items);
			toast.add({ title: "Harga dipakai", description: `${item.uraian}: ${rupiah(produk.harga)} per ${item.satuan ?? "satuan"}`, color: "success" });
			emit("applied");
		} catch (err) {
			toast.add({ title: "Gagal menyimpan harga", description: errorMessage(err), color: "error" });
		} finally {
			menyimpan.value = null;
		}
	};

	onMounted(muatToko);
	// Draft lain dipilih: hasil lama tidak berlaku lagi.
	watch(() => props.draft.id, () => {
		report.value = null;
	});
</script>
