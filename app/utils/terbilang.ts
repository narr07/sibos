// Terbilang rupiah dalam bahasa Indonesia.

const SATUAN = ["", "satu", "dua", "tiga", "empat", "lima", "enam", "tujuh", "delapan", "sembilan", "sepuluh", "sebelas"];

const BESAR: [number, string][] = [
	[1e12, "triliun"],
	[1e9, "miliar"],
	[1e6, "juta"],
	[1e3, "ribu"]
];

export const terbilang = (input: number): string => {
	const n = Math.trunc(input);
	if (n < 0) return `minus ${terbilang(-n)}`;
	if (n < 12) return SATUAN[n] ?? "";
	if (n < 20) return `${terbilang(n - 10)} belas`;
	if (n < 100) return `${terbilang(Math.floor(n / 10))} puluh ${terbilang(n % 10)}`.trim();
	if (n < 200) return `seratus ${terbilang(n - 100)}`.trim();
	if (n < 1000) return `${terbilang(Math.floor(n / 100))} ratus ${terbilang(n % 100)}`.trim();
	if (n < 2000) return `seribu ${terbilang(n - 1000)}`.trim();
	for (const [div, word] of BESAR) {
		if (n >= div) return `${terbilang(Math.floor(n / div))} ${word} ${terbilang(n % div)}`.trim();
	}
	return "";
};

/** 2750000 -> "Dua juta tujuh ratus lima puluh ribu rupiah". */
export const terbilangRupiah = (n: number): string => {
	const text = `${n === 0 ? "nol" : terbilang(n)} rupiah`;
	return text.charAt(0).toUpperCase() + text.slice(1);
};
