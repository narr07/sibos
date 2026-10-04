<template>
	<PrintSheet v-for="g in groups" :key="g.key">
		<div class="doc">
			<!-- Kop toko -->
			<div class="flex items-start justify-between border-b-2 border-black pb-2 mb-3">
				<div class="leading-tight max-w-[60%]">
					<p class="text-[15pt] font-bold uppercase">
						{{ g.nota?.namaToko || "Nama Toko" }}
					</p>
					<p v-if="g.nota?.alamatToko" class="text-[10pt]">
						{{ g.nota.alamatToko }}
					</p>
					<p v-if="g.nota?.noTelp" class="text-[10pt]">
						Telp. {{ g.nota.noTelp }}
					</p>
					<p v-if="g.nota?.npwp" class="text-[10pt]">
						NPWP {{ g.nota.npwp }}
					</p>
				</div>
				<div class="text-right">
					<p class="text-[18pt] font-bold tracking-[0.2em]">
						NOTA
					</p>
					<PrintFields class="text-[10pt] ml-auto" label-width="4.5rem" :rows="[['Nomor', nomor(g)], ['Tanggal', tanggalPanjang(tanggal(g))]]" />
				</div>
			</div>

			<PrintFields class="mb-3" label-width="6rem" :rows="[['Kepada Yth.', sekolah?.nama], ['Alamat', alamatSekolah]]" />

			<table class="tabel table-fixed">
				<colgroup>
					<col style="width: 6%">
					<col>
					<col style="width: 13%">
					<col style="width: 10%">
					<col style="width: 16%">
					<col style="width: 17%">
				</colgroup>
				<thead>
					<tr>
						<th>No</th>
						<th>Nama Barang / Jasa</th>
						<th>Banyaknya</th>
						<th>Satuan</th>
						<th>Harga Satuan (Rp)</th>
						<th>Jumlah (Rp)</th>
					</tr>
				</thead>
				<tbody>
					<tr v-for="(it, i) in g.items" :key="it.id">
						<td class="text-center">
							{{ i + 1 }}
						</td>
						<td>{{ namaBarang(it.uraian) }}</td>
						<td class="num">
							{{ it.volume ?? "" }}
						</td>
						<td>{{ it.satuan ?? "" }}</td>
						<td class="num">
							{{ harga(it) }}
						</td>
						<td class="num">
							{{ angka(it.nominal) }}
						</td>
					</tr>
					<tr v-for="n in kosong(g)" :key="`k${n}`">
						<td>&nbsp;</td>
						<td />
						<td />
						<td />
						<td />
						<td />
					</tr>
					<tr>
						<td colspan="5" class="text-right font-bold">
							JUMLAH
						</td>
						<td class="num font-bold">
							{{ angka(g.total) }}
						</td>
					</tr>
				</tbody>
			</table>

			<p class="mt-2">
				Terbilang: <i>{{ terbilangRupiah(g.total) }}</i>
			</p>

			<PrintTandaTangan :signers="signers(g)" space="24mm" />
		</div>
	</PrintSheet>
</template>

<script lang="ts" setup>
	defineProps<{ groups: NotaGroup[] }>();
	const { sekolah, pengaturan } = useDocContext();

	/** Minimal jumlah baris tabel agar nota tidak terlihat kosong/pendek. */
	const MIN_BARIS = 8;

	const alamatSekolah = computed(() =>
		[pengaturan.value?.kop.alamat || sekolah.value?.alamat, sekolah.value?.kecamatan, sekolah.value?.kabupaten].filter(Boolean).join(", "));

	const tanggal = (g: NotaGroup) => g.override?.tanggalNota || g.nota?.tanggalNota || g.tanggal;
	/** Nomor nota: isian sendiri, nomor dari ARKAS, atau otomatis dari nomor bukti. */
	const nomor = (g: NotaGroup) => g.override?.noNota || g.nota?.noNota || `NT/${g.noBukti || "-"}/${tanggal(g).slice(0, 4)}`;

	// Uraian ARKAS sering berbentuk "Kategori-Nama barang"; nota cukup nama barangnya.
	const namaBarang = (uraian: string) => {
		const i = uraian.indexOf("-");
		return i > 0 && i < 40 ? uraian.slice(i + 1).trim() : uraian;
	};

	const harga = (it: NotaItem) => {
		if (it.volume && it.volume > 0) return angka(Math.round(it.nominal / it.volume));
		return it.hargaSatuan ? angka(it.hargaSatuan) : "";
	};

	const kosong = (g: NotaGroup) => Math.max(0, MIN_BARIS - g.items.length);

	const signers = (g: NotaGroup): Penandatangan[] => [
		{ atas: "Tanda terima,", jabatan: "Bendahara", nama: pengaturan.value?.pejabat.bendahara, nip: undefined },
		{ atas: "Hormat kami,", jabatan: g.nota?.namaToko ?? "Penjual", nama: "" }
	];
</script>
