<template>
	<PrintSheet>
		<div class="doc">
			<PrintKopSurat />
			<p class="text-center text-[13pt] font-bold">
				BERITA ACARA REKONSILIASI DANA BOSP
			</p>
			<p class="text-center mb-4">
				Nomor: {{ pengaturan?.ba.nomor || "...................................." }}
			</p>
			<p class="text-justify">
				Pada hari ini, {{ pengaturan?.ba.tanggalSurat || "...................................." }}, bertempat di
				{{ pengaturan?.ba.tempat || sekolah?.nama }}, telah dilaksanakan rekonsiliasi penggunaan dana
				<b>{{ fundLabel }}</b> periode <b>{{ periodLabel }}</b> pada {{ sekolah?.nama }}
				<template v-if="pengaturan?.ba.nomorSk">
					berdasarkan SK Nomor {{ pengaturan.ba.nomorSk }}{{ pengaturan.ba.tanggalSk ? ` tanggal ${pengaturan.ba.tanggalSk}` : "" }}
				</template>, dengan hasil sebagai berikut:
			</p>

			<table class="tabel mt-3">
				<thead>
					<tr>
						<th class="w-8">
							No
						</th>
						<th>Uraian</th>
						<th class="w-40">
							Jumlah (Rp)
						</th>
					</tr>
				</thead>
				<tbody>
					<tr>
						<td class="text-center">
							1
						</td>
						<td class="font-bold">
							Saldo awal periode
						</td>
						<td class="num font-bold">
							{{ angka(saldoAwal) }}
						</td>
					</tr>
					<tr>
						<td class="text-center">
							2
						</td>
						<td class="font-bold" colspan="2">
							Penerimaan
						</td>
					</tr>
					<tr v-for="(p, i) in s.penerimaan" :key="`p${i}`">
						<td />
						<td class="pl-5">
							{{ p.uraian }} ({{ tanggalId(p.tanggal) }})
						</td>
						<td class="num">
							{{ angka(p.nominal) }}
						</td>
					</tr>
					<tr v-if="s.bunga">
						<td />
						<td class="pl-5">
							Bunga / jasa giro bank
						</td>
						<td class="num">
							{{ angka(s.bunga) }}
						</td>
					</tr>
					<tr>
						<td />
						<td class="pl-5 font-semibold">
							Jumlah penerimaan
						</td>
						<td class="num font-semibold">
							{{ angka(s.totalPenerimaan) }}
						</td>
					</tr>
					<tr>
						<td class="text-center">
							3
						</td>
						<td class="font-bold" colspan="2">
							Pengeluaran
						</td>
					</tr>
					<tr v-for="k in s.belanjaKelompok" :key="k.kode">
						<td />
						<td class="pl-5">
							{{ k.nama }}
						</td>
						<td class="num">
							{{ angka(k.total) }}
						</td>
					</tr>
					<tr v-if="s.pajakBunga">
						<td />
						<td class="pl-5">
							Pajak bunga / biaya administrasi bank
						</td>
						<td class="num">
							{{ angka(s.pajakBunga) }}
						</td>
					</tr>
					<tr v-if="s.pengembalian">
						<td />
						<td class="pl-5">
							Pengembalian dana
						</td>
						<td class="num">
							{{ angka(s.pengembalian) }}
						</td>
					</tr>
					<tr>
						<td />
						<td class="pl-5 font-semibold">
							Jumlah pengeluaran
						</td>
						<td class="num font-semibold">
							{{ angka(pengeluaran) }}
						</td>
					</tr>
					<tr>
						<td class="text-center">
							4
						</td>
						<td class="font-bold">
							Saldo akhir periode (1 + 2 - 3)
						</td>
						<td class="num font-bold">
							{{ angka(saldoAkhir) }}
						</td>
					</tr>
					<tr>
						<td />
						<td class="pl-5">
							Saldo bank
						</td>
						<td class="num">
							{{ angka(s.saldoAkhirBank) }}
						</td>
					</tr>
					<tr>
						<td />
						<td class="pl-5">
							Saldo tunai
						</td>
						<td class="num">
							{{ angka(s.saldoAkhirTunai) }}
						</td>
					</tr>
					<tr>
						<td class="text-center">
							5
						</td>
						<td>Pajak dipungut / disetor dalam periode</td>
						<td class="num">
							{{ angka(s.pajakDipungut) }} / {{ angka(s.pajakDisetor) }}
						</td>
					</tr>
				</tbody>
			</table>

			<p class="mt-3">
				Demikian berita acara ini dibuat untuk dipergunakan sebagaimana mestinya.
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
	const pengeluaran = computed(() => props.s.totalBelanja + props.s.pajakBunga + props.s.pengembalian);

	const signers = computed<Penandatangan[]>(() => {
		const p = pengaturan.value?.pejabat;
		return [
			{ atas: "Bendahara BOSP,", jabatan: "", nama: p?.bendahara, nip: p?.nipBendahara },
			{ atas: "Mengetahui,", jabatan: "Kepala Sekolah", nama: p?.kepalaSekolah, nip: p?.nipKepalaSekolah },
			{ atas: "Petugas Rekonsiliasi,", jabatan: "", nama: p?.petugasRekon, nip: p?.nipPetugasRekon }
		];
	});
</script>
