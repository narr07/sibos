<template>
	<LayoutPageShell title="Generator SPPD">
		<template #actions>
			<UButton color="neutral" variant="outline" icon="i-lucide-save" :loading="saving" :disabled="!form" @click="save()">
				Simpan
			</UButton>
			<UButton icon="i-lucide-printer" :disabled="!form || !jenis.length || !pelaksanaIds.size" @click="print">
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
				label-saring="Hanya belanja transport / perjalanan dinas"
				@pick="pick"
			/>

			<div v-if="form && selected" class="space-y-4 max-w-4xl pb-12">
				<GeneratorInfoBelanja :g="selected" :nama-kegiatan="namaKegiatanArkas" />

				<UCard>
					<template #header>
						<div class="flex items-center justify-between gap-2">
							<p class="font-medium">
								Perjalanan dinas
							</p>
							<UButton size="xs" color="neutral" variant="ghost" icon="i-lucide-rotate-ccw" @click="reset">
								Isi ulang dari ARKAS
							</UButton>
						</div>
					</template>
					<div class="grid sm:grid-cols-2 gap-3">
						<UFormField label="Maksud perjalanan dinas" class="sm:col-span-2" help="Tulis sebagai kalimat tugas, mis. &quot;Mengikuti Rapat K3S&quot; atau &quot;Melaksanakan tugas sebagai Pengawas TKA&quot;.">
							<UInput v-model="form.maksud" class="w-full" />
						</UFormField>
						<UFormField label="Tempat tujuan">
							<UInput v-model="form.tujuan" placeholder="mis. SD Negeri Rajagaluh I" class="w-full" />
						</UFormField>
						<UFormField label="Tempat berangkat">
							<UInput v-model="form.berangkatDari" class="w-full" />
						</UFormField>
						<UFormField label="Tanggal berangkat" :hint="hariDari(form.tanggalBerangkat)">
							<UInput v-model="form.tanggalBerangkat" type="date" class="w-full" />
						</UFormField>
						<UFormField label="Tanggal kembali" :hint="`Lama: ${lamaTeks(form)}`">
							<UInput v-model="form.tanggalKembali" type="date" class="w-full" />
						</UFormField>
						<UFormField label="Jam berangkat">
							<UInput v-model="form.jamBerangkat" type="time" />
						</UFormField>
						<UFormField label="Alat angkutan">
							<UInput v-model="form.angkutan" placeholder="Kendaraan umum / kendaraan pribadi" class="w-full" />
						</UFormField>
						<UFormField label="Jenis perjalanan">
							<USelect v-model="form.daerah" :items="DAERAH" class="w-full" />
						</UFormField>
					</div>
				</UCard>

				<UCard>
					<template #header>
						<div class="flex items-center justify-between gap-2">
							<p class="font-medium">
								Pelaksana · {{ pelaksanaIds.size }} orang
							</p>
							<UButton v-if="guru.length" size="xs" color="neutral" variant="ghost" @click="toggleSemua">
								{{ pelaksanaIds.size === guru.length ? "Kosongkan" : "Pilih semua" }}
							</UButton>
						</div>
					</template>
					<p v-if="!guru.length" class="text-sm">
						Daftar guru masih kosong.
						<NuxtLink to="/lainnya/pengaturan" class="text-primary underline">
							Isi di menu Pengaturan
						</NuxtLink>
						(nama, NIP, pangkat/golongan, tugas).
					</p>
					<div v-else class="grid sm:grid-cols-2 gap-x-4 gap-y-2">
						<UCheckbox
							v-for="g in guru"
							:key="g.id"
							:model-value="pelaksanaIds.has(g.id)"
							:label="g.nama"
							:description="[jabatanSppd(g), g.pangkat].filter(Boolean).join(' · ')"
							@update:model-value="(v) => setPelaksana(g.id, !!v)"
						/>
					</div>
					<p class="mt-2 text-xs text-muted">
						Surat Tugas memuat semua pelaksana; dokumen lainnya dicetak satu berkas per orang.
					</p>
				</UCard>

				<UCard>
					<template #header>
						<p class="font-medium">
							Biaya per orang · {{ rupiah(perOrang) }}
						</p>
					</template>
					<div class="grid grid-cols-2 sm:grid-cols-3 gap-3">
						<UFormField v-for="b in BIAYA_SPPD" :key="b.key" :label="b.label">
							<UInputNumber v-model="form.biaya[b.key]" :min="0" :step="1000" :format-options="{ maximumFractionDigits: 0 }" class="w-full" />
						</UFormField>
					</div>
					<p class="mt-3 text-xs" :class="totalSemua === selected.total ? 'text-muted' : 'text-warning'">
						{{ pelaksanaIds.size }} orang × {{ rupiah(perOrang) }} = {{ rupiah(totalSemua) }} · belanja di BKU: {{ rupiah(selected.total) }}
					</p>
				</UCard>

				<UCard>
					<template #header>
						<p class="font-medium">
							Surat tugas, SPD & kuitansi
						</p>
					</template>
					<div class="grid sm:grid-cols-2 gap-3">
						<UFormField label="Nomor surat tugas">
							<UInput v-model="form.nomorSuratTugas" :placeholder="`800/012/SD/${year}`" class="w-full" />
						</UFormField>
						<UFormField label="Tanggal surat tugas / SPD" :hint="hariDari(form.tanggalSurat)">
							<UInput v-model="form.tanggalSurat" type="date" class="w-full" />
						</UFormField>
						<UFormField label="Dasar surat tugas" class="sm:col-span-2" hint="Kosong = titik-titik">
							<UInput v-model="form.dasar" placeholder="mis. Surat Undangan Dinas Pendidikan Nomor ... tanggal ..." class="w-full" />
						</UFormField>
						<UFormField label="Nomor SPD">
							<UInput v-model="form.nomorSpd" :placeholder="`090/012/SPD/${year}`" class="w-full" />
						</UFormField>
						<UFormField label="Tingkat biaya perjalanan dinas" hint="Boleh kosong">
							<UInput v-model="form.tingkatBiaya" class="w-full" />
						</UFormField>
						<UFormField label="Instansi (pembebanan anggaran)">
							<UInput v-model="form.instansi" class="w-full" />
						</UFormField>
						<UFormField label="Kode rekening / akun">
							<UInput v-model="form.kodeRekening" class="w-full" />
						</UFormField>
						<UFormField label="Nomor kuitansi">
							<UInput v-model="form.nomorKuitansi" class="w-full" />
						</UFormField>
						<UFormField label="Tanggal kuitansi (dibayar)" :hint="hariDari(form.tanggalBayar)">
							<UInput v-model="form.tanggalBayar" type="date" class="w-full" />
						</UFormField>
						<UFormField label="Sumber dana" class="sm:col-span-2">
							<UInput v-model="form.sumberDana" placeholder="BOSP Reguler" class="w-full" />
						</UFormField>
					</div>
				</UCard>

				<UCard>
					<template #header>
						<p class="font-medium">
							Laporan pelaksanaan tugas
						</p>
					</template>
					<UFormField label="Rangkaian acara" hint="Satu acara per baris">
						<UTextarea v-model="form.acara" :rows="4" autoresize class="w-full" />
					</UFormField>
				</UCard>

				<UCard>
					<template #header>
						<p class="font-medium">
							Dokumen yang dicetak
						</p>
					</template>
					<div class="flex flex-wrap gap-x-6 gap-y-2">
						<UCheckbox
							v-for="d in DOKUMEN_SPPD"
							:key="d.value"
							:model-value="jenis.includes(d.value)"
							:label="d.label"
							@update:model-value="(v) => toggleJenis(d.value, !!v)"
						/>
					</div>
				</UCard>
			</div>
			<div v-else class="py-16 text-center text-muted">
				Pilih belanja transport di sebelah kiri untuk membuat SPPD-nya. Bila belanjanya tercatat di rekening lain
				(mis. konsumsi), matikan saringan lalu cari.
			</div>
		</div>
	</LayoutPageShell>
