import type { SiplahSyncProgress, SiplahSyncResult } from "~/utils/api";
import { isTauri } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

/**
 * Sinkronisasi katalog toko SIPLah dengan progres per halaman dan tombol batal.
 * Dipakai halaman Katalog SIPLah dan Belanja. `onDone` dipanggil setelah sinkron berhasil
 * (mis. untuk memuat ulang daftar toko dan pencocokan harga).
 */
export const useSiplahSync = (onDone: (res: SiplahSyncResult) => Promise<void> | void) => {
	const toast = useToast();
	const syncing = ref(false);
	const cancelling = ref(false);
	const progress = ref<SiplahSyncProgress | null>(null);

	/** Teks tombol saat sinkron berjalan, mis. "Halaman 3 · 90 produk". */
	const progressLabel = computed(() => (progress.value ? `Halaman ${progress.value.halaman} · ${progress.value.produk} produk` : ""));

	const sinkronkan = async (tokoId: number) => {
		syncing.value = true;
		cancelling.value = false;
		progress.value = null;
		const unlisten = isTauri()
			? await listen<SiplahSyncProgress>("siplah-sync-progress", (e) => {
				if (e.payload.tokoId === tokoId) progress.value = e.payload;
			})
			: null;
		try {
			const res = await api.siplahSyncProducts(tokoId);
			toast.add({ title: "Sinkronisasi berhasil", description: `Berhasil mengambil ${res.total} produk dari SIPLah.`, color: "success" });
			await onDone(res);
		} catch (err) {
			if (cancelling.value) {
				toast.add({ title: "Sinkronisasi dibatalkan", description: "Katalog toko tidak berubah.", color: "neutral" });
			} else {
				toast.add({ title: "Sinkronisasi gagal", description: errorMessage(err), color: "error" });
			}
		} finally {
			unlisten?.();
			syncing.value = false;
			cancelling.value = false;
			progress.value = null;
		}
	};

	/** Berhenti setelah halaman yang sedang diambil selesai (paling lama sekitar 30 detik). */
	const batalkan = async () => {
		cancelling.value = true;
		try {
			await api.siplahSyncCancel();
		} catch (err) {
			cancelling.value = false;
			toast.add({ title: "Gagal membatalkan", description: errorMessage(err), color: "error" });
		}
	};

	return { syncing, cancelling, progress, progressLabel, sinkronkan, batalkan };
};
