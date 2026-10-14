<template>
	<PrintSheet>
		<div class="doc sppd">
			<p class="mb-3 text-center text-[12pt] font-bold">
				RINCIAN PERJALANAN DINAS
			</p>
			<PrintFields
				class="mb-3"
				label-width="11rem"
				:rows="[['Lampiran SPPD Nomor', d.nomorSpd], ['Tanggal', tanggalPanjang(d.tanggalSurat)]]"
			/>

			<table class="tabel">
				<thead>
					<tr>
						<th class="w-10">
							No
						</th>
						<th>Perincian Biaya</th>
						<th class="w-40">
							Jumlah
						</th>
						<th class="w-44">
							Keterangan
						</th>
					</tr>
				</thead>
				<tbody>
					<tr v-for="(b, i) in BIAYA_SPPD" :key="b.key">
						<td class="text-center">
							{{ i + 1 }}
						</td>
						<td>{{ b.label }}</td>
						<td class="num">
							Rp {{ angka(d.biaya[b.key] || 0) }}
						</td>
						<td />
					</tr>
					<tr class="font-bold">
						<td />
						<td class="text-center">
							JUMLAH
						</td>
						<td class="num">
							Rp {{ angka(total) }}
						</td>
						<td />
					</tr>
					<tr>
						<td colspan="4">
							Terbilang: <i>{{ terbilangRupiah(total) }}</i>
						</td>
					</tr>
				</tbody>
			</table>

			<PrintTandaTangan
				space="16mm"
				:signers="[
					{ atas: `Telah dibayar sejumlah\nRp ${angka(total)}`, jabatan: 'Bendahara', nama: p?.bendahara, nip: p?.nipBendahara },
					{ atas: `${tempatTanggal}\nTelah menerima sejumlah uang sebesar\nRp ${angka(total)}`, jabatan: 'Yang menerima', nama: orang.nama, nip: orang.nip }
				]"
			/>

			<p class="mt-6 mb-2 border-t border-black pt-3 text-center font-bold">
				PERHITUNGAN SPPD RAMPUNG
			</p>
			<PrintFields
				label-width="13rem"
				:rows="[
					['Ditetapkan sejumlah', `Rp ${angka(total)}`],
					['Yang telah dibayar semula', `Rp ${angka(total)}`],
					['Sisa kurang/lebih', 'Rp 0']
				]"
			/>

			<PrintTandaTangan
				space="16mm"
				:signers="[{ atas: 'Mengetahui / Menyetujui', jabatan: 'Kepala Sekolah', nama: p?.kepalaSekolah, nip: p?.nipKepalaSekolah }]"
			/>
		</div>
	</PrintSheet>
</template>

<script lang="ts" setup>
	const props = defineProps<{ d: SppdData, orang: Guru }>();
	const { pengaturan } = useDocContext();

	const p = computed(() => pengaturan.value?.pejabat);
	const total = computed(() => totalBiaya(props.d.biaya));
	const tempatTanggal = computed(() => [pengaturan.value?.cetak.kota, tanggalPanjang(props.d.tanggalBayar)].filter(Boolean).join(", "));
</script>

<style scoped>
	/* Format SPPD sekolah memakai Arial */
	.sppd {
		font-family: Arial, Helvetica, sans-serif;
		font-size: 11pt;
	}
	.sppd table.tabel {
		font-size: 10pt;
	}
	.sppd :deep(.ttd) {
		font-size: 11pt;
		margin-top: 1.25rem;
	}
</style>
