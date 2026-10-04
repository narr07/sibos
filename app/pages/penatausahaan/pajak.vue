<template>
	<div class="contents">
		<BookView ref="bookView" kind="pajak" title="Buku Pembantu Pajak">
			<template #actions>
				<UButton color="neutral" variant="outline" icon="i-lucide-square-pen" @click="open = true">
					Pajak manual ({{ entries.length }})
				</UButton>
			</template>
		</BookView>

		<UModal v-model:open="open" title="Pajak manual" description="Catat pajak yang tidak ada di ARKAS (hutang tahun lalu, pajak terlewat, setoran). Tersimpan di SIBOS, ARKAS tidak berubah." :ui="{ content: 'max-w-4xl' }">
			<template #body>
				<div class="space-y-4">
					<UForm :state="form" class="grid sm:grid-cols-6 gap-3 items-end" @submit="save">
						<UFormField label="Tanggal" class="sm:col-span-2">
							<UInput v-model="form.tanggal" type="date" />
						</UFormField>
						<UFormField label="Arah" class="sm:col-span-2">
							<USelect v-model="form.arah" :items="arahItems" />
						</UFormField>
						<UFormField label="Jenis pajak" class="sm:col-span-2">
							<USelectMenu v-model="form.jenisPajak" :items="jenisItems" create-item @create="(v: string) => (form.jenisPajak = v)" />
						</UFormField>
						<UFormField label="Uraian" class="sm:col-span-4">
							<UInput v-model="form.uraian" placeholder="Contoh: Hutang PPh 21 Desember 2025" />
						</UFormField>
						<UFormField label="Nominal (Rp)" class="sm:col-span-2">
							<UInputNumber v-model="form.nominal" :min="0" :format-options="{ maximumFractionDigits: 0 }" locale="id-ID" />
						</UFormField>
						<UFormField label="No. bukti (opsional)" class="sm:col-span-2">
							<UInput :model-value="form.noBukti ?? ''" @update:model-value="(v) => (form.noBukti = String(v) || null)" />
						</UFormField>
						<UFormField label="Sumber dana" class="sm:col-span-2">
							<USelect v-model="form.sumberDana" :items="fundItems" />
						</UFormField>
						<div class="sm:col-span-2 flex gap-2 justify-end">
							<UButton v-if="form.id" color="neutral" variant="ghost" @click="reset">
								Batal ubah
							</UButton>
							<UButton type="submit" icon="i-lucide-check" :loading="busy">
								{{ form.id ? "Simpan perubahan" : "Tambah" }}
							</UButton>
						</div>
					</UForm>

					<UTable
						:data="entries"
						:columns="taxColumns"
						sticky
						empty="Belum ada entri pajak manual."
						class="max-h-80 border border-default rounded-md"
						:ui="{ td: 'py-1.5 text-sm', th: 'py-2 text-xs' }"
					>
						<template #uraian-cell="{ row }">
							<span class="whitespace-normal">{{ row.original.uraian }}</span>
							<UBadge v-if="row.original.refId" color="info" variant="subtle" size="sm" class="ml-1">
								dari nota
							</UBadge>
						</template>
						<template #arah-cell="{ row }">
							<UBadge :color="row.original.arah === 'pungut' ? 'warning' : 'success'" variant="subtle" size="sm">
								{{ row.original.arah === "pungut" ? "Pungut" : "Setor" }}
							</UBadge>
						</template>
						<template #aksi-cell="{ row }">
							<UButton v-if="!row.original.refId" icon="i-lucide-pencil" size="xs" color="neutral" variant="ghost" aria-label="Ubah" @click="edit(row.original)" />
							<UTooltip v-else text="Ubah lewat Cetak Kwitansi A2 → Atur cetak">
								<UButton icon="i-lucide-pencil" size="xs" color="neutral" variant="ghost" disabled aria-label="Ubah" />
							</UTooltip>
							<UButton icon="i-lucide-trash-2" size="xs" color="error" variant="ghost" aria-label="Hapus" @click="remove(row.original)" />
						</template>
					</UTable>
				</div>
			</template>
		</UModal>
	</div>
</template>

<script lang="ts" setup>
	import type { TableColumn } from "@nuxt/ui";

	const { year, funds } = useArkas();
	const toast = useToast();
	const bookView = useTemplateRef<{ reload: () => Promise<void> }>("bookView");

	const open = ref(false);
	const busy = ref(false);
	const entries = ref<ManualTax[]>([]);

	const arahItems = [{ label: "Pungut (hutang pajak)", value: "pungut" }, { label: "Setor ke kas negara", value: "setor" }];
	const jenisItems = ["PPN", "PPh 21", "PPh 22", "PPh 23", "PPh 4(2)", "Pajak Daerah"];
	const fundItems = computed(() => [{ label: "Tidak spesifik", value: 0 }, ...funds.value.map((f) => ({ label: f.name, value: f.id }))]);

	const taxColumns: TableColumn<ManualTax>[] = [
		{ accessorKey: "tanggal", header: "Tanggal", cell: ({ row }) => tanggalId(row.original.tanggal), meta: { class: { td: "whitespace-nowrap tabular" } } },
		{ accessorKey: "uraian", header: "Uraian" },
		{ accessorKey: "jenisPajak", header: "Jenis" },
		{ accessorKey: "arah", header: "Arah" },
		{ accessorKey: "nominal", header: "Nominal", cell: ({ row }) => angka(row.original.nominal), meta: { class: { th: "text-right", td: "text-right tabular" } } },
		{ id: "aksi", header: "", meta: { class: { th: "w-20", td: "whitespace-nowrap" } } }
	];

	const empty = (): ManualTax => ({
		id: "",
		tahun: year.value ?? new Date().getFullYear(),
		sumberDana: 0,
		tanggal: `${year.value ?? new Date().getFullYear()}-01-01`,
		noBukti: null,
		uraian: "",
		jenisPajak: "PPh 21",
		arah: "pungut",
		nominal: 0,
		keterangan: null
	});
	const form = reactive<ManualTax>(empty());

	const reset = () => Object.assign(form, empty());

	const load = async () => {
		if (!year.value) return;
		entries.value = await api.manualTaxList(year.value).catch(() => []);
	};

	const save = async () => {
		busy.value = true;
		try {
			await api.manualTaxSave({ ...form, tahun: year.value ?? form.tahun, noBukti: form.noBukti || null });
			toast.add({ title: form.id ? "Entri diubah" : "Entri ditambahkan", color: "success" });
			reset();
			await load();
			await bookView.value?.reload();
		} catch (err) {
			toast.add({ title: "Gagal menyimpan", description: errorMessage(err), color: "error" });
		} finally {
			busy.value = false;
		}
	};

	const edit = (e: ManualTax) => Object.assign(form, structuredClone(toRaw(e)));

	const remove = async (e: ManualTax) => {
		await api.manualTaxDelete(e.id).catch((err) => toast.add({ title: "Gagal menghapus", description: errorMessage(err), color: "error" }));
		await load();
		await bookView.value?.reload();
	};

	watch(year, () => {
		reset();
		load();
	}, { immediate: true });
</script>
