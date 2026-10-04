<template>
	<LayoutPageShell title="Dashboard">
		<div v-if="!connected" class="max-w-xl mx-auto py-16 text-center space-y-4">
			<UIcon name="i-lucide-database" class="size-12 text-dimmed mx-auto" />
			<h2 class="text-xl font-semibold">
				Hubungkan ARKAS
			</h2>
			<p class="text-muted">
				SIBOS membaca data dari database ARKAS di komputer ini. Pastikan ARKAS sudah terpasang dan data sekolah sudah diunduh.
			</p>
			<UButton to="/setup" icon="i-lucide-plug">
				Hubungkan sekarang
			</UButton>
		</div>

		<div v-else class="space-y-5">
			<div class="flex flex-wrap items-end justify-between gap-2">
				<div>
					<p class="text-lg font-semibold">
						{{ school?.nama ?? "Sekolah" }}
					</p>
					<p class="text-sm text-muted">
						NPSN {{ school?.npsn ?? "-" }} · KS {{ school?.kepalaSekolah ?? "-" }} · Bendahara {{ school?.bendahara ?? "-" }}
					</p>
				</div>
				<p v-if="d" class="text-xs text-muted">
					Data s.d. {{ BULAN[d.upto - 1] }} {{ d.year }}
				</p>
			</div>

			<UAlert v-if="error" color="error" variant="subtle" icon="i-lucide-triangle-alert" :title="error" />

			<template v-if="d">
				<div class="grid grid-cols-2 xl:grid-cols-4 gap-3">
					<UCard :ui="{ body: 'p-4 sm:p-4' }">
						<p class="text-xs text-muted">
							Pagu anggaran
						</p>
						<p class="text-2xl font-semibold tabular">
							{{ rupiah(d.pagu) }}
						</p>
					</UCard>
					<UCard :ui="{ body: 'p-4 sm:p-4' }">
						<p class="text-xs text-muted">
							Realisasi belanja
						</p>
						<p class="text-2xl font-semibold tabular">
							{{ rupiah(d.totalRealisasi) }}
						</p>
						<div class="flex items-center gap-2 mt-1">
							<UProgress :model-value="Math.min(100, d.persen)" size="sm" class="flex-1" />
							<span class="text-xs tabular">{{ d.persen.toLocaleString("id-ID") }}%</span>
						</div>
					</UCard>
					<UCard :ui="{ body: 'p-4 sm:p-4' }">
						<p class="text-xs text-muted">
							Saldo saat ini
						</p>
						<p class="text-2xl font-semibold tabular">
							{{ rupiah(d.summary.saldoAkhirBank + d.summary.saldoAkhirTunai) }}
						</p>
						<p class="text-xs text-muted tabular">
							Bank {{ rupiah(d.summary.saldoAkhirBank) }} · Tunai {{ rupiah(d.summary.saldoAkhirTunai) }}
						</p>
					</UCard>
					<UCard :ui="{ body: 'p-4 sm:p-4' }">
						<p class="text-xs text-muted">
							Penerimaan tahun ini
						</p>
						<p class="text-2xl font-semibold tabular">
							{{ rupiah(d.summary.totalPenerimaan) }}
						</p>
						<p class="text-xs text-muted">
							{{ d.summary.penerimaan.length }} kali terima dana
						</p>
					</UCard>
				</div>

				<div class="grid xl:grid-cols-[1fr_22rem] gap-4">
					<UCard>
						<template #header>
							<p class="font-medium">
								Rencana vs realisasi belanja per bulan
							</p>
						</template>
						<ChartMonthBars
							:series="[{ label: 'Rencana RKAS', values: d.rencanaBulan }, { label: 'Realisasi', values: d.realisasiBulan }]"
							aria-label="Grafik rencana dan realisasi belanja per bulan"
						/>
					</UCard>

					<div class="space-y-4">
						<UCard>
							<template #header>
								<p class="font-medium">
									Status
								</p>
							</template>
							<ul class="space-y-2 text-sm">
								<li v-for="a in activeAnggaran" :key="a.idAnggaran" class="flex items-start gap-2">
									<UIcon :name="a.isApprove ? 'i-lucide-badge-check' : 'i-lucide-clock'" :class="a.isApprove ? 'text-success' : 'text-warning'" class="size-4 mt-0.5 shrink-0" />
									<span>RKAS {{ a.fundName }}: {{ a.isApprove ? "disahkan" : "menunggu pengesahan" }}{{ revisionLabel(a) }}</span>
								</li>
								<li v-for="a in pendingAnggaran" :key="a.idAnggaran" class="flex items-start gap-2">
									<UIcon name="i-lucide-clock" class="size-4 mt-0.5 shrink-0 text-warning" />
									<span>{{ a.fundName }}: revisi {{ a.isRevisi }} belum disetujui{{ a.alasanPenolakan ? ` (${a.alasanPenolakan})` : "" }}</span>
								</li>
								<li class="flex items-start gap-2">
									<UIcon name="i-lucide-book-check" class="size-4 mt-0.5 shrink-0 text-info" />
									<span>BKU ditutup: {{ d.bulanDitutup.length ? d.bulanDitutup.map((m) => BULAN[m - 1]?.slice(0, 3)).join(", ") : "belum ada" }}</span>
								</li>
							</ul>
						</UCard>

						<UCard>
							<template #header>
								<p class="font-medium">
									Perlu perhatian
								</p>
							</template>
							<ul class="space-y-2 text-sm">
								<li v-if="d.pajakBelumDisetor" class="flex items-start gap-2">
									<UIcon name="i-lucide-receipt" class="size-4 mt-0.5 shrink-0 text-warning" />
									<NuxtLink to="/penatausahaan/pajak" class="hover:underline">
										Pajak belum disetor {{ rupiah(d.pajakBelumDisetor) }}
									</NuxtLink>
								</li>
								<li v-if="d.totalTertunda" class="flex items-start gap-2">
									<UIcon name="i-lucide-calendar-clock" class="size-4 mt-0.5 shrink-0 text-warning" />
									<NuxtLink to="/penganggaran/realisasi" class="hover:underline">
										{{ d.itemBelum }} item jatuh tempo belum dibelanjakan ({{ rupiah(d.totalTertunda) }})
									</NuxtLink>
								</li>
								<li v-if="d.itemMelampaui" class="flex items-start gap-2">
									<UIcon name="i-lucide-triangle-alert" class="size-4 mt-0.5 shrink-0 text-error" />
									<NuxtLink to="/penganggaran/realisasi" class="hover:underline">
										{{ d.itemMelampaui }} item melampaui pagu
									</NuxtLink>
								</li>
								<li v-for="(w, i) in d.summary.warnings" :key="i" class="flex items-start gap-2">
									<UIcon name="i-lucide-circle-x" class="size-4 mt-0.5 shrink-0 text-error" />
									<span>{{ w }}</span>
								</li>
								<li v-if="!d.pajakBelumDisetor && !d.totalTertunda && !d.itemMelampaui && !d.summary.warnings.length" class="flex items-start gap-2 text-success">
									<UIcon name="i-lucide-badge-check" class="size-4 mt-0.5 shrink-0" />
									<span>Tidak ada catatan.</span>
								</li>
							</ul>
						</UCard>
					</div>
				</div>

				<UCard>
					<template #header>
						<p class="font-medium">
							Komposisi belanja
						</p>
					</template>
					<div class="space-y-2.5">
						<div v-for="k in d.summary.belanjaKelompok" :key="k.kode" class="grid grid-cols-[14rem_1fr_9rem] items-center gap-3 text-sm">
							<span class="truncate">{{ k.nama }}</span>
							<div class="h-2.5 rounded-full bg-elevated overflow-hidden">
								<div class="h-full rounded-full bg-primary" :style="{ width: `${share(k.total)}%` }" />
							</div>
							<span class="text-right tabular">{{ rupiah(k.total) }} <span class="text-muted text-xs">{{ share(k.total).toLocaleString("id-ID", { maximumFractionDigits: 1 }) }}%</span></span>
						</div>
						<p v-if="!d.summary.belanjaKelompok.length" class="text-sm text-muted">
							Belum ada belanja.
						</p>
					</div>
				</UCard>
			</template>
			<div v-else-if="loading" class="py-16 text-center text-muted">
				Memuat data...
			</div>
		</div>
	</LayoutPageShell>
