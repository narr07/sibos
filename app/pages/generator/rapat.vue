<template>
	<LayoutPageShell title="Generator Konsumsi Rapat">
		<template #actions>
			<UButton color="neutral" variant="outline" icon="i-lucide-save" :loading="saving" :disabled="!form" @click="save()">
				Simpan
			</UButton>
			<UButton icon="i-lucide-printer" :disabled="!form || !jenis.length" @click="print">
				Pratinjau & cetak
			</UButton>
		</template>

		<div v-if="!connected" class="py-16 text-center text-muted">
			Hubungkan ARKAS terlebih dahulu.
		</div>

		<div v-else class="grid lg:grid-cols-[22rem_1fr] gap-4">
			<GeneratorDaftarBelanja
				v-model:month="month"
				v-model:search="search"
				v-model:hanya="hanyaCocok"
				:groups="filtered"
				:selected-key="selectedKey"
				:tersimpan="saved"
				:loading="loading"
				:error="error"
				label-saring="Hanya belanja konsumsi / rapat"
				@pick="pick"
			/>

			<div v-if="form && selected" class="space-y-4 max-w-4xl pb-12">
				<GeneratorInfoBelanja :g="selected" :nama-kegiatan="namaKegiatanArkas" />

				<UCard>
					<template #header>
						<div class="flex items-center justify-between gap-2">
							<p class="font-medium">
								Rapat
							</p>
							<UButton size="xs" color="neutral" variant="ghost" icon="i-lucide-rotate-ccw" @click="reset">
								Isi ulang dari ARKAS
							</UButton>
						</div>
					</template>
					<div class="grid sm:grid-cols-2 gap-3">
						<UFormField label="Judul / nama kegiatan rapat" class="sm:col-span-2" help="Agenda, pengarahan, pembahasan, dan keputusan ikut menyesuaikan judul, kecuali yang sudah Anda ubah sendiri.">
							<div class="flex gap-2">
								<UInput :model-value="form.namaKegiatan" placeholder="Rapat Penyusunan RKAS" class="flex-1" @update:model-value="(v) => gantiJudul(String(v))" />
								<UDropdownMenu v-if="saranMenu.length > 1" :items="saranMenu" :content="{ align: 'end' }" :ui="{ content: 'max-w-md' }">
									<UButton color="neutral" variant="outline" icon="i-lucide-list" trailing-icon="i-lucide-chevron-down">
										Saran
									</UButton>
								</UDropdownMenu>
								<UButton color="neutral" variant="outline" icon="i-lucide-wand-sparkles" @click="isiDariJudul">
									Isi ulang dari judul
								</UButton>
							</div>
						</UFormField>
						<UFormField label="Agenda" class="sm:col-span-2">
							<UInput v-model="form.agenda" placeholder="Pembahasan Rencana Kegiatan dan Anggaran Sekolah" class="w-full" />
						</UFormField>
						<UFormField label="Tanggal pelaksanaan" :hint="hariDari(form.tanggal)">
							<UInput v-model="form.tanggal" type="date" class="w-full" />
						</UFormField>
						<UFormField label="Tempat">
							<UInput v-model="form.tempat" placeholder="Ruang Guru" class="w-full" />
						</UFormField>
						<UFormField label="Waktu">
							<div class="flex items-center gap-2">
								<UInput v-model="form.waktuMulai" type="time" aria-label="Waktu mulai" />
								<span class="text-sm text-muted">s.d.</span>
								<UInput v-model="form.waktuSelesai" type="time" aria-label="Waktu selesai" />
								<USelect v-model="form.zona" :items="ZONA" class="w-24" aria-label="Zona waktu" />
							</div>
						</UFormField>
					</div>
				</UCard>

				<UCard>
					<template #header>
						<p class="font-medium">
							Surat undangan
						</p>
					</template>
					<div class="grid sm:grid-cols-2 gap-3">
						<UFormField label="Nomor surat">
							<UInput v-model="form.nomorSurat" :placeholder="`421.2/045/BOS/${year}`" class="w-full" />
						</UFormField>
						<UFormField label="Tanggal surat" :hint="hariDari(form.tanggalSurat)">
							<UInput v-model="form.tanggalSurat" type="date" class="w-full" />
						</UFormField>
						<UFormField label="Lampiran">
							<UInput v-model="form.lampiran" placeholder="- atau 1 lembar" class="w-full" />
						</UFormField>
						<UFormField label="Kepada Yth.">
							<UInput v-model="form.kepada" class="w-full" />
						</UFormField>
					</div>
				</UCard>

				<UCard>
					<template #header>
						<div class="flex items-center justify-between gap-2">
							<p class="font-medium">
								Daftar hadir · {{ jumlahHadir }} hadir, {{ guru.length - hadirIds.size }} tidak hadir
							</p>
							<UButton v-if="guru.length" size="xs" color="neutral" variant="ghost" @click="toggleSemua">
								{{ hadirIds.size === guru.length ? "Kosongkan" : "Pilih semua" }}
							</UButton>
						</div>
					</template>
					<p v-if="!guru.length" class="text-sm">
						Daftar guru masih kosong.
						<NuxtLink to="/lainnya/pengaturan" class="text-primary underline">
							Isi di menu Pengaturan
						</NuxtLink>
						(nama, tugas, wali kelas / mata pelajaran).
					</p>
					<div v-else class="grid sm:grid-cols-2 gap-x-4 gap-y-2">
						<UCheckbox
							v-for="g in guru"
							:key="g.id"
							:model-value="hadirIds.has(g.id)"
							:label="g.nama"
							:description="peranGuru(g)"
							@update:model-value="(v) => setHadir(g.id, !!v)"
						/>
					</div>
					<UFormField label="Baris kosong tambahan" hint="Untuk komite / tamu, ditulis tangan" class="mt-4 max-w-xs">
						<UInputNumber v-model="form.barisKosong" :min="0" :max="50" />
					</UFormField>
					<p v-if="porsi" class="mt-3 text-xs" :class="porsi === jumlahHadir ? 'text-muted' : 'text-warning'">
						Konsumsi di BKU: {{ angka(porsi) }} {{ satuanPorsi }} · peserta di daftar hadir: {{ jumlahHadir }} orang.
					</p>
				</UCard>

				<UCard>
					<template #header>
						<p class="font-medium">
							Berita acara & notulen
						</p>
					</template>
					<div class="grid sm:grid-cols-3 gap-3">
						<UFormField label="Nomor berita acara" class="sm:col-span-3">
							<UInput v-model="form.nomorBa" :placeholder="`005/012/BA/${year}`" class="w-full sm:w-1/2" />
						</UFormField>
						<UFormField label="Pimpinan rapat">
							<UInput v-model="form.pimpinan" class="w-full" />
						</UFormField>
						<UFormField label="Jabatan pimpinan">
							<UInput v-model="form.jabatanPimpinan" class="w-full" />
						</UFormField>
						<UFormField label="NIP pimpinan">
							<UInput v-model="form.nipPimpinan" class="w-full" />
						</UFormField>
						<UFormField label="Notulis" class="sm:col-span-2">
							<div class="flex gap-2">
								<UInput v-model="form.notulis" placeholder="Nama guru / staf pencatat rapat" class="flex-1" />
								<UDropdownMenu v-if="guru.length" :items="notulisMenu" :content="{ align: 'end' }">
									<UButton color="neutral" variant="outline" icon="i-lucide-users-round" trailing-icon="i-lucide-chevron-down">
										Pilih
									</UButton>
								</UDropdownMenu>
							</div>
						</UFormField>
						<UFormField label="NIP notulis">
							<UInput v-model="form.nipNotulis" class="w-full" />
						</UFormField>
						<UFormField label="Pengarahan pimpinan rapat" class="sm:col-span-3">
							<UTextarea v-model="form.arahan" :rows="2" autoresize class="w-full" />
						</UFormField>
						<UFormField label="Pembahasan inti" class="sm:col-span-3">
							<UTextarea v-model="form.pembahasan" :rows="3" autoresize class="w-full" />
						</UFormField>
						<UFormField label="Hasil / keputusan rapat" hint="Satu poin per baris" class="sm:col-span-3">
							<UTextarea v-model="form.keputusan" :rows="4" autoresize class="w-full" />
						</UFormField>
					</div>
				</UCard>

				<UCard>
					<template #header>
						<p class="font-medium">
							Dokumen yang dicetak
						</p>
					</template>
					<div class="flex flex-wrap gap-x-6 gap-y-2">
						<UCheckbox
							v-for="d in DOKUMEN_RAPAT"
							:key="d.value"
							:model-value="jenis.includes(d.value)"
							:label="d.label"
							@update:model-value="(v) => toggleJenis(d.value, !!v)"
						/>
					</div>
				</UCard>
			</div>
			<div v-else class="py-16 text-center text-muted">
				Pilih belanja konsumsi di sebelah kiri untuk membuat dokumen rapatnya.
			</div>
		</div>
	</LayoutPageShell>
