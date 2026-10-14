<template>
	<PrintSheet>
		<div class="doc rapat">
			<div class="text-center font-bold uppercase mb-4 leading-snug">
				<p class="text-[12pt]">
					Daftar Hadir
				</p>
				<p>{{ r.namaKegiatan }}</p>
				<p>{{ sekolah?.nama }}</p>
				<p>Tahun Anggaran {{ tahun }}</p>
			</div>

			<PrintFields
				class="mb-3"
				label-width="8rem"
				:rows="[
					['Hari / Tanggal', `${hariDari(r.tanggal)}, ${tanggalPanjang(r.tanggal)}`],
					['Waktu', `${jam(r.waktuMulai)} s.d. ${jam(r.waktuSelesai)} ${r.zona}`],
					['Tempat', r.tempat],
					['Acara / Agenda', r.agenda]
				]"
			/>

			<table class="tabel">
				<thead>
					<tr>
						<th class="w-10">
							No
						</th>
						<th>Nama / NIP</th>
						<th class="w-36">
							Jabatan
						</th>
						<th class="w-40">
							Tanda Tangan
						</th>
					</tr>
				</thead>
				<tbody>
					<tr v-for="(row, i) in rows" :key="i" class="baris">
						<td class="text-center">
							{{ i + 1 }}
						</td>
						<td>
							<!-- Nama satu baris, NIP di bawahnya -->
							<p class="whitespace-nowrap">
								{{ row?.nama }}
							</p>
							<p v-if="row?.nip" class="text-[10pt]">
								NIP. {{ row.nip }}
							</p>
						</td>
						<td class="text-center">
							{{ row ? jabatanHadir(row, r.notulis, pengaturan?.pejabat.kepalaSekolah) : "" }}
						</td>
						<!-- Tanda tangan selang-seling kiri/kanan -->
						<td :class="i % 2 ? 'pl-16!' : ''">
							{{ i + 1 }}.
						</td>
					</tr>
				</tbody>
			</table>

			<PrintTandaTangan :signers="signers" />
		</div>
	</PrintSheet>
</template>

<script lang="ts" setup>
	const props = defineProps<{
		r: RapatData
		/** Guru yang hadir, urut sesuai daftar di Pengaturan. */
		hadir: Guru[]
	}>();
	const { pengaturan, sekolah } = useDocContext();

	const tahun = computed(() => props.r.tanggal.slice(0, 4));

	/** Peserta hadir + baris kosong untuk peserta di luar daftar guru. */
	const rows = computed<(Guru | null)[]>(() => [
		...props.hadir,
		...Array.from<null>({ length: Math.max(0, props.r.barisKosong) }).fill(null)
	]);

	const signers = computed<Penandatangan[]>(() => {
		const p = pengaturan.value?.pejabat;
		return [
			{ atas: "Mengetahui,", jabatan: jabatanKepala(sekolah.value?.nama), nama: p?.kepalaSekolah, nip: p?.nipKepalaSekolah },
			{
				atas: [pengaturan.value?.cetak.kota, tanggalPanjang(props.r.tanggal)].filter(Boolean).join(", "),
				jabatan: "Notulis",
				nama: props.r.notulis,
				nip: props.r.nipNotulis
			}
		];
	});
</script>

<style scoped>
	.rapat {
		font-size: 12pt;
	}
	.rapat table.tabel {
		font-size: 11pt;
	}
	.baris {
		break-inside: avoid;
	}
	.baris td {
		height: 11mm;
		vertical-align: middle;
	}
	.rapat :deep(.ttd) {
		font-size: 12pt;
	}
</style>
