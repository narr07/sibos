<template>
	<PrintSheet>
		<div class="doc rapat">
			<PrintKopSurat ringkas />
			<PrintFields
				label-width="5.5rem"
				:rows="[
					['Nomor', r.nomorSurat || '....................................'],
					['Lampiran', r.lampiran || '-'],
					['Perihal', `Undangan ${r.namaKegiatan}`]
				]"
			>
				<template #value-2="{ value }">
					<b>{{ value }}</b>
				</template>
			</PrintFields>

			<div class="mt-5">
				<p>Kepada Yth.</p>
				<p>{{ r.kepada }}</p>
				<p>di</p>
				<p class="pl-8">
					Tempat
				</p>
			</div>

			<p class="mt-5">
				Dengan hormat,
			</p>
			<p class="mt-2 text-justify indent-8">
				Sehubungan dengan pelaksanaan program Bantuan Operasional Satuan Pendidikan (BOSP) Tahun Anggaran {{ tahun }},
				khususnya agenda {{ r.namaKegiatan }}, kami mengharapkan kehadiran Bapak/Ibu pada rapat yang akan dilaksanakan pada:
			</p>
			<PrintFields
				class="my-3 ml-8"
				label-width="8rem"
				:rows="[
					['Hari / Tanggal', `${hariDari(r.tanggal)}, ${tanggalPanjang(r.tanggal)}`],
					['Waktu', `${jam(r.waktuMulai)} s.d. ${jam(r.waktuSelesai)} ${r.zona}`],
					['Tempat', r.tempat],
					['Agenda Rapat', r.agenda]
				]"
			/>
			<p class="text-justify indent-8">
				Mengingat pentingnya agenda tersebut, kami mohon kehadiran Bapak/Ibu tepat pada waktunya.
			</p>
			<p class="mt-2 text-justify indent-8">
				Demikian undangan ini kami sampaikan. Atas perhatian dan kerja sama yang baik, kami ucapkan terima kasih.
			</p>

			<PrintTandaTangan :signers="signers" />
		</div>
	</PrintSheet>
</template>

<script lang="ts" setup>
	const props = defineProps<{ r: RapatData }>();
	const { pengaturan, sekolah } = useDocContext();

	const tahun = computed(() => props.r.tanggal.slice(0, 4));

	const signers = computed<Penandatangan[]>(() => {
		const p = pengaturan.value?.pejabat;
		return [{
			atas: [pengaturan.value?.cetak.kota, tanggalPanjang(props.r.tanggalSurat)].filter(Boolean).join(", "),
			jabatan: jabatanKepala(sekolah.value?.nama),
			nama: p?.kepalaSekolah,
			nip: p?.nipKepalaSekolah
		}];
	});
</script>

<style scoped>
	.rapat {
		font-size: 12pt;
	}
	.rapat :deep(.ttd) {
		font-size: 12pt;
	}
</style>
