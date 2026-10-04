<template>
	<PrintSheet paper="A4" :margin="15">
		<div class="lkk">
			<div class="text-center font-bold leading-tight mb-4">
				<p class="text-[13pt]">
					LEMBAR KERTAS KERJA
				</p>
				<p>UNIT KERJA</p>
				<p>{{ pemerintah }}</p>
				<p>TAHUN ANGGARAN {{ year }}</p>
			</div>

			<PrintFields
				class="mb-4"
				label-width="9.5rem"
				:rows="[
					['Urusan Pemerintahan', '1.01 - PENDIDIKAN'],
					['Organisasi', `${sekolah?.npsn ?? ''} - ${sekolah?.nama ?? ''}`]
				]"
			/>

			<p class="text-center font-bold mb-1">
				Rincian Anggaran Pendapatan dan Belanja Unit Kerja
			</p>
			<table class="grid-t mb-5">
				<thead>
					<tr>
						<th class="w-36">
							Kode Rekening
						</th>
						<th>Uraian</th>
						<th class="w-36">
							Jumlah (Rp)
						</th>
					</tr>
				</thead>
				<tbody>
					<tr class="font-bold">
						<td />
						<td>JUMLAH PENDAPATAN</td>
						<td />
					</tr>
					<tr v-for="r in rincian" :key="r.kode" :class="r.kode === '5' ? 'font-bold' : ''">
						<td>{{ r.kode }}</td>
						<td :style="{ paddingLeft: `${4 + Math.max(0, r.level - 2) * 12}px` }">
							{{ r.nama }}
						</td>
						<td class="num">
							{{ angka(r.jumlah) }}
						</td>
					</tr>
					<tr class="font-bold">
						<td />
						<td>Jumlah BELANJA</td>
						<td class="num">
							{{ angka(total) }}
						</td>
					</tr>
				</tbody>
			</table>

			<p class="text-center font-bold mb-1">
				Rencana Pelaksanaan Anggaran Unit Kerja per Triwulan
			</p>
			<table class="grid-t">
				<thead>
					<tr>
						<th class="w-10">
							No
						</th>
						<th>Uraian</th>
						<th v-for="r in ROMAWI" :key="r" class="w-24">
							TW {{ r }}
						</th>
						<th class="w-26">
							Jumlah
						</th>
					</tr>
				</thead>
				<tbody>
					<tr v-for="b in triwulan" :key="b.no">
						<td>{{ b.no }}</td>
						<td>{{ b.uraian }}</td>
						<td v-for="(v, i) in b.tw" :key="i" class="num">
							{{ angka(v) }}
						</td>
						<td class="num font-bold">
							{{ angka(b.tw.reduce((s, v) => s + v, 0)) }}
						</td>
					</tr>
				</tbody>
			</table>
		</div>
	</PrintSheet>
</template>

<script lang="ts" setup>
	/** Lembar Kertas Kerja Unit Kerja: ringkasan belanja per kode rekening induk dan rencana per triwulan. */
	const props = defineProps<{ items: RkasItem[], year: number }>();
	const { sekolah, pengaturan } = useDocContext();

	const ROMAWI = ["I", "II", "III", "IV"];

	const pemerintah = computed(() => pengaturan.value?.kop.pemerintah || `PEMERINTAH ${sekolah.value?.kabupaten ?? ""}`.toUpperCase());

	const total = computed(() => props.items.reduce((s, i) => s + i.jumlah, 0));

	/** Kode induk rekening: "5.1.02.01.01.0034" -> 5, 5.1, 5.1.02, 5.1.02.01. */
	const induk = (kode: string) => {
		const p = kode.split(".").filter(Boolean);
		return [1, 2, 3, 4].filter((n) => p.length >= n).map((n) => p.slice(0, n).join("."));
	};

	const rincian = computed(() => {
		const jumlah = new Map<string, number>();
		for (const it of props.items) {
			for (const k of induk(it.kodeRekening ?? "")) jumlah.set(k, (jumlah.get(k) ?? 0) + it.jumlah);
		}
		// Baris tetap seperti format dinas, ditambah rekening induk lain yang punya anggaran.
		const kode = new Set([...REKENING_INDUK_TETAP, ...jumlah.keys()]);
		return [...kode]
			.filter((k) => k.startsWith("5"))
			.sort((a, b) => a.localeCompare(b, undefined, { numeric: true }))
			.map((k) => ({ kode: k, level: k.split(".").length, nama: NAMA_REKENING_INDUK[k] ?? k, jumlah: jumlah.get(k) ?? 0 }));
	});

	const perTw = (filter: (it: RkasItem) => boolean) => ROMAWI.map((_, k) =>
		props.items.filter(filter).reduce((s, it) => s + it.bulan.slice(k * 3, k * 3 + 3).reduce((a, v) => a + v, 0), 0));

	const triwulan = computed(() => [
		{ no: "1", uraian: "Pendapatan", tw: perTw(() => true) },
		{ no: "2.1", uraian: "Belanja Operasi", tw: perTw((it) => !(it.kodeRekening ?? "").startsWith("5.2")) },
		{ no: "2.2", uraian: "Belanja Modal", tw: perTw((it) => (it.kodeRekening ?? "").startsWith("5.2")) }
	]);
</script>

<style scoped>
	.lkk {
		font-family: Arial, Helvetica, sans-serif;
		font-size: 9.5pt;
		color: #000;
	}
	.grid-t {
		width: 100%;
		border-collapse: collapse;
		line-height: 1.3;
	}
	.grid-t th,
	.grid-t td {
		border: 1px solid #000;
		padding: 3px 6px;
	}
	.grid-t th {
		text-align: center;
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
</style>
