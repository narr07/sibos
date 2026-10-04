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
				<UTable
					:data="penerimaanRows"
					:columns="penerimaanColumns"
					empty="Tidak ada penerimaan pada periode ini."
					:ui="{ td: 'py-1.5 text-sm', th: 'py-2 text-xs' }"
				/>
			</UCard>
			<UCard>
				<template #header>
					<p class="font-medium">
						Pengeluaran per kelompok
					</p>
				</template>
				<UTable
					:data="pengeluaranRows"
					:columns="pengeluaranColumns"
					:meta="{ class: { tr: (row) => row.original.bold ? 'font-semibold' : '' } }"
					:ui="{ td: 'py-1.5 text-sm', th: 'py-2 text-xs' }"
				/>
			</UCard>
		</div>
	</div>
</template>

<script lang="ts" setup>
	import type { TableColumn } from "@nuxt/ui";

	const props = defineProps<{ s: PeriodSummary }>();

	interface Baris { tanggal?: string, uraian: string, nilai: string, bold?: boolean }
	const nominal = { th: "text-right", td: "text-right tabular" };

	const penerimaanRows = computed<Baris[]>(() => [
		...props.s.penerimaan.map((p) => ({ tanggal: tanggalId(p.tanggal), uraian: p.uraian, nilai: angka(p.nominal) })),
		...(props.s.bunga ? [{ uraian: "Bunga bank", nilai: angka(props.s.bunga) }] : [])
	]);
	const penerimaanColumns: TableColumn<Baris>[] = [
		{ accessorKey: "tanggal", header: "Tanggal", meta: { class: { td: "tabular text-muted w-24" } } },
		{ accessorKey: "uraian", header: "Uraian", meta: { class: { td: "whitespace-normal" } } },
		{ accessorKey: "nilai", header: "Jumlah", meta: { class: nominal } }
	];

	const pengeluaranRows = computed<Baris[]>(() => [
		...props.s.belanjaKelompok.map((k) => ({ uraian: k.nama, nilai: angka(k.total) })),
		...(props.s.pajakBunga ? [{ uraian: "Pajak bunga / adm bank", nilai: angka(props.s.pajakBunga) }] : []),
		...(props.s.pengembalian ? [{ uraian: "Pengembalian dana", nilai: angka(props.s.pengembalian) }] : []),
		{ uraian: "Operasi / Modal", nilai: `${angka(props.s.belanjaOperasi)} / ${angka(props.s.belanjaModal)}`, bold: true }
	]);
	const pengeluaranColumns: TableColumn<Baris>[] = [
		{ accessorKey: "uraian", header: "Kelompok", meta: { class: { td: "whitespace-normal" } } },
		{ accessorKey: "nilai", header: "Jumlah", meta: { class: nominal } }
	];
</script>
