<template>
	<LayoutPageShell title="Cetak Kwitansi A2">
		<div v-if="!connected" class="py-16 text-center text-muted">
			Hubungkan ARKAS terlebih dahulu.
		</div>

		<div v-else class="flex min-h-0 flex-1 flex-col gap-4">
			<UAlert
				icon="i-lucide-info"
				color="neutral"
				variant="subtle"
				title="Gabungan, pengaturan cetak, dan foto nota disimpan di SIBOS. Data ARKAS tidak berubah."
				description="Pajak hanya tampil bila sudah dicatat; SIBOS tidak menghitung pajak otomatis."
			/>

			<div class="flex flex-wrap items-center gap-2">
				<USelect v-model="month" :items="monthItems" icon="i-lucide-calendar-days" class="w-44" />
				<UInput v-model="search" icon="i-lucide-search" placeholder="Cari no bukti, toko, uraian..." class="w-72" />
				<USelect v-model="metode" :items="metodeItems" class="w-44" aria-label="Metode belanja" />
				<USelect v-model="statusFilter" :items="statusItems" class="w-48" />
				<div class="ml-auto flex flex-wrap gap-2">
					<UButton :disabled="!selected.length" icon="i-lucide-files" color="primary" variant="solid" @click="openDoc">
						Cetak Dokumen ({{ selected.length }})
					</UButton>
					<UButton :disabled="!selected.length" icon="i-lucide-file-text" color="neutral" variant="outline" @click="printBukti">
						Bukti
					</UButton>
					<UButton :disabled="!selected.length" icon="i-lucide-receipt-text" color="secondary" @click="printA2">
						Cetak Kwitansi A2
					</UButton>
					<UButton :disabled="!selected.length" icon="i-lucide-scroll-text" color="neutral" @click="printNota">
						Cetak Nota
					</UButton>
					<UButton :disabled="selectedItemIds.length < 2" color="neutral" variant="outline" icon="i-lucide-merge" @click="openMerge">
						Gabungkan
					</UButton>
				</div>
			</div>

			<UAlert v-if="error" color="error" variant="subtle" icon="i-lucide-triangle-alert" :title="error" />

			<UTable
				:data="filtered"
				:columns="columns"
				:loading="loading"
				sticky
				empty="Tidak ada belanja pada periode ini."
				class="min-h-60 flex-1 border border-default rounded-md"
				:ui="{ td: 'py-1.5 text-sm align-top', th: 'py-2 text-xs' }"
			>
				<template #pilih-header>
					<UCheckbox :model-value="allChecked" aria-label="Pilih semua" @update:model-value="toggleAll" />
				</template>
				<template #pilih-cell="{ row }">
					<UCheckbox :model-value="checked.has(row.original.key)" :aria-label="`Pilih ${row.original.noBukti}`" @update:model-value="toggle(row.original.key)" />
				</template>
				<template #noBukti-cell="{ row }">
					{{ row.original.noBukti || "-" }}
					<UBadge v-if="row.original.kind === 'merge'" color="info" variant="subtle" size="sm" class="ml-1">
						gabungan
					</UBadge>
				</template>
				<template #toko-cell="{ row }">
					<div class="whitespace-normal">
						{{ row.original.nota?.namaToko || "-" }}
						<UBadge v-if="row.original.isSiplah" color="info" variant="subtle" size="sm" class="ml-1">
							SIPLah
						</UBadge>
						<p v-if="row.original.nota?.noNota" class="text-xs text-muted">
							Nota {{ row.original.nota.noNota }}
						</p>
					</div>
				</template>
				<template #uraian-cell="{ row }">
					<div class="whitespace-normal">
						{{ row.original.override?.keperluan || row.original.override?.uraian || row.original.uraian }}
						<p class="text-xs text-muted">
							{{ row.original.items.length }} item{{ row.original.fileCount ? ` · ${row.original.fileCount} foto nota` : "" }}
						</p>
					</div>
				</template>
				<template #totalPajak-cell="{ row }">
					<span v-if="row.original.totalPajak">{{ angka(row.original.totalPajak) }}</span>
					<span v-else class="text-muted">-</span>
				</template>
				<template #status-cell="{ row }">
					<div class="flex gap-1">
						<UBadge v-if="row.original.printedBukti" color="success" variant="subtle" size="sm">
							Bukti
						</UBadge>
						<UBadge v-if="row.original.printedA2" color="success" variant="subtle" size="sm">
							A2
						</UBadge>
						<UBadge v-if="row.original.printedNota" color="success" variant="subtle" size="sm">
							Nota
						</UBadge>
					</div>
				</template>
				<template #aksi-cell="{ row }">
					<UDropdownMenu :items="rowMenu(row.original)" :content="{ align: 'end' }">
						<UButton icon="i-lucide-ellipsis-vertical" color="neutral" variant="ghost" size="xs" aria-label="Aksi" />
					</UDropdownMenu>
				</template>
			</UTable>
			<p class="text-sm text-muted">
				{{ filtered.length }} bukti · total {{ rupiah(filtered.reduce((s, g) => s + g.total, 0)) }}
			</p>
		</div>

		<!-- Cetak dokumen dari template -->
		<UModal v-model:open="docOpen" title="Cetak dokumen" :description="`${selected.length} transaksi terpilih. Dokumen dicetak berurutan per transaksi.`">
			<template #body>
				<div class="space-y-2">
					<p v-if="!docTemplates.length" class="text-sm">
						Belum ada template.
						<NuxtLink to="/lainnya/template" class="text-primary underline">
							Buat di menu Template Dokumen
						</NuxtLink>
						(bisa langsung dari contoh Excel).
					</p>
					<div v-for="j in JENIS_DOKUMEN" :key="j.value" class="flex items-center gap-3 rounded-md border border-default px-3 py-2" :class="hasTemplate(j.value) ? '' : 'opacity-50'">
						<UCheckbox :model-value="docJenis.includes(j.value)" :disabled="!hasTemplate(j.value)" :label="j.label" @update:model-value="(v) => toggleJenis(j.value, !!v)" />
						<span class="ml-auto text-xs text-muted text-right">{{ templateLabel(j.value) }}</span>
					</div>
					<p class="text-xs text-muted">
						Template khusus toko dipakai otomatis bila nama toko cocok; selain itu template umum.
					</p>
				</div>
			</template>
			<template #footer>
				<div class="flex w-full justify-end gap-2">
					<UButton color="neutral" variant="outline" @click="docOpen = false">
						Batal
					</UButton>
					<UButton icon="i-lucide-printer" :disabled="!docJenis.length" :loading="busy" @click="printDocs">
						Pratinjau & cetak
					</UButton>
				</div>
			</template>
		</UModal>

		<!-- Gabungkan -->
		<UModal v-model:open="mergeOpen" title="Gabungkan menjadi satu bukti" :description="`${selectedItemIds.length} transaksi akan dicetak sebagai satu bukti. Data ARKAS tidak berubah.`">
			<template #body>
				<div class="space-y-3">
					<UFormField label="Nomor bukti gabungan">
						<UInput v-model="mergeForm.noBukti" />
					</UFormField>
					<UFormField label="Uraian / untuk keperluan">
						<UTextarea v-model="mergeForm.uraian" :rows="3" class="w-full" />
					</UFormField>
					<UFormField label="Tanggal (opsional)">
						<UInput v-model="mergeForm.tanggal" type="date" />
					</UFormField>
				</div>
			</template>
			<template #footer>
				<div class="flex w-full justify-end gap-2">
					<UButton color="neutral" variant="outline" @click="mergeOpen = false">
						Batal
					</UButton>
					<UButton icon="i-lucide-merge" :loading="busy" @click="saveMerge">
						Simpan gabungan
					</UButton>
				</div>
			</template>
		</UModal>

		<!-- Atur cetak -->
		<UModal v-model:open="ovOpen" title="Atur cetak" description="Hanya tersimpan di SIBOS. Data ARKAS tidak berubah." :ui="{ content: 'max-w-2xl' }">
			<template #body>
				<div class="grid sm:grid-cols-2 gap-3">
					<UFormField label="Nomor nota" class="sm:col-span-2" :hint="ovTarget?.isSiplah ? 'Belanja SIPLah sudah punya invoice' : 'Untuk Cetak Nota (non-SIPLah)'">
						<UInput v-model="ovForm.noNota" :placeholder="ovTarget?.nota?.noNota || `NT/${ovTarget?.noBukti || '-'}/${ovTarget?.tanggal.slice(0, 4) ?? ''}`" />
					</UFormField>
					<UFormField label="Tanggal nota">
						<UInput v-model="ovForm.tanggalNota" type="date" />
					</UFormField>
					<UFormField label="Tanggal bayar">
						<UInput v-model="ovForm.tanggalBayar" type="date" />
					</UFormField>
					<UFormField label="Untuk keperluan" class="sm:col-span-2">
						<UTextarea v-model="ovForm.keperluan" :rows="3" class="w-full" :placeholder="ovTarget?.uraian" />
					</UFormField>
				</div>

				<USeparator class="my-4" />
				<div class="space-y-2">
					<div class="flex items-center gap-2">
						<p class="font-medium text-sm">
							Pajak (input manual)
						</p>
						<UButton size="xs" color="neutral" variant="outline" icon="i-lucide-plus" class="ml-auto" @click="addTax">
							Tambah pajak
						</UButton>
					</div>
					<p class="text-xs text-muted">
						Jumlah belanja di kwitansi sudah termasuk pajak; yang dibayarkan ke penyedia = belanja − pajak.
						Pajak ini otomatis masuk Buku Pembantu Pajak sebagai pungut (dan setor bila tanggal setor diisi).
					</p>
					<UAlert
						v-if="ovTarget?.arkasTaxes.length"
						color="info"
						variant="subtle"
						icon="i-lucide-info"
						:title="`Tercatat di ARKAS: ${ovTarget.arkasTaxes.map((t) => `${t.jenis ?? 'Pajak'} Rp ${angka(t.nominal)}`).join(', ')}`"
						description="Ini PPN yang dipungut & disetor oleh SIPLah (terima dan setor di hari yang sama), tidak memotong pembayaran ke penyedia. Tidak dicetak sebagai pajak nota."
					/>
					<div v-for="(tx, i) in taxForm" :key="i" class="grid grid-cols-[9rem_1fr_10rem_auto] gap-2 items-end">
						<UFormField :label="i === 0 ? 'Jenis' : undefined">
							<USelectMenu v-model="tx.jenis" :items="JENIS_PAJAK" create-item size="sm" @create="(v: string) => (tx.jenis = v)" />
						</UFormField>
						<UFormField :label="i === 0 ? 'Nominal (Rp)' : undefined">
							<UInputNumber v-model="tx.nominal" :min="0" locale="id-ID" :format-options="{ maximumFractionDigits: 0 }" size="sm" />
						</UFormField>
						<UFormField :label="i === 0 ? 'Tanggal setor' : undefined">
							<UInput :model-value="tx.tanggalSetor ?? ''" type="date" size="sm" @update:model-value="(v) => (tx.tanggalSetor = String(v) || null)" />
						</UFormField>
						<UButton icon="i-lucide-trash-2" color="error" variant="ghost" size="sm" aria-label="Hapus pajak" @click="taxForm.splice(i, 1)" />
					</div>
					<p v-if="!taxForm.length" class="text-sm text-muted">
						Tidak ada pajak (tercetak "-").
					</p>
					<p v-else-if="ovTarget" class="text-sm tabular">
						Belanja {{ rupiah(ovTarget.total) }} − pajak {{ rupiah(taxTotal) }} = dibayarkan <b>{{ rupiah(ovTarget.total - taxTotal) }}</b>
					</p>
				</div>
			</template>
			<template #footer>
				<div class="flex w-full gap-2">
					<UButton color="neutral" variant="ghost" icon="i-lucide-undo-2" @click="resetOverride">
						Kosongkan
					</UButton>
					<UButton class="ml-auto" color="neutral" variant="outline" @click="ovOpen = false">
						Batal
					</UButton>
					<UButton icon="i-lucide-check" :loading="busy" @click="saveOverride">
						Simpan
					</UButton>
				</div>
			</template>
		</UModal>

		<!-- Foto nota -->
		<UModal v-model:open="fotoOpen" :title="`Foto nota ${fotoTarget?.noBukti ?? ''}`" :ui="{ content: 'max-w-3xl' }">
			<template #body>
				<div class="space-y-3">
					<div class="grid grid-cols-2 sm:grid-cols-3 gap-3">
						<div v-for="f in fotoFiles" :key="f.id" class="relative border border-default rounded-md overflow-hidden">
							<img v-if="f.mime.startsWith('image/') && fotoData[f.id]" :src="fotoData[f.id]" alt="Foto nota" class="w-full h-40 object-cover">
							<div v-else class="h-40 flex items-center justify-center text-muted text-sm">
								{{ f.mime === "application/pdf" ? "PDF" : "Memuat..." }}
							</div>
							<UButton icon="i-lucide-trash-2" color="error" variant="solid" size="xs" class="absolute top-1 right-1" aria-label="Hapus foto" @click="deleteFoto(f.id)" />
						</div>
					</div>
					<p v-if="!fotoFiles.length" class="text-sm text-muted">
						Belum ada foto nota.
					</p>
				</div>
			</template>
			<template #footer>
				<div class="flex w-full justify-between gap-2">
					<p class="text-xs text-muted self-center">
						JPG/PNG dikompres otomatis; PDF disimpan apa adanya.
					</p>
					<UButton icon="i-lucide-upload" :loading="busy" @click="uploadFoto">
						Tambah foto
					</UButton>
				</div>
			</template>
		</UModal>
	</LayoutPageShell>
