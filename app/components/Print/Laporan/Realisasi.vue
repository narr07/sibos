<template>
	<PrintSheet landscape :margin="12">
		<div class="doc realisasi">
			<p class="text-center text-[13pt] font-bold">
				LAPORAN REALISASI BELANJA
			</p>
			<p class="text-center font-bold mb-3">
				ACUAN S.D. {{ BULAN[upto - 1]!.toUpperCase() }} TAHUN {{ year }}
			</p>
			<PrintFields
				class="mb-2"
				label-width="9rem"
				:rows="[
					['Nama Sekolah', sekolah?.nama],
					['NPSN', sekolah?.npsn],
					['Sumber Dana', fundLabel],
					['Pagu', rupiah(totalPagu)],
					['Realisasi', `${rupiah(totalRealisasi)} (${persen}%)`],
					['Sisa anggaran', rupiah(totalPagu - totalRealisasi)]
				]"
			/>

			<table class="tabel">
				<thead>
					<tr>
						<th class="w-8">
							No
						</th>
						<th class="w-36">
							Kegiatan
						</th>
						<th>Uraian</th>
						<th class="w-28">
							Kode Rekening
						</th>
						<th class="w-22">
							Pagu
						</th>
						<th class="w-22">
							Rencana s.d.
						</th>
						<th class="w-22">
							Realisasi
						</th>
						<th class="w-22">
							Sisa
						</th>
						<th class="w-24">
							Status
						</th>
					</tr>
				</thead>
				<tbody>
					<tr v-for="(r, i) in items" :key="r.idRapbs">
						<td class="text-center">
							{{ i + 1 }}
						</td>
						<td>
							<span class="font-mono">{{ r.kodeKegiatan }}</span><br>{{ r.namaKegiatan }}
						</td>
						<td>{{ r.uraian }}</td>
						<td class="text-center">
							{{ r.kodeRekening }}
						</td>
						<td class="num">
							{{ angka(r.pagu) }}
						</td>
						<td class="num">
							{{ angka(r.rencanaSd) }}
						</td>
						<td class="num">
							{{ angka(r.totalRealisasi) }}
						</td>
						<td class="num">
							{{ angka(r.sisa) }}
						</td>
						<td class="text-center">
							{{ statusLabel[r.status] }}
						</td>
					</tr>
					<tr v-for="o in luar" :key="o.id">
						<td />
						<td>Di luar RKAS aktif</td>
						<td>{{ o.uraian }}</td>
						<td class="text-center">
							{{ o.kodeRekening }}
						</td>
						<td />
						<td />
						<td class="num">
							{{ angka(o.nominal) }}
						</td>
						<td />
						<td class="text-center">
							{{ tanggalId(o.tanggal) }}
						</td>
					</tr>
					<tr class="font-bold">
						<td colspan="4" class="text-center">
							Jumlah
						</td>
						<td class="num">
							{{ angka(items.reduce((s, r) => s + r.pagu, 0)) }}
						</td>
						<td class="num">
							{{ angka(items.reduce((s, r) => s + r.rencanaSd, 0)) }}
						</td>
						<td class="num">
							{{ angka(items.reduce((s, r) => s + r.totalRealisasi, 0) + luar.reduce((s, o) => s + o.nominal, 0)) }}
						</td>
						<td class="num">
							{{ angka(items.reduce((s, r) => s + r.sisa, 0)) }}
						</td>
						<td />
					</tr>
				</tbody>
			</table>

			<PrintTandaTangan :tanggal="tanggalPanjang(akhirBulan(year, upto))" />
		</div>
	</PrintSheet>
</template>

<script lang="ts" setup>
	defineProps<{
		items: RealisasiItem[]
		luar: OutsideItem[]
		year: number
		upto: number
		fundLabel: string
		totalPagu: number
		totalRealisasi: number
		persen: number
		statusLabel: Record<RealisasiStatus, string>
	}>();
	const { sekolah } = useDocContext();
</script>

<style scoped>
	.realisasi :deep(table.tabel) {
		font-size: 9pt;
	}
	.realisasi :deep(table.tabel td),
	.realisasi :deep(table.tabel th) {
		padding: 2px 4px;
	}
	.realisasi :deep(table.tabel tr) {
		break-inside: avoid;
	}
</style>
