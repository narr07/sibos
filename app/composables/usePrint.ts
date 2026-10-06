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
	/** Kertas cetak pilihan di pratinjau: A4 (210 × 297 mm) atau F4 (210 × 330 mm). */
	const kertas = useState<"A4" | "F4" | null>("print-kertas", () => null);

	const open = (next: PrintJob) => {
		// Otomatis A4 (atau F4 bila itu yang diatur di Pengaturan); pengguna bisa ganti di toolbar.
		const pengaturan = useState<Pengaturan | null>("doc-pengaturan");
		kertas.value = pengaturan.value?.cetak.kertas === "F4" ? "F4" : "A4";
		job.value = { ...next, component: markRaw(next.component) };
	};

	const close = () => {
		job.value = null;
	};

	return { job, kertas, open, close };
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
