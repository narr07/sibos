const REPO = "narr07/sibos";
export const HALAMAN_RILIS = `https://github.com/${REPO}/releases/latest`;
const AWALAN_V = /^v/;

interface Rilis {
	versi: string
	/** Tautan langsung installer .exe; kosong bila tidak ditemukan. */
	url: string
	ukuran: number
}

/** Rilis terbaru SIBOS dari GitHub Releases (dibaca di browser, sekali per kunjungan). */
export const useRilisTerbaru = () => {
	const rilis = useState<Rilis | null>("sibos-rilis", () => null);

	onMounted(async () => {
		if (rilis.value) return;
		try {
			const data = await $fetch<{ tag_name: string, assets: { name: string, browser_download_url: string, size: number }[] }>(
				`https://api.github.com/repos/${REPO}/releases/latest`
			);
			const exe = data.assets.find((a) => a.name.endsWith("-setup.exe"));
			rilis.value = { versi: data.tag_name.replace(AWALAN_V, ""), url: exe?.browser_download_url ?? "", ukuran: exe?.size ?? 0 };
		} catch {
			// Tanpa koneksi ke API GitHub: tombol tetap mengarah ke halaman rilis.
		}
	});

	return { rilis };
};
