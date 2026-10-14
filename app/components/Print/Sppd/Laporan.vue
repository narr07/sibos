<template>
	<PrintSheet>
		<div class="doc sppd">
			<p class="mb-5 text-center text-[12pt] font-bold underline">
				LAPORAN PELAKSANAAN TUGAS
			</p>
			<p class="text-justify indent-8">
				Adapun hasil pelaksanaan tugas secara ringkas dapat saya laporkan sebagai berikut:
			</p>
			<ol class="mt-2 list-decimal pl-6 text-justify">
				<li>
					Pada hari {{ hariDari(d.tanggalBerangkat) }} tanggal {{ tanggalPanjang(d.tanggalBerangkat) }} pukul {{ jam(d.jamBerangkat) }}
					saya berangkat menuju tempat kegiatan dalam rangka {{ maksud }}, bertempat di {{ d.tujuan || "...................." }}.
				</li>
				<li>
					Kegiatan dilaksanakan selama {{ lamaTeks(d) }} dengan rangkaian acara sebagai berikut:
					<ol class="list-[lower-alpha] pl-6">
						<li v-for="(a, i) in acara" :key="i">
							{{ a }}
						</li>
					</ol>
				</li>
			</ol>
			<p class="mt-3 text-justify indent-8">
				Demikian laporan ini kami buat untuk menjadi bahan sebagaimana mestinya.
			</p>

			<PrintTandaTangan
				:signers="[{ atas: tempatTanggal, jabatan: 'Yang Melaksanakan Tugas', nama: orang.nama, nip: orang.nip }]"
			/>
		</div>
	</PrintSheet>
</template>

<script lang="ts" setup>
	const props = defineProps<{ d: SppdData, orang: Guru }>();
	const { pengaturan } = useDocContext();

	/** Maksud di tengah kalimat: huruf awal dikecilkan ("Mengikuti ..." -> "mengikuti ..."). */
	const maksud = computed(() => {
		const m = props.d.maksud.trim();
		// Singkatan di awal (mis. "TKA") dibiarkan.
		return m.slice(0, 2) === m.slice(0, 2).toUpperCase() ? m : m.charAt(0).toLowerCase() + m.slice(1);
	});
	const acara = computed(() => barisTeks(props.d.acara));
	const tempatTanggal = computed(() => [pengaturan.value?.cetak.kota, tanggalPanjang(props.d.tanggalKembali)].filter(Boolean).join(", "));
</script>

<style scoped>
	.sppd {
		font-size: 12pt;
		line-height: 1.5;
	}
	.sppd :deep(.ttd) {
		font-size: 12pt;
	}
</style>
