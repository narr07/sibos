<template>
	<LayoutPageShell title="Pengaturan" :picker="false">
		<template #actions>
			<UButton icon="i-lucide-save" :loading="saving" :disabled="!form" @click="save">
				Simpan
			</UButton>
		</template>

		<div v-if="form && view" class="max-w-4xl mx-auto space-y-6 pb-12">
			<UAlert
				icon="i-lucide-info"
				color="neutral"
				variant="subtle"
				title="Kolom yang dikosongkan memakai data dari ARKAS (tampil samar sebagai contoh)."
				description="Pengaturan ini dipakai di kop, tanda tangan, dan Berita Acara. Data ARKAS tidak diubah."
			/>

			<UCard>
				<template #header>
					<p class="font-medium">
						Pejabat penandatangan
					</p>
				</template>
				<div class="grid sm:grid-cols-2 gap-4">
					<UFormField v-for="f in pejabatFields" :key="f.key" :label="f.label">
						<UInput v-model="form.pejabat[f.key]" :placeholder="view.bawaan.pejabat[f.key] || f.hint" :inputmode="f.key.startsWith('nip') ? 'numeric' : undefined" />
					</UFormField>
				</div>
			</UCard>

			<UCard>
				<template #header>
					<p class="font-medium">
						Kop surat
					</p>
				</template>
				<p class="text-xs text-muted mb-2">
					Pratinjau kop (dipakai di SP, Bukti Pengeluaran, SPTJM, BA, dan template berkop sekolah):
				</p>
				<div class="rounded-md border border-default bg-white text-black p-4 mb-4 overflow-x-auto">
					<div class="min-w-[170mm]">
						<PrintKopSurat :kop="previewKop" />
					</div>
				</div>
				<div class="grid sm:grid-cols-[1fr_auto_auto] gap-4 items-start">
					<div class="grid gap-3">
						<UFormField v-for="f in kopFields" :key="f.key" :label="f.label">
							<UInput v-model="form.kop[f.key]" :placeholder="view.bawaan.kop[f.key] || f.hint" />
						</UFormField>
					</div>
					<UFormField v-for="side in logoSides" :key="side.key" :label="side.label" :hint="side.hint">
						<div class="flex flex-col items-center gap-2">
							<div class="size-28 rounded-md border-2 border-dashed border-default flex items-center justify-center bg-white">
								<img v-if="form.kop[side.key]" :src="form.kop[side.key]" alt="" class="max-w-full max-h-full object-contain">
								<span v-else class="text-xs text-muted text-center px-2">Persegi 1:1, otomatis diratakan tengah</span>
							</div>
							<label class="cursor-pointer text-xs text-primary underline">
								Pilih gambar
								<input type="file" accept="image/*" class="hidden" @change="(e) => uploadLogo(e, side.key)">
							</label>
							<UButton v-if="form.kop[side.key]" size="xs" color="neutral" variant="ghost" icon="i-lucide-x" @click="form.kop[side.key] = ''">
								Hapus
							</UButton>
						</div>
					</UFormField>
				</div>
				<p class="text-xs text-muted mt-3">
					Logo yang diunggah ditempatkan di tengah bingkai persegi tanpa ditarik, sehingga kedua logo sama besar, simetris, dan tidak gepeng.
					Gunakan gambar PNG dengan latar transparan bila ada.
				</p>
				<details class="mt-3">
					<summary class="text-sm cursor-pointer">
						Data kontak sekolah (opsional)
					</summary>
					<div class="grid sm:grid-cols-2 gap-3 mt-3">
						<UFormField v-for="f in kontakFields" :key="f.key" :label="f.label">
							<UInput v-model="form.kop[f.key]" :placeholder="view.bawaan.kop[f.key] || f.hint" />
						</UFormField>
					</div>
				</details>
			</UCard>

			<UCard>
				<template #header>
					<p class="font-medium">
						Berita Acara
					</p>
				</template>
				<div class="grid sm:grid-cols-2 gap-4">
					<UFormField v-for="f in baFields" :key="f.key" :label="f.label">
						<UInput v-model="form.ba[f.key]" :placeholder="view.bawaan.ba[f.key] || f.hint" />
					</UFormField>
				</div>
			</UCard>

			<UCard>
				<template #header>
					<p class="font-medium">
						Cetak
					</p>
				</template>
				<div class="grid sm:grid-cols-2 gap-4">
					<UFormField label="Kota di atas tanda tangan">
						<UInput v-model="form.cetak.kota" :placeholder="view.bawaan.cetak.kota || 'Contoh: Majalengka'" />
					</UFormField>
					<UFormField label="Ukuran kertas">
						<USelect v-model="form.cetak.kertas" :items="kertasItems" />
					</UFormField>
				</div>
			</UCard>
		</div>

		<div v-else-if="error" class="max-w-xl mx-auto py-12">
			<UAlert color="error" variant="subtle" icon="i-lucide-triangle-alert" :title="error" />
		</div>
	</LayoutPageShell>
</template>

