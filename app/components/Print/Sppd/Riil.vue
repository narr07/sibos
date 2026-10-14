<template>
	<PrintSheet>
		<div class="doc sppd">
			<PrintKopSurat ringkas />
			<p class="mb-4 text-center text-[12pt] font-bold">
				DAFTAR PENGELUARAN RIIL
			</p>
			<p>Yang bertanda tangan di bawah ini:</p>
			<PrintFields
				class="my-2 ml-6"
				label-width="6rem"
				:rows="[['Nama', orang.nama], ['NIP', orang.nip || '-'], ['Jabatan', jabatanSppd(orang)]]"
			/>
			<p class="text-justify">
				Berdasarkan Surat Perintah Perjalanan Dinas (SPPD) Nomor: {{ d.nomorSpd || "...................." }}
				tanggal {{ tanggalPanjang(d.tanggalSurat) }}, dengan ini kami menyatakan dengan sesungguhnya bahwa:
			</p>
			<ol class="mt-2 list-decimal pl-6 text-justify">
				<li>
					Biaya di bawah ini tidak dapat diperoleh bukti-bukti pengeluarannya, meliputi:
					<table class="tabel my-2">
						<thead>
							<tr>
								<th class="w-10">
									No
								</th>
								<th>Uraian</th>
								<th class="w-44">
									Jumlah
								</th>
							</tr>
						</thead>
						<tbody>
							<tr v-for="(b, i) in riil" :key="b.key">
								<td class="text-center">
									{{ i + 1 }}
								</td>
								<td>{{ b.key === "tol" ? "Biaya E-Tol" : b.label }}</td>
								<td class="num">
									Rp {{ angka(d.biaya[b.key] || 0) }}
								</td>
							</tr>
							<tr class="font-bold">
								<td />
								<td class="text-center">
									Jumlah
								</td>
								<td class="num">
									Rp {{ angka(total) }}
								</td>
							</tr>
						</tbody>
					</table>
				</li>
				<li>
					Jumlah uang tersebut pada angka 1 di atas benar-benar dikeluarkan untuk pelaksanaan perjalanan dinas dimaksud
					dan apabila di kemudian hari terdapat kelebihan atas pembayaran, kami bersedia untuk menyetorkan kelebihan
					tersebut ke Kas Daerah.
				</li>
			</ol>
			<p class="mt-2 text-justify">
				Demikian pernyataan ini kami buat dengan sebenarnya, untuk dipergunakan sebagaimana mestinya.
			</p>

			<PrintTandaTangan
				:signers="[{ atas: tempatTanggal, jabatan: 'Yang melaksanakan perjalanan dinas', nama: orang.nama, nip: orang.nip }]"
			/>
		</div>
	</PrintSheet>
</template>

<script lang="ts" setup>
	const props = defineProps<{ d: SppdData, orang: Guru }>();
	const { pengaturan } = useDocContext();

	const riil = BIAYA_SPPD.filter((b) => b.riil);
	const total = computed(() => totalBiaya(props.d.biaya, true));
	const tempatTanggal = computed(() => [pengaturan.value?.cetak.kota, tanggalPanjang(props.d.tanggalKembali)].filter(Boolean).join(", "));
</script>

<style scoped>
	/* Format SPPD sekolah memakai Arial */
	.sppd {
		font-family: Arial, Helvetica, sans-serif;
		font-size: 11pt;
	}
	.sppd table.tabel {
		font-size: 10pt;
	}
	.sppd :deep(.ttd) {
		font-size: 11pt;
	}
</style>
