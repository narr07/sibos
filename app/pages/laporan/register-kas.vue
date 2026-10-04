<template>
	<LayoutPageShell title="Register Kas">
		<template #actions>
			<UButton color="neutral" variant="outline" icon="i-lucide-save" :loading="saving" :disabled="!reg" @click="save">
				Simpan
			</UButton>
			<UButton icon="i-lucide-printer" :disabled="!reg" @click="print">
				Cetak BA Pemeriksaan Kas
			</UButton>
		</template>
		<div v-if="!connected" class="py-16 text-center text-muted">
			Hubungkan ARKAS terlebih dahulu.
		</div>
		<div v-else class="space-y-4">
			<USelect v-model="month" :items="monthItems" icon="i-lucide-calendar-days" class="w-44" />
			<UAlert v-if="error" color="error" variant="subtle" icon="i-lucide-triangle-alert" :title="error" />

			<div v-if="reg" class="grid lg:grid-cols-[1fr_22rem] gap-4">
				<UCard>
					<template #header>
						<p class="font-medium">
							Hitung uang tunai di brankas
						</p>
					</template>
					<div class="grid sm:grid-cols-2 gap-6">
						<div>
							<p class="text-sm font-medium mb-2">
								Uang kertas (lembar)
							</p>
							<div v-for="v in KERTAS" :key="v" class="flex items-center gap-2 mb-1.5">
								<span class="w-24 text-right tabular text-sm">{{ angka(v) }}</span>
								<UInputNumber v-model="pecahan.kertas[v]" :min="0" size="sm" class="w-32" />
								<span class="text-xs text-muted tabular">{{ angka(v * (pecahan.kertas[v] ?? 0)) }}</span>
							</div>
						</div>
						<div>
							<p class="text-sm font-medium mb-2">
								Uang logam (keping)
							</p>
							<div v-for="v in LOGAM" :key="v" class="flex items-center gap-2 mb-1.5">
								<span class="w-24 text-right tabular text-sm">{{ angka(v) }}</span>
								<UInputNumber v-model="pecahan.logam[v]" :min="0" size="sm" class="w-32" />
								<span class="text-xs text-muted tabular">{{ angka(v * (pecahan.logam[v] ?? 0)) }}</span>
							</div>
						</div>
					</div>
					<UFormField label="Catatan" class="mt-4">
						<UTextarea v-model="catatan" :rows="2" class="w-full" />
					</UFormField>
				</UCard>

				<UCard>
					<template #header>
						<p class="font-medium">
							Posisi akhir {{ BULAN[month - 1] }}
						</p>
					</template>
					<dl class="space-y-2 text-sm">
						<div class="flex justify-between">
							<dt class="text-muted">
								Penerimaan s.d. bulan ini
							</dt>
							<dd class="tabular">
								{{ rupiah(reg.penerimaanSd) }}
							</dd>
						</div>
						<div class="flex justify-between">
							<dt class="text-muted">
								Pengeluaran s.d. bulan ini
							</dt>
							<dd class="tabular">
								{{ rupiah(reg.pengeluaranSd) }}
							</dd>
						</div>
						<div class="flex justify-between font-semibold">
							<dt>Saldo buku</dt>
							<dd class="tabular">
								{{ rupiah(reg.saldoBuku) }}
							</dd>
						</div>
						<div class="flex justify-between">
							<dt class="text-muted">
								Saldo bank
							</dt>
							<dd class="tabular">
								{{ rupiah(reg.saldoBank) }}
							</dd>
						</div>
						<div class="flex justify-between">
							<dt class="text-muted">
								Saldo tunai (buku)
							</dt>
							<dd class="tabular">
								{{ rupiah(reg.saldoTunai) }}
							</dd>
						</div>
						<USeparator />
						<div class="flex justify-between font-semibold">
							<dt>Uang fisik</dt>
							<dd class="tabular">
								{{ rupiah(fisik) }}
							</dd>
						</div>
						<div class="flex justify-between font-semibold" :class="selisih === 0 ? 'text-success' : 'text-error'">
							<dt>{{ selisih === 0 ? "Sesuai" : selisih > 0 ? "Fisik lebih" : "Fisik kurang" }}</dt>
							<dd class="tabular">
								{{ rupiah(selisih) }}
							</dd>
						</div>
						<div v-if="reg.pajakBelumDisetor" class="flex justify-between text-warning">
							<dt>Pajak belum disetor</dt>
							<dd class="tabular">
								{{ rupiah(reg.pajakBelumDisetor) }}
							</dd>
						</div>
					</dl>
				</UCard>
			</div>
		</div>
	</LayoutPageShell>
</template>

<script lang="ts" setup>
	import BaPemeriksaanKas from "~/components/Print/Laporan/BaPemeriksaanKas.vue";

	const KERTAS = [100000, 50000, 20000, 10000, 5000, 2000, 1000];
	const LOGAM = [1000, 500, 200, 100];

	const { connected, year, fund } = useArkas();
	const { open } = usePrint();
	const toast = useToast();

	const month = ref(new Date().getMonth() + 1);
	const monthItems = BULAN.map((label, i) => ({ label, value: i + 1 }));
	const reg = ref<RegisterKas | null>(null);
	const pecahan = reactive<{ kertas: Record<number, number>, logam: Record<number, number> }>({ kertas: {}, logam: {} });
	const catatan = ref("");
	const saving = ref(false);
	const error = ref("");

	const fundArg = computed(() => (fund.value === ALL_FUNDS ? null : fund.value));
	const fisik = computed(() =>
		KERTAS.reduce((s, v) => s + v * (pecahan.kertas[v] ?? 0), 0) + LOGAM.reduce((s, v) => s + v * (pecahan.logam[v] ?? 0), 0));
	const selisih = computed(() => fisik.value - (reg.value?.saldoTunai ?? 0));

	const toRecord = (src: Record<number, number>, values: number[]) =>
		Object.fromEntries(values.filter((v) => (src[v] ?? 0) > 0).map((v) => [String(v), src[v] ?? 0]));

	const load = async () => {
		if (!connected.value || !year.value) return;
		error.value = "";
		try {
			reg.value = await api.registerKasGet(year.value, month.value, fundArg.value);
			const saved = reg.value.saved?.pecahan;
			pecahan.kertas = Object.fromEntries(KERTAS.map((v) => [v, saved?.kertas[String(v)] ?? 0]));
			pecahan.logam = Object.fromEntries(LOGAM.map((v) => [v, saved?.logam[String(v)] ?? 0]));
			catatan.value = reg.value.saved?.catatan ?? "";
		} catch (err) {
			error.value = errorMessage(err);
		}
	};
	watch([month, year, fund, connected], load, { immediate: true });

	const value = (): CashRegister => ({
		pecahan: { kertas: toRecord(pecahan.kertas, KERTAS), logam: toRecord(pecahan.logam, LOGAM) },
		catatan: catatan.value || null
	});

	const save = async () => {
		if (!year.value) return;
		saving.value = true;
		try {
			await api.registerKasSet(year.value, month.value, fundArg.value, value());
			toast.add({ title: "Register kas tersimpan", color: "success" });
			await load();
		} catch (err) {
			toast.add({ title: "Gagal menyimpan", description: errorMessage(err), color: "error" });
		} finally {
			saving.value = false;
		}
	};

	const print = () => {
		if (!reg.value) return;
		const v = value();
		open({ title: "BA Pemeriksaan Kas", component: BaPemeriksaanKas, props: { reg: reg.value, pecahan: v.pecahan, catatan: v.catatan } });
	};
</script>
