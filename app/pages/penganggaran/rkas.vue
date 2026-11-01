<template>
	<LayoutPageShell title="Penganggaran RKAS">
		<div v-if="!connected" class="py-16 text-center text-muted">
			Hubungkan ARKAS terlebih dahulu.
		</div>
		<div v-else class="space-y-4">
			<!-- Toolbar: pilih draft & aksi. Dulu kolom kiri tetap; dipindah ke atas agar tabel selebar layar. -->
			<div class="flex flex-wrap items-center gap-2">
				<USelectMenu
					:model-value="selectedId ?? undefined"
					:items="draftOptions"
					value-key="value"
					icon="i-lucide-files"
					placeholder="Pilih draft"
					:search-input="drafts.length > 6 ? { placeholder: 'Cari draft...' } : false"
					class="w-80"
					@update:model-value="(v) => (selectedId = v ?? null)"
				/>
				<UButton icon="i-lucide-download" color="neutral" variant="outline" @click="createOpen = true">
					Draft baru dari ARKAS
				</UButton>
				<p class="text-xs text-muted flex items-center gap-1.5">
					<UIcon name="i-lucide-info" class="size-4 shrink-0" />
					Draft hanya disimpan di Lajur untuk simulasi; RKAS di ARKAS tidak berubah.
				</p>
				<div v-if="detail" class="ml-auto flex flex-wrap gap-2">
					<UButton color="neutral" variant="outline" icon="i-lucide-plus" @click="editItem(null)">
						Item
					</UButton>
					<UDropdownMenu :items="draftMenu">
						<UButton color="neutral" variant="outline" icon="i-lucide-ellipsis" aria-label="Aksi draft" />
					</UDropdownMenu>
				</div>
			</div>

			<UEmpty
				v-if="!drafts.length"
				icon="i-lucide-files"
				title="Belum ada draft untuk tahun ini"
				description="Salin RKAS yang berlaku di ARKAS sebagai draft Awal, lalu ubah item, harga, dan volumenya di sini."
				:actions="[{ label: 'Draft baru dari ARKAS', icon: 'i-lucide-download', onClick: () => { createOpen = true } }]"
			/>

			<div v-else-if="detail" class="space-y-4 min-w-0">
				<p class="text-sm text-muted flex flex-wrap items-center gap-2">
					<UBadge :color="jenisColor[detail.draft.jenis]" variant="subtle" size="sm">
						{{ jenisLabel[detail.draft.jenis] }}
					</UBadge>
					<span>{{ detail.fundName || `Sumber dana ${detail.draft.sumberDana}` }}</span>
					<span v-if="detail.parent">· dari "{{ detail.parent.nama }}"</span>
				</p>

				<div class="grid grid-cols-2 lg:grid-cols-3 gap-3">
					<UCard :ui="{ body: 'p-3 sm:p-3' }">
						<p class="text-xs text-muted">
							Total draft
						</p>
						<p class="text-lg font-semibold tabular">
							{{ rupiah(total) }}
						</p>
					</UCard>
					<UCard v-if="detail.parent" :ui="{ body: 'p-3 sm:p-3' }">
						<p class="text-xs text-muted">
							Total versi asal
						</p>
						<p class="text-lg font-semibold tabular">
							{{ rupiah(detail.parent.total) }}
						</p>
					</UCard>
					<UCard v-if="detail.parent" :ui="{ body: 'p-3 sm:p-3' }">
						<p class="text-xs text-muted">
							Selisih
						</p>
						<p class="text-lg font-semibold tabular" :class="selisih === 0 ? 'text-success' : 'text-warning'">
							{{ rupiah(selisih) }}
						</p>
						<p v-if="detail.draft.jenis === 'pergeseran' && selisih !== 0" class="text-xs text-warning">
							Pergeseran harus berjumlah sama dengan versi asal.
						</p>
					</UCard>
				</div>

				<UTabs v-model="tab" :items="tabItems" :content="false" variant="link" />

				<RkasBandingSiplah v-if="tab === 'siplah'" :draft="detail.draft" :items="detail.items" @applied="onHargaDipakai" />

				<UInput v-if="tab === 'item'" v-model="search" icon="i-lucide-search" placeholder="Cari item..." class="w-72" />

				<UTable
					v-if="tab === 'item'"
					:data="filteredItems"
					:columns="itemColumns"
					sticky
					empty="Belum ada item."
					class="max-h-[calc(100vh-26rem)] border border-default"
					:ui="{ td: 'py-1.5 text-sm align-top', th: 'py-2 text-xs' }"
				>
					<template #kode-cell="{ row }">
						<p class="font-mono text-xs">
							{{ row.original.kodeKegiatan }}
						</p>
						<p class="font-mono text-xs text-muted">
							{{ row.original.kodeRekening }}
						</p>
					</template>
					<template #uraian-cell="{ row }">
						<p class="whitespace-normal">
							{{ row.original.uraian }}
						</p>
						<p class="text-xs text-muted whitespace-normal">
							{{ row.original.kodeKegiatan ? detail.kodeNames[row.original.kodeKegiatan] : "" }}
						</p>
					</template>
					<template #aksi-cell="{ row }">
						<UButton icon="i-lucide-pencil" size="xs" color="neutral" variant="ghost" aria-label="Ubah" @click="editItem(row.original)" />
						<UButton icon="i-lucide-trash-2" size="xs" color="error" variant="ghost" aria-label="Hapus" @click="deleteItem(row.original)" />
					</template>
				</UTable>
			</div>
			<p v-else class="py-16 text-center text-muted">
				Pilih draft RKAS.
			</p>
		</div>

		<!-- Draft baru -->
		<UModal v-model:open="createOpen" title="Draft baru dari RKAS ARKAS" description="Item RKAS yang berlaku di ARKAS disalin sebagai draft Awal.">
			<template #body>
				<div class="space-y-3">
					<UFormField label="Sumber dana">
						<USelect v-model="createForm.fund" :items="fundItems" />
					</UFormField>
					<UFormField label="Nama draft">
						<UInput v-model="createForm.nama" />
					</UFormField>
				</div>
			</template>
			<template #footer>
				<div class="flex w-full justify-end gap-2">
					<UButton color="neutral" variant="outline" @click="createOpen = false">
						Batal
					</UButton>
					<UButton icon="i-lucide-check" :loading="busy" @click="create">
						Buat draft
					</UButton>
				</div>
			</template>
		</UModal>

		<!-- Salin versi -->
		<UModal v-model:open="copyOpen" :title="`Buat versi ${copyForm.jenis === 'perubahan' ? 'Perubahan' : 'Pergeseran'}`">
			<template #body>
				<UFormField label="Nama versi">
					<UInput v-model="copyForm.nama" />
				</UFormField>
			</template>
			<template #footer>
				<div class="flex w-full justify-end gap-2">
					<UButton color="neutral" variant="outline" @click="copyOpen = false">
						Batal
					</UButton>
					<UButton icon="i-lucide-copy" :loading="busy" @click="copy">
						Buat
					</UButton>
				</div>
			</template>
		</UModal>

		<!-- Ubah item -->
		<UModal v-model:open="itemOpen" :title="itemForm.id ? 'Ubah item' : 'Tambah item'" :ui="{ content: 'max-w-3xl' }">
			<template #body>
				<div class="space-y-3">
					<div class="grid sm:grid-cols-2 gap-3">
						<UFormField label="Kode kegiatan">
							<UInput :model-value="itemForm.kodeKegiatan ?? ''" placeholder="05.02.08." @update:model-value="(v) => (itemForm.kodeKegiatan = String(v) || null)" />
						</UFormField>
						<UFormField label="Kode rekening">
							<UInput :model-value="itemForm.kodeRekening ?? ''" placeholder="5.1.02.01.01.0024" @update:model-value="(v) => (itemForm.kodeRekening = String(v) || null)" />
						</UFormField>
						<UFormField label="Uraian" class="sm:col-span-2">
							<UInput v-model="itemForm.uraian" />
						</UFormField>
						<UFormField label="Satuan">
							<UInput :model-value="itemForm.satuan ?? ''" @update:model-value="(v) => (itemForm.satuan = String(v) || null)" />
						</UFormField>
						<UFormField label="Harga satuan">
							<UInputNumber v-model="itemForm.hargaSatuan" :min="0" locale="id-ID" :format-options="{ maximumFractionDigits: 0 }" />
						</UFormField>
					</div>
					<p class="text-sm font-medium">
						Volume per bulan
					</p>
					<div class="grid grid-cols-3 sm:grid-cols-6 gap-2">
						<UFormField v-for="(m, i) in BULAN" :key="m" :label="m.slice(0, 3)">
							<UInputNumber v-model="itemForm.volumeBulan[i]" :min="0" :step="1" size="sm" />
						</UFormField>
					</div>
					<p class="text-sm tabular">
						Jumlah: <b>{{ rupiah(jumlah(itemForm)) }}</b> ({{ volume(itemForm).toLocaleString("id-ID") }} × {{ angka(itemForm.hargaSatuan) }})
					</p>
				</div>
			</template>
			<template #footer>
				<div class="flex w-full justify-end gap-2">
					<UButton color="neutral" variant="outline" @click="itemOpen = false">
						Batal
					</UButton>
					<UButton icon="i-lucide-check" :loading="busy" @click="saveItem">
						Simpan
					</UButton>
				</div>
			</template>
		</UModal>
	</LayoutPageShell>
