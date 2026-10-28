<template>
	<PrintSheet>
		<div class="doc si-bank" style="font-family: 'Times New Roman', Times, serif; font-size: 10pt; line-height: 1.25; color: black;">
			<!-- Kop Surat Sekolah -->
			<PrintKopSurat :ringkas="true" />

			<!-- Metadata Surat: Nomor & Perihal di kiri, Standing Instruction & Tanggal di kanan -->
			<div class="grid grid-cols-2 gap-4 my-2 text-[10pt]">
				<div>
					<table class="border-collapse w-full">
						<tbody>
							<tr>
								<td class="w-20 align-top py-0.5">
									Nomor
								</td>
								<td class="w-3 align-top py-0.5">
									:
								</td>
								<td class="align-top py-0.5 font-medium">
									{{ si.nomorSurat || '................................................' }}
								</td>
							</tr>
							<tr>
								<td class="align-top py-0.5">
									Sifat
								</td>
								<td class="align-top py-0.5">
									:
								</td>
								<td class="align-top py-0.5">
									{{ si.sifat || 'Biasa' }}
								</td>
							</tr>
							<tr>
								<td class="align-top py-0.5">
									Lampiran
								</td>
								<td class="align-top py-0.5">
									:
								</td>
								<td class="align-top py-0.5">
									{{ si.lampiran || '-' }}
								</td>
							</tr>
							<tr>
								<td class="align-top py-0.5">
									Perihal
								</td>
								<td class="align-top py-0.5">
									:
								</td>
								<td class="align-top py-0.5 font-medium">
									{{ si.perihal || 'Pemindah bukuan/Transfer' }}
								</td>
							</tr>
						</tbody>
					</table>
				</div>

				<div class="text-right">
					<p class="font-bold text-[12pt] uppercase tracking-wide">
						Standing Instruction
					</p>
					<p class="mt-1">
						{{ si.kota || sekolah?.kecamatan || 'Tempat' }}, {{ tanggalId(si.tanggal) }}
					</p>
				</div>
			</div>

			<!-- Tujuan Surat -->
			<div class="my-2.5">
				<p>Kepada Yth.</p>
				<p class="font-bold">
					{{ si.bankTujuan || 'Bank BJB Cabang / KCP' }}
				</p>
				<p>Di tempat</p>
			</div>

			<!-- Pernyataan Pihak Bertandatangan -->
			<div class="my-2 text-justify">
				<p class="mb-1.5">
					Yang bertandatangan di bawah ini,
				</p>
				<div class="grid grid-cols-2 gap-4 pl-3 mb-2">
					<div>
						<p class="text-[9pt] text-gray-700 italic">
							1. Kepala Sekolah:
						</p>
						<p><span class="inline-block w-16">Nama</span>: <strong>{{ si.namaKepalaSekolah || pejabat?.kepalaSekolah || '-' }}</strong></p>
						<p><span class="inline-block w-16">No KTP</span>: {{ si.ktpKepalaSekolah || '-' }}</p>
					</div>
					<div>
						<p class="text-[9pt] text-gray-700 italic">
							2. Bendahara:
						</p>
						<p><span class="inline-block w-16">Nama</span>: <strong>{{ si.namaBendahara || pejabat?.bendahara || '-' }}</strong></p>
						<p><span class="inline-block w-16">No KTP</span>: {{ si.ktpBendahara || '-' }}</p>
					</div>
				</div>

				<div class="pl-3 space-y-0.5 border-l-2 border-black/30 my-2 text-[9.5pt]">
					<p><span class="inline-block w-36">Atas Nama Rekening</span>: <strong>{{ si.namaRekeningSumber || sekolah?.nama || '-' }}</strong></p>
					<p><span class="inline-block w-36">No. Rekening</span>: <strong>{{ si.noRekeningSumber || '-' }}</strong></p>
					<p><span class="inline-block w-36">Uraian</span>: {{ si.uraianSumber || 'Belanja Barang dan Jasa' }}</p>
				</div>

				<p class="mt-2 text-justify">
					Maka dengan ini kami mohon kepada Bapak/Ibu untuk dapat melakukan pemindahbukuan / mentransfer dana dari rekening kami ke rekening penerima berikut ini:
				</p>
			</div>

			<!-- Tabel Rincian Transfer Penerima -->
			<table class="w-full border-collapse border border-black my-2.5 text-[8.5pt]">
				<thead>
					<tr class="bg-gray-100 font-bold text-center">
						<th class="border border-black px-1.5 py-1 w-7">
							No
						</th>
						<th class="border border-black px-2 py-1">
							Atas Nama Rekening
						</th>
						<th class="border border-black px-1.5 py-1 w-14">
							Nama Bank
						</th>
						<th class="border border-black px-2 py-1 w-28">
							No Rekening
						</th>
						<th class="border border-black px-2 py-1 w-24">
							Jumlah Uang
						</th>
						<th class="border border-black px-1.5 py-1 w-16">
							Pajak
						</th>
						<th class="border border-black px-2 py-1 w-24">
							Jumlah Uang Diterima
						</th>
						<th class="border border-black px-2 py-1">
							Keterangan
						</th>
					</tr>
				</thead>
				<tbody>
					<tr v-for="(item, idx) in si.items" :key="item.id || idx">
						<td class="border border-black px-1.5 py-1 text-center align-top">
							{{ idx + 1 }}
						</td>
						<td class="border border-black px-2 py-1 align-top font-medium">
							{{ item.namaPenerima }}
						</td>
						<td class="border border-black px-1.5 py-1 text-center align-top font-mono">
							{{ item.namaBank || 'BJB' }}
						</td>
						<td class="border border-black px-2 py-1 text-center align-top font-mono">
							{{ item.noRekening }}
						</td>
						<td class="border border-black px-2 py-1 text-right align-top tabular-nums font-mono">
							{{ angka(item.jumlah) }}
						</td>
						<td class="border border-black px-1.5 py-1 text-right align-top tabular-nums font-mono">
							{{ item.pajak > 0 ? angka(item.pajak) : '-' }}
						</td>
						<td class="border border-black px-2 py-1 text-right align-top tabular-nums font-bold font-mono">
							{{ angka(item.jumlah - (item.pajak || 0)) }}
						</td>
						<td class="border border-black px-2 py-1 align-top">
							{{ item.keterangan }}
						</td>
					</tr>

					<!-- Total Row -->
					<tr class="font-bold bg-gray-50">
						<td colspan="4" class="border border-black px-2 py-1 text-center uppercase tracking-wider">
							Total
						</td>
						<td class="border border-black px-2 py-1 text-right tabular-nums font-mono">
							{{ angka(totalJumlah) }}
						</td>
						<td class="border border-black px-1.5 py-1 text-right tabular-nums font-mono">
							{{ totalPajak > 0 ? angka(totalPajak) : '-' }}
						</td>
						<td class="border border-black px-2 py-1 text-right tabular-nums font-mono text-[9pt]">
							{{ angka(totalDiterima) }}
						</td>
						<td class="border border-black px-2 py-1" />
					</tr>
				</tbody>
			</table>

			<!-- Terbilang -->
			<div class="my-2 p-1.5 border border-black/40 bg-gray-50/60 text-[9pt] italic">
				<strong>Terbilang :</strong> {{ terbilangRupiah(totalDiterima) }}
			</div>

			<!-- Penutup Surat -->
			<p class="my-3 text-justify text-[9.5pt]">
				Demikian Standing Instruction kami buat untuk dipergunakan sebagaimana mestinya dan akibat apapun yang timbul atas pelaksanaan instruksi pemindahbukuan ini menjadi tanggung jawab kami sepenuhnya.
			</p>

			<!-- Tanda Tangan -->
			<div class="mt-6 pt-2">
				<p class="text-center font-bold uppercase text-[10pt] mb-4">
					{{ sekolah?.nama }}
					<br>
					KECAMATAN {{ sekolah?.kecamatan || '....................' }}
				</p>

				<div class="grid grid-cols-2 gap-8 text-center text-[10pt]">
					<!-- Kolom Kiri: Kepala Sekolah -->
					<div>
						<p>Kepala Sekolah,</p>
						<div class="h-20 flex items-end justify-center">
							<p class="font-bold underline">
								{{ si.namaKepalaSekolah || pejabat?.kepalaSekolah || '............................................' }}
							</p>
						</div>
						<p class="text-[9pt]">
							NIP. {{ si.nipKepalaSekolah || pejabat?.nipKepalaSekolah || '........................................' }}
						</p>
					</div>

					<!-- Kolom Kanan: Bendahara -->
					<div>
						<p>Bendahara,</p>
						<div class="h-20 flex items-end justify-center">
							<p class="font-bold underline">
								{{ si.namaBendahara || pejabat?.bendahara || '............................................' }}
							</p>
						</div>
						<p class="text-[9pt]">
							NIP. {{ si.nipBendahara || pejabat?.nipBendahara || '........................................' }}
						</p>
					</div>
				</div>
			</div>
		</div>
	</PrintSheet>
