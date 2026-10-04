<template>
	<PrintSheet>
		<div class="doc">
			<PrintKopSurat />
			<p class="text-center text-[13pt] font-bold">
				BERITA ACARA PEMERIKSAAN KAS
			</p>
			<p class="text-center mb-4">
				dan Register Penutupan Kas Bulan {{ BULAN[reg.month - 1] }} {{ reg.year }}
			</p>
			<p class="text-justify">
				Pada hari ini, tanggal {{ tanggalPanjang(akhirBulan(reg.year, reg.month)) }}, yang bertanda tangan di bawah ini,
				Kepala Sekolah {{ sekolah?.nama }}, telah melakukan pemeriksaan kas kepada:
			</p>
			<PrintFields class="my-2 ml-4" label-width="10rem" :rows="[['Nama', pengaturan?.pejabat.bendahara], ['Jabatan', 'Bendahara BOS / Pemegang Kas']]" />
			<p>Berdasarkan pemeriksaan kas serta bukti-bukti yang ada, hasilnya sebagai berikut:</p>

			<table class="w-full my-2">
				<tbody>
					<tr>
						<td class="w-8">
							A.
						</td>
						<td>Jumlah penerimaan s.d. akhir bulan (termasuk saldo awal)</td>
						<td class="num w-40">
							Rp {{ angka(reg.penerimaanSd) }}
						</td>
					</tr>
					<tr>
						<td>B.</td>
						<td>Jumlah pengeluaran s.d. akhir bulan</td>
						<td class="num">
							Rp {{ angka(reg.pengeluaranSd) }}
						</td>
					</tr>
					<tr class="font-bold">
						<td>C.</td>
						<td>Saldo buku kas umum (A - B)</td>
						<td class="num">
							Rp {{ angka(reg.saldoBuku) }}
						</td>
					</tr>
					<tr>
						<td>D.</td>
						<td>Saldo di bank</td>
						<td class="num">
							Rp {{ angka(reg.saldoBank) }}
						</td>
					</tr>
					<tr>
						<td>E.</td>
						<td>Saldo kas tunai menurut buku</td>
						<td class="num">
							Rp {{ angka(reg.saldoTunai) }}
						</td>
					</tr>
				</tbody>
			</table>

			<p class="font-semibold mt-3">
				Uang tunai yang ada di brankas (hasil hitung fisik):
			</p>
			<table class="tabel mt-1">
				<thead>
					<tr>
						<th>Pecahan</th>
						<th>Jenis</th>
						<th>Jumlah lembar/keping</th>
						<th>Nilai (Rp)</th>
					</tr>
				</thead>
				<tbody>
					<tr v-for="row in rows" :key="row.key">
						<td class="num">
							{{ angka(row.nilai) }}
						</td>
						<td>{{ row.jenis }}</td>
						<td class="num">
							{{ row.jumlah }}
						</td>
						<td class="num">
							{{ angka(row.nilai * row.jumlah) }}
						</td>
					</tr>
					<tr class="font-bold">
						<td colspan="3" class="text-right">
							Jumlah uang fisik
						</td>
						<td class="num">
							{{ angka(fisik) }}
						</td>
					</tr>
					<tr>
						<td colspan="3" class="text-right">
							Selisih (fisik - saldo tunai buku)
						</td>
						<td class="num">
							{{ angka(fisik - reg.saldoTunai) }}
						</td>
					</tr>
				</tbody>
			</table>
			<p v-if="catatan" class="mt-2">
				Catatan: {{ catatan }}
			</p>

			<PrintTandaTangan :signers="signers" />
		</div>
	</PrintSheet>
</template>

<script lang="ts" setup>
	const props = defineProps<{ reg: RegisterKas, pecahan: Pecahan, catatan?: string | null }>();
	const { pengaturan, sekolah } = useDocContext();

	const rows = computed(() => [
		...Object.entries(props.pecahan.kertas).map(([v, n]) => ({ key: `k${v}`, nilai: Number(v), jumlah: n, jenis: "Kertas" })),
		...Object.entries(props.pecahan.logam).map(([v, n]) => ({ key: `l${v}`, nilai: Number(v), jumlah: n, jenis: "Logam" }))
	].filter((r) => r.jumlah > 0).sort((a, b) => b.nilai - a.nilai));
	const fisik = computed(() => rows.value.reduce((s, r) => s + r.nilai * r.jumlah, 0));

	const signers = computed<Penandatangan[]>(() => {
		const p = pengaturan.value?.pejabat;
		return [
			{ atas: "Yang diperiksa,", jabatan: "Bendahara", nama: p?.bendahara, nip: p?.nipBendahara },
			{ atas: [pengaturan.value?.cetak.kota, tanggalPanjang(akhirBulan(props.reg.year, props.reg.month))].filter(Boolean).join(", "), jabatan: "Kepala Sekolah", nama: p?.kepalaSekolah, nip: p?.nipKepalaSekolah }
		];
	});
</script>
