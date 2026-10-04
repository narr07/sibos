import type { ConnectionStatus, FundSource } from "~/utils/api";

const SETTING_YEAR = "ui.year";
const SETTING_FUND = "ui.fund";

/** Sumber dana "semua" dipakai sebagai nilai default filter. */
export const ALL_FUNDS = 0;

/**
 * Status koneksi ARKAS dan pilihan global tahun anggaran + sumber dana.
 * Dipakai bersama oleh semua halaman.
 */
export const useArkas = () => {
	const status = useState<ConnectionStatus | null>("arkas-status", () => null);
	const years = useState<number[]>("arkas-years", () => []);
	const year = useState<number | null>("arkas-year", () => null);
	const funds = useState<FundSource[]>("arkas-funds", () => []);
	const fund = useState<number>("arkas-fund", () => ALL_FUNDS);
	const loading = useState<boolean>("arkas-loading", () => false);

	const connected = computed(() => status.value?.state === "connected");
	const school = computed(() => status.value?.school ?? null);

	const loadFunds = async () => {
		if (!connected.value || !year.value) {
			funds.value = [];
			return;
		}
		funds.value = await api.fundSources(year.value);
		if (fund.value !== ALL_FUNDS && !funds.value.some((f) => f.id === fund.value)) {
			fund.value = ALL_FUNDS;
		}
	};

	const loadYears = async () => {
		if (!connected.value) {
			years.value = [];
			year.value = null;
			return;
		}
		years.value = await api.availableYears();
		const saved = await api.settingsGet<number>(SETTING_YEAR);
		const current = new Date().getFullYear();
		year.value = [saved, current, years.value[0]].find((y) => y != null && years.value.includes(y)) ?? null;
		fund.value = (await api.settingsGet<number>(SETTING_FUND)) ?? ALL_FUNDS;
		await loadFunds();
	};

	const applyStatus = async (next: ConnectionStatus) => {
		status.value = next;
		await loadYears();
		return next;
	};

	const refresh = async () => applyStatus(await api.arkasStatus());

	const connect = async (args: { path?: string, key?: string, rememberKey?: boolean } = {}) => {
		loading.value = true;
		try {
			return await applyStatus(await api.arkasConnect(args));
		} finally {
			loading.value = false;
		}
	};

	const disconnect = async (forgetKey = false) => applyStatus(await api.arkasDisconnect(forgetKey));

	const setYear = async (value: number) => {
		year.value = value;
		await api.settingsSet(SETTING_YEAR, value);
		await loadFunds();
	};

	const setFund = async (value: number) => {
		fund.value = value;
		await api.settingsSet(SETTING_FUND, value);
	};

	return { status, years, year, funds, fund, loading, connected, school, refresh, connect, disconnect, setYear, setFund };
};
