<template>
	<div class="space-y-4">
		<div class="grid grid-cols-2 lg:grid-cols-4 gap-3">
			<UCard :ui="{ body: 'p-3 sm:p-3' }">
				<p class="text-xs text-muted">
					Saldo awal
				</p>
				<p class="text-lg font-semibold tabular">
					{{ rupiah(s.saldoAwalBank + s.saldoAwalTunai) }}
				</p>
			</UCard>
			<UCard :ui="{ body: 'p-3 sm:p-3' }">
				<p class="text-xs text-muted">
					Penerimaan
				</p>
				<p class="text-lg font-semibold tabular text-success">
					{{ rupiah(s.totalPenerimaan) }}
				</p>
			</UCard>
			<UCard :ui="{ body: 'p-3 sm:p-3' }">
				<p class="text-xs text-muted">
					Pengeluaran
				</p>
				<p class="text-lg font-semibold tabular text-error">
					{{ rupiah(s.totalBelanja + s.pajakBunga + s.pengembalian) }}
				</p>
			</UCard>
			<UCard :ui="{ body: 'p-3 sm:p-3' }">
				<p class="text-xs text-muted">
					Saldo akhir
				</p>
				<p class="text-lg font-semibold tabular">
					{{ rupiah(s.saldoAkhirBank + s.saldoAkhirTunai) }}
				</p>
				<p class="text-xs text-muted tabular">
					Bank {{ rupiah(s.saldoAkhirBank) }} · Tunai {{ rupiah(s.saldoAkhirTunai) }}
				</p>
			</UCard>
		</div>

		<UAlert v-for="(w, i) in s.warnings" :key="i" color="warning" variant="subtle" icon="i-lucide-triangle-alert" :title="w" />

		<div class="grid lg:grid-cols-2 gap-4">
			<UCard>
				<template #header>
					<p class="font-medium">
						Penerimaan
					</p>
				</template>
				<table class="w-full text-sm">
					<tbody>
						<tr v-for="(p, i) in s.penerimaan" :key="i" class="border-b border-default last:border-0">
							<td class="py-1.5 tabular text-muted w-24">
								{{ tanggalId(p.tanggal) }}
							</td>
							<td class="py-1.5">
								{{ p.uraian }}
							</td>
							<td class="py-1.5 text-right tabular">
								{{ angka(p.nominal) }}
							</td>
						</tr>
						<tr v-if="s.bunga">
							<td />
							<td class="py-1.5">
								Bunga bank
							</td>
							<td class="py-1.5 text-right tabular">
								{{ angka(s.bunga) }}
							</td>
						</tr>
						<tr v-if="!s.penerimaan.length && !s.bunga">
							<td colspan="3" class="py-3 text-center text-muted">
								Tidak ada penerimaan pada periode ini.
							</td>
						</tr>
					</tbody>
				</table>
			</UCard>
			<UCard>
				<template #header>
					<p class="font-medium">
						Pengeluaran per kelompok
					</p>
				</template>
				<table class="w-full text-sm">
					<tbody>
						<tr v-for="k in s.belanjaKelompok" :key="k.kode" class="border-b border-default last:border-0">
							<td class="py-1.5">
								{{ k.nama }}
							</td>
							<td class="py-1.5 text-right tabular">
								{{ angka(k.total) }}
							</td>
						</tr>
						<tr v-if="s.pajakBunga">
							<td class="py-1.5">
								Pajak bunga / adm bank
							</td>
							<td class="py-1.5 text-right tabular">
								{{ angka(s.pajakBunga) }}
							</td>
						</tr>
						<tr v-if="s.pengembalian">
							<td class="py-1.5">
								Pengembalian dana
							</td>
							<td class="py-1.5 text-right tabular">
								{{ angka(s.pengembalian) }}
							</td>
						</tr>
						<tr class="font-semibold">
							<td class="py-1.5">
								Operasi / Modal
							</td>
							<td class="py-1.5 text-right tabular">
								{{ angka(s.belanjaOperasi) }} / {{ angka(s.belanjaModal) }}
							</td>
						</tr>
					</tbody>
				</table>
			</UCard>
		</div>
	</div>
</template>

<script lang="ts" setup>
	defineProps<{ s: PeriodSummary }>();
</script>
