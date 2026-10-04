<template>
	<PrintSheet>
		<div class="doc">
			<PrintKopSurat />
			<p class="text-center text-[13pt] font-bold">
				SURAT PERNYATAAN TANGGUNG JAWAB MUTLAK
			</p>
			<p class="text-center mb-4">
				Nomor: ....................................
			</p>
			<p>Yang bertanda tangan di bawah ini:</p>
			<PrintFields class="my-2 ml-4" label-width="14rem" :rows="[['Nama Sekolah', sekolah?.nama], ['Kode Sekolah (NPSN)', sekolah?.npsn], ['Nomor/Tanggal DPA SKPD', '....................................'], ['Kegiatan', fundLabel]]" />
			<p class="text-justify">
				Menyatakan bahwa saya bertanggung jawab penuh atas segala penerimaan dan pengeluaran yang telah dilaksanakan
				dan dimanfaatkan untuk membiayai kegiatan sesuai petunjuk teknis pada periode <b>{{ periodLabel }}</b>
				dengan rincian sebagai berikut:
			</p>
			<table class="my-3 ml-4 w-[90%]">
				<tbody>
					<tr>
						<td class="w-8">
							A.
						</td>
						<td>Saldo Awal</td>
						<td class="num w-40">
							Rp {{ angka(saldoAwal) }}
						</td>
					</tr>
					<tr>
						<td>B.</td>
						<td>Penerimaan</td>
						<td class="num">
							Rp {{ angka(s.totalPenerimaan) }}
						</td>
					</tr>
					<tr>
						<td>C.</td>
						<td>Pengeluaran, terdiri atas:</td>
						<td class="num">
							Rp {{ angka(pengeluaran) }}
						</td>
					</tr>
					<tr>
						<td />
						<td class="pl-6">
							1. Belanja Operasi
						</td>
						<td class="num">
							Rp {{ angka(s.belanjaOperasi) }}
						</td>
					</tr>
					<tr>
						<td />
						<td class="pl-6">
							2. Belanja Modal
						</td>
						<td class="num">
							Rp {{ angka(s.belanjaModal) }}
						</td>
					</tr>
					<tr v-if="lainnya">
						<td />
						<td class="pl-6">
							3. Pajak bunga, pengembalian dana, dan lainnya
						</td>
						<td class="num">
							Rp {{ angka(lainnya) }}
						</td>
					</tr>
					<tr>
						<td>D.</td>
						<td>Saldo Akhir, terdiri atas:</td>
						<td class="num">
							Rp {{ angka(saldoAkhir) }}
						</td>
					</tr>
					<tr>
						<td />
						<td class="pl-6">
							1. Saldo Bank
						</td>
						<td class="num">
							Rp {{ angka(s.saldoAkhirBank) }}
						</td>
					</tr>
					<tr>
						<td />
						<td class="pl-6">
							2. Saldo Tunai
						</td>
						<td class="num">
							Rp {{ angka(s.saldoAkhirTunai) }}
						</td>
					</tr>
				</tbody>
			</table>
			<p class="text-justify">
				Bukti-bukti atas belanja tersebut pada huruf C disimpan pada sekolah untuk kelengkapan administrasi dan
				keperluan pemeriksaan sesuai peraturan perundang-undangan. Apabila bukti-bukti tersebut tidak benar, saya
				bertanggung jawab secara penuh atas segala kerugian keuangan negara yang timbul sesuai kewenangan saya.
			</p>
			<p class="mt-2">
				Demikian surat pernyataan ini dibuat dengan sebenarnya.
			</p>
			<PrintTandaTangan :signers="signers" />
		</div>
	</PrintSheet>
</template>

<script lang="ts" setup>
	const props = defineProps<{ s: PeriodSummary, fundLabel: string, periodLabel: string }>();
	const { pengaturan, sekolah } = useDocContext();

	const saldoAwal = computed(() => props.s.saldoAwalBank + props.s.saldoAwalTunai);
	const saldoAkhir = computed(() => props.s.saldoAkhirBank + props.s.saldoAkhirTunai);
	const lainnya = computed(() => props.s.pajakBunga + props.s.pengembalian);
	const pengeluaran = computed(() => props.s.totalBelanja + lainnya.value);

	const signers = computed<Penandatangan[]>(() => {
		const p = pengaturan.value?.pejabat;
		const tanggal = tanggalPanjang(akhirBulan(props.s.year, props.s.end));
		return [
			{ atas: [pengaturan.value?.cetak.kota, tanggal].filter(Boolean).join(", "), jabatan: "Kepala Sekolah", nama: p?.kepalaSekolah, nip: p?.nipKepalaSekolah }
		];
	});
</script>
