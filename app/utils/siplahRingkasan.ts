// Hitungan pencocokan harga SIPLah di sisi frontend, dipakai saat pengguna mengganti produk secara manual.
// Rumusnya HARUS sama dengan `match_totals` dan penentuan status di src-tauri/src/commands/siplah.rs.

import type { SiplahMatchItem, SiplahMatchSummary, SiplahProduk } from "./api";
import { hargaBanding } from "./siplahPlatform";

/**
 * Pasangkan `produk` ke item, lalu tentukan ulang selisih per satuan, persentase, dan status.
 * Harga zona memakai harga tertinggi (lihat `hargaBanding`).
 */
export const pasangProduk = (item: SiplahMatchItem, produk: SiplahProduk): void => {
	const selisih = hargaBanding(produk) - item.hargaRkas;
	item.matchedProduct = produk;
	item.selisih = selisih;
	item.persenSelisih = item.hargaRkas > 0 ? (selisih / item.hargaRkas) * 100 : 0;
	item.status = selisih === 0 ? "sesuai" : selisih < 0 ? "lebih_murah" : "lebih_mahal";
};

/** Perkiraan biaya satu item di toko: harga produk (zona tertinggi) × volume (volume kosong dianggap 1). */
export const estimasiToko = (item: SiplahMatchItem): number | null =>
	item.matchedProduct ? Math.round(hargaBanding(item.matchedProduct) * (item.volume > 0 ? item.volume : 1)) : null;

/**
 * Ringkasan laporan. Hemat hanya membandingkan item yang punya produk cocok; item tanpa padanan
 * tetap ikut di `totalPerkiraan` dengan pagunya sendiri.
 */
export const ringkasanSiplah = (items: SiplahMatchItem[]): SiplahMatchSummary => {
	let totalAnggaranRkas = 0;
	let totalAnggaranCocok = 0;
	let totalEstimasiToko = 0;
	let totalCocok = 0;
	for (const it of items) {
		totalAnggaranRkas += it.totalRkas;
		const estimasi = estimasiToko(it);
		if (estimasi !== null) {
			totalCocok += 1;
			totalAnggaranCocok += it.totalRkas;
			totalEstimasiToko += estimasi;
		}
	}
	const count = (status: SiplahMatchItem["status"]) => items.filter((i) => i.matchedProduct && i.status === status).length;
	return {
		totalItems: items.length,
		totalCocok,
		totalTidakCocok: items.length - totalCocok,
		totalSesuai: count("sesuai"),
		totalLebihMurah: count("lebih_murah"),
		totalLebihMahal: count("lebih_mahal"),
		totalAnggaranRkas,
		totalAnggaranCocok,
		totalEstimasiToko,
		totalHemat: totalAnggaranCocok - totalEstimasiToko,
		totalPerkiraan: totalEstimasiToko + (totalAnggaranRkas - totalAnggaranCocok)
	};
};