</template>

<script lang="ts" setup>
	import type { DropdownMenuItem } from "@nuxt/ui";
	import Dokumen from "~/components/Print/Rapat/Dokumen.vue";

	const { open: openPrint } = usePrint();

	const ZONA = ["WIB", "WITA", "WIT"];

	const {
		connected,
		year,
		guru,
		saved,
		loading,
		saving,
		error,
		month,
		search,
		hanyaCocok,
		selectedKey,
		selected,
		form,
		filtered,
		namaKegiatanArkas,
		pick,
		reset,
		save
	} = useGeneratorBelanja<RapatData>({
		setting: settingRapat,
		cocok: isKonsumsi,
		pesanSimpan: "Dokumen rapat tersimpan",
		buat: (g, tersimpan, ctx) => {
			// Notulis bergilir menurut jumlah rapat lain yang sudah tersimpan tahun ini.
			const urutan = Object.keys(ctx.saved).filter((k) => k !== g.key).length;
			const notulis = notulisBergilir(ctx.guru, urutan, ctx.pejabat?.kepalaSekolah);
			const bawaan: RapatData = {
				...rapatBawaan(g, { namaKegiatan: ctx.namaKegiatan, pejabat: ctx.pejabat, guru: ctx.guru }),
				notulis: notulis?.nama ?? "",
				nipNotulis: notulis?.nip ?? ""
			};
			if (!tersimpan) return bawaan;
			const ids = new Set(ctx.guru.map((x) => x.id));
			return { ...bawaan, ...tersimpan, peserta: tersimpan.peserta.filter((id) => ids.has(id)) };
		}
	});

	const jenis = ref<JenisDokumenRapat[]>(DOKUMEN_RAPAT.map((d) => d.value));

	/** Jumlah porsi konsumsi di BKU (volume terbesar), sebagai pembanding jumlah peserta. */
	const itemPorsi = computed(() => selected.value?.items.reduce<NotaItem | null>((max, i) => ((i.volume ?? 0) > (max?.volume ?? 0) ? i : max), null) ?? null);
	const porsi = computed(() => itemPorsi.value?.volume ?? 0);
	const satuanPorsi = computed(() => itemPorsi.value?.satuan ?? "porsi");

	const hadirIds = computed(() => new Set(form.value?.peserta ?? []));
	const jumlahHadir = computed(() => hadirIds.value.size + Math.max(0, form.value?.barisKosong ?? 0));

	const setHadir = (id: string, hadir: boolean) => {
		if (!form.value) return;
		// Urutan mengikuti daftar guru di Pengaturan.
		form.value.peserta = guru.value.map((g) => g.id).filter((x) => (x === id ? hadir : hadirIds.value.has(x)));
	};
	const toggleSemua = () => {
		if (form.value) form.value.peserta = hadirIds.value.size === guru.value.length ? [] : guru.value.map((g) => g.id);
	};
	const toggleJenis = (j: JenisDokumenRapat, on: boolean) => {
		jenis.value = DOKUMEN_RAPAT.map((d) => d.value).filter((v) => (v === j ? on : jenis.value.includes(v)));
	};

	const notulisMenu = computed<DropdownMenuItem[]>(() => guru.value.map((g) => ({
		label: g.nama,
		onSelect: () => {
			if (!form.value) return;
			form.value.notulis = g.nama;
			form.value.nipNotulis = g.nip;
		}
	})));

	/** Ganti judul: kolom isi yang masih sama dengan isi otomatis judul lama ikut diperbarui. */
	const gantiJudul = (judul: string) => {
		if (!form.value) return;
		const lama = isiRapat(form.value.namaKegiatan);
		const baru = isiRapat(judul);
		for (const k of KOLOM_ISI) {
			if (form.value[k].trim() === lama[k].trim()) form.value[k] = baru[k];
		}
		form.value.namaKegiatan = judul;
	};
	/** Timpa agenda dan isi notulen dengan isi otomatis dari judul saat ini. */
	const isiDariJudul = () => {
		if (form.value) Object.assign(form.value, isiRapat(form.value.namaKegiatan));
	};

	/** Pilihan judul dari uraian BKU, uraian item, dan nama kegiatan RKAS. */
	const saranMenu = computed<DropdownMenuItem[]>(() =>
		(selected.value ? saranJudul(selected.value, namaKegiatanArkas.value) : []).map((judul) => ({
			label: judul,
			onSelect: () => gantiJudul(judul)
		})));

	const print = async () => {
		if (!form.value || !selected.value) return;
		await save(true);
		openPrint({
			title: form.value.namaKegiatan,
			component: Dokumen,
			props: { r: structuredClone(toRaw(form.value)), guru: toRaw(guru.value), jenis: [...jenis.value] }
		});
	};
</script>
