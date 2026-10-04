<template>
	<LayoutPageShell title="Template Dokumen" :picker="false">
		<template #actions>
			<UPopover>
				<UButton color="neutral" variant="outline" icon="i-lucide-braces">
					Daftar isian
				</UButton>
				<template #content>
					<div class="p-3 max-h-96 overflow-auto w-96 text-sm">
						<p class="text-xs text-muted mb-2">
							Klik untuk menyalin, lalu tempel di teks template.
						</p>
						<button v-for="v in DAFTAR_ISIAN" :key="v.kunci" class="flex w-full items-center gap-2 rounded px-1.5 py-1 hover:bg-elevated text-left" @click="copyVar(v.kunci)">
							<code class="text-primary">{{ kurung(v.kunci) }}</code>
							<span class="text-xs text-muted ml-auto text-right">{{ v.ket }}</span>
						</button>
					</div>
				</template>
			</UPopover>
			<UButton icon="i-lucide-save" :loading="saving" :disabled="!draft" @click="save">
				Simpan
			</UButton>
		</template>

		<div class="grid xl:grid-cols-[16rem_minmax(0,1fr)_minmax(0,1fr)] lg:grid-cols-[16rem_minmax(0,1fr)] gap-4">
			<!-- Daftar template -->
			<div class="space-y-3">
				<UButton block icon="i-lucide-plus" @click="createOpen = true">
					Template baru
				</UButton>
				<UButton v-if="!templates.length" block color="neutral" variant="outline" icon="i-lucide-wand-sparkles" :loading="saving" @click="seed">
					Buat dari contoh Excel
				</UButton>
				<div v-for="j in JENIS_DOKUMEN" :key="j.value">
					<p v-if="templates.some((t) => t.jenis === j.value)" class="text-xs font-medium text-muted uppercase mb-1">
						{{ j.label }}
					</p>
					<button
						v-for="t in templates.filter((x) => x.jenis === j.value)"
						:key="t.id"
						class="w-full text-left rounded-md border px-3 py-2 mb-1.5 transition-colors"
						:class="t.id === selectedId ? 'border-primary bg-primary/5' : 'border-default hover:bg-elevated'"
						@click="select(t.id)"
					>
						<p class="font-medium text-sm truncate">
							{{ t.nama }}
						</p>
						<p class="text-xs text-muted">
							{{ t.data.kertas }} {{ t.data.orientasi === "landscape" ? "landscape" : "" }}{{ t.tokoMatch ? ` · toko: ${t.tokoMatch}` : " · umum" }}
						</p>
					</button>
				</div>
			</div>

			<!-- Editor -->
			<div v-if="draft" class="space-y-3 min-w-0">
				<UCard :ui="{ body: 'p-3 sm:p-3' }">
					<div class="grid sm:grid-cols-2 gap-2">
						<UFormField label="Nama template">
							<UInput v-model="draft.nama" size="sm" />
						</UFormField>
						<UFormField label="Jenis dokumen">
							<USelect v-model="draft.jenis" :items="JENIS_DOKUMEN" size="sm" />
						</UFormField>
						<UFormField label="Khusus toko (kata kunci nama toko)" hint="Kosong = umum">
							<UInput :model-value="draft.tokoMatch ?? ''" size="sm" placeholder="mis. KPR" @update:model-value="(v) => (draft!.tokoMatch = String(v) || null)" />
						</UFormField>
						<UFormField label="Format nomor">
							<UInput v-model="draft.data.nomorFormat" size="sm" placeholder="{urut}/SP/{bulan_romawi}/{tahun}" />
						</UFormField>
						<div class="grid grid-cols-2 gap-2">
							<UFormField label="Kertas">
								<USelect v-model="draft.data.kertas" :items="['A4', 'F4', 'A5']" size="sm" />
							</UFormField>
							<UFormField label="Orientasi">
								<USelect v-model="draft.data.orientasi" :items="[{ label: 'Tegak', value: 'portrait' }, { label: 'Mendatar', value: 'landscape' }]" size="sm" />
							</UFormField>
						</div>
						<div class="grid grid-cols-4 gap-2">
							<UFormField label="Margin (mm)">
								<UInputNumber v-model="draft.data.margin" :min="5" :max="30" size="sm" />
							</UFormField>
							<UFormField label="Margin atas / jilid (mm)">
								<UInputNumber :model-value="draft.data.marginAtas ?? draft.data.margin" :min="5" :max="60" size="sm" @update:model-value="(v) => (draft!.data.marginAtas = Number(v) || undefined)" />
							</UFormField>
							<UFormField label="Huruf (pt)">
								<UInputNumber v-model="draft.data.ukuranHuruf" :min="8" :max="14" size="sm" />
							</UFormField>
							<UFormField label="Geser tanggal" hint="hari">
								<UInputNumber v-model="draft.data.geserHari" :min="-30" :max="30" size="sm" />
							</UFormField>
						</div>
					</div>
					<div class="grid grid-cols-3 gap-2 mt-2">
						<UFormField label="Mulai kerja: mundur min. (hari)">
							<UInputNumber :model-value="jadwal.mulaiMin" :min="0" :max="60" size="sm" @update:model-value="(v) => setJadwal('mulaiMin', v)" />
						</UFormField>
						<UFormField label="Mulai kerja: mundur maks. (hari)">
							<UInputNumber :model-value="jadwal.mulaiMax" :min="0" :max="60" size="sm" @update:model-value="(v) => setJadwal('mulaiMax', v)" />
						</UFormField>
						<UFormField label="Selesai: maks. sebelum BKU (hari)">
							<UInputNumber :model-value="jadwal.selesaiMax" :min="0" :max="30" size="sm" @update:model-value="(v) => setJadwal('selesaiMax', v)" />
						</UFormField>
					</div>
					<p class="text-xs text-muted mt-2">
						{tanggal_mulai} dan {tanggal_selesai} dihitung mundur acak dari tanggal BKU, tetap sama setiap dicetak ulang.
						"Geser tanggal" mengatur {tanggal_dokumen} dari tanggal nota.
					</p>
				</UCard>

				<div v-for="(b, i) in draft.data.blocks" :key="b.id" class="rounded-md border border-default">
					<div class="flex items-center gap-1 px-2 py-1.5 bg-elevated/60 rounded-t-md">
						<button class="flex-1 text-left text-sm font-medium" @click="toggle(b.id)">
							<UIcon :name="open.has(b.id) ? 'i-lucide-chevron-down' : 'i-lucide-chevron-right'" class="size-3.5 align-middle" />
							{{ BLOK_LABEL[b.type] }}
							<span class="text-xs text-muted font-normal">{{ ringkas(b) }}</span>
						</button>
						<UButton icon="i-lucide-arrow-up" size="xs" color="neutral" variant="ghost" :disabled="i === 0" aria-label="Naik" @click="move(i, -1)" />
						<UButton icon="i-lucide-arrow-down" size="xs" color="neutral" variant="ghost" :disabled="i === draft.data.blocks.length - 1" aria-label="Turun" @click="move(i, 1)" />
						<UButton icon="i-lucide-copy" size="xs" color="neutral" variant="ghost" aria-label="Duplikat" @click="dup(i)" />
						<UButton icon="i-lucide-trash-2" size="xs" color="error" variant="ghost" aria-label="Hapus" @click="draft.data.blocks.splice(i, 1)" />
					</div>
					<div v-if="open.has(b.id)" class="p-2">
						<DokumenBlockEditor v-model="draft.data.blocks[i]!" />
					</div>
				</div>

				<div class="flex flex-wrap gap-2">
					<UDropdownMenu :items="addItems">
						<UButton icon="i-lucide-plus" color="neutral" variant="outline">
							Tambah blok
						</UButton>
					</UDropdownMenu>
					<UButton color="neutral" variant="ghost" icon="i-lucide-copy-plus" @click="duplicateTemplate">
						Duplikat template
					</UButton>
					<UButton color="error" variant="ghost" icon="i-lucide-trash-2" class="ml-auto" @click="remove">
						Hapus template
					</UButton>
				</div>
			</div>
			<div v-else class="py-16 text-center text-muted">
				Pilih template atau buat baru.
			</div>

			<!-- Pratinjau -->
			<div v-if="draft" class="space-y-2 min-w-0 lg:col-span-2 xl:col-span-1">
				<div class="flex items-center gap-2">
					<p class="text-sm font-medium">
						Pratinjau
					</p>
					<USelectMenu v-model="sampleKey" :items="sampleItems" value-key="value" size="sm" class="flex-1 min-w-0" placeholder="Pilih contoh nota" />
				</div>
				<div class="overflow-auto rounded-md bg-elevated p-3 max-h-[calc(100vh-12rem)]">
					<div v-if="sample" class="origin-top-left" :style="{ transform: `scale(${zoom})`, width: `${100 / zoom}%` }">
						<PrintDocRender :template="draft.data" :g="sample" :penyedia="penyediaFor(sample)" :urut="urutan[sample.key] ?? 1" />
					</div>
					<p v-else class="text-sm text-muted p-6 text-center">
						{{ connected ? "Belum ada nota untuk contoh." : "Hubungkan ARKAS untuk melihat contoh dengan data asli." }}
					</p>
				</div>
				<div class="flex items-center gap-2 text-xs text-muted">
					<span>Zoom</span>
					<USlider v-model="zoom" :min="0.4" :max="1" :step="0.05" class="w-40" />
				</div>
			</div>
		</div>

		<UModal v-model:open="createOpen" title="Template baru">
			<template #body>
				<div class="space-y-3">
					<UFormField label="Nama">
						<UInput v-model="createForm.nama" />
					</UFormField>
					<UFormField label="Jenis dokumen">
						<USelect v-model="createForm.jenis" :items="JENIS_DOKUMEN" />
					</UFormField>
					<UFormField label="Mulai dari">
						<USelect v-model="createForm.dari" :items="dariItems" />
					</UFormField>
				</div>
			</template>
			<template #footer>
				<div class="flex w-full justify-end gap-2">
					<UButton color="neutral" variant="outline" @click="createOpen = false">
						Batal
					</UButton>
					<UButton icon="i-lucide-check" :loading="saving" @click="create">
						Buat
					</UButton>
				</div>
			</template>
		</UModal>
	</LayoutPageShell>
