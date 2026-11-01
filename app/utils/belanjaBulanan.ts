// Status belanja RKAS per bulan: pagu bulan ini dibanding realisasi ARKAS di bulan yang sama.
// Fungsi murni (tanpa state Vue) supaya bisa dites dan dipakai ulang antarhalaman.

import type { RealisasiItem, RkasItem } from "./types";
import { rupiah } from "./format";

export type StatusBelanja = "belum" | "sebagian" | "lunas";

export interface BelanjaBulan {
	/** Anggaran item di bulan ini. */
	pagu: number
	/** Realisasi ARKAS item di bulan ini. */
	terpakai: number
	/** Pagu yang belum dibelanjakan (tidak pernah negatif). */
	sisa: number
	/** null bila item tidak punya pagu di bulan ini. */
	status: StatusBelanja | null
}

/** Hitung pagu, realisasi, sisa, dan status satu item RKAS di `bulan` (1-12). */
export const belanjaBulan = (
	item: Pick<RkasItem, "bulan">,
	realisasi: Pick<RealisasiItem, "realisasi"> | undefined,
	bulan: number
): BelanjaBulan => {
	const pagu = item.bulan[bulan - 1] ?? 0;
	const terpakai = realisasi?.realisasi[bulan - 1] ?? 0;
	const sisa = Math.max(0, pagu - terpakai);
	const status: StatusBelanja | null = pagu <= 0 ? null : terpakai >= pagu ? "lunas" : terpakai > 0 ? "sebagian" : "belum";
	return { pagu, terpakai, sisa, status };
};

/** Label status untuk lencana, mis. "Sebagian (Rp 50.000)". Kosong bila tidak ada pagu. */
export const labelStatusBelanja = (b: BelanjaBulan | undefined): string => {
	switch (b?.status) {
		case "lunas": return "Lunas";
		case "sebagian": return `Sebagian (${rupiah(b.terpakai)})`;
		case "belum": return "Belum Dibelanjakan";
		default: return "";
	}
};

/** Warna lencana Nuxt UI untuk status belanja. */
export const warnaStatusBelanja = (status: StatusBelanja | null | undefined) =>
	status === "lunas" ? "success" : status === "sebagian" ? "info" : "warning";

export interface FilterBelanjaBulan {
	/** 0 = semua sumber dana. */
	fundId: number
	/** Sembunyikan item yang sudah lunas atau tidak bersisa. */
	belumSaja: boolean
	/** Kata kunci uraian atau kode rekening (tanpa beda huruf besar/kecil). */
	cari: string
}

/** Apakah item ditampilkan di daftar rencana belanja bulan ini. */
export const tampilDiBulan = (
	item: Pick<RkasItem, "volumeBulan" | "fundId" | "uraian" | "kodeRekening">,
	b: BelanjaBulan,
	bulan: number,
	filter: FilterBelanjaBulan
): boolean => {
	const volume = item.volumeBulan[bulan - 1] ?? 0;
	if (b.pagu <= 0 && volume <= 0) return false;
	if (filter.fundId !== 0 && item.fundId !== filter.fundId) return false;
	if (filter.belumSaja && (b.status === "lunas" || b.sisa <= 0)) return false;
	const q = filter.cari.trim().toLowerCase();
	if (q && !item.uraian.toLowerCase().includes(q) && !(item.kodeRekening?.toLowerCase().includes(q) ?? false)) return false;
	return true;
};

/**
 * Nominal yang dimasukkan ke rincian SI: sisa bulan ini; bila tidak bersisa, pagu bulan ini;
 * bila pagu bulan kosong, harga satuan × volume (minimal 1).
 */
export const nominalImpor = (item: Pick<RkasItem, "hargaSatuan" | "volume" | "volumeBulan">, b: BelanjaBulan, bulan: number): number => {
	if (b.sisa > 0) return b.sisa;
	if (b.pagu > 0) return b.pagu;
	return item.hargaSatuan * Math.max(1, Math.round(item.volumeBulan[bulan - 1] || item.volume));
};