</template>

<script lang="ts" setup>
	const { connected, school, year, fund } = useArkas();

	const d = ref<Dashboard | null>(null);
	const loading = ref(false);
	const error = ref("");
	const fundArg = computed(() => (fund.value === ALL_FUNDS ? null : fund.value));

	const load = async () => {
		if (!connected.value || !year.value) return;
		loading.value = true;
		error.value = "";
		try {
			d.value = await api.dashboard(year.value, fundArg.value);
		} catch (err) {
			error.value = errorMessage(err);
		} finally {
			loading.value = false;
		}
	};
	watch([year, fund, connected], load, { immediate: true });

	const activeAnggaran = computed(() => {
		const best = new Map<number, AnggaranInfo>();
		for (const a of d.value?.anggaran ?? []) {
			if (!a.isApprove) continue;
			const cur = best.get(a.fundId);
			if (!cur || a.isRevisi > cur.isRevisi) best.set(a.fundId, a);
		}
		return [...best.values()];
	});
	const pendingAnggaran = computed(() => (d.value?.anggaran ?? []).filter((a) =>
		!a.isApprove && activeAnggaran.value.some((b) => b.fundId === a.fundId && a.isRevisi > b.isRevisi)));

	const revisionLabel = (a: AnggaranInfo) => {
		if (a.isRevisi >= 100) return ` (perubahan ke-${a.isRevisi - 99})`;
		if (a.isRevisi > 0) return ` (revisi ${a.isRevisi})`;
		return "";
	};

	const share = (v: number) => {
		const total = d.value?.summary.totalBelanja ?? 0;
		return total ? (v / total) * 100 : 0;
	};
</script>