</template>

<script lang="ts" setup>
	import type { DropdownMenuItem } from "@nuxt/ui";

	const { connected, year } = useArkas();
	const { load: loadDoc } = useDocContext();
	const toast = useToast();
	const { copy } = useClipboard();

	const templates = ref<DocTemplate[]>([]);
	const penyedia = ref<Penyedia[]>([]);
	const selectedId = ref<string | null>(null);
	const draft = ref<DocTemplate | null>(null);
	const saving = ref(false);
	const open = ref(new Set<string>());
	const zoom = ref(0.6);

	// Contoh nota untuk pratinjau
	const notas = ref<NotaGroup[]>([]);
	const sampleKey = ref<string | undefined>();
	const sample = computed(() => notas.value.find((g) => g.key === sampleKey.value) ?? notas.value[0] ?? null);
	const sampleItems = computed(() => notas.value.map((g) => ({ label: `${tanggalId(g.tanggal)} · ${g.noBukti} · ${g.nota?.namaToko ?? "-"} · ${angka(g.total)}`, value: g.key })));
	const urutan = computed(() => Object.fromEntries([...notas.value].sort((a, b) => a.tanggal.localeCompare(b.tanggal)).map((g, i) => [g.key, i + 1])));

	const penyediaFor = (g: NotaGroup) => {
		const n = g.nota?.namaToko?.trim().toLowerCase();
		return penyedia.value.find((p) => p.nama.trim().toLowerCase() === n)?.data ?? null;
	};

	const select = (id: string) => {
		const t = templates.value.find((x) => x.id === id);
		if (!t) return;
		selectedId.value = id;
		draft.value = structuredClone(toRaw(t));
		open.value = new Set();
	};

	const load = async () => {
		templates.value = await api.docTemplateList().catch(() => []);
		penyedia.value = await api.penyediaList().catch(() => []);
		if (!selectedId.value && templates.value[0]) select(templates.value[0].id);
	};

	const loadSamples = async () => {
		if (!connected.value || !year.value) return;
		notas.value = await api.notaList(year.value, null, null).catch(() => []);
		await loadDoc().catch(() => {});
	};

	onMounted(load);
	watch([year, connected], loadSamples, { immediate: true });

	const jadwal = computed(() => ({ ...JADWAL_BAWAAN, ...draft.value?.data.jadwal }));
	const setJadwal = (k: keyof typeof JADWAL_BAWAAN, v: number | null | undefined) => {
		if (!draft.value) return;
		draft.value.data.jadwal = { ...jadwal.value, [k]: Number(v) || 0 };
	};

	const toggle = (id: string) => {
		const next = new Set(open.value);
		if (next.has(id)) next.delete(id);
		else next.add(id);
		open.value = next;
	};

	const move = (i: number, d: number) => {
		const blocks = draft.value?.data.blocks;
		if (!blocks) return;
		const j = i + d;
		if (j < 0 || j >= blocks.length) return;
		const [x] = blocks.splice(i, 1);
		if (x) blocks.splice(j, 0, x);
	};

	const dup = (i: number) => {
		const blocks = draft.value?.data.blocks;
		const b = blocks?.[i];
		if (!blocks || !b) return;
		blocks.splice(i + 1, 0, { ...structuredClone(toRaw(b)), id: Math.random().toString(36).slice(2, 10) });
	};

	const addItems = computed<DropdownMenuItem[]>(() => (Object.keys(BLOK_LABEL) as JenisBlok[]).map((type) => ({
		label: BLOK_LABEL[type],
		onSelect: () => {
			if (!draft.value) return;
			const b = blokBaru(type);
			draft.value.data.blocks.push(b);
			open.value = new Set([...open.value, b.id]);
		}
	})));

	const ringkas = (b: Blok) => {
		switch (b.type) {
		case "judul": return `· ${b.teks}`;
		case "teks": return `· ${b.isi.slice(0, 40)}${b.isi.length > 40 ? "…" : ""}`;
		case "kop": return `· ${b.sumber}`;
		case "tabel": return `· ${b.kolom.length} kolom`;
		case "ttd": return `· ${b.kolom.length} orang`;
		case "isian": return `· ${b.baris.length} baris`;
		default: return "";
		}
	};

	const kurung = (k: string) => `{${k}}`;

	const copyVar = async (k: string) => {
		await copy(`{${k}}`);
		toast.add({ title: `{${k}} disalin`, color: "success" });
	};

	const save = async () => {
		if (!draft.value) return;
		saving.value = true;
		try {
			const id = await api.docTemplateSave(toRaw(draft.value));
			toast.add({ title: "Template tersimpan", color: "success" });
			await load();
			select(id);
		} catch (err) {
			toast.add({ title: "Gagal menyimpan", description: errorMessage(err), color: "error" });
		} finally {
			saving.value = false;
		}
	};

	/** Buat 4 template awal dari file Excel (SP, Nota, Kwitansi, BA) + contoh profil KPR Rajagaluh. */
	const seed = async () => {
		saving.value = true;
		try {
			for (const [i, t] of TEMPLATE_AWAL().entries()) {
				await api.docTemplateSave({ ...t, id: "", urutan: i });
			}
			if (!penyedia.value.some((p) => p.nama === PENYEDIA_CONTOH.nama)) {
				await api.penyediaSave(PENYEDIA_CONTOH);
			}
			toast.add({ title: "4 template contoh dibuat", description: "SP, Nota, Kwitansi, BA Serah Terima. Profil KPR Rajagaluh ditambahkan di Data Penyedia.", color: "success" });
			await load();
		} catch (err) {
			toast.add({ title: "Gagal membuat template", description: errorMessage(err), color: "error" });
		} finally {
			saving.value = false;
		}
	};

	// Template baru
	const createOpen = ref(false);
	const createForm = reactive<{ nama: string, jenis: JenisDokumen, dari: string }>({ nama: "", jenis: "nota", dari: "kosong" });
	const dariItems = computed(() => [
		{ label: "Kosong", value: "kosong" },
		...TEMPLATE_AWAL().map((t, i) => ({ label: `Contoh: ${t.nama}`, value: `awal:${i}` })),
		...templates.value.map((t) => ({ label: `Salin: ${t.nama}`, value: `salin:${t.id}` }))
	]);
	const create = async () => {
		let data: TemplateData = { kertas: "A4", orientasi: "portrait", geserHari: 0, nomorFormat: "{no_bukti}", ukuranHuruf: 11, margin: 15, blocks: [blokBaru("judul")] };
		if (createForm.dari.startsWith("awal:")) data = TEMPLATE_AWAL()[Number(createForm.dari.slice(5))]?.data ?? data;
		if (createForm.dari.startsWith("salin:")) data = structuredClone(toRaw(templates.value.find((t) => t.id === createForm.dari.slice(6))?.data)) ?? data;
		saving.value = true;
		try {
			const id = await api.docTemplateSave({ id: "", nama: createForm.nama || "Template baru", jenis: createForm.jenis, tokoMatch: null, data, urutan: templates.value.length });
			createOpen.value = false;
			await load();
			select(id);
		} catch (err) {
			toast.add({ title: "Gagal membuat", description: errorMessage(err), color: "error" });
		} finally {
			saving.value = false;
		}
	};

	const duplicateTemplate = async () => {
		if (!draft.value) return;
		const id = await api.docTemplateSave({ ...structuredClone(toRaw(draft.value)), id: "", nama: `${draft.value.nama} (salinan)` }).catch((err) => {
			toast.add({ title: "Gagal", description: errorMessage(err), color: "error" });
			return null;
		});
		if (id) {
			await load();
			select(id);
		}
	};

	const remove = async () => {
		if (!draft.value?.id) return;
		await api.docTemplateDelete(draft.value.id);
		selectedId.value = null;
		draft.value = null;
		await load();
	};
</script>
