<template>
	<LayoutPageShell title="Data Penyedia" :picker="false">
		<template #actions>
			<UButton icon="i-lucide-save" :loading="saving" :disabled="!form" @click="save">
				Simpan
			</UButton>
		</template>
		<div class="grid lg:grid-cols-[18rem_1fr] gap-4">
			<div class="space-y-2">
				<UInput v-model="search" icon="i-lucide-search" placeholder="Cari toko..." />
				<p class="text-xs text-muted">
					Daftar toko dari nota ARKAS tahun {{ year ?? "-" }}.
				</p>
				<div class="max-h-[calc(100vh-14rem)] overflow-auto space-y-1">
					<UButton
						v-for="t in filtered"
						:key="t.nama"
						block
						:color="t.nama === selected ? 'primary' : 'neutral'"
						:variant="t.nama === selected ? 'soft' : 'outline'"
						class="justify-start text-left"
						@click="pick(t.nama)"
					>
						<div class="min-w-0">
							<p class="text-sm font-medium truncate">
								{{ t.nama }}
							</p>
							<p class="text-xs text-muted">
								{{ t.jumlah }} nota{{ t.siplah ? " · SIPLah" : "" }}{{ t.tersimpan ? " · profil tersimpan" : "" }}
							</p>
						</div>
					</UButton>
				</div>
			</div>

			<div v-if="form && selected" class="space-y-4 max-w-3xl">
				<UAlert
					icon="i-lucide-info"
					color="neutral"
					variant="subtle"
					title="Data yang tidak ada di ARKAS untuk dokumen (SP, Nota, Kwitansi, BA)."
					description="Kolom kosong memakai data ARKAS (alamat, telepon, NPWP). Data ini tersimpan di SIBOS."
				/>
				<UCard>
					<template #header>
						<p class="font-medium">
							{{ selected }}
						</p>
					</template>
					<div class="grid sm:grid-cols-2 gap-3">
						<UFormField label="Penanggung jawab / penanda tangan">
							<UInput v-model="form.penanggungJawab" placeholder="mis. Drs. H. Ahmad Kholid, M.M" />
						</UFormField>
						<UFormField label="Jabatan">
							<UInput v-model="form.jabatan" placeholder="mis. Pemilik / Bendahara" />
						</UFormField>
						<UFormField label="Alamat" class="sm:col-span-2">
							<UInput v-model="form.alamat" :placeholder="arkasInfo?.alamatToko ?? ''" />
						</UFormField>
						<UFormField label="Kota (untuk tanggal di nota/kwitansi)">
							<UInput v-model="form.kota" placeholder="mis. Rajagaluh" />
						</UFormField>
						<UFormField label="Telepon">
							<UInput v-model="form.telp" :placeholder="arkasInfo?.noTelp ?? ''" />
						</UFormField>
						<UFormField label="NPWP">
							<UInput v-model="form.npwp" :placeholder="arkasInfo?.npwp ?? ''" />
						</UFormField>
						<UFormField label="Kop toko (satu baris per baris)" class="sm:col-span-2" hint="Kosong = nama, alamat, telepon">
							<UTextarea v-model="form.kop" :rows="5" class="w-full" />
						</UFormField>
						<UFormField label="Keterangan layanan toko (isian {layanan_toko})" class="sm:col-span-2">
							<UTextarea v-model="form.layanan" :rows="3" class="w-full" />
						</UFormField>
						<UFormField label="Logo toko" class="sm:col-span-2">
							<div class="flex items-center gap-3">
								<img v-if="form.logo" :src="form.logo" alt="Logo" class="h-14 border border-default rounded">
								<UFileUpload v-slot="{ open }" accept="image/*" reset :preview="false" @update:model-value="uploadLogo">
									<UButton size="sm" color="neutral" variant="outline" icon="i-lucide-image-up" @click="open()">
										Pilih logo
									</UButton>
								</UFileUpload>
								<UButton v-if="form.logo" size="xs" color="neutral" variant="ghost" icon="i-lucide-x" @click="form.logo = ''">
									Hapus logo
								</UButton>
							</div>
						</UFormField>
					</div>
				</UCard>
				<div class="flex gap-2">
					<UButton v-if="selected.toUpperCase().includes('KPR')" color="neutral" variant="outline" icon="i-lucide-wand-sparkles" @click="isiContoh">
						Isi dari contoh Excel (KPRI-KPR)
					</UButton>
					<UButton v-if="savedNames.has(selected)" color="error" variant="ghost" icon="i-lucide-trash-2" class="ml-auto" @click="remove">
						Hapus profil
					</UButton>
				</div>
			</div>
			<div v-else class="py-16 text-center text-muted">
				Pilih toko di sebelah kiri.
			</div>
		</div>
	</LayoutPageShell>
