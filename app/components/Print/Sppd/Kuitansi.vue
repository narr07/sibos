<template>
	<PrintSheet>
		<div class="doc sppd">
			<PrintKopSurat ringkas />
			<p class="mb-4 text-center text-[12pt] font-bold underline">
				KUITANSI
			</p>

			<PrintFields
				class="isi"
				label-width="10rem"
				:rows="[
					['Nomor', d.nomorKuitansi],
					['Sudah terima dari', `Bendahara Pengeluaran ${namaSekolah}`],
					['Banyaknya Uang', terbilangRupiah(total)],
					['Untuk Pembayaran', untuk],
					['Kode Rekening', d.kodeRekening],
					['Jumlah', `Rp ${angka(total)},-`]
				]"
			>
				<template #value-2="{ value }">
					<i>{{ value }}</i>
				</template>
				<template #value-5="{ value }">
					<b>{{ value }}</b>
				</template>
			</PrintFields>

			<PrintTandaTangan
				space="16mm"
				:signers="[{ atas: tempatTanggal, jabatan: 'Yang Menerima', nama: orang.nama, nip: orang.nip }]"
			/>
			<PrintTandaTangan
				space="16mm"
				:signers="[
					{ atas: 'Mengetahui / Menyetujui', jabatan: 'Kepala Sekolah', nama: p?.kepalaSekolah, nip: p?.nipKepalaSekolah },
					{ jabatan: 'Bendahara', nama: p?.bendahara, nip: p?.nipBendahara }
				]"
			/>
		</div>
	</PrintSheet>
</template>

<script lang="ts" setup>
	const props = defineProps<{ d: SppdData, orang: Guru }>();
	const { pengaturan, sekolah } = useDocContext();

	const p = computed(() => pengaturan.value?.pejabat);
	const namaSekolah = computed(() => namaSekolahRapi(sekolah.value?.nama));
	const total = computed(() => totalBiaya(props.d.biaya));
	const tempatTanggal = computed(() => [pengaturan.value?.cetak.kota, tanggalPanjang(props.d.tanggalBayar)].filter(Boolean).join(", "));

	const untuk = computed(() => {
		const d = props.d;
		return `Belanja Perjalanan Dinas Biasa (perjalanan dinas ${d.daerah}) ke ${d.tujuan || "...................."} tanggal ${rentangTanggal(d)} `
			+ `pada ${namaSekolah.value} sumber Dana ${d.sumberDana} Tahun Anggaran ${d.tanggalBayar.slice(0, 4)}.`;
	});
</script>

<style scoped>
	/* Format SPPD sekolah memakai Arial */
	.sppd {
		font-family: Arial, Helvetica, sans-serif;
		font-size: 11pt;
	}
	.sppd :deep(.ttd) {
		font-size: 11pt;
		margin-top: 1.25rem;
	}
	.isi :deep(td) {
		padding-top: 0.3rem;
		padding-bottom: 0.3rem;
	}
</style>
