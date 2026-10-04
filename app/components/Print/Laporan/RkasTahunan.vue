<template>
	<PrintSheet landscape paper="A4" :margin="10">
		<div class="rkas">
			<div class="text-center">
				<p class="text-[15pt] font-bold">
					RKAS - {{ judul.toUpperCase() }}
				</p>
				<p class="text-[11pt] font-bold text-[#8a7350]">
					{{ sekolah?.nama }} (NPSN: {{ sekolah?.npsn }})
				</p>
				<p v-if="alamat">
					{{ alamat }}
				</p>
			</div>
			<div class="border-t-2 border-[#8a7350] my-2" />
			<div class="flex justify-between mb-2">
				<p><b>Sumber Dana:</b> {{ fundLabel }}</p>
				<p><b>Tahun Anggaran:</b> {{ year }}</p>
			</div>

			<p class="judul">
				A. PENERIMAAN
			</p>
			<table class="grid-t">
				<thead>
					<tr>
						<th class="w-28">
							No. Kode
						</th>
						<th>Penerimaan</th>
						<th class="w-40">
							Jumlah
						</th>
					</tr>
				</thead>
				<tbody>
					<tr v-for="p in penerimaan" :key="p.nama">
						<td class="text-center">
							{{ p.kode }}
						</td>
						<td>{{ p.nama }}</td>
						<td class="num">
							Rp {{ angka(p.jumlah) }}
						</td>
					</tr>
					<tr class="font-bold">
						<td colspan="2">
							Total Penerimaan
						</td>
						<td class="num">
							Rp {{ angka(totalBelanja) }}
						</td>
					</tr>
				</tbody>
			</table>

			<p class="judul mt-2">
				B. BELANJA
			</p>

			<!-- Triwulan / Lembar Kertas Kerja: kolom per triwulan (lembar juga volume, satuan, harga) -->
			<table v-if="kolom !== 'alokasi'" class="grid-t">
				<thead>
					<tr>
						<th rowspan="2" class="w-8">
							No. Urut
						</th>
						<th rowspan="2" class="w-24">
							Kode Rekening
						</th>
						<th rowspan="2" class="w-16">
							Kode Program
						</th>
						<th rowspan="2">
							Uraian Kegiatan
						</th>
						<th v-if="kolom === 'lembar'" rowspan="2" class="w-12">
							Volume
						</th>
						<th v-if="kolom === 'lembar'" rowspan="2" class="w-12">
							Satuan
						</th>
						<th v-if="kolom === 'lembar'" rowspan="2" class="w-20">
							Harga Satuan
						</th>
						<th rowspan="2" class="w-22">
							Jumlah
						</th>
						<th colspan="4">
							Rencana Belanja per Triwulan
						</th>
					</tr>
					<tr>
						<th v-for="r in ROMAWI" :key="r" class="w-20">
							Triwulan {{ r }}
						</th>
					</tr>
				</thead>
				<tbody>
					<tr class="font-bold bg-total">
						<td :colspan="kolom === 'lembar' ? 7 : 4">
							TOTAL BELANJA
						</td>
						<td class="num">
							Rp {{ angka(totalBelanja) }}
						</td>
						<td v-for="(v, i) in grandTriwulan" :key="i" class="num">
							Rp {{ angka(v) }}
						</td>
					</tr>
					<tr v-for="(r, i) in rows" :key="r.key" :class="r.item ? '' : 'font-bold bg-grup'">
						<td>{{ i + 1 }}.</td>
						<td>{{ r.item?.kodeRekening ?? "" }}</td>
						<td>{{ r.kodeProgram }}</td>
						<td :class="r.item ? 'pl-3' : ''">
							{{ r.uraian }}
						</td>
						<td v-if="kolom === 'lembar'" class="num">
							{{ r.item ? r.item.volume.toLocaleString("id-ID") : "" }}
						</td>
						<td v-if="kolom === 'lembar'">
							{{ r.item?.satuan ?? "" }}
						</td>
						<td v-if="kolom === 'lembar'" class="num">
							{{ r.item ? angka(r.item.hargaSatuan) : "" }}
						</td>
						<td class="num">
							Rp {{ angka(r.jumlah) }}
						</td>
						<td v-for="(v, j) in r.triwulan" :key="j" class="num">
							{{ angka(v) }}
						</td>
					</tr>
				</tbody>
			</table>

			<!-- Rincian RKAS: alokasi per sumber dana × operasi/modal -->
			<table v-else class="grid-t">
				<thead>
					<tr>
						<th rowspan="3" class="w-8">
							No. Urut
						</th>
						<th rowspan="3" class="w-24">
							Kode Rekening
						</th>
						<th rowspan="3" class="w-16">
							Kode Program
						</th>
						<th rowspan="3">
							Uraian Kegiatan
						</th>
						<th rowspan="3" class="w-20">
							Jumlah
						</th>
						<th :colspan="KOLOM.length * 2">
							Sumber Dana dan Alokasi Anggaran
						</th>
					</tr>
					<tr>
						<th v-for="k in KOLOM" :key="k.key" colspan="2">
							{{ k.label }}
						</th>
					</tr>
					<tr>
						<template v-for="k in KOLOM" :key="k.key">
							<th class="w-14">
								Belanja Operasi
							</th>
							<th class="w-14">
								Belanja Modal
							</th>
						</template>
					</tr>
				</thead>
				<tbody>
					<tr class="font-bold bg-total">
						<td colspan="4">
							TOTAL BELANJA
						</td>
						<td class="num">
							Rp {{ angka(totalBelanja) }}
						</td>
						<td v-for="(v, i) in grandAlokasi" :key="i" class="num">
							Rp {{ angka(v) }}
						</td>
					</tr>
					<tr v-for="(r, i) in rows" :key="r.key" :class="r.item ? '' : 'font-bold bg-grup'">
						<td>{{ i + 1 }}.</td>
						<td>{{ r.item?.kodeRekening ?? "" }}</td>
						<td>{{ r.kodeProgram }}</td>
						<td :class="r.item ? 'pl-3' : ''">
							{{ r.uraian }}
						</td>
						<td class="num">
							Rp {{ angka(r.jumlah) }}
						</td>
						<td v-for="(v, j) in r.alokasi" :key="j" class="num">
							{{ angka(v) }}
						</td>
					</tr>
				</tbody>
			</table>
		</div>
	</PrintSheet>