</template>

<script lang="ts" setup>
	import type { DropdownMenuItem, TableColumn } from "@nuxt/ui";
	import DocBatch from "~/components/Print/DocBatch.vue";
	import BuktiPengeluaran from "~/components/Print/Nota/BuktiPengeluaran.vue";
	import KwitansiA2 from "~/components/Print/Nota/KwitansiA2.vue";
	import NotaToko from "~/components/Print/Nota/NotaToko.vue";

	const nominal = { th: "text-right", td: "text-right tabular whitespace-nowrap" };
	const columns: TableColumn<NotaGroup>[] = [
		{ id: "pilih", meta: { class: { th: "w-8", td: "w-8" } } },
		{ accessorKey: "tanggal", header: "Tanggal", cell: ({ row }) => tanggalId(row.original.override?.tanggalBayar || row.original.tanggal), meta: { class: { td: "whitespace-nowrap tabular" } } },
		{ accessorKey: "noBukti", header: "No. Bukti", meta: { class: { td: "whitespace-nowrap" } } },
		{ id: "toko", header: "Toko / Penyedia" },
		{ accessorKey: "uraian", header: "Uraian", meta: { class: { td: "min-w-64" } } },
		{ accessorKey: "total", header: "Jumlah", cell: ({ row }) => angka(row.original.total), meta: { class: nominal } },
		{ accessorKey: "totalPajak", header: "Pajak", meta: { class: nominal } },
		{ id: "status", header: "Status" },
		{ id: "aksi", meta: { class: { th: "w-10" } } }
	];

	const { connected, year, fund, funds } = useArkas();
	const { open: openPrint } = usePrint();
	const toast = useToast();

	const groups = ref<NotaGroup[]>([]);
	const loading = ref(false);
	const busy = ref(false);
	const error = ref("");
	const month = ref(0);
	const search = ref("");
	const statusFilter = ref("semua");
	const metode = ref<"semua" | "siplah" | "langsung">("semua");
	const metodeItems = [
		{ label: "Semua belanja", value: "semua" },
		{ label: "SIPLah", value: "siplah" },
		{ label: "Non-SIPLah", value: "langsung" }
	];
	const checked = ref(new Set<string>());

	const monthItems = [{ label: "Semua bulan", value: 0 }, ...BULAN.map((label, i) => ({ label, value: i + 1 }))];
	const statusItems = [
		{ label: "Semua status", value: "semua" },
		{ label: "Bukti belum dicetak", value: "bukti" },
		{ label: "A2 belum dicetak", value: "a2" },
		{ label: "Nota belum dicetak", value: "nota" }
	];

	const filtered = computed(() => {
		const q = search.value.trim().toLowerCase();
		return groups.value.filter((g) => {
			if (statusFilter.value === "bukti" && g.printedBukti) return false;
			if (statusFilter.value === "a2" && g.printedA2) return false;
			if (statusFilter.value === "nota" && g.printedNota) return false;
			if (metode.value === "siplah" && !g.isSiplah) return false;
			if (metode.value === "langsung" && g.isSiplah) return false;
			if (!q) return true;
			return [g.noBukti, g.uraian, g.nota?.namaToko, g.nota?.noNota, ...g.items.map((i) => i.uraian)]
				.some((v) => v?.toLowerCase().includes(q));
		});
	});

	const selected = computed(() => filtered.value.filter((g) => checked.value.has(g.key)));
	const selectedItemIds = computed(() => selected.value.filter((g) => g.kind !== "merge").flatMap((g) => g.items.map((i) => i.id)));
	const allChecked = computed(() => filtered.value.length > 0 && filtered.value.every((g) => checked.value.has(g.key)));

	const toggle = (key: string) => {
		const next = new Set(checked.value);
		if (next.has(key)) next.delete(key);
		else next.add(key);
		checked.value = next;
	};
	const toggleAll = () => {
		checked.value = allChecked.value ? new Set() : new Set(filtered.value.map((g) => g.key));
	};

	const fundArg = computed(() => (fund.value === ALL_FUNDS ? null : fund.value));

	const load = async () => {
		if (!connected.value || !year.value) return;
		loading.value = true;
		error.value = "";
		try {
			groups.value = await api.notaList(year.value, month.value || null, fundArg.value);
			const keys = new Set(groups.value.map((g) => g.key));
			checked.value = new Set([...checked.value].filter((k) => keys.has(k)));
		} catch (err) {
			error.value = errorMessage(err);
		} finally {
			loading.value = false;
		}
	};

	watch(year, async (y) => {
		if (!y || !connected.value) return;
		const last = await api.lastActiveMonth(y).catch(() => null);
		if ((last ?? 0) === month.value) await load();
		else month.value = last ?? 0;
	}, { immediate: true });
	watch([month, fund, connected], load);

	// Cetak
	const printTemplates = {
		bukti: { title: "Bukti Pengeluaran", component: BuktiPengeluaran },
		a2: { title: "Kwitansi A2", component: KwitansiA2 },
		nota: { title: "Nota Pembelian", component: NotaToko }
	};

	const doPrint = (kind: "bukti" | "a2" | "nota") => {
		const list = selected.value;
		openPrint({
			title: `${printTemplates[kind].title} (${list.length})`,
			component: printTemplates[kind].component,
			props: { groups: list },
			onPrinted: async () => {
				await api.printStatusSet(kind, list.map((g) => g.key), true).catch(() => {});
				await load();
			}
		});
	};
	const printBukti = () => doPrint("bukti");
	const printA2 = () => doPrint("a2");
	const printNota = () => {
		const siplah = selected.value.filter((g) => g.isSiplah).length;
		if (siplah) {
			toast.add({
				title: `${siplah} belanja SIPLah ikut dipilih`,
				description: "Belanja SIPLah sudah punya invoice resmi dari SIPLah. Nota tetap dibuat untuk semua yang dipilih.",
				color: "info"
			});
		}
		doPrint("nota");
	};

	// Cetak dokumen dari template (SP, Nota, Kwitansi, BA, ...)
	const DOC_KEY = "sibos-doc-jenis";
	const docOpen = ref(false);
	const docTemplates = ref<DocTemplate[]>([]);
	const docPenyedia = ref<Penyedia[]>([]);
	const docJenis = ref<JenisDokumen[]>(["sp", "nota", "kwitansi", "ba"]);
	try {
		const saved = localStorage.getItem(DOC_KEY);
		if (saved) docJenis.value = JSON.parse(saved);
	} catch {}

	const hasTemplate = (j: JenisDokumen) => docTemplates.value.some((t) => t.jenis === j);
	const templateLabel = (j: JenisDokumen) => {
		const names = [...new Set(selected.value.map((g) => templateUntuk(docTemplates.value, j, g.nota?.namaToko)?.nama).filter(Boolean))];
		return names.length ? names.join(", ") : "tidak ada template";
	};
	const toggleJenis = (j: JenisDokumen, on: boolean) => {
		const set = new Set(docJenis.value);
		if (on) set.add(j);
		else set.delete(j);
		docJenis.value = JENIS_DOKUMEN.map((x) => x.value).filter((x) => set.has(x));
		try {
			localStorage.setItem(DOC_KEY, JSON.stringify(docJenis.value));
		} catch {}
	};

	const openDoc = async () => {
		docOpen.value = true;
		[docTemplates.value, docPenyedia.value] = await Promise.all([api.docTemplateList().catch(() => []), api.penyediaList().catch(() => [])]);
	};

	const printDocs = async () => {
		if (!year.value) return;
		busy.value = true;
		try {
			// Nomor urut nota dalam setahun untuk isian {urut}.
			const all = await api.notaList(year.value, null, null);
			const urutan = Object.fromEntries([...all].sort((a, b) => a.tanggal.localeCompare(b.tanggal) || a.noBukti.localeCompare(b.noBukti)).map((g, i) => [g.key, i + 1]));
			const jenis = docJenis.value.filter(hasTemplate);
			const list = selected.value;
			docOpen.value = false;
			openPrint({
				title: `Dokumen (${list.length} transaksi)`,
				component: DocBatch,
				props: { groups: list, jenis, templates: docTemplates.value, penyedia: docPenyedia.value, urutan }
			});
		} catch (err) {
			toast.add({ title: "Gagal menyiapkan dokumen", description: errorMessage(err), color: "error" });
		} finally {
			busy.value = false;
		}
	};

	// Gabungkan
	const mergeOpen = ref(false);
	const mergeForm = reactive({ noBukti: "", uraian: "", tanggal: "" });
	const openMerge = () => {
		const first = selected.value[0];
		const nums = [...new Set(selected.value.map((g) => g.noBukti).filter(Boolean))];
		mergeForm.noBukti = nums.length > 1 ? `${nums[0]}-${nums.at(-1)}` : (nums[0] ?? "");
		mergeForm.uraian = first?.uraian ?? "";
		mergeForm.tanggal = "";
		mergeOpen.value = true;
	};
	const saveMerge = async () => {
		if (!year.value) return;
		busy.value = true;
		try {
			await api.mergeCreate({ year: year.value, noBukti: mergeForm.noBukti, uraian: mergeForm.uraian, tanggal: mergeForm.tanggal || null, items: selectedItemIds.value });
			mergeOpen.value = false;
			checked.value = new Set();
			toast.add({ title: "Gabungan disimpan", color: "success" });
			await load();
		} catch (err) {
			toast.add({ title: "Gagal menggabungkan", description: errorMessage(err), color: "error" });
		} finally {
			busy.value = false;
		}
	};

	// Atur cetak
	const JENIS_PAJAK = ["PPh 21", "PPh 22", "PPh 23", "PPh 4(2)", "PPN", "Pajak Daerah"];
	const taxForm = ref<{ jenis: string, nominal: number, tanggalSetor: string | null }[]>([]);
	const taxTotal = computed(() => taxForm.value.reduce((s, t) => s + (Number(t.nominal) || 0), 0));
	const addTax = () => taxForm.value.push({ jenis: "PPh 23", nominal: 0, tanggalSetor: null });

	const ovOpen = ref(false);
	const ovTarget = ref<NotaGroup | null>(null);
	const ovForm = reactive({ noNota: "", tanggalNota: "", tanggalBayar: "", keperluan: "" });
	const openOverride = (g: NotaGroup) => {
		ovTarget.value = g;
		ovForm.noNota = g.override?.noNota ?? "";
		ovForm.tanggalNota = g.override?.tanggalNota ?? "";
		ovForm.tanggalBayar = g.override?.tanggalBayar ?? "";
		ovForm.keperluan = g.override?.keperluan ?? "";
		taxForm.value = g.taxes.map((t) => ({ jenis: t.jenis ?? "Pajak", nominal: t.nominal, tanggalSetor: t.tanggalSetor }));
		ovOpen.value = true;
	};
	const saveOverride = async () => {
		if (!ovTarget.value) return;
		busy.value = true;
		try {
			const g = ovTarget.value;
			if (taxTotal.value > g.total) throw new Error("Total pajak melebihi jumlah belanja.");
			await api.printOverrideSet(g.key, { ...ovForm });
			if (year.value) {
				await api.notaTaxSet({
					year: year.value,
					refId: g.key,
					tanggal: ovForm.tanggalBayar || g.tanggal,
					noBukti: g.noBukti || null,
					keterangan: [g.noBukti, g.nota?.namaToko].filter(Boolean).join(" "),
					fund: funds.value.find((f) => f.name === g.fundName)?.id ?? null,
					taxes: taxForm.value.filter((t) => t.jenis && t.nominal > 0)
				});
			}
			ovOpen.value = false;
			await load();
		} catch (err) {
			toast.add({ title: "Gagal menyimpan", description: errorMessage(err), color: "error" });
		} finally {
			busy.value = false;
		}
	};
	const resetOverride = () => {
		ovForm.noNota = "";
		ovForm.tanggalNota = "";
		ovForm.tanggalBayar = "";
		ovForm.keperluan = "";
	};

	// Foto nota
	const fotoOpen = ref(false);
	const fotoTarget = ref<NotaGroup | null>(null);
	const fotoFiles = ref<NotaFile[]>([]);
	const fotoData = ref<Record<string, string>>({});

	const loadFotos = async () => {
		if (!year.value || !fotoTarget.value) return;
		const all = await api.notaFileList(year.value);
		fotoFiles.value = all.filter((f) => f.refId === fotoTarget.value?.key);
		for (const f of fotoFiles.value) {
			if (f.mime.startsWith("image/") && !fotoData.value[f.id]) {
				fotoData.value[f.id] = await api.notaFileData(f.id).catch(() => "");
			}
		}
	};
	const openFoto = async (g: NotaGroup) => {
		fotoTarget.value = g;
		fotoFiles.value = [];
		fotoOpen.value = true;
		await loadFotos();
	};
	const uploadFoto = async () => {
		if (!year.value || !fotoTarget.value) return;
		const picked = await useTauriDialogOpen({ multiple: true, filters: [{ name: "Foto/PDF nota", extensions: ["jpg", "jpeg", "png", "pdf"] }] });
		const paths = Array.isArray(picked) ? picked : picked ? [picked] : [];
		if (!paths.length) return;
		busy.value = true;
		const m = Number(fotoTarget.value.tanggal.slice(5, 7)) || 1;
		try {
			for (const p of paths) {
				await api.notaFileUpload(year.value, fotoTarget.value.key, m, p);
			}
			await loadFotos();
			await load();
		} catch (err) {
			toast.add({ title: "Gagal mengunggah", description: errorMessage(err), color: "error" });
		} finally {
			busy.value = false;
		}
	};
	const deleteFoto = async (id: string) => {
		await api.notaFileDelete(id).catch((err) => toast.add({ title: "Gagal menghapus", description: errorMessage(err), color: "error" }));
		await loadFotos();
		await load();
	};

	const unmerge = async (g: NotaGroup) => {
		if (!g.mergeId) return;
		await api.mergeDelete(g.mergeId).catch((err) => toast.add({ title: "Gagal", description: errorMessage(err), color: "error" }));
		await load();
	};

	const resetStatus = async (g: NotaGroup) => {
		await api.printStatusSet("bukti", [g.key], false);
		await api.printStatusSet("a2", [g.key], false);
		await api.printStatusSet("nota", [g.key], false);
		await load();
	};

	const rowMenu = (g: NotaGroup): DropdownMenuItem[][] => [
		[
			{ label: "Atur cetak", icon: "i-lucide-sliders-horizontal", onSelect: () => openOverride(g) },
			{ label: "Foto nota", icon: "i-lucide-image", onSelect: () => openFoto(g) }
		],
		[
			...(g.kind === "merge" ? [{ label: "Batalkan gabungan", icon: "i-lucide-split", onSelect: () => unmerge(g) }] : []),
			...(g.printedBukti || g.printedA2 || g.printedNota ? [{ label: "Tandai belum dicetak", icon: "i-lucide-rotate-ccw", onSelect: () => resetStatus(g) }] : [])
		]
	];
</script>
