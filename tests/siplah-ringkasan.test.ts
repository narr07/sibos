// Test ringkasan pencocokan harga SIPLah (harus sama dengan match_totals di Rust).

import type { SiplahMatchItem, SiplahProduk } from "../app/utils/api";
import { describe, expect, test } from "bun:test";
import { estimasiToko, pasangProduk, ringkasanSiplah } from "../app/utils/siplahRingkasan";

const produk = (harga: number): SiplahProduk => ({ id: harga, tokoId: 1, nama: `Produk ${harga}`, harga, stok: 0, terjual: 0, gambar: null, kenaPajak: true });

const item = (hargaRkas: number, volume: number, cocok: number | null): SiplahMatchItem => ({
	sourceId: `i-${hargaRkas}-${volume}`,
	namaRkas: "Barang",
	satuan: "pcs",
	volume,
	hargaRkas,
	totalRkas: Math.round(hargaRkas * volume),
	realisasiRkas: 0,
	statusBelanja: "belum",
	kodeRekening: null,
	sumberDana: null,
	matchedProduct: cocok === null ? null : produk(cocok),
	score: 0,
	status: cocok === null ? "tidak_cocok" : cocok < hargaRkas ? "lebih_murah" : cocok > hargaRkas ? "lebih_mahal" : "sesuai",
	selisih: cocok === null ? 0 : cocok - hargaRkas,
	persenSelisih: 0,
	candidates: []
});

describe("pasangProduk", () => {
	test("menentukan selisih, persen, dan status", () => {
		const it = item(10000, 3, null);
		pasangProduk(it, produk(8000));
		expect(it.selisih).toBe(-2000);
		expect(it.persenSelisih).toBe(-20);
		expect(it.status).toBe("lebih_murah");
		pasangProduk(it, produk(12000));
		expect(it.status).toBe("lebih_mahal");
		pasangProduk(it, produk(10000));
		expect(it.status).toBe("sesuai");
	});

	test("harga draft 0 tidak membagi dengan nol", () => {
		const it = item(0, 1, null);
		pasangProduk(it, produk(5000));
		expect(it.persenSelisih).toBe(0);
	});
});

describe("ringkasanSiplah", () => {
	test("hemat hanya membandingkan item yang cocok; perkiraan memakai pagu untuk yang tidak cocok", () => {
		const s = ringkasanSiplah([item(10000, 10, 9000), item(5000, 10, null), item(2000, 2, 2500)]);
		expect(s.totalAnggaranRkas).toBe(100000 + 50000 + 4000);
		expect(s.totalAnggaranCocok).toBe(104000);
		expect(s.totalEstimasiToko).toBe(90000 + 5000);
		expect(s.totalHemat).toBe(9000);
		expect(s.totalPerkiraan).toBe(95000 + 50000);
		expect(s.totalCocok).toBe(2);
		expect(s.totalTidakCocok).toBe(1);
		expect(s.totalLebihMurah).toBe(1);
		expect(s.totalLebihMahal).toBe(1);
	});

	test("volume pecahan tidak dibulatkan; volume 0 dianggap 1", () => {
		expect(estimasiToko(item(1000, 2.5, 1000))).toBe(2500);
		expect(estimasiToko(item(1000, 0, 700))).toBe(700);
		expect(estimasiToko(item(1000, 1, null))).toBeNull();
	});
});
