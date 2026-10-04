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

			<div v-if="data" class="border border-default rounded-md overflow-auto">
				<table class="w-full text-sm">
					<thead class="bg-elevated text-xs text-muted">
						<tr>
							<th class="p-2 text-left">
								Bulan
							</th>
							<th class="p-2 text-right">
								Penerimaan
							</th>
							<th class="p-2 text-right">
								Belanja
							</th>
							<th class="p-2 text-right">
								Saldo bank (BKU)
							</th>
							<th class="p-2 text-right w-48">
								Saldo rekening koran
							</th>
							<th class="p-2 text-right">
								Selisih
							</th>
							<th class="p-2 text-left">
								Keterangan
							</th>
						</tr>
					</thead>
					<tbody>
						<tr v-for="m in data.months" :key="m.month" class="border-t border-default">
							<td class="p-2">
								{{ BULAN[m.month - 1] }}
							</td>
							<td class="p-2 text-right tabular">
								{{ angka(m.penerimaan) }}
							</td>
							<td class="p-2 text-right tabular">
								{{ angka(m.belanja) }}
							</td>
							<td class="p-2 text-right tabular font-medium">
								{{ angka(m.saldoBank) }}
							</td>
							<td class="p-1">
								<UInputNumber
									:model-value="m.rekeningKoran ?? undefined"
									:format-options="{ maximumFractionDigits: 0 }"
									locale="id-ID"
									placeholder="-"
									:increment="false"
									:decrement="false"
									size="sm"
									@update:model-value="(v) => save(m.month, v ?? null, m.keterangan)"
								/>
							</td>
							<td class="p-2 text-right tabular" :class="m.selisih ? 'text-error font-semibold' : 'text-success'">
								{{ m.selisih === null ? "" : angka(m.selisih) }}
							</td>
							<td class="p-1">
								<UInput
									:model-value="m.keterangan ?? ''"
									size="sm"
									placeholder="Keterangan"
									@change="(e: Event) => save(m.month, m.rekeningKoran, (e.target as HTMLInputElement).value)"
								/>
							</td>
						</tr>
					</tbody>
				</table>
			</div>
		</div>
	</LayoutPageShell>
</template>

<script lang="ts" setup>
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

	const print = () => {
		if (!data.value) return;
		open({ title: "Rekonsiliasi Bank", component: RekonBankDoc, props: { data: data.value, fundLabel: fundLabel.value } });
	};
</script>