</template>

<script lang="ts" setup>
	const { connected, year } = useArkas();
	const toast = useToast();

	const saved = ref<Penyedia[]>([]);
	const notas = ref<NotaGroup[]>([]);
	const selected = ref<string | null>(null);
	const form = ref<PenyediaData | null>(null);
	const search = ref("");
	const saving = ref(false);

	const savedNames = computed(() => new Set(saved.value.map((p) => p.nama)));

	const tokoList = computed(() => {
		const map = new Map<string, { nama: string, jumlah: number, siplah: boolean, tersimpan: boolean }>();
		for (const g of notas.value) {
			const nama = g.nota?.namaToko?.trim();
			if (!nama) continue;
			const cur = map.get(nama) ?? { nama, jumlah: 0, siplah: false, tersimpan: savedNames.value.has(nama) };
			cur.jumlah += 1;
			cur.siplah ||= g.isSiplah;
			map.set(nama, cur);
		}
		for (const p of saved.value) {
			if (!map.has(p.nama)) map.set(p.nama, { nama: p.nama, jumlah: 0, siplah: false, tersimpan: true });
		}
		return [...map.values()].sort((a, b) => b.jumlah - a.jumlah || a.nama.localeCompare(b.nama));
	});

	const filtered = computed(() => {
		const q = search.value.trim().toLowerCase();
		return q ? tokoList.value.filter((t) => t.nama.toLowerCase().includes(q)) : tokoList.value;
	});

	const arkasInfo = computed(() => notas.value.find((g) => g.nota?.namaToko?.trim() === selected.value)?.nota ?? null);

	const load = async () => {
		saved.value = await api.penyediaList().catch(() => []);
		if (connected.value && year.value) notas.value = await api.notaList(year.value, null, null).catch(() => []);
	};
	watch([year, connected], load, { immediate: true });

	const pick = (nama: string) => {
		selected.value = nama;
		form.value = { ...penyediaKosong(), ...(saved.value.find((p) => p.nama === nama)?.data ?? {}) };
	};

	const isiContoh = () => {
		if (form.value) Object.assign(form.value, { ...PENYEDIA_CONTOH.data });
	};

	const uploadLogo = async (file: File | null | undefined) => {
		if (!file || !form.value) return;
		try {
			form.value.logo = await fileToDataUrl(file);
		} catch (err) {
			toast.add({ title: "Gagal memuat logo", description: errorMessage(err), color: "error" });
		}
	};

	const save = async () => {
		if (!form.value || !selected.value) return;
		saving.value = true;
		try {
			await api.penyediaSave({ nama: selected.value, data: toRaw(form.value), updatedAt: "" });
			toast.add({ title: "Data penyedia tersimpan", color: "success" });
			await load();
		} catch (err) {
			toast.add({ title: "Gagal menyimpan", description: errorMessage(err), color: "error" });
		} finally {
			saving.value = false;
		}
	};

	const remove = async () => {
		if (!selected.value) return;
		await api.penyediaDelete(selected.value);
		await load();
		pick(selected.value);
	};
</script>