</template>

<script lang="ts" setup>
	import type { DropdownMenuItem, TableColumn } from "@nuxt/ui";
	import RkasTahunan from "~/components/Print/Laporan/RkasTahunan.vue";

	const { connected, year, funds } = useArkas();
	const toast = useToast();
	const { xlsx } = useSaveFile();

	const drafts = ref<Draft[]>([]);
	const selectedId = ref<string | null>(null);
	const detail = ref<DraftDetail | null>(null);
	const busy = ref(false);
	const search = ref("");
	const tab = ref<"item" | "siplah">("item");
	const tabItems = [
		{ label: "Item draft", value: "item", icon: "i-lucide-list" },
		{ label: "Bandingkan harga SIPLah", value: "siplah", icon: "i-lucide-scale" }
	];

	const jenisLabel: Record<Draft["jenis"], string> = { awal: "Awal", perubahan: "Perubahan", pergeseran: "Pergeseran" };
	const jenisColor: Record<Draft["jenis"], "neutral" | "info" | "warning"> = { awal: "neutral", perubahan: "info", pergeseran: "warning" };
	const fundItems = computed(() => funds.value.map((f) => ({ label: f.name, value: f.id })));
	const draftOptions = computed(() => drafts.value.map((d) => ({
		label: d.nama,
		description: `${jenisLabel[d.jenis]} · ${d.itemCount} item · ${rupiah(d.total)}`,
		value: d.id
	})));

	const volume = (it: Pick<DraftItem, "volumeBulan">) => it.volumeBulan.reduce((a, b) => a + (b || 0), 0);
	const jumlah = (it: Pick<DraftItem, "volumeBulan" | "hargaSatuan">) => Math.round(volume(it) * it.hargaSatuan);
	const total = computed(() => (detail.value?.items ?? []).reduce((s, i) => s + jumlah(i), 0));
	const selisih = computed(() => total.value - (detail.value?.parent?.total ?? 0));

	const nominal = { th: "text-right", td: "text-right tabular whitespace-nowrap" };
	const itemColumns: TableColumn<DraftItem>[] = [
		{ id: "kode", header: "Kegiatan / Rekening" },
		{ accessorKey: "uraian", header: "Uraian", meta: { class: { td: "min-w-64" } } },
		{ id: "volume", header: "Volume", cell: ({ row }) => `${volume(row.original).toLocaleString("id-ID")} ${row.original.satuan ?? ""}`, meta: { class: nominal } },
		{ accessorKey: "hargaSatuan", header: "Harga", cell: ({ row }) => angka(row.original.hargaSatuan), meta: { class: nominal } },
		{ id: "jumlah", header: "Jumlah", cell: ({ row }) => angka(jumlah(row.original)), meta: { class: nominal } },
		{ id: "aksi", header: "", meta: { class: { th: "w-20", td: "whitespace-nowrap" } } }
	];

	const filteredItems = computed(() => {
		const q = search.value.trim().toLowerCase();
		const items = detail.value?.items ?? [];
		return q ? items.filter((i) => [i.uraian, i.kodeRekening, i.kodeKegiatan].some((v) => v?.toLowerCase().includes(q))) : items;
	});

	const loadList = async () => {
		if (!year.value) return;
		drafts.value = await api.rkasDraftList(year.value).catch(() => []);
		if (selectedId.value && !drafts.value.some((d) => d.id === selectedId.value)) selectedId.value = null;
		if (!selectedId.value && drafts.value.length) selectedId.value = drafts.value.at(-1)?.id ?? null;
	};
	const loadDetail = async () => {
		detail.value = selectedId.value ? await api.rkasDraftDetail(selectedId.value).catch(() => null) : null;
	};
	watch([year, connected], loadList, { immediate: true });
	watch(selectedId, loadDetail);

	const run = async (fn: () => Promise<unknown>, ok?: string) => {
		busy.value = true;
		try {
			await fn();
			if (ok) toast.add({ title: ok, color: "success" });
			return true;
		} catch (err) {
			toast.add({ title: "Gagal", description: errorMessage(err), color: "error" });
			return false;
		} finally {
			busy.value = false;
		}
	};

	// Draft baru
	const createOpen = ref(false);
	const createForm = reactive({ fund: 0, nama: "" });
	watch(createOpen, (o) => {
		if (!o) return;
		createForm.fund = funds.value[0]?.id ?? 0;
		createForm.nama = `RKAS ${year.value ?? ""} Awal`;
	});
	const create = async () => {
		if (!year.value) return;
		const y = year.value;
		let id = "";
		if (await run(async () => {
			id = await api.rkasDraftFromArkas(y, createForm.fund, createForm.nama);
		}, "Draft dibuat")) {
			createOpen.value = false;
			await loadList();
			selectedId.value = id;
		}
	};

	// Salin versi
	const copyOpen = ref(false);
	const copyForm = reactive<{ jenis: "perubahan" | "pergeseran", nama: string }>({ jenis: "perubahan", nama: "" });
	const openCopy = (jenis: "perubahan" | "pergeseran") => {
		copyForm.jenis = jenis;
		copyForm.nama = `${jenis === "perubahan" ? "Perubahan" : "Pergeseran"} ${(drafts.value.filter((d) => d.jenis === jenis).length + 1)}`;
		copyOpen.value = true;
	};
	const copy = async () => {
		if (!selectedId.value) return;
		const source = selectedId.value;
		let id = "";
		if (await run(async () => {
			id = await api.rkasDraftCopy(source, copyForm.jenis, copyForm.nama);
		}, "Versi dibuat")) {
			copyOpen.value = false;
			await loadList();
			selectedId.value = id;
		}
	};

	// Item
	const itemOpen = ref(false);
	const emptyItem = (): DraftItem => ({ id: "", urutan: 0, kodeKegiatan: null, kodeRekening: null, uraian: "", satuan: null, hargaSatuan: 0, volumeBulan: Array.from<number>({ length: 12 }).fill(0), sumberIdRapbs: null });
	const itemForm = reactive<DraftItem>(emptyItem());
	const editItem = (it: DraftItem | null) => {
		Object.assign(itemForm, it ? structuredClone(toRaw(it)) : emptyItem());
		itemOpen.value = true;
	};
	const saveItem = async () => {
		if (!selectedId.value) return;
		const id = selectedId.value;
		const item = { ...itemForm, volumeBulan: itemForm.volumeBulan.map((v) => Number(v) || 0) };
		if (await run(() => api.rkasDraftItemSave(id, item))) {
			itemOpen.value = false;
			await loadDetail();
			await loadList();
		}
	};
	// Harga toko dipakai dari tab SIPLah: muat ulang total draft tanpa mengosongkan hasil perbandingan.
	const onHargaDipakai = async () => {
		await loadDetail();
		await loadList();
	};
	const deleteItem = async (it: DraftItem) => {
		if (!selectedId.value) return;
		const id = selectedId.value;
		await run(() => api.rkasDraftItemDelete(id, it.id));
		await loadDetail();
		await loadList();
	};

	const { open: openPrint } = usePrint();
	/** PDF draft memakai lembar RKAS yang sama dengan Kertas Kerja. */
	const printDraft = () => {
		const d = detail.value;
		if (!d) return;
		const items: RkasItem[] = d.items.map((it) => ({
			idRapbs: it.id,
			fundId: d.draft.sumberDana,
			fundName: d.fundName,
			kodeKegiatan: it.kodeKegiatan,
			kodeRekening: it.kodeRekening,
			uraian: it.uraian,
			satuan: it.satuan,
			volume: volume(it),
			hargaSatuan: it.hargaSatuan,
			jumlah: jumlah(it),
			bulan: it.volumeBulan.map((v) => Math.round((v || 0) * it.hargaSatuan)),
			volumeBulan: it.volumeBulan
		}));
		openPrint({
			title: `Draft RKAS ${d.draft.nama}`,
			landscape: true,
			component: RkasTahunan,
			props: {
				items,
				kodeNames: d.kodeNames,
				fundLabel: d.fundName,
				year: d.draft.tahun,
				funds: funds.value,
				judul: `Draft ${jenisLabel[d.draft.jenis]} - ${d.draft.nama}`
			}
		});
	};

	const exportDraft = () => {
		if (!detail.value) return;
		const { id, nama } = detail.value.draft;
		xlsx(`Draft_RKAS_${nama}`, (path) => api.rkasDraftExportXlsx(id, path));
	};

	const removeDraft = async () => {
		if (!selectedId.value) return;
		const id = selectedId.value;
		if (await run(() => api.rkasDraftDelete(id), "Draft dihapus")) {
			selectedId.value = null;
			await loadList();
		}
	};

	const draftMenu = computed<DropdownMenuItem[][]>(() => [
		[
			{ label: "Buat versi Perubahan", icon: "i-lucide-file-plus", onSelect: () => openCopy("perubahan") },
			{ label: "Buat versi Pergeseran", icon: "i-lucide-arrow-left-right", onSelect: () => openCopy("pergeseran") }
		],
		[
			{ label: "Export Excel", icon: "i-lucide-file-spreadsheet", onSelect: exportDraft },
			{ label: "Cetak PDF", icon: "i-lucide-printer", onSelect: printDraft }
		],
		[{ label: "Hapus draft", icon: "i-lucide-trash-2", color: "error", onSelect: removeDraft }]
	]);
</script>
