<template>
	<!-- Tabel agar semua kolom sejajar per baris: judul (rata bawah), ruang ttd, nama, NIP -->
	<table class="ttd mt-8 text-[10pt] break-inside-avoid">
		<colgroup>
			<col v-for="(_, i) in kolom" :key="i" :style="{ width: `${100 / kolom.length}%` }">
		</colgroup>
		<tbody>
			<tr v-for="r in jumlahBarisJudul" :key="`j${r}`">
				<td v-for="(s, i) in kolom" :key="i">
					{{ s ? judul(s)[r - 1] ?? "" : "" }}
				</td>
			</tr>
			<tr>
				<td v-for="(_, i) in kolom" :key="i" :style="{ height: space }" />
			</tr>
			<tr>
				<td v-for="(s, i) in kolom" :key="i">
					<template v-if="s">
						<span v-if="s.nama" class="font-bold underline">{{ s.nama }}</span>
						<!-- Nama kosong: cukup garis untuk ditulis tangan -->
						<span v-else class="garis-isi" />
					</template>
				</td>
			</tr>
			<tr v-if="adaNip">
				<td v-for="(s, i) in kolom" :key="i">
					<!-- NIP kosong: tidak dicetak, agar bisa diisi pulpen -->
					<template v-if="s?.nip">
						NIP. {{ s.nip }}
					</template>
				</td>
			</tr>
		</tbody>
	</table>
</template>

<script lang="ts" setup>
	const props = withDefaults(defineProps<{
		signers?: Penandatangan[]
		tanggal?: string
		/** Tinggi ruang tanda tangan; untuk nota/kwitansi dibuat cukup untuk meterai. */
		space?: string
	}>(), { space: "20mm" });
	const { pengaturan } = useDocContext();

	const list = computed<Penandatangan[]>(() => {
		if (props.signers?.length) return props.signers;
		const p = pengaturan.value?.pejabat;
		const kota = pengaturan.value?.cetak.kota;
		const tempatTanggal = [kota, props.tanggal].filter(Boolean).join(", ");
		return [
			{ atas: "Mengetahui,", jabatan: "Kepala Sekolah", nama: p?.kepalaSekolah, nip: p?.nipKepalaSekolah },
			{ atas: tempatTanggal, jabatan: "Bendahara", nama: p?.bendahara, nip: p?.nipBendahara }
		];
	});

	/** Satu penanda tangan ditaruh di kolom kanan (kolom kiri kosong). */
	const kolom = computed<(Penandatangan | null)[]>(() => (list.value.length === 1 ? [null, list.value[0]!] : list.value));

	/** Baris judul (atas + jabatan), tanpa baris kosong. */
	const judulMentah = (s: Penandatangan) => [...(s.atas ?? "").split("\n"), s.jabatan ?? ""].map((l) => l.trim()).filter(Boolean);
	const jumlahBarisJudul = computed(() => Math.max(1, ...kolom.value.map((s) => (s ? judulMentah(s).length : 0))));
	/** Judul dirata bawah: kolom dengan baris lebih sedikit diberi baris kosong di atas, sehingga jabatan sejajar. */
	const judul = (s: Penandatangan) => {
		const lines = judulMentah(s);
		return [...Array.from<string>({ length: jumlahBarisJudul.value - lines.length }).fill(""), ...lines];
	};

	const adaNip = computed(() => list.value.some((s) => s.nip !== undefined));
</script>

<style scoped>
	.ttd {
		width: 100%;
		border-collapse: collapse;
		table-layout: fixed;
	}
	.ttd td {
		text-align: center;
		vertical-align: bottom;
		padding: 0 4px;
		line-height: 1.35;
		height: 1.35em;
	}
</style>
