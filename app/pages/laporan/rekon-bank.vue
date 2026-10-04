<template>
	<LayoutPageShell title="Rekonsiliasi Bank">
		<template #actions>
			<UButton icon="i-lucide-printer" :disabled="!data" @click="print">
				Cetak
			</UButton>
		</template>
		<div v-if="!connected" class="py-16 text-center text-muted">
			Hubungkan ARKAS terlebih dahulu.
		</div>
		<div v-else class="space-y-4">
			<UAlert
				icon="i-lucide-info"
				color="neutral"
				variant="subtle"
				title="Isi saldo rekening koran tiap akhir bulan. Selisih dengan saldo bank di BKU dihitung otomatis."
				description="Saldo rekening koran disimpan di SIBOS."
			/>
			<UAlert v-if="error" color="error" variant="subtle" icon="i-lucide-triangle-alert" :title="error" />

			<UTable
				v-if="data"
				:data="data.months"
				:columns="columns"
				class="border border-default rounded-md"
				:ui="{ td: 'py-1.5 text-sm', th: 'py-2 text-xs' }"
			>
				<template #rekeningKoran-cell="{ row }">
					<UInputNumber
						:model-value="row.original.rekeningKoran ?? undefined"
						:format-options="{ maximumFractionDigits: 0 }"
						locale="id-ID"
						placeholder="-"
						:increment="false"
						:decrement="false"
						size="sm"
						@update:model-value="(v) => save(row.original.month, v ?? null, row.original.keterangan)"
					/>
				</template>
				<template #selisih-cell="{ row }">
					<span :class="row.original.selisih ? 'text-error font-semibold' : 'text-success'">
						{{ row.original.selisih === null ? "" : angka(row.original.selisih) }}
					</span>
				</template>
				<template #keterangan-cell="{ row }">
					<UInput
						:model-value="row.original.keterangan ?? ''"
						size="sm"
						placeholder="Keterangan"
						@change="(e: Event) => save(row.original.month, row.original.rekeningKoran, (e.target as HTMLInputElement).value)"
					/>
				</template>
			</UTable>
		</div>
	</LayoutPageShell>
</template>

<script lang="ts" setup>
	import type { TableColumn } from "@nuxt/ui";
	import RekonBankDoc from "~/components/Print/Laporan/RekonBank.vue";

	const { connected, year, fund, funds } = useArkas();
	const { open } = usePrint();
	const toast = useToast();

	const data = ref<RekonBank | null>(null);
	const error = ref("");
	const fundArg = computed(() => (fund.value === ALL_FUNDS ? null : fund.value));
	const fundLabel = computed(() => funds.value.find((f) => f.id === fund.value)?.name ?? "Semua sumber dana");

	const load = async () => {
		if (!connected.value || !year.value) return;
		error.value = "";
		try {
			data.value = await api.rekonBank(year.value, fundArg.value);
		} catch (err) {
			error.value = errorMessage(err);
		}
	};
	watch([year, fund, connected], load, { immediate: true });

	const save = async (month: number, saldo: number | null, keterangan: string | null) => {
		if (!year.value) return;
		try {
			await api.bankStatementSet(year.value, fundArg.value, month, saldo, keterangan);
			await load();
		} catch (err) {
			toast.add({ title: "Gagal menyimpan", description: errorMessage(err), color: "error" });
		}
	};

	const nominal = { th: "text-right", td: "text-right tabular" };
	const columns: TableColumn<RekonBankMonth>[] = [
		{ accessorKey: "month", header: "Bulan", cell: ({ row }) => BULAN[row.original.month - 1] },
		{ accessorKey: "penerimaan", header: "Penerimaan", cell: ({ row }) => angka(row.original.penerimaan), meta: { class: nominal } },
		{ accessorKey: "belanja", header: "Belanja", cell: ({ row }) => angka(row.original.belanja), meta: { class: nominal } },
		{ accessorKey: "saldoBank", header: "Saldo bank (BKU)", cell: ({ row }) => angka(row.original.saldoBank), meta: { class: { ...nominal, td: `${nominal.td} font-medium` } } },
		{ accessorKey: "rekeningKoran", header: "Saldo rekening koran", meta: { class: { th: "text-right w-48", td: "py-1" } } },
		{ accessorKey: "selisih", header: "Selisih", meta: { class: nominal } },
		{ accessorKey: "keterangan", header: "Keterangan", meta: { class: { td: "py-1" } } }
	];

	const print = () => {
		if (!data.value) return;
		open({ title: "Rekonsiliasi Bank", component: RekonBankDoc, props: { data: data.value, fundLabel: fundLabel.value } });
	};
</script>
