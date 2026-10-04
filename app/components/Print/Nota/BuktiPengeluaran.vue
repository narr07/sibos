<template>
	<PrintSheet v-for="g in groups" :key="g.key">
		<div class="doc">
			<PrintKopSurat />
			<p class="text-center text-[13pt] font-bold underline">
				BUKTI PENGELUARAN
			</p>
			<p class="text-center mb-4">
				Nomor: {{ g.noBukti || "-" }}
			</p>

			<PrintFields class="w-full mb-3" hide-empty :rows="fields(g)" />

			<table class="tabel table-fixed">
				<colgroup>
					<col style="width: 6%">
					<col>
					<col style="width: 9%">
					<col style="width: 11%">
					<col style="width: 15%">
					<col style="width: 17%">
				</colgroup>
				<thead>
					<tr>
						<th>No</th>
						<th>Uraian</th>
						<th>Volume</th>
						<th>Satuan</th>
						<th>Harga Satuan</th>
						<th>Jumlah (Rp)</th>
					</tr>
				</thead>
				<tbody>
					<tr v-for="(it, i) in g.items" :key="it.id">
						<td class="text-center">
							{{ i + 1 }}
						</td>
						<td>{{ it.uraian }}</td>
						<td class="num">
							{{ it.volume ?? "" }}
						</td>
						<td>{{ it.satuan ?? "" }}</td>
						<td class="num">
							{{ it.hargaSatuan ? angka(it.hargaSatuan) : "" }}
						</td>
						<td class="num">
							{{ angka(it.nominal) }}
						</td>
					</tr>
					<tr>
						<td colspan="5" class="text-right font-bold">
							Jumlah belanja (termasuk pajak)
						</td>
						<td class="num font-bold">
							{{ angka(g.total) }}
						</td>
					</tr>
					<tr v-for="t in g.taxes" :key="t.id">
						<td colspan="5" class="text-right">
							Pajak {{ t.jenis || "" }}
						</td>
						<td class="num">
							{{ angka(t.nominal) }}
						</td>
					</tr>
					<tr v-if="!g.taxes.length">
						<td colspan="5" class="text-right">
							Pajak
						</td>
						<td class="text-center">
							-
						</td>
					</tr>
					<tr>
						<td colspan="5" class="text-right font-bold">
							Dibayarkan kepada penyedia
						</td>
						<td class="num font-bold">
							{{ angka(g.diterima) }}
						</td>
					</tr>
				</tbody>
			</table>

			<p class="mt-3">
				Terbilang: <i>{{ terbilangRupiah(g.total) }}</i>
			</p>

			<PrintTandaTangan :signers="signers(g)" space="24mm" />
		</div>
	</PrintSheet>
</template>

<script lang="ts" setup>
	defineProps<{ groups: NotaGroup[] }>();
	const { pengaturan } = useDocContext();

	const bayar = (g: NotaGroup) => g.override?.tanggalBayar || g.tanggal;
	const keperluan = (g: NotaGroup) => g.override?.keperluan || g.override?.uraian || g.uraian;

	const fields = (g: NotaGroup): [string, string | null | undefined][] => {
		const tglNota = g.override?.tanggalNota || g.nota?.tanggalNota;
		return [
			["Tanggal", tanggalPanjang(bayar(g))],
			["Sumber dana", g.fundName],
			["Dibayarkan kepada", g.nota?.namaToko || "-"],
			["Alamat", g.nota?.alamatToko],
			["NPWP", g.nota?.npwp],
			["No. nota / faktur", g.nota?.noNota ? `${g.nota.noNota}${tglNota ? `, tanggal ${tanggalPanjang(tglNota)}` : ""}` : null],
			["Untuk keperluan", keperluan(g)]
		];
	};

	const signers = (g: NotaGroup): Penandatangan[] => {
		const p = pengaturan.value?.pejabat;
		const kota = pengaturan.value?.cetak.kota;
		return [
			{ atas: "Menyetujui,", jabatan: "Kepala Sekolah", nama: p?.kepalaSekolah, nip: p?.nipKepalaSekolah },
			{ atas: [kota, tanggalPanjang(bayar(g))].filter(Boolean).join(", "), jabatan: "Bendahara", nama: p?.bendahara, nip: p?.nipBendahara },
			{ atas: "Yang menerima,", jabatan: g.nota?.isBadanUsaha ? "Penyedia" : "Penerima", nama: g.nota?.namaToko ?? "" }
		];
	};
</script>
