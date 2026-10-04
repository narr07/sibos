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

					<div class="border border-default rounded-md overflow-auto max-h-80">
						<table class="w-full text-sm">
							<thead class="bg-elevated text-xs text-muted sticky top-0">
								<tr class="text-left">
									<th class="p-2">
										Tanggal
									</th>
									<th class="p-2">
										Uraian
									</th>
									<th class="p-2">
										Jenis
									</th>
									<th class="p-2">
										Arah
									</th>
									<th class="p-2 text-right">
										Nominal
									</th>
									<th class="p-2 w-20" />
								</tr>
							</thead>
							<tbody>
								<tr v-for="e in entries" :key="e.id" class="border-t border-default">
									<td class="p-2 whitespace-nowrap tabular">
										{{ tanggalId(e.tanggal) }}
									</td>
									<td class="p-2">
										{{ e.uraian }}
										<UBadge v-if="e.refId" color="info" variant="subtle" size="sm" class="ml-1">
											dari nota
										</UBadge>
									</td>
									<td class="p-2">
										{{ e.jenisPajak }}
									</td>
									<td class="p-2">
										<UBadge :color="e.arah === 'pungut' ? 'warning' : 'success'" variant="subtle" size="sm">
											{{ e.arah === "pungut" ? "Pungut" : "Setor" }}
										</UBadge>
									</td>
									<td class="p-2 text-right tabular">
										{{ angka(e.nominal) }}
									</td>
									<td class="p-2 whitespace-nowrap">
										<UButton v-if="!e.refId" icon="i-lucide-pencil" size="xs" color="neutral" variant="ghost" aria-label="Ubah" @click="edit(e)" />
										<UTooltip v-else text="Ubah lewat Cetak Kwitansi A2 → Atur cetak">
											<UButton icon="i-lucide-pencil" size="xs" color="neutral" variant="ghost" disabled aria-label="Ubah" />
										</UTooltip>
										<UButton icon="i-lucide-trash-2" size="xs" color="error" variant="ghost" aria-label="Hapus" @click="remove(e)" />
									</td>
								</tr>
								<tr v-if="!entries.length">
									<td colspan="6" class="p-6 text-center text-muted">
										Belum ada entri pajak manual.
									</td>
								</tr>
							</tbody>
						</table>
					</div>
				</div>
			</template>
		</UModal>
	</div>
</template>

<script lang="ts" setup>
	const { year, funds } = useArkas();
	const toast = useToast();
	const bookView = useTemplateRef<{ reload: () => Promise<void> }>("bookView");

	const open = ref(false);
	const busy = ref(false);
	const entries = ref<ManualTax[]>([]);

	const arahItems = [{ label: "Pungut (hutang pajak)", value: "pungut" }, { label: "Setor ke kas negara", value: "setor" }];
	const jenisItems = ["PPN", "PPh 21", "PPh 22", "PPh 23", "PPh 4(2)", "Pajak Daerah"];
	const fundItems = computed(() => [{ label: "Tidak spesifik", value: 0 }, ...funds.value.map((f) => ({ label: f.name, value: f.id }))]);

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
