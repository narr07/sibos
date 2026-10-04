<template>
	<!-- Lembar 1: Register Penutupan Kas (tanpa kop) -->
	<PrintSheet>
		<div class="doc rpk">
			<p class="text-center text-[14pt] font-bold mb-5">
				REGISTER PENUTUPAN KAS
			</p>

			<PrintFields
				label-width="17rem"
				:rows="[
					['Tanggal Penutupan Kas', tanggalPanjang(tglTutup)],
					['Nama Penutup Kas (Pemegang Kas)', pejabat?.bendahara],
					['Tanggal Penutupan Kas yang lalu', tanggalPanjang(tglLalu)],
					['Jumlah penerimaan dan saldo/jasa bank (D)', rp(reg.penerimaanSd)],
					['Jumlah total pengeluaran (K)', rp(reg.pengeluaranSd)]
				]"
			/>
			<PrintFields
				class="my-3 font-bold"
				label-width="17rem"
				:rows="[
					['Saldo Kas (A = D - K)', rp(saldoA)],
					['Saldo Kas (B)', rp(saldoB)]
				]"
			/>

			<p class="mb-1">
				Saldo Kas B terdiri dari:
			</p>
			<table class="tabel">
				<thead>
					<tr>
						<th>Keterangan</th>
						<th class="w-28">
							Nominal
						</th>
						<th class="w-32">
							Jumlah
						</th>
						<th class="w-36">
							Total
						</th>
					</tr>
				</thead>
				<tbody>
					<template v-for="(g, gi) in groups" :key="g.label">
						<tr v-for="(row, i) in g.rows" :key="`${g.label}${row.nilai}`">
							<td>{{ i === 0 ? `${gi + 1}. ` : "" }}<span :class="i === 0 ? '' : 'pl-4'">{{ g.label }}</span></td>
							<td class="num">
								Rp {{ angka(row.nilai) }}
							</td>
							<td class="text-center">
								{{ row.jumlah }} {{ g.satuan }}
							</td>
							<td class="num">
								Rp {{ angka(row.nilai * row.jumlah) }}
							</td>
						</tr>
						<tr class="font-bold">
							<td colspan="3" class="text-right">
								Sub Jumlah ({{ gi + 1 }})
							</td>
							<td class="num">
								Rp {{ angka(g.total) }}
							</td>
						</tr>
					</template>
					<tr>
						<td>3. Saldo Bank, Surat Berharga dll</td>
						<td />
						<td class="text-right font-bold">
							Sub Jumlah (3)
						</td>
						<td class="num font-bold">
							Rp {{ angka(reg.saldoBank) }}
						</td>
					</tr>
					<tr class="font-bold">
						<td colspan="3" class="text-right">
							Jumlah (1 + 2 + 3)
						</td>
						<td class="num">
							Rp {{ angka(saldoB) }}
						</td>
					</tr>
					<tr>
						<td colspan="3" class="text-right">
							Perbedaan (A - B)
						</td>
						<td class="num">
							Rp {{ angka(perbedaan) }}
						</td>
					</tr>
				</tbody>
			</table>

			<p class="mt-3">
				Penjelasan perbedaan:
			</p>
			<p class="italic">
				{{ penjelasan }}
			</p>

			<div class="ttd">
				<div>
					<p>&nbsp;</p>
					<p>Yang diperiksa,</p>
					<p>Bendahara/Pemegang Kas</p>
					<p class="nama">
						{{ pejabat?.bendahara || "(..............................)" }}
					</p>
					<p>NIP. {{ pejabat?.nipBendahara || "-" }}</p>
				</div>
				<div>
					<p>{{ tempatTanggal }}</p>
					<p>Yang memeriksa,</p>
					<p>Kepala Sekolah</p>
					<p class="nama">
						{{ pejabat?.kepalaSekolah || "(..............................)" }}
					</p>
					<p>NIP. {{ pejabat?.nipKepalaSekolah || "-" }}</p>
				</div>
			</div>
		</div>
	</PrintSheet>

	<!-- Lembar 2: Berita Acara Pemeriksaan Kas (tanpa kop) -->
	<PrintSheet>
		<div class="doc rpk">
			<p class="text-center text-[14pt] font-bold mb-5">
				BERITA ACARA PEMERIKSAAN KAS
			</p>
			<p>
				Pada hari ini <i>{{ hari }}</i> tanggal <i>{{ terbilang(tglDate.getDate()) }}</i> bulan <i>{{ BULAN[reg.month - 1] }}</i>
				tahun <i>{{ terbilang(reg.year) }}</i>, yang bertanda tangan di bawah ini, kami Kepala Sekolah yang ditunjuk berdasarkan
				Surat Keputusan Nomor: {{ pejabat?.skKepalaSekolah || "........................................" }}
			</p>
			<PrintFields
				class="my-2"
				label-width="9rem"
				:rows="[['Nama', pejabat?.kepalaSekolah], ['Jabatan', `Kepala ${namaSekolah}`]]"
			>
				<template #value-0="{ value }">
					<b>{{ value }}</b>
				</template>
			</PrintFields>
			<p>Melakukan pemeriksaan kas kepada:</p>
			<PrintFields
				class="my-2"
				label-width="9rem"
				:rows="[['Nama', pejabat?.bendahara], ['Jabatan', `Bendahara BOS ${namaSekolah}`]]"
			>
				<template #value-0="{ value }">
					<b>{{ value }}</b>
				</template>
			</PrintFields>
			<p>
				Yang berdasarkan Surat Keputusan Nomor {{ pejabat?.skBendahara || "........................................" }}
				tanggal {{ pejabat?.tanggalSkBendahara || "...................." }} ditugaskan dengan pengurusan uang dana
				Bantuan Operasional Sekolah (BOS). Berdasarkan pemeriksaan kas serta bukti-bukti dalam pengurusan itu,
				kami menemui kenyataan sebagai berikut:
			</p>

			<table class="ba-rincian my-3">
				<tbody>
					<tr>
						<td class="w-8">
							a)
						</td>
						<td>Uang kertas bank, uang logam</td>
						<td class="w-10">
							: Rp
						</td>
						<td class="num w-36">
							{{ angka(fisik) }}
						</td>
					</tr>
					<tr>
						<td>b)</td>
						<td>Saldo Bank</td>
						<td>: Rp</td>
						<td class="num">
							{{ angka(reg.saldoBank) }}
						</td>
					</tr>
					<tr>
						<td>c)</td>
						<td>Surat berharga dll</td>
						<td>: Rp</td>
						<td class="num">
							0
						</td>
					</tr>
					<tr class="font-bold">
						<td />
						<td class="text-center">
							Jumlah
						</td>
						<td>: Rp</td>
						<td class="num">
							{{ angka(saldoB) }}
						</td>
					</tr>
				</tbody>
			</table>
			<PrintFields
				label-width="20rem"
				:rows="[
					['Saldo uang menurut Buku Kas Umum (BKU)', rp(saldoA)],
					['Perbedaan antara saldo kas dan saldo buku', rp(perbedaan)]
				]"
			/>

			<div class="ttd">
				<div>
					<p>&nbsp;</p>
					<p>Yang diperiksa,</p>
					<p>Bendahara/Pemegang Kas</p>
					<p class="nama">
						{{ pejabat?.bendahara || "(..............................)" }}
					</p>
					<p>NIP. {{ pejabat?.nipBendahara || "-" }}</p>
				</div>
				<div>
					<p>{{ tempatTanggal }}</p>
					<p>Yang memeriksa,</p>
					<p>Kepala Sekolah</p>
					<p class="nama">
						{{ pejabat?.kepalaSekolah || "(..............................)" }}
					</p>
					<p>NIP. {{ pejabat?.nipKepalaSekolah || "-" }}</p>
				</div>
			</div>
		</div>
	</PrintSheet>
