// Test fungsi yang menentukan angka dan tanggal di dokumen SPJ (kwitansi, laporan, nama file).
// Jalankan: bun test

import { describe, expect, test } from "bun:test";
import { akhirBulan, angka, fileSafe, labelPeriode, rupiah, slugify, tanggalId, tanggalPanjang, teksCss } from "../app/utils/format";
import { terbilang, terbilangRupiah } from "../app/utils/terbilang";

describe("terbilang", () => {
	test.each([
		[0, ""],
		[1, "satu"],
		[10, "sepuluh"],
		[11, "sebelas"],
		[12, "dua belas"],
		[19, "sembilan belas"],
		[20, "dua puluh"],
		[21, "dua puluh satu"],
		[100, "seratus"],
		[101, "seratus satu"],
		[111, "seratus sebelas"],
		[200, "dua ratus"],
		[999, "sembilan ratus sembilan puluh sembilan"],
		[1000, "seribu"],
		[1001, "seribu satu"],
		[1500, "seribu lima ratus"],
		[2000, "dua ribu"],
		[11000, "sebelas ribu"],
		[100000, "seratus ribu"],
		[110000, "seratus sepuluh ribu"],
		[1000000, "satu juta"],
		[1000001, "satu juta satu"],
		[2750000, "dua juta tujuh ratus lima puluh ribu"],
		[1000000000, "satu miliar"],
		[1250000000000, "satu triliun dua ratus lima puluh miliar"]
	])("%d -> %s", (n, expected) => {
		expect(terbilang(n)).toBe(expected);
	});

	test("pecahan dibuang, bukan dibulatkan", () => {
		expect(terbilang(1999.99)).toBe("seribu sembilan ratus sembilan puluh sembilan");
	});

	test("bilangan negatif", () => {
		expect(terbilang(-1500)).toBe("minus seribu lima ratus");
	});

	test("tidak ada spasi ganda atau spasi di ujung", () => {
		for (const n of [10, 100, 1000, 10000, 100000, 1000000, 10000000, 100000000, 1000000000]) {
			const text = terbilang(n);
			expect(text).toBe(text.trim());
			expect(text).not.toContain("  ");
		}
	});
});

describe("terbilangRupiah", () => {
	test("huruf pertama kapital dan diakhiri rupiah", () => {
		expect(terbilangRupiah(2750000)).toBe("Dua juta tujuh ratus lima puluh ribu rupiah");
	});

	test("nol", () => {
		expect(terbilangRupiah(0)).toBe("Nol rupiah");
	});
});

describe("format angka", () => {
	test("angka memakai titik ribuan tanpa desimal", () => {
		expect(angka(1234567)).toBe("1.234.567");
		expect(angka(0)).toBe("0");
		expect(angka(999.6)).toBe("1.000");
	});

	test("rupiah positif dan negatif", () => {
		expect(rupiah(1500)).toBe("Rp 1.500");
		expect(rupiah(-1500)).toBe("-Rp 1.500");
	});
});

describe("format tanggal", () => {
	test("tanggalId", () => {
		expect(tanggalId("2026-09-04")).toBe("04-09-2026");
		expect(tanggalId("bukan tanggal")).toBe("bukan tanggal");
	});

	test("tanggalPanjang", () => {
		expect(tanggalPanjang("2026-09-04")).toBe("4 September 2026");
		expect(tanggalPanjang("2026-12-31 10:00:00")).toBe("31 Desember 2026");
		expect(tanggalPanjang(null)).toBe("");
		expect(tanggalPanjang("")).toBe("");
	});

	test("akhirBulan memperhitungkan tahun kabisat", () => {
		expect(akhirBulan(2024, 2)).toBe("2024-02-29");
		expect(akhirBulan(2026, 2)).toBe("2026-02-28");
		expect(akhirBulan(2026, 12)).toBe("2026-12-31");
	});

	test("labelPeriode", () => {
		expect(labelPeriode(2026, 3, 3)).toBe("Maret 2026");
		expect(labelPeriode(2026, 1, 6)).toBe("Januari - Juni 2026");
	});
});

describe("teks aman", () => {
	test("fileSafe membuang karakter terlarang Windows", () => {
		expect(fileSafe("  BKU Januari: SD/1? \"x\"  ")).toBe("BKU_Januari_SD1_x");
	});

	test("slugify", () => {
		expect(slugify("Buku Kas Umum")).toBe("buku-kas-umum");
	});

	test("teksCss membuang kutip dan backslash", () => {
		expect(teksCss("Judul \"A\"\\B\n  C")).toBe("\"Judul AB C\"");
	});
});
