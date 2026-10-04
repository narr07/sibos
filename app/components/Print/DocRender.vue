<template>
	<PrintSheet :paper="template.kertas" :landscape="template.orientasi === 'landscape'" :margin="template.margin" :margin-top="template.marginAtas || undefined">
		<div class="doc" :style="{ fontSize: `${template.ukuranHuruf || 11}pt` }">
			<template v-for="b in template.blocks" :key="b.id">
				<!-- Kop sekolah: sama dengan kop di Pengaturan (2 logo, 5 baris, garis ganda) -->
				<PrintKopSurat v-if="b.type === 'kop' && b.sumber === 'sekolah'" />
				<!-- Kop toko / teks sendiri -->
				<div v-else-if="b.type === 'kop'" class="flex items-center gap-3 pb-2 mb-3" :class="b.garis ? 'border-b-[3px] border-double border-black' : ''">
					<img v-if="logo(b)" :src="logo(b)" alt="" class="h-[22mm] w-auto object-contain shrink-0">
					<div class="flex-1 text-center leading-tight">
						<p v-for="(line, i) in kopLines(b)" :key="i" :class="kopClass(b, i)">
							{{ line }}
						</p>
					</div>
					<div v-if="logo(b)" class="w-[22mm] shrink-0" />
				</div>

				<!-- Judul -->
				<p
					v-else-if="b.type === 'judul'"
					class="font-bold leading-tight mt-1"
					:class="b.garisBawah ? 'underline' : ''"
					:style="{ fontSize: `${b.ukuran}pt`, textAlign: b.rata, letterSpacing: b.ukuran >= 16 ? '0.2em' : undefined }"
				>
					{{ fill(b.teks) }}
				</p>

				<!-- Paragraf -->
				<p
					v-else-if="b.type === 'teks' && fill(b.isi).trim()"
					class="whitespace-pre-line my-1"
					:class="b.tebal ? 'font-bold' : ''"
					:style="{ textAlign: b.rata, fontSize: b.ukuran ? `${b.ukuran}pt` : undefined }"
				>
					{{ fill(b.isi) }}
				</p>

				<!-- Daftar isian -->
				<PrintFields
					v-else-if="b.type === 'isian'"
					class="my-1"
					:style="{ marginLeft: `${b.indent || 0}mm` }"
					:label-width="`${b.lebarLabel}mm`"
					:rows="b.baris.map((r) => [fill(r.label), fill(r.nilai)])"
				/>

				<!-- Tabel barang -->
				<table v-else-if="b.type === 'tabel'" class="tabel table-fixed my-2" :style="{ fontSize: `${b.ukuran || 10}pt` }">
					<colgroup>
						<col v-for="k in b.kolom" :key="k.kunci" :style="k.lebar ? { width: `${k.lebar}%` } : {}">
					</colgroup>
					<thead>
						<tr>
							<th v-for="k in b.kolom" :key="k.kunci">
								{{ k.judul }}
							</th>
						</tr>
					</thead>
					<tbody>
						<tr v-for="(it, i) in g.items" :key="it.id">
							<td v-for="k in b.kolom" :key="k.kunci" :class="numeric(k.kunci) ? 'num' : k.kunci === 'no' ? 'text-center' : ''">
								{{ cell(k.kunci, it, i, b.bersihkanNama) }}
							</td>
						</tr>
						<tr v-for="n in Math.max(0, (b.minBaris || 0) - g.items.length)" :key="`k${n}`">
							<td v-for="k in b.kolom" :key="k.kunci">
								&nbsp;
							</td>
						</tr>
						<tr v-if="b.total">
							<td :colspan="Math.max(1, b.kolom.length - 1)" class="text-right font-bold">
								{{ b.labelTotal }}
							</td>
							<td class="num font-bold">
								{{ angka(g.total) }}
							</td>
						</tr>
					</tbody>
				</table>

				<!-- Terbilang -->
				<p v-else-if="b.type === 'terbilang'" class="my-1">
					{{ b.label }}: <i>{{ vars.terbilang }}</i>
				</p>

				<!-- Kotak jumlah -->
				<div v-else-if="b.type === 'kotakTotal'" class="my-2">
					<span class="inline-block border-2 border-black px-4 py-1.5 text-[14pt] font-bold">{{ b.label }} {{ vars.total }}</span>
				</div>

				<!-- Tanda tangan -->
				<div
					v-else-if="b.type === 'ttd'"
					class="mt-5 grid text-center break-inside-avoid"
					:style="{ gridTemplateColumns: `repeat(${ttdCols(b)}, minmax(0, 1fr))` }"
				>
					<div v-for="(s, i) in b.kolom" :key="i" class="flex flex-col items-center px-2" :class="ttdPos(b, i)">
						<p class="whitespace-pre-line min-h-[1.3em]">
							{{ fill(s.atas) }}
						</p>
						<p v-if="fill(s.jabatan)" class="whitespace-pre-line">
							{{ fill(s.jabatan) }}
						</p>
						<div :style="{ height: `${b.tinggi || 20}mm` }" />
						<p class="font-bold underline">
							{{ fill(s.nama) || "(..............................)" }}
						</p>
						<p v-if="fill(s.nip)">
							NIP. {{ fill(s.nip) }}
						</p>
					</div>
				</div>

				<!-- Kwitansi: kotak tinggi tetap seperti format toko, plus visum kepala sekolah & bendahara -->
				<div v-else-if="b.type === 'kwitansi'" class="kwitansi break-inside-avoid mx-auto" :style="{ width: b.lebarKotak ? `${b.lebarKotak}mm` : '100%', maxWidth: '100%' }">
					<div class="flex border-[1.5px] border-black overflow-hidden" :style="{ height: `${b.tinggiKotak || 75}mm` }">
						<div
							v-if="b.panelToko"
							class="shrink-0 h-full border-r-[2px] border-black flex items-stretch gap-[2mm] pl-[2mm] pr-[1mm] py-[2mm]"
							:style="b.lebarPanel ? { width: `${b.lebarPanel}mm` } : {}"
						>
							<!-- Kolom 1: kop toko, logo di bawah -->
							<div class="h-full flex flex-col items-center justify-between border-r border-black pr-[2mm]">
								<p class="kw-vertical flex-1 min-h-0 text-center leading-[1.2] overflow-hidden">
									<span v-for="(line, i) in kopTokoLines" :key="i" class="block" :class="i < Math.max(1, kopTokoLines.length - 2) ? 'text-[7pt] font-bold' : 'text-[5.5pt]'">{{ line }}</span>
								</p>
								<img v-if="penyedia?.logo" :src="penyedia.logo" alt="" class="kw-logo mt-[1mm]">
							</div>
							<!-- Kolom 2: daftar layanan (dipisah baris kosong; kelompok pertama di bawah) -->
							<div v-if="layananGroups.length" class="h-full flex flex-col-reverse justify-between">
								<p v-for="(grp, gi) in layananGroups" :key="gi" class="kw-vertical whitespace-pre-line text-[5.5pt] leading-[1.25] overflow-hidden" :style="{ maxHeight: `${100 / layananGroups.length}%` }">
									<span class="font-bold block text-[6pt]">{{ grp[0] }}</span>{{ grp.slice(1).join("\n") }}
								</p>
							</div>
							<!-- Kolom 3: motto -->
							<p v-if="b.motto" class="kw-vertical h-full text-center italic text-[10.5pt] leading-none whitespace-nowrap overflow-hidden" style="font-family: 'Monotype Corsiva', 'Segoe Script', cursive">
								{{ b.motto }}
							</p>
						</div>

						<div class="flex-1 min-w-0 h-full flex flex-col px-[6mm] py-[2mm] text-[11pt]">
							<span class="italic text-[10pt]">No. {{ vars.nomor }}</span>
							<p class="text-center text-[22pt] tracking-[0.4em] leading-none -mt-[2mm] mb-[3mm]" style="font-family: 'Algerian', 'Times New Roman', serif">
								KWITANSI
							</p>
							<table class="w-full kw-fields">
								<tbody>
									<tr>
										<td class="kw-label">
											Telah terima dari
										</td>
										<td class="kw-colon">
											:
										</td>
										<td>Bendahara {{ vars.nama_sekolah }}</td>
									</tr>
									<tr>
										<td class="kw-label">
											Uang Sejumlah
										</td>
										<td class="kw-colon">
											:
										</td>
										<td>
											<div class="kw-sorot w-[92%] italic" :style="{ background: b.warna }">
												{{ vars.terbilang }}
											</div>
										</td>
									</tr>
									<tr>
										<td class="kw-label">
											Untuk pembayaran
										</td>
										<td class="kw-colon">
											:
										</td>
										<td>{{ vars.untuk_pembayaran }}</td>
									</tr>
								</tbody>
							</table>
							<div class="flex-1 flex items-end justify-between gap-4">
								<div class="flex items-center gap-[8mm] mb-[1mm] pl-[14mm]">
									<span class="italic font-bold">Jumlah</span>
									<span>:</span>
									<span class="kw-sorot font-bold italic min-w-[55mm]" :style="{ background: b.warna }">Rp&nbsp;&nbsp;&nbsp;{{ vars.total }},00</span>
								</div>
								<div class="text-[10.5pt] leading-tight min-w-[60mm]">
									<p>{{ vars.kota_toko }}, {{ vars.tanggal_bku }}</p>
									<p>Yang menerima,</p>
									<div :style="{ height: `${b.tinggiTtd || 14}mm` }" />
									<p class="font-bold italic">
										{{ vars.penanggung_jawab }}
									</p>
								</div>
							</div>
						</div>
					</div>
					<div v-if="b.visum !== 'tidak'" class="grid grid-cols-2 text-[10pt] leading-tight" :style="{ marginTop: `${b.jarakVisum ?? 10}mm` }">
						<div v-for="(s, i) in visum" :key="i" class="px-[3mm]" :class="i === 1 ? 'justify-self-end' : ''">
							<p class="whitespace-pre-line">
								{{ s.atas }}
							</p>
							<div :style="{ height: `${b.tinggiTtd || 14}mm` }" />
							<p class="font-bold underline">
								{{ s.nama || "(..............................)" }}
							</p>
							<p v-if="s.nip">
								NIP. {{ s.nip }}
							</p>
						</div>
					</div>
				</div>

				<div v-else-if="b.type === 'spasi'" :style="{ height: `${b.tinggi}mm` }" />
				<hr v-else-if="b.type === 'garis'" class="border-black my-2">
			</template>
		</div>
	</PrintSheet>
