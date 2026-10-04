<template>
	<PrintSheet landscape :margin="12">
		<div class="doc buku">
			<p class="text-center text-[13pt] font-bold">
				{{ JUDUL[book.kind] }}
			</p>
			<p class="text-center font-bold mb-3">
				{{ book.month ? `BULAN ${BULAN[book.month - 1]!.toUpperCase()} TAHUN ${book.year}` : `TAHUN ANGGARAN ${book.year}` }}
			</p>
			<PrintFields
				class="mb-2"
				label-width="9rem"
				:rows="[
					['Nama Sekolah', sekolah?.nama],
					['NPSN', sekolah?.npsn],
					['Kecamatan', sekolah?.kecamatan],
					['Kabupaten/Kota', sekolah?.kabupaten],
					['Provinsi', sekolah?.provinsi],
					['Sumber Dana', fundLabel]
				]"
			/>

			<table class="tabel">
				<thead>
					<tr>
						<th class="w-8">
							No
						</th>
						<th class="w-20">
							Tanggal
						</th>
						<template v-if="pajak">
							<th class="w-24">
								No. Bukti
							</th>
							<th>Uraian</th>
							<th class="w-24">
								Jenis Pajak
							</th>
						</template>
						<template v-else>
							<th class="w-20">
								Kode Kegiatan
							</th>
							<th class="w-28">
								Kode Rekening
							</th>
							<th class="w-24">
								No. Bukti
							</th>
							<th>Uraian</th>
						</template>
						<th class="w-24">
							{{ pajak ? "Pungut" : "Penerimaan" }}
						</th>
						<th class="w-24">
							{{ pajak ? "Setor" : "Pengeluaran" }}
						</th>
						<th class="w-26">
							Saldo
						</th>
					</tr>
				</thead>
				<tbody>
					<tr>
						<td :colspan="pajak ? 3 : 5" />
						<td>Saldo awal</td>
						<td />
						<td />
						<td class="num">
							{{ angka(book.opening) }}
						</td>
					</tr>
					<tr v-for="(l, i) in book.lines" :key="l.id">
						<td class="text-center">
							{{ i + 1 }}
						</td>
						<td class="text-center whitespace-nowrap">
							{{ tanggalId(l.tanggal) }}
						</td>
						<template v-if="pajak">
							<td class="text-center">
								{{ l.noBukti }}
							</td>
							<td>{{ l.uraian }}</td>
							<td class="text-center">
								{{ l.jenisPajak }}
							</td>
						</template>
						<template v-else>
							<td class="text-center">
								{{ l.kodeKegiatan }}
							</td>
							<td class="text-center">
								{{ l.kodeRekening }}
							</td>
							<td class="text-center">
								{{ l.noBukti }}{{ l.siplah ? " (SIPLah)" : "" }}
							</td>
							<td>{{ l.uraian }}</td>
						</template>
						<td class="num">
							{{ l.penerimaan ? angka(l.penerimaan) : "" }}
						</td>
						<td class="num">
							{{ l.pengeluaran ? angka(l.pengeluaran) : "" }}
						</td>
						<td class="num">
							{{ angka(l.saldo) }}
						</td>
					</tr>
					<tr class="font-bold">
						<td :colspan="pajak ? 5 : 6" class="text-center">
							Jumlah
						</td>
						<td class="num">
							{{ angka(book.totalPenerimaan) }}
						</td>
						<td class="num">
							{{ angka(book.totalPengeluaran) }}
						</td>
						<td class="num">
							{{ angka(book.closing) }}
						</td>
					</tr>
				</tbody>
			</table>

			<div v-if="book.kind === 'umum'" class="mt-3 break-inside-avoid">
				<p>Pada tanggal {{ tanggalTtd }}, buku ditutup dengan posisi sebagai berikut:</p>
				<PrintFields
					class="ml-6"
					label-width="12rem"
					:rows="[
						['Saldo buku', rupiah(book.closing)],
						['Terdiri dari: saldo bank', rupiah(book.closingBank)],
						['saldo tunai', rupiah(book.closingTunai)]
					]"
				/>
			</div>

			<PrintTandaTangan :signers="signers" />
		</div>
	</PrintSheet>
</template>

<script lang="ts" setup>
	const props = defineProps<{ book: Book, fundLabel: string }>();
	const { pengaturan, sekolah } = useDocContext();

	const JUDUL: Record<BookKind, string> = {
		umum: "BUKU KAS UMUM",
		bank: "BUKU PEMBANTU BANK",
		tunai: "BUKU PEMBANTU KAS TUNAI",
		pajak: "BUKU PEMBANTU PAJAK"
	};
	const pajak = computed(() => props.book.kind === "pajak");
	/** Akhir bulan, atau 31 Desember untuk satu tahun. */
	const tanggalTtd = computed(() => tanggalPanjang(akhirBulan(props.book.year, props.book.month ?? 12)));

	const signers = computed<Penandatangan[]>(() => {
		const p = pengaturan.value?.pejabat;
		return [
			{ atas: "Menyetujui,", jabatan: "Kepala Sekolah", nama: p?.kepalaSekolah, nip: p?.nipKepalaSekolah },
			{ atas: [pengaturan.value?.cetak.kota, tanggalTtd.value].filter(Boolean).join(", "), jabatan: "Bendahara", nama: p?.bendahara, nip: p?.nipBendahara }
		];
	});
</script>

<style scoped>
	.buku :deep(table.tabel) {
		font-size: 9pt;
	}
	.buku :deep(table.tabel td),
	.buku :deep(table.tabel th) {
		padding: 2px 4px;
	}
	.buku :deep(table.tabel tr) {
		break-inside: avoid;
	}
	.buku :deep(table.tabel thead) {
		display: table-header-group;
	}
</style>