<script lang="ts" setup>
	type PejabatKey = keyof Pengaturan["pejabat"];
	type KopKey = keyof Pengaturan["kop"];
	type BaKey = keyof Pengaturan["ba"];

	const { year } = useArkas();
	const toast = useToast();

	const view = ref<PengaturanView | null>(null);
	const form = ref<Pengaturan | null>(null);
	const saving = ref(false);
	const error = ref("");

	const pejabatFields: { key: PejabatKey, label: string, hint: string }[] = [
		{ key: "kepalaSekolah", label: "Kepala Sekolah", hint: "Nama kepala sekolah" },
		{ key: "nipKepalaSekolah", label: "NIP Kepala Sekolah", hint: "Hanya angka" },
		{ key: "bendahara", label: "Bendahara", hint: "Nama bendahara" },
		{ key: "nipBendahara", label: "NIP Bendahara", hint: "Hanya angka" },
		{ key: "pemegangBarang", label: "Pemegang Barang", hint: "Nama pemegang barang" },
		{ key: "nipPemegangBarang", label: "NIP Pemegang Barang", hint: "Hanya angka" },
		{ key: "petugasRekon", label: "Petugas Rekonsiliasi", hint: "Nama petugas" },
		{ key: "nipPetugasRekon", label: "NIP Petugas Rekonsiliasi", hint: "Hanya angka" }
	];
	const kopFields: { key: KopKey, label: string, hint: string }[] = [
		{ key: "pemerintah", label: "Baris 1 (pemerintah)", hint: "PEMERINTAH KABUPATEN ..." },
		{ key: "dinas", label: "Baris 2 (dinas)", hint: "DINAS PENDIDIKAN" },
		{ key: "namaSekolah", label: "Baris 3 (nama sekolah, huruf besar)", hint: "SEKOLAH DASAR NEGERI ..." },
		{ key: "barisAlamat", label: "Baris 4 (alamat)", hint: "Alamat ..., Kecamatan ..., Kabupaten ...-kode pos" },
		{ key: "barisKontak", label: "Baris 5 (NPSN / e-mail / website)", hint: "NPSN. ... E-mail: ... website: ..." }
	];
	const kontakFields: { key: KopKey, label: string, hint: string }[] = [
		{ key: "alamat", label: "Alamat singkat", hint: "Jl. ..." },
		{ key: "telepon", label: "Telepon / Fax", hint: "(0233) ..." },
		{ key: "email", label: "Email", hint: "sekolah@contoh.sch.id" },
		{ key: "laman", label: "Laman", hint: "https://..." }
	];
	const logoSides: { key: "logoKiri" | "logoKanan", label: string, hint: string }[] = [
		{ key: "logoKiri", label: "Logo kiri", hint: "Pemda" },
		{ key: "logoKanan", label: "Logo kanan", hint: "Sekolah" }
	];

	/** Pratinjau: isian yang diketik menimpa nilai bawaan dari ARKAS. */
	const previewKop = computed(() => {
		if (!form.value || !view.value) return null;
		const saved = form.value.kop;
		const base = view.value.bawaan.kop;
		return Object.fromEntries(Object.keys(base).map((k) => {
			const key = k as KopKey;
			return [key, saved[key] || base[key]];
		})) as Pengaturan["kop"];
	});

	const uploadLogo = async (e: Event, key: "logoKiri" | "logoKanan") => {
		const file = (e.target as HTMLInputElement).files?.[0];
		if (!file || !form.value) return;
		try {
			form.value.kop[key] = await fileToSquareLogo(file);
		} catch (err) {
			toast.add({ title: "Gagal memuat logo", description: errorMessage(err), color: "error" });
		}
	};
	const baFields: { key: BaKey, label: string, hint: string }[] = [
		{ key: "nomor", label: "Nomor Berita Acara", hint: "Contoh: 900/001/SD/2026" },
		{ key: "tanggalSurat", label: "Tanggal surat", hint: "Contoh: 01 Juli 2026" },
		{ key: "nomorSk", label: "Nomor SK", hint: "Nomor SK penunjukan" },
		{ key: "tanggalSk", label: "Tanggal SK", hint: "Contoh: 02 Januari 2026" },
		{ key: "tempat", label: "Tempat rekonsiliasi", hint: "Nama tempat" }
	];
	const kertasItems = [{ label: "A4", value: "A4" }, { label: "F4 / Folio", value: "F4" }];

	const load = async () => {
		error.value = "";
		try {
			view.value = await api.pengaturanGet(year.value);
			form.value = structuredClone(toRaw(view.value.tersimpan));
			if (!form.value.cetak.kertas) form.value.cetak.kertas = view.value.efektif.cetak.kertas || "A4";
		} catch (err) {
			error.value = errorMessage(err);
		}
	};

	const save = async () => {
		if (!form.value) return;
		saving.value = true;
		try {
			await api.pengaturanSet(form.value);
			toast.add({ title: "Pengaturan tersimpan", color: "success" });
			await load();
		} catch (err) {
			toast.add({ title: "Gagal menyimpan", description: errorMessage(err), color: "error" });
		} finally {
			saving.value = false;
		}
	};

	watch(year, load, { immediate: true });
</script>
