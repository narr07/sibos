<template>
	<PrintSheet v-for="g in groups" :key="g.key">
		<div class="doc">
			<div class="flex items-start justify-between mb-2">
				<div class="text-[10pt] leading-tight">
					<p class="font-bold uppercase">
						{{ sekolah?.nama }}
					</p>
					<p>{{ [sekolah?.kecamatan, sekolah?.kabupaten].filter(Boolean).join(", ") }}</p>
				</div>
				<div class="border border-black px-3 py-1 text-[10pt] text-right">
					<p>Model A2</p>
					<p>No. {{ g.noBukti || "-" }}</p>
				</div>
			</div>
			<p class="text-center text-[16pt] font-bold tracking-[0.3em] my-4">
				KWITANSI
			</p>

			<PrintFields class="w-full kwitansi-fields" label-width="11rem" hide-empty :rows="fields(g)">
				<template #value-1="{ value }">
					<span class="italic font-semibold bg-gray-100 px-1">{{ value }}</span>
				</template>
			</PrintFields>

			<div class="mt-6 flex items-end justify-between">
				<div class="border-2 border-black px-4 py-2 text-[14pt] font-bold">
					Rp {{ angka(g.total) }}
				</div>
				<div class="text-center text-[10pt]">
					<p>{{ [pengaturan?.cetak.kota, tanggalPanjang(bayar(g))].filter(Boolean).join(", ") }}</p>
					<p>Yang menerima,</p>
					<!-- Ruang kosong cukup untuk tanda tangan di atas meterai bila diperlukan -->
					<div class="h-[24mm] w-[50mm]" />
					<p class="font-bold underline">
						<template v-if="g.nota?.namaToko">
							{{ g.nota?.namaToko }}
						</template><span v-else class="garis-isi" />
					</p>
				</div>
			</div>

			<PrintTandaTangan :signers="signers" space="22mm" />
		</div>
	</PrintSheet>
</template>

<script lang="ts" setup>
	defineProps<{ groups: NotaGroup[] }>();
	const { pengaturan, sekolah } = useDocContext();

	const bayar = (g: NotaGroup) => g.override?.tanggalBayar || g.tanggal;
	const tanggalNota = (g: NotaGroup) => g.override?.tanggalNota || g.nota?.tanggalNota || null;
	const keperluan = (g: NotaGroup) => g.override?.keperluan || g.override?.uraian || g.uraian;

	const fields = (g: NotaGroup): [string, string | null | undefined][] => {
		const tgl = tanggalNota(g);
		return [
			["Sudah terima dari", `Bendahara BOS ${sekolah.value?.nama ?? ""}`],
			["Uang sebanyak", terbilangRupiah(g.total)],
			["Untuk pembayaran", keperluan(g)],
			["Sesuai nota", g.nota?.noNota ? `No. ${g.nota.noNota}${tgl ? `, tanggal ${tanggalPanjang(tgl)}` : ""}` : null],
			["Pajak", g.taxes.length ? g.taxes.map((t) => `${t.jenis || "Pajak"} Rp ${angka(t.nominal)}`).join("; ") : "-"],
			["Dibayarkan kepada penyedia", g.taxes.length ? `Rp ${angka(g.diterima)}` : null]
		];
	};

	const signers = computed<Penandatangan[]>(() => {
		const p = pengaturan.value?.pejabat;
		return [
			{ atas: "Setuju dibayar,", jabatan: "Kepala Sekolah", nama: p?.kepalaSekolah, nip: p?.nipKepalaSekolah },
			{ atas: "Lunas dibayar,", jabatan: "Bendahara", nama: p?.bendahara, nip: p?.nipBendahara }
		];
	});
</script>
