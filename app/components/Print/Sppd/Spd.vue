<template>
	<!-- Lembar depan -->
	<PrintSheet>
		<div class="doc spd">
			<PrintKopSurat ringkas />
			<div class="nomor">
				<PrintFields label-width="5.5rem" :rows="[['Lembar Ke', ''], ['Kode No.', ''], ['Nomor', d.nomorSpd]]" />
			</div>
			<p class="judul">
				SURAT PERJALANAN DINAS (SPD)
			</p>

			<table class="kotak depan">
				<colgroup>
					<col class="w-[7mm]">
					<col class="w-[44%]">
					<col>
					<col>
				</colgroup>
				<tbody>
					<tr>
						<td>1</td>
						<td>Pejabat Pembuat Komitmen</td>
						<td colspan="2">
							{{ kepala?.kepalaSekolah }}
						</td>
					</tr>
					<tr>
						<td>2</td>
						<td>Nama/NIP Pegawai yang melaksanakan Perjalanan Dinas</td>
						<td colspan="2">
							<p>{{ orang.nama }}</p>
							<p>NIP. {{ orang.nip || "-" }}</p>
						</td>
					</tr>
					<tr>
						<td>3</td>
						<td>
							<p>a. Pangkat dan golongan</p>
							<p>b. Jabatan/Instansi</p>
							<p>c. Tingkat Biaya Perjalanan Dinas</p>
						</td>
						<td colspan="2">
							<p>a. {{ orang.pangkat || "-" }}</p>
							<p>b. {{ jabatanSppd(orang) }} / {{ d.instansi }}</p>
							<p>c. {{ d.tingkatBiaya }}</p>
						</td>
					</tr>
					<tr>
						<td>4</td>
						<td class="h-[11mm]">
							Maksud Perjalanan Dinas
						</td>
						<td colspan="2">
							{{ d.maksud }}
						</td>
					</tr>
					<tr>
						<td>5</td>
						<td>Alat angkutan yang dipergunakan</td>
						<td colspan="2">
							{{ d.angkutan }}
						</td>
					</tr>
					<tr>
						<td>6</td>
						<td>
							<p>a. Tempat berangkat</p>
							<p>b. Tempat tujuan</p>
						</td>
						<td colspan="2">
							<p>a. {{ d.berangkatDari }}</p>
							<p>b. {{ d.tujuan }}</p>
						</td>
					</tr>
					<tr>
						<td>7</td>
						<td>
							<p>a. Lama Perjalanan Dinas</p>
							<p>b. Tanggal Berangkat</p>
							<p>c. Tanggal harus kembali/tiba di tempat baru*)</p>
						</td>
						<td colspan="2">
							<p>a. {{ lamaTeks(d) }}</p>
							<p>b. {{ tanggalPanjang(d.tanggalBerangkat) }}</p>
							<p>c. {{ tanggalPanjang(d.tanggalKembali) }}</p>
						</td>
					</tr>
					<tr>
						<td>8</td>
						<td>Pengikut : Nama</td>
						<td class="text-center">
							Tanggal Lahir
						</td>
						<td class="text-center">
							Keterangan
						</td>
					</tr>
					<tr>
						<td />
						<td>
							<p>1.</p>
							<p>2.</p>
							<p>3.</p>
						</td>
						<td />
						<td />
					</tr>
					<tr>
						<td>9</td>
						<td>
							<p>Pembebanan Anggaran</p>
							<p>a. Instansi</p>
							<p>b. Akun</p>
						</td>
						<td colspan="2">
							<p>&nbsp;</p>
							<p>a. {{ d.instansi }}</p>
							<p>b. {{ d.kodeRekening }}</p>
						</td>
					</tr>
					<tr>
						<td>10</td>
						<td>Keterangan lain-lain</td>
						<td colspan="2" />
					</tr>
				</tbody>
			</table>
			<p class="ml-[10mm]">
				* Coret yang tidak perlu
			</p>

			<div class="ppk mt-6">
				<PrintFields
					class="border-b border-black w-full"
					label-width="6.5rem"
					:rows="[['Dikeluarkan di', kota], ['Tanggal', tanggalPanjang(d.tanggalSurat)]]"
				/>
				<p>Pejabat Pembuat Komitmen,</p>
				<p class="mt-[20mm] font-bold">
					{{ kepala?.kepalaSekolah }}
				</p>
				<p>NIP. {{ kepala?.nipKepalaSekolah }}</p>
			</div>
		</div>
	</PrintSheet>

	<!-- Lembar belakang: pengesahan berangkat / tiba -->
	<PrintSheet>
		<div class="doc spd">
			<table class="kotak belakang">
				<colgroup>
					<col class="w-1/2">
					<col class="w-1/2">
				</colgroup>
				<tbody>
					<tr v-for="(pasang, i) in pengesahan" :key="i">
						<td v-for="(sel, j) in pasang" :key="j">
							<div v-if="sel" class="sel">
								<span class="romawi">{{ sel.romawi }}</span>
								<div class="flex flex-1 flex-col">
									<PrintFields label-width="8.5rem" :rows="sel.rows" />
									<p>{{ sel.kepala }}</p>
									<p class="mt-auto pt-[18mm]">
										{{ sel.nama || "(..........................................)" }}
									</p>
									<p>NIP. {{ sel.nip }}</p>
								</div>
							</div>
							<p v-else-if="i === pengesahan.length - 1" class="text-justify">
								Telah diperiksa dengan keterangan bahwa perjalanan dinas tersebut atas perintahnya dan semata-mata untuk
								kepentingan jabatan dalam waktu yang sesingkat-singkatnya.
							</p>
						</td>
					</tr>
					<tr>
						<td class="text-justify" colspan="2">
							VI. Catatan Lain-lain
						</td>
					</tr>
					<tr>
						<td class="text-justify" colspan="2">
							<p>VII. PERHATIAN:</p>
							<p class="pl-[7mm]">
								PPK yang menerbitkan SPD, pegawai yang melakukan perjalanan dinas, para pejabat yang mengesahkan tanggal
								berangkat/tiba, serta bendahara bertanggung jawab berdasarkan peraturan-peraturan Keuangan Negara apabila
								negara menderita rugi akibat kesalahan, kelalaian, dan kealpaannya.
							</p>
						</td>
					</tr>
				</tbody>
			</table>

			<div class="ppk mt-6">
				<p>Pejabat Pembuat Komitmen</p>
				<p class="mt-[20mm]">
					{{ kepala?.kepalaSekolah }}
				</p>
				<p>NIP. {{ kepala?.nipKepalaSekolah }}</p>
			</div>
		</div>
	</PrintSheet>
