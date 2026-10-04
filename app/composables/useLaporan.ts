/** Logika bersama halaman laporan periode (SPTJM, BA Rekonsiliasi). */
export const usePeriodSummary = () => {
	const { connected, year, fund, funds } = useArkas();
	const range = ref({ start: 1, end: 6 });
	const summary = ref<PeriodSummary | null>(null);
	const loading = ref(false);
	const error = ref("");

	const fundArg = computed(() => (fund.value === ALL_FUNDS ? null : fund.value));
	const fundLabel = computed(() => funds.value.find((f) => f.id === fund.value)?.name ?? "Dana BOSP (semua sumber dana)");
	const periodLabel = computed(() => (year.value ? labelPeriode(year.value, range.value.start, range.value.end) : ""));

	const load = async () => {
		if (!connected.value || !year.value) return;
		loading.value = true;
		error.value = "";
		try {
			summary.value = await api.periodSummary(year.value, range.value.start, range.value.end, fundArg.value);
		} catch (err) {
			error.value = errorMessage(err);
			summary.value = null;
		} finally {
			loading.value = false;
		}
	};

	watch([range, year, fund, connected], load, { immediate: true, deep: true });

	return { connected, year, range, summary, loading, error, fundLabel, periodLabel, reload: load };
};