</template>

<script lang="ts" setup>
	import Dokumen from "~/components/Print/Sppd/Dokumen.vue";

	const { open: openPrint } = usePrint();

	const DAERAH = [{ label: "Dalam daerah", value: "dalam daerah" }, { label: "Luar daerah", value: "luar daerah" }];

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
	} = useGeneratorBelanja<SppdData>({
		setting: settingSppd,
		cocok: isTransport,
		pesanSimpan: "SPPD tersimpan",
		buat: (g, tersimpan, ctx) => {
			const bawaan = sppdBawaan(g, { sekolah: ctx.sekolah });
			if (!tersimpan) return bawaan;
			const ids = new Set(ctx.guru.map((x) => x.id));
			return { ...bawaan, ...tersimpan, biaya: { ...bawaan.biaya, ...tersimpan.biaya }, pelaksana: tersimpan.pelaksana.filter((id) => ids.has(id)) };
		}
	});

	const jenis = ref<JenisDokumenSppd[]>(DOKUMEN_SPPD.map((d) => d.value));

	const pelaksanaIds = computed(() => new Set(form.value?.pelaksana ?? []));
	const perOrang = computed(() => (form.value ? totalBiaya(form.value.biaya) : 0));
	const totalSemua = computed(() => perOrang.value * pelaksanaIds.value.size);

	const setPelaksana = (id: string, ikut: boolean) => {
		if (!form.value) return;
		// Urutan mengikuti daftar guru di Pengaturan.
		form.value.pelaksana = guru.value.map((g) => g.id).filter((x) => (x === id ? ikut : pelaksanaIds.value.has(x)));
	};
	const toggleSemua = () => {
		if (form.value) form.value.pelaksana = pelaksanaIds.value.size === guru.value.length ? [] : guru.value.map((g) => g.id);
	};
	const toggleJenis = (j: JenisDokumenSppd, on: boolean) => {
		jenis.value = DOKUMEN_SPPD.map((d) => d.value).filter((v) => (v === j ? on : jenis.value.includes(v)));
	};

	const print = async () => {
		if (!form.value || !selected.value) return;
		await save(true);
		openPrint({
			title: `SPPD ${form.value.maksud}`,
			component: Dokumen,
			props: { d: structuredClone(toRaw(form.value)), guru: toRaw(guru.value), jenis: [...jenis.value] }
		});
	};
</script>
