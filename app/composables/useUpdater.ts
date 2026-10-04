import type { Update } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";
import { check } from "@tauri-apps/plugin-updater";

/** Status pembaruan aplikasi, dibagi antara notifikasi dan halaman Tentang. */
const update = shallowRef<Update | null>(null);
const open = ref(false);
const checking = ref(false);
const installing = ref(false);
const progress = ref(0);
const error = ref("");

/** Cek & pasang pembaruan dari GitHub Releases (file ditandatangani, diverifikasi oleh updater). */
export const useUpdater = () => {
	/** `silent`: tanpa pesan bila offline/gagal/tidak ada pembaruan (dipakai saat aplikasi dibuka). */
	const cek = async (silent = false) => {
		if (!inTauri()) return false;
		if (!navigator.onLine) {
			if (!silent) error.value = "Tidak ada koneksi internet.";
			return false;
		}
		checking.value = true;
		error.value = "";
		try {
			const found = await check();
			update.value = found;
			if (found) open.value = true;
			return !!found;
		} catch (err) {
			if (!silent) error.value = errorMessage(err);
			return false;
		} finally {
			checking.value = false;
		}
	};

	const pasang = async () => {
		const u = update.value;
		if (!u) return;
		installing.value = true;
		progress.value = 0;
		error.value = "";
		let total = 0;
		let done = 0;
		try {
			await u.downloadAndInstall((ev) => {
				if (ev.event === "Started") total = ev.data.contentLength ?? 0;
				if (ev.event === "Progress") {
					done += ev.data.chunkLength;
					progress.value = total ? Math.min(100, Math.round((done / total) * 100)) : 0;
				}
				if (ev.event === "Finished") progress.value = 100;
			});
			// Windows: installer berjalan lalu aplikasi ditutup; di sistem lain buka ulang aplikasi.
			await relaunch();
		} catch (err) {
			error.value = errorMessage(err);
			installing.value = false;
		}
	};

	const nanti = () => {
		open.value = false;
	};

	return { update, open, checking, installing, progress, error, cek, pasang, nanti };
};