</template>

<script lang="ts" setup>
	const props = defineProps<{ d: SppdData, orang: Guru }>();
	const { pengaturan } = useDocContext();

	const kepala = computed(() => pengaturan.value?.pejabat);
	const kota = computed(() => pengaturan.value?.cetak.kota ?? "");

	interface Sel { romawi: string, rows: [string, string][], kepala: string, nama?: string, nip?: string }

	const tiba = (romawi: string, tempat = "", tanggal = ""): Sel =>
		({ romawi, rows: [["Tiba di", tempat], ["Pada Tanggal", tanggal]], kepala: "Kepala" });
	const berangkat = (romawi: string, dari = "", ke = "", tanggal = ""): Sel =>
		({ romawi, rows: [["Berangkat dari\n(Tempat Kedudukan)", dari], ["Ke", ke], ["Pada Tanggal", tanggal]], kepala: "Kepala" });

	/** Lembar belakang I-V: [kiri, kanan]; kanan baris V berisi keterangan pemeriksaan (sel null = teks). */
	const pengesahan = computed<[Sel | null, Sel | null][]>(() => {
		const d = props.d;
		const ks = { kepala: "Kepala Sekolah", nama: kepala.value?.kepalaSekolah, nip: kepala.value?.nipKepalaSekolah };
		return [
			[null, { ...berangkat("I.", d.berangkatDari, d.tujuan, tanggalPanjang(d.tanggalBerangkat)), ...ks }],
			[tiba("II.", d.tujuan, tanggalPanjang(d.tanggalBerangkat)), berangkat("", d.tujuan, d.berangkatDari, tanggalPanjang(d.tanggalKembali))],
			[tiba("III."), berangkat("")],
			[tiba("IV."), berangkat("")],
			[{ ...tiba("V.", d.berangkatDari, tanggalPanjang(d.tanggalKembali)), ...ks }, null]
		];
	});
</script>

<style scoped>
	.spd {
		font-family: Arial, Helvetica, sans-serif;
		font-size: 10pt;
		line-height: 1.3;
	}
	.nomor {
		margin-left: 57%;
		font-size: 11pt;
	}
	.judul {
		margin: 4mm 0 2mm;
		text-align: center;
		font-size: 11pt;
		font-weight: bold;
		text-decoration: underline;
	}
	.kotak {
		width: 100%;
		border-collapse: collapse;
		table-layout: fixed;
	}
	.kotak td {
		border: 1px solid #000;
		padding: 2px 6px;
		vertical-align: top;
	}
	.kotak p + p {
		margin-top: 2px;
	}
	.depan td:first-child {
		text-align: center;
	}
	/* Blok PPK: rata kiri, dimulai sedikit lewat tengah halaman */
	.ppk {
		margin-left: 60%;
		break-inside: avoid;
	}
	.belakang tr {
		break-inside: avoid;
	}
	.sel {
		display: flex;
		gap: 4px;
		min-height: 52mm;
	}
	.romawi {
		width: 7mm;
		flex-shrink: 0;
	}
	.belakang :deep(.fields-label) {
		white-space: pre-line;
	}
</style>
