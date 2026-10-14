import type { Ref } from "vue";

/**
 * Dasar halaman generator dokumen: daftar belanja BKU yang bisa disaring dan dicari, isian per
 * belanja yang tersimpan di SIBOS (per tahun anggaran), dan data pendukung (guru, nama kegiatan).
 */
/** Data pendukung yang diberikan ke pembuat isian. */
export interface KonteksGenerator<T> {
	guru: Guru[]
	/** Isian tersimpan tahun ini, per key nota. */
	saved: Record<string, T>
	pejabat: Pengaturan["pejabat"] | null
	/** Nama sekolah dari ARKAS. */
	sekolah: string
	/** Nama kegiatan RKAS untuk belanja ini. */
	namaKegiatan: string
}

export const useGeneratorBelanja = <T>(opts: {
	/** Kunci pengaturan tempat isian disimpan. */
	setting: (year: number) => string
	/** Belanja yang relevan untuk generator ini (saringan bawaan, bisa dimatikan pengguna). */
	cocok: (g: NotaGroup, rekeningNames: Record<string, string>) => boolean
	/** Isian untuk satu belanja: dari yang tersimpan bila ada, selain itu isian awal dari ARKAS. */
	buat: (g: NotaGroup, tersimpan: T | undefined, ctx: KonteksGenerator<T>) => T
	pesanSimpan: string
}) => {
	const { connected, year, fund } = useArkas();
	const { pengaturan, sekolah, load: loadDoc } = useDocContext();
	const toast = useToast();

	const groups = ref<NotaGroup[]>([]);
	const kodeNames = ref<Record<string, string>>({});
	const rekeningNames = ref<Record<string, string>>({});
	const guru = ref<Guru[]>([]);
	/** Isian tersimpan tahun ini, per key nota. */
	const saved = ref({}) as Ref<Record<string, T>>;
	const loading = ref(false);
	const saving = ref(false);
	const error = ref("");
	const month = ref(0);
	const search = ref("");
	const hanyaCocok = ref(true);

	const selectedKey = ref<string | null>(null);
	const form = ref(null) as Ref<T | null>;

	const filtered = computed(() => {
		const q = search.value.trim().toLowerCase();
		return groups.value.filter((g) => {
			if (hanyaCocok.value && !opts.cocok(g, rekeningNames.value)) return false;
			if (!q) return true;
			return [g.noBukti, g.uraian, g.nota?.namaToko, ...g.items.map((i) => i.uraian)].some((v) => v?.toLowerCase().includes(q));
		});
	});

	const selected = computed(() => groups.value.find((g) => g.key === selectedKey.value) ?? null);

	const namaKegiatan = (g: NotaGroup | null) => {
		const kode = g?.items.find((i) => i.kodeKegiatan)?.kodeKegiatan;
		return kode ? kodeNames.value[kode] ?? "" : "";
	};
	const namaKegiatanArkas = computed(() => namaKegiatan(selected.value));

	const konteks = (g: NotaGroup): KonteksGenerator<T> => ({
		guru: guru.value,
		saved: saved.value,
		pejabat: pengaturan.value?.pejabat ?? null,
		sekolah: sekolah.value?.nama ?? "",
		namaKegiatan: namaKegiatan(g)
	});

	const pick = (g: NotaGroup) => {
		selectedKey.value = g.key;
		const tersimpan = saved.value[g.key];
		form.value = opts.buat(g, tersimpan ? structuredClone(toRaw(tersimpan)) : undefined, konteks(g));
	};

	/** Buang isian tersimpan di layar dan isi ulang dari ARKAS. */
	const reset = () => {
		if (selected.value) form.value = opts.buat(selected.value, undefined, konteks(selected.value));
	};

	const fundArg = computed(() => (fund.value === ALL_FUNDS ? null : fund.value));

	const loadGroups = async () => {
		if (!connected.value || !year.value) return;
		loading.value = true;
		error.value = "";
		try {
			groups.value = await api.notaList(year.value, month.value || null, fundArg.value);
			if (!selected.value) {
				selectedKey.value = null;
				form.value = null;
			}
		} catch (err) {
			error.value = errorMessage(err);
		} finally {
			loading.value = false;
		}
	};

	/** Data per tahun: nama kegiatan/rekening dari RKAS, daftar guru, dan isian tersimpan. */
	const loadYear = async () => {
		selectedKey.value = null;
		form.value = null;
		if (!connected.value || !year.value) return;
		const [kk, tersimpan, daftarGuru] = await Promise.all([
			api.kertasKerja(year.value, null).catch(() => null),
			api.settingsGet<Record<string, T>>(opts.setting(year.value)).catch(() => null),
			api.settingsGet<Guru[]>(SETTING_GURU).catch(() => null),
			loadDoc().catch(() => {})
		]);
		kodeNames.value = kk?.kodeNames ?? {};
		rekeningNames.value = kk?.rekeningNames ?? {};
		saved.value = tersimpan ?? {};
		guru.value = daftarGuru ?? [];
		await loadGroups();
	};

	watch([year, connected], loadYear, { immediate: true });
	watch([month, fund], loadGroups);

	const save = async (diam = false) => {
		if (!form.value || !selectedKey.value || !year.value) return false;
		saving.value = true;
		try {
			const next = { ...toRaw(saved.value), [selectedKey.value]: structuredClone(toRaw(form.value)) };
			await api.settingsSet(opts.setting(year.value), next);
			saved.value = next;
			if (!diam) toast.add({ title: opts.pesanSimpan, color: "success" });
			return true;
		} catch (err) {
			toast.add({ title: "Gagal menyimpan", description: errorMessage(err), color: "error" });
			return false;
		} finally {
			saving.value = false;
		}
	};

	return {
		connected,
		year,
		pengaturan,
		sekolah,
		groups,
		kodeNames,
		guru,
		saved,
		loading,
		saving,
		error,
		month,
		search,
		hanyaCocok,
		selectedKey,
		selected,
		form,
		filtered,
		namaKegiatan,
		namaKegiatanArkas,
		pick,
		reset,
		save
	};
};
