<template>
	<PrintSheet>
		<div class="doc sppd">
			<PrintKopSurat ringkas />
			<div class="text-center mb-4">
				<p class="text-[12pt] font-bold underline">
					SURAT TUGAS
				</p>
				<p>Nomor : {{ d.nomorSuratTugas || "...................................." }}</p>
			</div>

			<table class="susun">
				<tbody>
					<tr>
						<td class="w-20">
							Dasar
						</td>
						<td class="w-4">
							:
						</td>
						<td class="text-justify">
							{{ d.dasar || "........................................................................................................" }}
						</td>
					</tr>
				</tbody>
			</table>

			<p class="my-4 text-center font-bold tracking-wide">
				MEMERINTAHKAN
			</p>

			<table class="susun">
				<tbody>
					<tr v-for="(g, i) in pelaksana" :key="g.id">
						<td class="w-20">
							{{ i === 0 ? "Kepada" : "" }}
						</td>
						<td class="w-4">
							{{ i === 0 ? ":" : "" }}
						</td>
						<td class="w-6">
							{{ pelaksana.length > 1 ? `${i + 1}.` : "" }}
						</td>
						<td class="pb-2">
							<PrintFields
								label-width="7rem"
								:rows="[['Nama', g.nama], ['Pangkat/Gol', g.pangkat || '-'], ['NIP', g.nip || '-'], ['Jabatan', jabatanSppd(g)]]"
							/>
						</td>
					</tr>
					<tr>
						<td>Untuk</td>
						<td>:</td>
						<td colspan="2">
							<ol class="list-decimal pl-5 text-justify">
								<li>{{ d.maksud }} pada tanggal {{ rentangTanggal(d) }} bertempat di {{ d.tujuan || "...................." }}.</li>
								<li>Menyampaikan laporan hasil pelaksanaan tugas kepada pimpinan.</li>
								<li>Dilaksanakan dengan penuh rasa tanggung jawab.</li>
							</ol>
						</td>
					</tr>
				</tbody>
			</table>

			<PrintTandaTangan :signers="signers" />
		</div>
	</PrintSheet>
</template>

<script lang="ts" setup>
	const props = defineProps<{ d: SppdData, pelaksana: Guru[] }>();
	const { pengaturan, sekolah } = useDocContext();

	const signers = computed<Penandatangan[]>(() => {
		const p = pengaturan.value?.pejabat;
		return [{
			atas: [pengaturan.value?.cetak.kota, tanggalPanjang(props.d.tanggalSurat)].filter(Boolean).join(", "),
			jabatan: jabatanKepala(sekolah.value?.nama),
			nama: p?.kepalaSekolah,
			nip: p?.nipKepalaSekolah
		}];
	});
</script>

<style scoped>
	/* Format SPPD sekolah memakai Arial */
	.sppd {
		font-family: Arial, Helvetica, sans-serif;
		font-size: 11pt;
	}
	.sppd :deep(.ttd) {
		font-size: 11pt;
	}
	.susun {
		width: 100%;
		border-collapse: collapse;
	}
	.susun > tbody > tr > td {
		vertical-align: top;
		padding: 1px 0;
	}
</style>
