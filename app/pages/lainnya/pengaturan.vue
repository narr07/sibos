<template>
	<LayoutPageShell title="Pengaturan" :picker="false">
		<template #actions>
			<UButton icon="i-lucide-save" :loading="saving" :disabled="!form" @click="save">
				Simpan
			</UButton>
		</template>

		<div v-if="form && view" class="w-full max-w-4xl mx-auto space-y-6 pb-12">
			<UAlert
				icon="i-lucide-info"
				color="neutral"
				variant="subtle"
				title="Isian otomatis diambil dari ARKAS dan tetap bisa diubah manual."
				description="Kolom bertanda ARKAS mengikuti data ARKAS terbaru. Yang diubah manual disimpan di SIBOS; data ARKAS tidak diubah."
			/>

			<UCard>
				<template #header>
					<p class="font-medium">
						Pejabat penandatangan
					</p>
				</template>
				<div class="grid sm:grid-cols-2 gap-4">
					<UFormField v-for="f in pejabatFields" :key="f.key" :label="f.label">
						<template #hint>
							<SumberIsian :sumber="sumber('pejabat', f.key)" @reset="resetKe('pejabat', f.key)" />
						</template>
						<UInput v-model="form.pejabat[f.key]" :placeholder="f.hint" :inputmode="f.key.startsWith('nip') ? 'numeric' : undefined" />
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
							<template #hint>
								<SumberIsian :sumber="sumber('kop', f.key)" @reset="resetKe('kop', f.key)" />
							</template>
							<UInput v-model="form.kop[f.key]" :placeholder="f.hint" />
						</UFormField>
					</div>
					<UFormField v-for="side in logoSides" :key="side.key" :label="side.label" :hint="side.hint">
						<div class="flex flex-col items-center gap-2">
							<div class="size-28 rounded-md border-2 border-dashed border-default flex items-center justify-center bg-white">
								<img v-if="form.kop[side.key]" :src="form.kop[side.key]" alt="" class="max-w-full max-h-full object-contain">
								<span v-else class="text-xs text-muted text-center px-2">Persegi 1:1, otomatis diratakan tengah</span>
							</div>
							<UFileUpload v-slot="{ open }" accept="image/*" reset :preview="false" @update:model-value="(f) => uploadLogo(f, side.key)">
								<UButton size="xs" color="neutral" variant="outline" icon="i-lucide-image-up" @click="open()">
									Pilih gambar
								</UButton>
							</UFileUpload>
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
				<UCollapsible class="mt-3">
					<UButton color="neutral" variant="ghost" size="sm" trailing-icon="i-lucide-chevron-down" :ui="{ trailingIcon: 'group-data-[state=open]:rotate-180 transition-transform' }">
						Data kontak sekolah (opsional)
					</UButton>
					<template #content>
						<div class="grid sm:grid-cols-2 gap-3 mt-3">
							<UFormField v-for="f in kontakFields" :key="f.key" :label="f.label">
								<UInput v-model="form.kop[f.key]" :placeholder="f.hint" />
							</UFormField>
						</div>
					</template>
				</UCollapsible>
			</UCard>

			<UCard>
				<template #header>
					<div class="flex items-center justify-between gap-2">
						<p class="font-medium">
							Guru & tenaga kependidikan
						</p>
						<UButton size="xs" color="neutral" variant="ghost" icon="i-lucide-user-round-check" @click="guruDariPejabat">
							Tambah dari pejabat
						</UButton>
					</div>
				</template>
				<p class="text-xs text-muted mb-3">
					Dipakai untuk daftar hadir rapat dan SPPD di menu Generator (pangkat/golongan hanya untuk SPPD). Urutan di sini menjadi urutan di daftar hadir.
				</p>
				<div class="space-y-2">
					<div v-for="(g, i) in guru" :key="g.id" class="grid grid-cols-[1.5rem_1.5fr_1.1fr_1fr_0.9fr_1fr_auto] items-center gap-2">
						<span class="text-sm text-muted text-right">{{ i + 1 }}</span>
						<UInput v-model="g.nama" placeholder="Nama lengkap dan gelar" :aria-label="`Nama guru ${i + 1}`" />
						<UInput v-model="g.nip" placeholder="NIP / NUPTK" :aria-label="`NIP guru ${i + 1}`" />
						<UInput v-model="g.pangkat" placeholder="Pangkat / Gol." :aria-label="`Pangkat golongan guru ${i + 1}`" />
						<UInput v-model="g.tugas" placeholder="Tugas, mis. Guru Kelas" :aria-label="`Tugas guru ${i + 1}`" />
						<UInput v-model="g.kelasMapel" placeholder="Wali Kelas 4 / PJOK" :aria-label="`Kelas atau mata pelajaran guru ${i + 1}`" />
						<div class="flex">
							<UButton size="xs" color="neutral" variant="ghost" icon="i-lucide-arrow-up" :disabled="i === 0" aria-label="Naikkan" @click="geserGuru(i, -1)" />
							<UButton size="xs" color="neutral" variant="ghost" icon="i-lucide-arrow-down" :disabled="i === guru.length - 1" aria-label="Turunkan" @click="geserGuru(i, 1)" />
							<UButton size="xs" color="error" variant="ghost" icon="i-lucide-trash-2" aria-label="Hapus" @click="guru.splice(i, 1)" />
						</div>
					</div>
				</div>
				<UButton class="mt-3" size="sm" color="neutral" variant="outline" icon="i-lucide-plus" @click="guru.push(guruKosong())">
					Tambah guru
				</UButton>
			</UCard>

			<UCard>
				<template #header>
					<p class="font-medium">
						Berita Acara
					</p>
				</template>
				<div class="grid sm:grid-cols-2 gap-4">
					<UFormField v-for="f in baFields" :key="f.key" :label="f.label">
						<UInput v-model="form.ba[f.key]" :placeholder="f.hint" />
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
						<UInput v-model="form.cetak.kota" placeholder="Contoh: Majalengka" />
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
	const guru = ref<Guru[]>([]);

	const geserGuru = (i: number, arah: -1 | 1) => {
		const [g] = guru.value.splice(i, 1);
		if (g) guru.value.splice(i + arah, 0, g);
	};
	/** Tambahkan kepala sekolah dan bendahara ke daftar guru bila belum ada. */
	const guruDariPejabat = () => {
		const p = form.value?.pejabat;
		if (!p) return;
		const ada = new Set(guru.value.map((g) => g.nama.trim().toLowerCase()));
		const calon = [
			{ nama: p.kepalaSekolah, nip: p.nipKepalaSekolah, tugas: "Kepala Sekolah" },
			{ nama: p.bendahara, nip: p.nipBendahara, tugas: "Bendahara BOSP" }
		].filter((c) => c.nama.trim() && !ada.has(c.nama.trim().toLowerCase()));
		// Kepala sekolah dan bendahara ditaruh di urutan teratas.
		guru.value.unshift(...calon.map((c) => ({ ...guruKosong(), ...c })));
	};

	const pejabatFields: { key: PejabatKey, label: string, hint: string }[] = [
		{ key: "kepalaSekolah", label: "Kepala Sekolah", hint: "Nama kepala sekolah" },
		{ key: "nipKepalaSekolah", label: "NIP Kepala Sekolah", hint: "Hanya angka" },
		{ key: "bendahara", label: "Bendahara", hint: "Nama bendahara" },
		{ key: "nipBendahara", label: "NIP Bendahara", hint: "Hanya angka" },
		{ key: "komite", label: "Ketua Komite", hint: "Nama ketua komite sekolah" },
		{ key: "nipKomite", label: "NIP Ketua Komite", hint: "Hanya angka, boleh kosong" },
		{ key: "pemegangBarang", label: "Pemegang Barang", hint: "Nama pemegang barang" },
		{ key: "nipPemegangBarang", label: "NIP Pemegang Barang", hint: "Hanya angka" },
		{ key: "petugasRekon", label: "Petugas Rekonsiliasi", hint: "Nama petugas" },
		{ key: "nipPetugasRekon", label: "NIP Petugas Rekonsiliasi", hint: "Hanya angka" },
		{ key: "skKepalaSekolah", label: "Nomor SK Kepala Sekolah", hint: "Contoh: 100.3.3.2/KEP.148-DISDIK/2026" },
		{ key: "skBendahara", label: "Nomor SK Bendahara", hint: "Contoh: 400.3.3.5/084-SD/2026" },
		{ key: "tanggalSkBendahara", label: "Tanggal SK Bendahara", hint: "Contoh: 2 Januari 2026" }
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

	const uploadLogo = async (file: File | null | undefined, key: "logoKiri" | "logoKanan") => {
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
	const kertasItems = [{ label: "A4 (210 × 297 mm)", value: "A4" }, { label: "F4 (210 × 330 mm)", value: "F4" }];

	const load = async () => {
		error.value = "";
		try {
			view.value = await api.pengaturanGet(year.value);
			form.value = structuredClone(toRaw(view.value.efektif));
			if (!form.value.cetak.kertas) form.value.cetak.kertas = "A4";
			guru.value = (await api.settingsGet<Guru[]>(SETTING_GURU)) ?? [];
		} catch (err) {
			error.value = errorMessage(err);
		}
	};

	type Bagian = "pejabat" | "kop" | "ba" | "cetak";
	type Isian = Record<string, string>;

	/** "arkas" bila isian sama dengan data ARKAS, "manual" bila diubah, null bila ARKAS tidak punya datanya. */
	const sumber = (bagian: Bagian, key: string): "arkas" | "manual" | null => {
		const asal = (view.value?.bawaan[bagian] as unknown as Isian | undefined)?.[key]?.trim() ?? "";
		if (!asal || !form.value) return null;
		return ((form.value[bagian] as unknown as Isian)[key] ?? "").trim() === asal ? "arkas" : "manual";
	};
	const resetKe = (bagian: Bagian, key: string) => {
		if (!form.value || !view.value) return;
		(form.value[bagian] as unknown as Isian)[key] = (view.value.bawaan[bagian] as unknown as Isian)[key] ?? "";
	};

	/** Yang sama dengan data ARKAS tidak disimpan, supaya tetap mengikuti ARKAS bila datanya berubah. */
	const tanpaBawaan = (p: Pengaturan): Pengaturan => {
		const out = structuredClone(toRaw(p));
		const bawaan = view.value?.bawaan;
		if (!bawaan) return out;
		for (const bagian of ["pejabat", "kop", "ba", "cetak"] as const) {
			const isi = out[bagian] as unknown as Isian;
			const asal = bawaan[bagian] as unknown as Isian;
			for (const key of Object.keys(isi)) {
				if (asal[key] && isi[key]?.trim() === asal[key].trim()) isi[key] = "";
			}
		}
		return out;
	};

	const save = async () => {
		if (!form.value) return;
		saving.value = true;
		try {
			await api.pengaturanSet(tanpaBawaan(form.value));
			await api.settingsSet(SETTING_GURU, toRaw(guru.value).filter((g) => g.nama.trim()).map((g) => ({ ...g, nama: g.nama.trim() })));
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
