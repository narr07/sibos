<template>
	<PrintSheet>
		<div class="doc rapat">
			<div class="text-center font-bold uppercase mb-4 leading-snug">
				<p class="text-[12pt]">
					Berita Acara Rapat
				</p>
				<p>{{ r.namaKegiatan }}</p>
				<p>{{ sekolah?.nama }}</p>
				<p class="normal-case font-normal">
					Nomor: {{ r.nomorBa || "...................................." }}
				</p>
			</div>

			<p class="text-justify">
				Pada hari ini, <i>{{ tgl?.hari }}</i> tanggal <i>{{ tgl?.tanggal }}</i> bulan <i>{{ tgl?.bulan }}</i>
				tahun <i>{{ tgl?.tahun }}</i> ({{ tanggalId(r.tanggal) }}), bertempat di {{ r.tempat }} {{ namaSekolahRapi(sekolah?.nama) }},
				telah dilaksanakan rapat dalam rangka:
			</p>
			<p class="my-3 text-center font-bold">
				"{{ r.namaKegiatan }}"
			</p>

			<PrintFields
				label-width="11rem"
				:rows="[
					['Rapat dipimpin oleh', [r.pimpinan, r.jabatanPimpinan && `(${r.jabatanPimpinan})`].filter(Boolean).join(' ')],
					['Notulis', r.notulis],
					['Jumlah peserta hadir', `${jumlahHadir} orang (daftar hadir terlampir)`]
				]"
			/>

			<p class="mt-3 text-justify">
				Setelah melalui proses pemaparan, pembahasan, dan musyawarah mufakat, maka disepakati hal-hal sebagai berikut:
			</p>
			<ol class="my-2 list-decimal pl-8 text-justify">
				<li v-for="(poin, i) in poinKeputusan" :key="i">
					{{ poin }}
				</li>
			</ol>

			<p class="text-justify">
				Demikian Berita Acara ini dibuat dengan sebenarnya dan ditandatangani untuk dipergunakan sebagaimana mestinya
				sebagai bukti pertanggungjawaban kegiatan BOSP.
			</p>

			<PrintTandaTangan :signers="signers" />

			<!-- Bila pimpinan rapat bukan kepala sekolah, kepala sekolah ikut mengetahui -->
			<div v-if="perluMengetahui" class="mengetahui">
				<p>Mengetahui,</p>
				<p>{{ jabatanKepala(sekolah?.nama) }}</p>
				<p class="nama">
					<template v-if="pejabat?.kepalaSekolah">
						{{ pejabat.kepalaSekolah }}
					</template><span v-else class="garis-isi" />
				</p>
				<p v-if="pejabat?.nipKepalaSekolah">
					NIP. {{ pejabat.nipKepalaSekolah }}
				</p>
			</div>
		</div>
	</PrintSheet>
</template>

<script lang="ts" setup>
	const props = defineProps<{ r: RapatData, jumlahHadir: number }>();
	const { pengaturan, sekolah } = useDocContext();

	const pejabat = computed(() => pengaturan.value?.pejabat);
	const tgl = computed(() => tanggalTerbilang(props.r.tanggal));
	const poinKeputusan = computed(() => barisTeks(props.r.keputusan));

	const sama = (a?: string, b?: string) => (a ?? "").trim().toLowerCase() === (b ?? "").trim().toLowerCase();
	const perluMengetahui = computed(() => !sama(props.r.pimpinan, pejabat.value?.kepalaSekolah));

	const signers = computed<Penandatangan[]>(() => [
		{ jabatan: "Pimpinan Rapat,", nama: props.r.pimpinan, nip: props.r.nipPimpinan },
		{ jabatan: "Notulis Rapat,", nama: props.r.notulis, nip: props.r.nipNotulis }
	]);
</script>

<style scoped>
	.rapat {
		font-size: 12pt;
	}
	.rapat :deep(.ttd) {
		font-size: 12pt;
	}
	.mengetahui {
		margin-top: 1.5rem;
		text-align: center;
		break-inside: avoid;
	}
	.mengetahui .nama {
		margin-top: 20mm;
		font-weight: bold;
		text-decoration: underline;
	}
</style>