</template>

<script lang="ts" setup>
	const props = defineProps<{
		template: TemplateData
		g: NotaGroup
		penyedia: PenyediaData | null
		urut: number
	}>();
	const { pengaturan, sekolah } = useDocContext();

	const vars = computed(() => buildVars({
		g: props.g,
		sekolah: sekolah.value,
		pengaturan: pengaturan.value,
		penyedia: props.penyedia,
		urut: props.urut,
		template: props.template
	}));
	const fill = (text: string) => fillVars(text ?? "", vars.value);

	type KopBlok = Extract<Blok, { type: "kop" }>;
	type TtdBlok = Extract<Blok, { type: "ttd" }>;

	const kopLines = (b: KopBlok): string[] => {
		if (b.sumber === "teks") return fill(b.baris).split("\n").filter((l) => l.trim());
		if (b.sumber === "toko") {
			const custom = props.penyedia?.kop?.trim();
			if (custom) return custom.split("\n").filter((l) => l.trim());
			return [vars.value.nama_toko ?? "", vars.value.alamat_toko ?? "", vars.value.telp_toko ? `Telp. ${vars.value.telp_toko}` : ""].filter(Boolean);
		}
		return [
			pengaturan.value?.kop.pemerintah ?? "",
			pengaturan.value?.kop.dinas ?? "",
			(sekolah.value?.nama ?? "").toUpperCase(),
			`Alamat : ${vars.value.alamat_sekolah}`
		].filter((l) => l.trim());
	};

	/** Baris kop: dua baris pertama tebal, baris nama terbesar, baris alamat kecil. */
	const kopClass = (b: KopBlok, i: number) => {
		const n = kopLines(b).length;
		if (b.sumber === "sekolah") return i === n - 1 ? "text-[9pt]" : i === n - 2 ? "text-[14pt] font-bold" : "text-[12pt] font-bold";
		return i < Math.max(1, n - 2) ? "text-[12pt] font-bold uppercase" : "text-[9pt]";
	};

	const logo = (b: KopBlok) => {
		if (b.logo === "custom") return b.logoData || "";
		if (b.logo === "toko") return props.penyedia?.logo || "";
		return "";
	};

	/** Baris kop toko untuk panel kiri kwitansi. */
	const kopTokoLines = computed(() => {
		const custom = props.penyedia?.kop?.trim();
		if (custom) return custom.split("\n").filter((l) => l.trim());
		return [vars.value.nama_toko ?? "", vars.value.alamat_toko ?? ""].filter(Boolean);
	});

	/** Daftar layanan toko dipecah per kelompok (dipisah baris kosong); baris pertama tiap kelompok = judul. */
	const layananGroups = computed(() => {
		const groups: string[][] = [[]];
		for (const raw of (props.penyedia?.layanan ?? "").split("\n")) {
			const line = raw.trim();
			if (line) groups.at(-1)?.push(line);
			else if (groups.at(-1)?.length) groups.push([]);
		}
		return groups.filter((g) => g.length);
	});

	/** Visum kwitansi: Mengetahui Kepala Sekolah dan Lunas dibayar Bendahara. */
	const visum = computed(() => [
		{ atas: `Mengetahui,\nKepala ${vars.value.nama_sekolah ?? ""}`, nama: vars.value.kepala_sekolah ?? "", nip: vars.value.nip_kepala_sekolah ?? "" },
		{ atas: `Lunas dibayar ${vars.value.tanggal_bayar ?? ""}\nBendahara`, nama: vars.value.bendahara ?? "", nip: vars.value.nip_bendahara ?? "" }
	]);

	const numeric = (k: KolomTabel["kunci"]) => ["volume", "harga", "jumlah", "dipesan", "diterima", "rusak", "sesuai"].includes(k);

	const fmtVol = (v: number | null) => (v === null || v === undefined ? "" : v.toLocaleString("id-ID"));

	const cell = (k: KolomTabel["kunci"], it: NotaItem, i: number, bersih: boolean) => {
		const harga = it.volume && it.volume > 0 ? Math.round(it.nominal / it.volume) : it.hargaSatuan;
		switch (k) {
		case "no": return String(i + 1);
		case "nama": return bersih ? bersihkanNama(it.uraian) : it.uraian;
		case "volume":
		case "dipesan":
		case "diterima":
		case "sesuai": return fmtVol(it.volume);
		case "satuan": return it.satuan ?? "";
		case "volume_satuan": return `${fmtVol(it.volume)} ${it.satuan ?? ""}`.trim();
		case "harga": return harga ? angka(harga) : "";
		case "jumlah": return angka(it.nominal);
		case "rusak": return "-";
		case "kode_rekening": return it.kodeRekening ?? "";
		}
		return "";
	};

	/** "rata": kolom dibagi rata; satu penanda tangan "kanan": separuh kanan; "tengah": di tengah. */
	const ttdCols = (b: TtdBlok) => (b.kolom.length === 1 && b.rata === "kanan" ? 2 : Math.max(1, b.kolom.length));
	const ttdPos = (b: TtdBlok, _i: number) => (b.kolom.length === 1 && b.rata === "kanan" ? "col-start-2" : "");
</script>

<style scoped>
	/* Teks tegak dibaca dari bawah ke atas, seperti panel kiri kwitansi toko */
	.kw-vertical {
		writing-mode: vertical-rl;
		transform: rotate(180deg);
	}
	.kw-logo {
		width: 17mm;
		height: 17mm;
		object-fit: contain;
		transform: rotate(-90deg);
	}
	.kw-fields td {
		padding: 1.6mm 0;
		vertical-align: middle;
	}
	.kw-label {
		width: 52mm;
		font-style: italic;
	}
	.kw-colon {
		width: 6mm;
	}
	/* Kotak miring berwarna untuk terbilang & jumlah */
	.kw-sorot {
		display: inline-block;
		padding: 1.2mm 7mm;
		transform: skewX(-25deg);
		border: 1px solid rgba(0, 0, 0, 0.45);
		-webkit-print-color-adjust: exact;
		print-color-adjust: exact;
	}
</style>
