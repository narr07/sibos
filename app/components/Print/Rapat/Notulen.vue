<template>
	<PrintSheet>
		<div class="doc rapat">
			<div class="text-center font-bold uppercase mb-4 leading-snug">
				<p class="text-[12pt]">
					Notulen Rapat
				</p>
				<p>{{ r.namaKegiatan }}</p>
				<p>{{ sekolah?.nama }}</p>
			</div>

			<PrintFields
				label-width="9rem"
				:rows="[
					['Hari / Tanggal', `${hariDari(r.tanggal)}, ${tanggalPanjang(r.tanggal)}`],
					['Waktu', `${jam(r.waktuMulai)} s.d. ${jam(r.waktuSelesai)} ${r.zona}`],
					['Tempat', r.tempat],
					['Pimpinan Rapat', r.pimpinan],
					['Notulis', r.notulis],
					['Peserta', `${jumlahHadir} orang (daftar hadir terlampir)`],
					['Agenda', r.agenda]
				]"
			/>

			<p class="bagian">
				Ringkasan Pembahasan
			</p>
			<p v-if="r.arahan.trim()" class="text-justify whitespace-pre-line">
				{{ r.arahan }}
			</p>
			<p v-if="r.pembahasan.trim()" class="mt-1 text-justify whitespace-pre-line">
				{{ r.pembahasan }}
			</p>

			<p class="bagian">
				Hasil Rapat
			</p>
			<ol class="list-decimal pl-6 text-justify">
				<li v-for="(poin, i) in poinKeputusan" :key="i">
					{{ poin }}
				</li>
			</ol>

			<PrintTandaTangan :signers="signers" />
		</div>
	</PrintSheet>
</template>

<script lang="ts" setup>
	const props = defineProps<{ r: RapatData, jumlahHadir: number }>();
	const { pengaturan, sekolah } = useDocContext();

	const poinKeputusan = computed(() => barisTeks(props.r.keputusan));

	const signers = computed<Penandatangan[]>(() => {
		const p = pengaturan.value?.pejabat;
		return [
			{ atas: "Mengetahui,", jabatan: jabatanKepala(sekolah.value?.nama), nama: p?.kepalaSekolah, nip: p?.nipKepalaSekolah },
			{ jabatan: "Notulis Rapat,", nama: props.r.notulis, nip: props.r.nipNotulis }
		];
	});
</script>

<style scoped>
	.rapat {
		font-size: 12pt;
	}
	.rapat :deep(.ttd) {
		font-size: 12pt;
	}
	.bagian {
		margin-top: 0.9rem;
		margin-bottom: 0.15rem;
		font-weight: bold;
		break-after: avoid;
	}
</style>
