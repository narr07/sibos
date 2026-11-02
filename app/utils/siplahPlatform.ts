// Mitra SIPLah yang katalognya bisa diambil Lajur, dan cara menampilkan harganya.
// Harus sejalan dengan src-tauri/src/siplah (kode platform & harga_banding).

import type { SiplahProduk } from "./api";
import { rupiah } from "./format";

export type SiplahPlatform = "tokoladang" | "gramedia" | "intan" | "temprina" | "masmedia";

export const PLATFORM_SIPLAH: Record<SiplahPlatform, { label: string, contoh: string }> = {
	tokoladang: { label: "Toko Ladang", contoh: "https://siplah.tokoladang.co.id/m/nama-toko.12345" },
	gramedia: { label: "Gramedia", contoh: "https://siplahgramedia.id/main/penyedia/..." },
	intan: { label: "Intan Pariwara", contoh: "https://siplah.intanonline.com/frontend/web/store/nama-toko" },
	temprina: { label: "Temprina", contoh: "https://siplah.temprina.co.id/store/nama-toko" },
	masmedia: { label: "Masmedia", contoh: "https://siplahmasmedia.co.id/vendor/3347_1234" }
};

/** Label platform; nilai tak dikenal (data lama) dianggap Toko Ladang. */
export const labelPlatform = (platform: string | null | undefined) =>
	PLATFORM_SIPLAH[(platform ?? "tokoladang") as SiplahPlatform]?.label ?? platform ?? "Toko Ladang";

/** Harga untuk perbandingan & perkiraan: harga zona tertinggi agar anggaran tidak kurang. */
export const hargaBanding = (p: Pick<SiplahProduk, "harga" | "hargaMax">) => Math.max(p.harga, p.hargaMax ?? p.harga);

/** "Rp 86.900 – Rp 181.100" untuk harga zona, atau satu harga. */
export const labelHarga = (p: Pick<SiplahProduk, "harga" | "hargaMax">) =>
	p.hargaMax && p.hargaMax > p.harga ? `${rupiah(p.harga)} – ${rupiah(p.hargaMax)}` : rupiah(p.harga);

/** Halaman produk di situs asal (produk Toko Ladang lama belum punya url tersimpan). */
export const urlProduk = (p: Pick<SiplahProduk, "id" | "url">) => p.url ?? `https://siplah.tokoladang.co.id/p/produk.${p.id}`;
