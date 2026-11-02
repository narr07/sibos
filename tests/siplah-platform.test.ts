// Test tampilan harga & tautan produk dari banyak platform SIPLah.

import { describe, expect, test } from "bun:test";
import { hargaBanding, labelHarga, labelPlatform, urlProduk } from "../app/utils/siplahPlatform";

describe("harga zona", () => {
	test("perbandingan memakai harga tertinggi", () => {
		expect(hargaBanding({ harga: 86900, hargaMax: 181100 })).toBe(181100);
		expect(hargaBanding({ harga: 5000, hargaMax: null })).toBe(5000);
		// Data janggal (max di bawah min) tidak boleh menurunkan harga.
		expect(hargaBanding({ harga: 5000, hargaMax: 4000 })).toBe(5000);
	});

	test("label rentang atau satu harga", () => {
		expect(labelHarga({ harga: 86900, hargaMax: 181100 })).toBe("Rp 86.900 – Rp 181.100");
		expect(labelHarga({ harga: 5000, hargaMax: null })).toBe("Rp 5.000");
		expect(labelHarga({ harga: 5000, hargaMax: 5000 })).toBe("Rp 5.000");
	});
});

describe("platform dan tautan", () => {
	test("label platform, data lama dianggap Toko Ladang", () => {
		expect(labelPlatform("gramedia")).toBe("Gramedia");
		expect(labelPlatform("intan")).toBe("Intan Pariwara");
		expect(labelPlatform(undefined)).toBe("Toko Ladang");
	});

	test("url produk tersimpan dipakai, produk Toko Ladang lama memakai pola lama", () => {
		expect(urlProduk({ id: -5, url: "https://siplahgramedia.id/main/detail/abc" })).toBe("https://siplahgramedia.id/main/detail/abc");
		expect(urlProduk({ id: 123, url: null })).toBe("https://siplah.tokoladang.co.id/p/produk.123");
	});
});