</template>

<script lang="ts" setup>
	const props = withDefaults(defineProps<{
		items: RkasItem[]
		kodeNames: Record<string, string>
		fundLabel: string
		year: number
		funds: FundSource[]
		/** Mis. "Tahunan", "Triwulan I", "Bulan Januari". */
		judul?: string
		/**
		 * alokasi: per sumber dana × operasi/modal; triwulan: jumlah per triwulan;
		 * lembar: triwulan + volume, satuan, harga satuan.
		 */
		kolom?: "alokasi" | "triwulan" | "lembar"
	}>(), { judul: "Tahunan", kolom: "alokasi" });
	const { sekolah, pengaturan } = useDocContext();
	/** Alamat lengkap dari Pengaturan (baris alamat kop), cadangan alamat ARKAS. */
	const alamat = computed(() => pengaturan.value?.kop.barisAlamat || pengaturan.value?.kop.alamat || sekolah.value?.alamat || "");

	const TITIK_AKHIR = /\.$/;
	const ROMAWI = ["I", "II", "III", "IV"];

	/** Kolom sumber dana seperti format RKAS dinas; dana lain masuk "BOSP LAINNYA". */
	const KOLOM = [
		{ key: "reguler", label: "BOSP REGULER", cocok: /reguler/i },
		{ key: "daerah", label: "BOSP DAERAH", cocok: /daerah/i },
		{ key: "kinerja", label: "AFIRMASI / KINERJA", cocok: /afirmasi|kinerja/i },
		{ key: "silpa", label: "SILPA", cocok: /silpa|sisa/i },
		{ key: "lainnya", label: "BOSP LAINNYA", cocok: /.*/ }
	];

	/** Indeks kolom alokasi: (sumber dana × 2) + (0 operasi / 1 modal). */
	const kolomOf = (it: RkasItem) => {
		const fund = KOLOM.findIndex((k) => k.cocok.test(it.fundName));
		const modal = (it.kodeRekening ?? "").startsWith("5.2") ? 1 : 0;
		return fund * 2 + modal;
	};
	const kosong = (n: number) => Array.from<number>({ length: n }).fill(0);
	const perTriwulan = (bulan: number[]) => ROMAWI.map((_, k) => bulan.slice(k * 3, k * 3 + 3).reduce((s, v) => s + v, 0));

	const prefixes = (kode: string) => {
		const parts = kode.split(".").filter(Boolean);
		return parts.map((_, n) => `${parts.slice(0, n + 1).join(".")}.`);
	};

	interface Baris { key: string, kodeProgram: string, uraian: string, jumlah: number, alokasi: number[], triwulan: number[], item?: RkasItem }

	const rows = computed<Baris[]>(() => {
		const items = [...props.items].sort((a, b) =>
			(a.kodeKegiatan ?? "").localeCompare(b.kodeKegiatan ?? "") || (a.kodeRekening ?? "").localeCompare(b.kodeRekening ?? ""));
		const out: Baris[] = [];
		const grup = new Map<string, Baris>();
		for (const it of items) {
			const col = kolomOf(it);
			const tw = perTriwulan(it.bulan);
			const codes = it.kodeKegiatan ? prefixes(it.kodeKegiatan) : ["-"];
			codes.forEach((code, level) => {
				let g = grup.get(code);
				if (!g) {
					const nama = props.kodeNames[code] ?? (level === 1 ? `Kegiatan ${code.replace(TITIK_AKHIR, "")}` : code === "-" ? "Tanpa kegiatan" : "");
					g = { key: `k${code}`, kodeProgram: code, uraian: nama, jumlah: 0, alokasi: kosong(KOLOM.length * 2), triwulan: kosong(4) };
					grup.set(code, g);
					out.push(g);
				}
				g.jumlah += it.jumlah;
				g.alokasi[col]! += it.jumlah;
				tw.forEach((v, k) => (g!.triwulan[k]! += v));
			});
			const alokasi = kosong(KOLOM.length * 2);
			alokasi[col] = it.jumlah;
			out.push({ key: `i${it.idRapbs}`, kodeProgram: it.kodeKegiatan ?? "", uraian: it.uraian, jumlah: it.jumlah, alokasi, triwulan: tw, item: it });
		}
		return out;
	});

	const totalBelanja = computed(() => props.items.reduce((s, i) => s + i.jumlah, 0));
	const grandAlokasi = computed(() => {
		const t = kosong(KOLOM.length * 2);
		for (const it of props.items) t[kolomOf(it)]! += it.jumlah;
		return t;
	});
	const grandTriwulan = computed(() => {
		const t = kosong(4);
		for (const it of props.items) perTriwulan(it.bulan).forEach((v, k) => (t[k]! += v));
		return t;
	});
	const penerimaan = computed(() => {
		const m = new Map<string, number>();
		for (const it of props.items) m.set(it.fundName, (m.get(it.fundName) ?? 0) + it.jumlah);
		return Array.from(m, ([nama, jumlah]) => ({ nama, jumlah, kode: props.funds.find((f) => f.name === nama)?.kode ?? "" }));
	});
</script>

<style scoped>
	.rkas {
		font-family: Arial, Helvetica, sans-serif;
		font-size: 8pt;
		color: #000;
		-webkit-print-color-adjust: exact;
		print-color-adjust: exact;
	}
	.judul {
		font-weight: bold;
		font-size: 10pt;
	}
	.grid-t {
		width: 100%;
		border-collapse: collapse;
		line-height: 1.25;
	}
	.grid-t th,
	.grid-t td {
		border: 1px solid #000;
		padding: 2px 4px;
		vertical-align: top;
	}
	.grid-t th {
		text-align: center;
		vertical-align: middle;
		font-weight: bold;
	}
	.grid-t tr {
		break-inside: avoid;
	}
	.num {
		text-align: right;
		white-space: nowrap;
		font-variant-numeric: tabular-nums;
	}
	.bg-total {
		background: #fde8e8;
	}
	.bg-grup {
		background: #e9f7ef;
	}
</style>