</template>

<script lang="ts" setup>
	const props = defineProps<{ reg: RegisterKas, pecahan: Pecahan, catatan?: string | null }>();
	const { pengaturan, sekolah } = useDocContext();

	const HARI = ["Minggu", "Senin", "Selasa", "Rabu", "Kamis", "Jumat", "Sabtu"];

	const pejabat = computed(() => pengaturan.value?.pejabat);
	const namaSekolah = computed(() => sekolah.value?.nama ?? "");
	const rp = (n: number) => `Rp ${angka(n)}`;

	const tglTutup = computed(() => akhirBulan(props.reg.year, props.reg.month));
	const tglLalu = computed(() => (props.reg.month === 1 ? `${props.reg.year - 1}-12-31` : akhirBulan(props.reg.year, props.reg.month - 1)));
	const tglDate = computed(() => new Date(props.reg.year, props.reg.month, 0));
	const hari = computed(() => HARI[tglDate.value.getDay()]);
	const tempatTanggal = computed(() => [pengaturan.value?.cetak.kota, tanggalPanjang(tglTutup.value)].filter(Boolean).join(", "));

	/** Semua pecahan selalu tampil, termasuk yang jumlahnya 0. */
	const groups = computed(() => [
		{ label: "Lembaran uang kertas", satuan: "lembar", list: PECAHAN_KERTAS, src: props.pecahan.kertas },
		{ label: "Keping uang logam", satuan: "keping", list: PECAHAN_LOGAM, src: props.pecahan.logam }
	].map((g) => {
		const rows = g.list.map((nilai) => ({ nilai, jumlah: Number(g.src[String(nilai)] ?? 0) }));
		return { ...g, rows, total: rows.reduce((t, r) => t + r.nilai * r.jumlah, 0) };
	}));

	const fisik = computed(() => groups.value.reduce((t, g) => t + g.total, 0));
	const saldoA = computed(() => props.reg.saldoBuku);
	const saldoB = computed(() => fisik.value + props.reg.saldoBank);
	const perbedaan = computed(() => saldoA.value - saldoB.value);
	const penjelasan = computed(() => props.catatan?.trim()
		|| (perbedaan.value === 0
			? `Setelah pemeriksaan kas antara jumlah penerimaan dan pengeluaran jadi saldo Rp ${angka(saldoA.value)}`
			: `Terdapat perbedaan sebesar Rp ${angka(perbedaan.value)} antara saldo buku dan saldo kas.`));
</script>

<style scoped>
	.rpk {
		font-size: 11pt;
	}
	.rpk :deep(table.tabel) {
		font-size: 10pt;
	}
	.rpk :deep(table.tabel td) {
		padding: 2px 6px;
	}
	.ba-rincian td {
		padding: 1px 4px;
	}
	.ttd {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 2rem;
		margin-top: 2rem;
		break-inside: avoid;
		padding-left: 0.5rem;
	}
	.ttd .nama {
		margin-top: 18mm;
		font-weight: bold;
		text-decoration: underline;
	}
</style>
