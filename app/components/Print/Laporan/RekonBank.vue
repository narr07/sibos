<template>
	<PrintSheet>
		<div class="doc">
			<PrintKopSurat />
			<p class="text-center text-[13pt] font-bold">
				REKONSILIASI BANK
			</p>
			<p class="text-center mb-4">
				Tahun Anggaran {{ data.year }} · {{ fundLabel }}
			</p>
			<table class="tabel">
				<thead>
					<tr>
						<th>Bulan</th>
						<th>Saldo Bank menurut BKU</th>
						<th>Saldo Rekening Koran</th>
						<th>Selisih</th>
						<th>Keterangan</th>
					</tr>
				</thead>
				<tbody>
					<tr v-for="m in data.months" :key="m.month">
						<td>{{ BULAN[m.month - 1] }}</td>
						<td class="num">
							{{ angka(m.saldoBank) }}
						</td>
						<td class="num">
							{{ m.rekeningKoran === null ? "" : angka(m.rekeningKoran) }}
						</td>
						<td class="num">
							{{ m.selisih === null ? "" : angka(m.selisih) }}
						</td>
						<td>{{ m.keterangan ?? (m.selisih === 0 ? "Sesuai" : "") }}</td>
					</tr>
				</tbody>
			</table>
			<PrintTandaTangan :tanggal="tanggalPanjang(akhirBulan(data.year, lastMonth))" />
		</div>
	</PrintSheet>
</template>

<script lang="ts" setup>
	const props = defineProps<{ data: RekonBank, fundLabel: string }>();
	const lastMonth = computed(() => [...props.data.months].reverse().find((m) => m.rekeningKoran !== null)?.month ?? 12);
</script>
