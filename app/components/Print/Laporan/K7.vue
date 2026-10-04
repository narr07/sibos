<template>
	<PrintSheet landscape>
		<div class="doc text-[10pt]">
			<p class="text-center text-[13pt] font-bold">
				REKAPITULASI REALISASI PENGGUNAAN DANA BOSP (K7/K7a)
			</p>
			<p class="text-center mb-3">
				Periode {{ periodLabel }}
			</p>
			<PrintFields class="mb-3" label-width="10rem" :rows="[['Nama Sekolah', sekolah?.nama], ['Kecamatan', sekolah?.kecamatan], ['Kabupaten/Kota', sekolah?.kabupaten], ['Provinsi', sekolah?.provinsi]]" />
			<table class="tabel">
				<thead>
					<tr>
						<th class="w-10">
							No
						</th>
						<th>Uraian / Standar</th>
						<th v-for="f in funds" :key="f.fund.id">
							{{ f.fund.name }}
						</th>
						<th>Jumlah</th>
					</tr>
				</thead>
				<tbody>
					<tr class="font-semibold">
						<td />
						<td>Saldo periode sebelumnya</td>
						<td v-for="f in funds" :key="f.fund.id" class="num">
							{{ angka(f.s.saldoAwalBank + f.s.saldoAwalTunai) }}
						</td>
						<td class="num">
							{{ angka(sum((s) => s.saldoAwalBank + s.saldoAwalTunai)) }}
						</td>
					</tr>
					<tr class="font-semibold">
						<td />
						<td>Penerimaan periode ini</td>
						<td v-for="f in funds" :key="f.fund.id" class="num">
							{{ angka(f.s.totalPenerimaan) }}
						</td>
						<td class="num">
							{{ angka(sum((s) => s.totalPenerimaan)) }}
						</td>
					</tr>
					<tr v-for="(p, i) in programs" :key="p.kode">
						<td class="text-center">
							{{ i + 1 }}
						</td>
						<td>{{ p.kode }} {{ p.nama }}</td>
						<td v-for="f in funds" :key="f.fund.id" class="num">
							{{ angka(programTotal(f.s, p.kode)) }}
						</td>
						<td class="num">
							{{ angka(sum((s) => programTotal(s, p.kode))) }}
						</td>
					</tr>
					<tr v-if="sum((s) => s.pajakBunga + s.pengembalian)">
						<td />
						<td>Pajak bunga, pengembalian dana, dan lain-lain</td>
						<td v-for="f in funds" :key="f.fund.id" class="num">
							{{ angka(f.s.pajakBunga + f.s.pengembalian) }}
						</td>
						<td class="num">
							{{ angka(sum((s) => s.pajakBunga + s.pengembalian)) }}
						</td>
					</tr>
					<tr class="font-bold">
						<td />
						<td>Jumlah pengeluaran</td>
						<td v-for="f in funds" :key="f.fund.id" class="num">
							{{ angka(f.s.totalBelanja + f.s.pajakBunga + f.s.pengembalian) }}
						</td>
						<td class="num">
							{{ angka(sum((s) => s.totalBelanja + s.pajakBunga + s.pengembalian)) }}
						</td>
					</tr>
					<tr class="font-bold">
						<td />
						<td>Saldo akhir periode</td>
						<td v-for="f in funds" :key="f.fund.id" class="num">
							{{ angka(f.s.saldoAkhirBank + f.s.saldoAkhirTunai) }}
						</td>
						<td class="num">
							{{ angka(sum((s) => s.saldoAkhirBank + s.saldoAkhirTunai)) }}
						</td>
					</tr>
				</tbody>
			</table>
			<PrintTandaTangan :tanggal="tanggalPanjang(akhirBulan(year, end))" />
		</div>
	</PrintSheet>
</template>

<script lang="ts" setup>
	const props = defineProps<{ funds: { fund: FundSource, s: PeriodSummary }[], periodLabel: string, year: number, end: number }>();
	const { sekolah } = useDocContext();

	const programs = computed(() => {
		const map = new Map<string, string>();
		for (const f of props.funds) {
			for (const p of f.s.belanjaProgram) {
				if (!map.has(p.kode) || !map.get(p.kode)) map.set(p.kode, p.nama);
			}
		}
		return [...map.entries()].sort(([a], [b]) => a.localeCompare(b)).map(([kode, nama]) => ({ kode, nama }));
	});

	const programTotal = (s: PeriodSummary, kode: string) => s.belanjaProgram.find((p) => p.kode === kode)?.total ?? 0;
	const sum = (fn: (s: PeriodSummary) => number) => props.funds.reduce((acc, f) => acc + fn(f.s), 0);
</script>
