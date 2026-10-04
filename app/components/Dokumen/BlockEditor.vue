<template>
	<div class="space-y-2">
		<template v-if="b.type === 'kop'">
			<div class="grid grid-cols-2 gap-2">
				<UFormField label="Sumber kop">
					<USelect v-model="b.sumber" :items="[{ label: 'Sekolah (Pengaturan)', value: 'sekolah' }, { label: 'Toko (Data Penyedia)', value: 'toko' }, { label: 'Teks sendiri', value: 'teks' }]" size="sm" />
				</UFormField>
				<UFormField label="Logo">
					<USelect v-model="b.logo" :items="[{ label: 'Tanpa logo', value: 'none' }, { label: 'Logo toko', value: 'toko' }, { label: 'Unggah logo', value: 'custom' }]" size="sm" />
				</UFormField>
			</div>
			<UFormField v-if="b.sumber === 'teks'" label="Baris kop (satu baris per baris)">
				<UTextarea v-model="b.baris" :rows="4" class="w-full" size="sm" />
			</UFormField>
			<div v-if="b.logo === 'custom'" class="flex items-center gap-2">
				<img v-if="b.logoData" :src="b.logoData" alt="" class="h-10">
				<UFileUpload v-slot="{ open }" accept="image/*" reset :preview="false" @update:model-value="uploadLogo">
					<UButton size="xs" color="neutral" variant="outline" icon="i-lucide-image-up" @click="open()">
						Pilih logo
					</UButton>
				</UFileUpload>
			</div>
			<UCheckbox v-model="b.garis" label="Garis ganda di bawah kop" />
		</template>

		<template v-else-if="b.type === 'judul'">
			<UFormField label="Teks judul">
				<UInput v-model="b.teks" size="sm" />
			</UFormField>
			<div class="grid grid-cols-3 gap-2">
				<UFormField label="Ukuran (pt)">
					<UInputNumber v-model="b.ukuran" :min="8" :max="28" size="sm" />
				</UFormField>
				<UFormField label="Rata">
					<USelect v-model="b.rata" :items="rataItems" size="sm" />
				</UFormField>
				<UFormField label=" ">
					<UCheckbox v-model="b.garisBawah" label="Garis bawah" />
				</UFormField>
			</div>
		</template>

		<template v-else-if="b.type === 'teks'">
			<UFormField label="Isi (boleh pakai isian {nama_isian})">
				<UTextarea v-model="b.isi" :rows="4" class="w-full" size="sm" autoresize />
			</UFormField>
			<div class="grid grid-cols-3 gap-2">
				<UFormField label="Rata">
					<USelect v-model="b.rata" :items="rataItems" size="sm" />
				</UFormField>
				<UFormField label="Ukuran (0 = bawaan)">
					<UInputNumber v-model="b.ukuran" :min="0" :max="20" size="sm" />
				</UFormField>
				<UFormField label=" ">
					<UCheckbox v-model="b.tebal" label="Tebal" />
				</UFormField>
			</div>
		</template>

		<template v-else-if="b.type === 'isian'">
			<div v-for="(row, i) in b.baris" :key="i" class="grid grid-cols-[10rem_1fr_auto] gap-2">
				<UInput v-model="row.label" size="sm" placeholder="Label" />
				<UInput v-model="row.nilai" size="sm" placeholder="Nilai, mis. {nama_toko}" />
				<UButton icon="i-lucide-x" size="xs" color="neutral" variant="ghost" aria-label="Hapus baris" @click="b.baris.splice(i, 1)" />
			</div>
			<div class="flex items-center gap-2">
				<UButton size="xs" color="neutral" variant="outline" icon="i-lucide-plus" @click="b.baris.push({ label: '', nilai: '' })">
					Baris
				</UButton>
				<span class="text-xs text-muted ml-auto">Lebar label (mm)</span>
				<UInputNumber v-model="b.lebarLabel" :min="10" :max="80" size="xs" class="w-24" />
				<span class="text-xs text-muted">Masuk (mm)</span>
				<UInputNumber v-model="b.indent" :min="0" :max="40" size="xs" class="w-24" />
			</div>
		</template>

		<template v-else-if="b.type === 'tabel'">
			<div v-for="(k, i) in b.kolom" :key="i" class="grid grid-cols-[12rem_1fr_6rem_auto] gap-2 items-center">
				<USelect v-model="k.kunci" :items="kolomItems" size="sm" />
				<UInput v-model="k.judul" size="sm" placeholder="Judul kolom" />
				<UInputNumber v-model="k.lebar" :min="0" :max="80" size="sm" title="Lebar % (0 = sisa)" />
				<div class="flex">
					<UButton icon="i-lucide-arrow-up" size="xs" color="neutral" variant="ghost" :disabled="i === 0" aria-label="Naik" @click="move(b.kolom, i, -1)" />
					<UButton icon="i-lucide-x" size="xs" color="neutral" variant="ghost" aria-label="Hapus kolom" @click="b.kolom.splice(i, 1)" />
				</div>
			</div>
			<p class="text-xs text-muted">
				Lebar dalam persen; 0 = memakai sisa lebar (biasanya kolom nama barang).
			</p>
			<div class="flex flex-wrap items-center gap-3">
				<UButton size="xs" color="neutral" variant="outline" icon="i-lucide-plus" @click="b.kolom.push({ kunci: 'jumlah', judul: 'Jumlah', lebar: 15 })">
					Kolom
				</UButton>
				<UCheckbox v-model="b.total" label="Baris jumlah" />
				<UInput v-if="b.total" v-model="b.labelTotal" size="xs" class="w-32" />
				<UCheckbox v-model="b.bersihkanNama" label="Buang awalan kategori nama barang" />
				<span class="text-xs text-muted">Min. baris</span>
				<UInputNumber v-model="b.minBaris" :min="0" :max="30" size="xs" class="w-20" />
				<span class="text-xs text-muted">Huruf (pt)</span>
				<UInputNumber v-model="b.ukuran" :min="7" :max="14" size="xs" class="w-20" />
			</div>
		</template>

		<template v-else-if="b.type === 'terbilang' || b.type === 'kotakTotal'">
			<UFormField label="Label">
				<UInput v-model="b.label" size="sm" />
			</UFormField>
		</template>

		<template v-else-if="b.type === 'ttd'">
			<div v-for="(s, i) in b.kolom" :key="i" class="rounded-md border border-default p-2 space-y-1.5">
				<div class="flex items-center">
					<span class="text-xs font-medium">Penanda tangan {{ i + 1 }}</span>
					<UButton icon="i-lucide-x" size="xs" color="neutral" variant="ghost" class="ml-auto" aria-label="Hapus" @click="b.kolom.splice(i, 1)" />
				</div>
				<UTextarea v-model="s.atas" :rows="2" size="sm" class="w-full" placeholder="Baris atas, mis. {kota}, {tanggal_dokumen}" />
				<UTextarea v-model="s.jabatan" :rows="1" size="sm" class="w-full" placeholder="Jabatan" autoresize />
				<div class="grid grid-cols-2 gap-2">
					<UInput v-model="s.nama" size="sm" placeholder="Nama, mis. {bendahara}" />
					<UInput v-model="s.nip" size="sm" placeholder="NIP (kosongkan bila tidak ada)" />
				</div>
			</div>
			<div class="flex flex-wrap items-center gap-3">
				<UButton size="xs" color="neutral" variant="outline" icon="i-lucide-plus" @click="b.kolom.push({ atas: '', jabatan: '', nama: '', nip: '' })">
					Penanda tangan
				</UButton>
				<span class="text-xs text-muted">Tinggi ruang ttd (mm)</span>
				<UInputNumber v-model="b.tinggi" :min="8" :max="40" size="xs" class="w-20" />
				<USelect v-model="b.rata" :items="[{ label: 'Dibagi rata', value: 'rata' }, { label: 'Satu, di kanan', value: 'kanan' }, { label: 'Satu, di tengah', value: 'tengah' }]" size="xs" class="w-40" />
			</div>
		</template>

		<template v-else-if="b.type === 'spasi'">
			<UFormField label="Tinggi (mm)">
				<UInputNumber v-model="b.tinggi" :min="1" :max="60" size="sm" class="w-32" />
			</UFormField>
		</template>

		<template v-else-if="b.type === 'kwitansi'">
			<p class="text-xs text-muted">
				Kotak kwitansi lengkap: panel kop toko (dari Data Penyedia), terbilang & jumlah dalam kotak berwarna, "Yang menerima", dan visum Kepala Sekolah & Bendahara.
			</p>
			<div class="grid grid-cols-2 gap-2">
				<UCheckbox v-model="b.panelToko" label="Panel kop toko di kiri" />
				<UFormField label="Visum kepala sekolah & bendahara">
					<USelect v-model="b.visum" :items="[{ label: 'Di bawah kotak', value: 'bawah' }, { label: 'Di dalam kotak', value: 'dalam' }, { label: 'Tidak ada', value: 'tidak' }]" size="sm" />
				</UFormField>
				<UFormField label="Motto panel">
					<UInput v-model="b.motto" size="sm" />
				</UFormField>
				<div class="grid grid-cols-2 gap-2">
					<UFormField label="Warna sorotan">
						<UPopover>
							<UButton color="neutral" variant="outline" size="sm" block class="justify-start">
								<span class="size-4 rounded-sm ring ring-default" :style="{ backgroundColor: b.warna }" />
								<span class="font-mono text-xs">{{ b.warna }}</span>
							</UButton>
							<template #content>
								<UColorPicker v-model="b.warna" class="p-2" />
							</template>
						</UPopover>
					</UFormField>
					<UFormField label="Ruang ttd (mm)">
						<UInputNumber v-model="b.tinggiTtd" :min="8" :max="40" size="sm" />
					</UFormField>
				</div>
				<UFormField label="Lebar kwitansi (mm, 0 = selebar halaman)">
					<UInputNumber :model-value="b.lebarKotak ?? 0" :min="0" :max="290" size="sm" @update:model-value="(v) => b.type === 'kwitansi' && (b.lebarKotak = Number(v) || 0)" />
				</UFormField>
				<UFormField label="Jarak kotak ke visum (mm)">
					<UInputNumber :model-value="b.jarakVisum ?? 10" :min="0" :max="40" size="sm" @update:model-value="(v) => b.type === 'kwitansi' && (b.jarakVisum = Number(v) || 0)" />
				</UFormField>
				<UFormField label="Tinggi kotak (mm)">
					<UInputNumber :model-value="b.tinggiKotak ?? 75" :min="40" :max="150" size="sm" @update:model-value="(v) => b.type === 'kwitansi' && (b.tinggiKotak = Number(v) || 75)" />
				</UFormField>
				<UFormField label="Lebar panel kop toko (mm, 0 = otomatis)">
					<UInputNumber :model-value="b.lebarPanel ?? 0" :min="0" :max="110" size="sm" @update:model-value="(v) => b.type === 'kwitansi' && (b.lebarPanel = Number(v) || 0)" />
				</UFormField>
			</div>
			<p class="text-xs text-muted">
				Daftar layanan toko diambil dari Data Penyedia; pisahkan kelompok dengan satu baris kosong, baris pertama tiap kelompok menjadi judul.
			</p>
		</template>

		<p v-else-if="b.type === 'garis'" class="text-xs text-muted">
			Garis horizontal penuh.
		</p>
	</div>
</template>

<script lang="ts" setup>
	const b = defineModel<Blok>({ required: true });
	const toast = useToast();

	const rataItems = [{ label: "Kiri", value: "left" }, { label: "Tengah", value: "center" }, { label: "Kanan", value: "right" }, { label: "Rata kiri-kanan", value: "justify" }];
	const kolomItems = Object.entries(KOLOM_LABEL).map(([value, label]) => ({ label, value }));

	function move<T>(arr: T[], i: number, d: number) {
		const j = i + d;
		if (j < 0 || j >= arr.length) return;
		const [x] = arr.splice(i, 1);
		if (x !== undefined) arr.splice(j, 0, x);
	}

	const uploadLogo = async (file: File | null | undefined) => {
		if (!file || b.value.type !== "kop") return;
		try {
			b.value.logoData = await fileToDataUrl(file);
		} catch (err) {
			toast.add({ title: "Gagal memuat logo", description: errorMessage(err), color: "error" });
		}
	};
</script>