</template>

<script lang="ts" setup>
	export interface SiTransferItem {
		id: string
		namaPenerima: string
		namaBank: string
		noRekening: string
		jumlah: number
		pajak: number
		keterangan: string
	}

	export interface SiData {
		nomorSurat: string
		sifat: string
		lampiran: string
		perihal: string
		tanggal: string
		kota: string
		bankTujuan: string
		namaRekeningSumber: string
		noRekeningSumber: string
		uraianSumber: string
		namaKepalaSekolah: string
		nipKepalaSekolah: string
		ktpKepalaSekolah: string
		namaBendahara: string
		nipBendahara: string
		ktpBendahara: string
		items: SiTransferItem[]
	}

	const props = defineProps<{
		si: SiData
	}>();

	const { pengaturan, sekolah } = useDocContext();
	const pejabat = computed(() => pengaturan.value?.pejabat ?? null);

	const totalJumlah = computed(() => props.si.items.reduce((acc, it) => acc + (it.jumlah || 0), 0));
	const totalPajak = computed(() => props.si.items.reduce((acc, it) => acc + (it.pajak || 0), 0));
	const totalDiterima = computed(() => totalJumlah.value - totalPajak.value);

	const terbilangRupiah = (val: number): string => {
		if (val <= 0) return "Nol rupiah";
		const str = terbilang(val);
		return `${str.charAt(0).toUpperCase() + str.slice(1)} rupiah`;
	};
</script>
