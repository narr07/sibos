// Test status & sisa belanja RKAS per bulan (dipakai Generator Standing Instruction).

import { describe, expect, test } from "bun:test";
import { belanjaBulan, labelStatusBelanja, nominalImpor, tampilDiBulan, warnaStatusBelanja } from "../app/utils/belanjaBulanan";
import { formatVolume } from "../app/utils/format";

const bulanan = (bulan: number, nilai: number) => Array.from({ length: 12 }, (_, i) => (i === bulan - 1 ? nilai : 0));

const item = (pagu: number, volume = 0) => ({
	bulan: bulanan(3, pagu),
	volumeBulan: bulanan(3, volume),
	fundId: 1,
	uraian: "Kertas HVS A4",
	kodeRekening: "5.1.02.01.01.0024",
	hargaSatuan: 50000,
	volume: 4
});

const semua = { fundId: 0, belumSaja: false, cari: "" };

describe("belanjaBulan", () => {
	test("belum dibelanjakan", () => {
		expect(belanjaBulan(item(200000), undefined, 3)).toEqual({ pagu: 200000, terpakai: 0, sisa: 200000, status: "belum" });
	});

	test("sebagian", () => {
		const b = belanjaBulan(item(200000), { realisasi: bulanan(3, 50000) }, 3);
		expect(b).toEqual({ pagu: 200000, terpakai: 50000, sisa: 150000, status: "sebagian" });
		expect(labelStatusBelanja(b)).toBe("Sebagian (Rp 50.000)");
	});

	test("lunas, dan realisasi melebihi pagu tidak membuat sisa negatif", () => {
		const b = belanjaBulan(item(200000), { realisasi: bulanan(3, 250000) }, 3);
		expect(b.status).toBe("lunas");
		expect(b.sisa).toBe(0);
	});

	test("tanpa pagu di bulan itu: status null", () => {
		const b = belanjaBulan(item(200000), undefined, 4);
		expect(b.status).toBeNull();
		expect(labelStatusBelanja(b)).toBe("");
	});

	test("warna lencana", () => {
		expect(warnaStatusBelanja("lunas")).toBe("success");
		expect(warnaStatusBelanja("sebagian")).toBe("info");
		expect(warnaStatusBelanja("belum")).toBe("warning");
	});
});

describe("tampilDiBulan", () => {
	test("item tanpa pagu dan volume di bulan itu disembunyikan", () => {
		const it = item(0, 0);
		expect(tampilDiBulan(it, belanjaBulan(it, undefined, 3), 3, semua)).toBe(false);
	});

	test("filter sumber dana", () => {
		const it = item(100);
		const b = belanjaBulan(it, undefined, 3);
		expect(tampilDiBulan(it, b, 3, { ...semua, fundId: 2 })).toBe(false);
		expect(tampilDiBulan(it, b, 3, { ...semua, fundId: 1 })).toBe(true);
	});

	test("hanya yang belum dibelanjakan", () => {
		const it = item(100);
		const lunas = belanjaBulan(it, { realisasi: bulanan(3, 100) }, 3);
		expect(tampilDiBulan(it, lunas, 3, { ...semua, belumSaja: true })).toBe(false);
		expect(tampilDiBulan(it, lunas, 3, semua)).toBe(true);
	});

	test("pencarian uraian dan kode rekening tanpa beda huruf besar/kecil", () => {
		const it = item(100);
		const b = belanjaBulan(it, undefined, 3);
		expect(tampilDiBulan(it, b, 3, { ...semua, cari: "  hvs " })).toBe(true);
		expect(tampilDiBulan(it, b, 3, { ...semua, cari: "0024" })).toBe(true);
		expect(tampilDiBulan(it, b, 3, { ...semua, cari: "tinta" })).toBe(false);
	});
});

describe("nominalImpor", () => {
	test("pakai sisa bila ada", () => {
		const it = item(200000);
		expect(nominalImpor(it, belanjaBulan(it, { realisasi: bulanan(3, 50000) }, 3), 3)).toBe(150000);
	});

	test("tidak bersisa: pakai pagu bulan", () => {
		const it = item(200000);
		expect(nominalImpor(it, belanjaBulan(it, { realisasi: bulanan(3, 200000) }, 3), 3)).toBe(200000);
	});

	test("pagu bulan kosong: harga satuan × volume", () => {
		const it = item(0, 2);
		expect(nominalImpor(it, belanjaBulan(it, undefined, 3), 3)).toBe(100000);
		// Volume bulan kosong: pakai volume tahunan.
		expect(nominalImpor(it, belanjaBulan(it, undefined, 5), 5)).toBe(200000);
	});
});

describe("formatVolume", () => {
	test("bulat apa adanya, pecahan maksimal 2 desimal", () => {
		expect(formatVolume(12)).toBe("12");
		expect(formatVolume(2.5)).toBe("2,5");
		expect(formatVolume(1.2345)).toBe("1,23");
	});
});
