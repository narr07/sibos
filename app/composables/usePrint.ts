import type { Component } from "vue";

export interface PrintJob {
	title: string
	component: Component
	props: Record<string, unknown>
	/** Dipanggil setelah dialog cetak ditutup (mis. untuk menandai sudah dicetak). */
	onPrinted?: () => void | Promise<void>
	landscape?: boolean
}

/** Pratinjau cetak global (lihat components/Print/PrintPreview.vue). */
export const usePrint = () => {
	const job = useState<PrintJob | null>("print-job", () => null);

	const open = (next: PrintJob) => {
		job.value = { ...next, component: markRaw(next.component) };
	};

	const close = () => {
		job.value = null;
	};

	return { job, open, close };
};

/** Data untuk kop & tanda tangan dokumen: pengaturan efektif + profil sekolah. */
export const useDocContext = () => {
	const { year, school } = useArkas();
	const pengaturan = useState<Pengaturan | null>("doc-pengaturan", () => null);
	const sekolah = useState<SchoolInfo | null>("doc-sekolah", () => null);

	const load = async () => {
		const [view, info] = await Promise.all([api.pengaturanGet(year.value), api.schoolInfo(year.value)]);
		pengaturan.value = view.efektif;
		sekolah.value = info;
	};

	return { pengaturan, sekolah: computed(() => sekolah.value ?? school.value), load };
};
