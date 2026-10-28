<template>
	<LayoutPageShell title="Generator Standing Instruction (SI)">
		<template #actions>
			<UButton
				color="neutral"
				variant="outline"
				icon="i-lucide-file-plus"
				@click="buatDraftBaru"
			>
				Buat SI Baru
			</UButton>
			<UButton
				color="neutral"
				variant="outline"
				icon="i-lucide-save"
				:loading="saving"
				@click="simpanDraft"
			>
				Simpan Draft
			</UButton>
			<UButton
				color="primary"
				variant="solid"
				icon="i-lucide-printer"
				:disabled="form.items.length === 0"
				@click="cetakSi"
			>
				Pratinjau & Cetak
			</UButton>
		</template>

		<!-- Empty state jika ARKAS belum terhubung -->
		<div v-if="!connected" class="py-16 text-center text-muted">
			<UIcon name="i-lucide-database" class="size-10 mx-auto mb-2 opacity-50" />
			<p class="font-medium text-highlighted">
				Database ARKAS Belum Terhubung
			</p>
			<p class="text-sm mt-1">
				Hubungkan ke file database ARKAS terlebih dahulu untuk menggunakan generator Standing Instruction.
			</p>
		</div>

		<!-- Main Workspace -->
		<div v-else class="space-y-4 max-w-6xl pb-16">
			<!-- Toolbar Riwayat Draft SI -->
			<div v-if="daftarDraft.length > 0" class="flex flex-wrap items-center gap-2 p-3 bg-card rounded-xl border border-default">
				<span class="text-xs font-semibold text-highlighted flex items-center gap-1.5">
					<UIcon name="i-lucide-history" class="size-4 text-muted" />
					Draft Tersimpan:
				</span>
				<div class="flex flex-wrap items-center gap-1.5">
					<UButton
						v-for="d in daftarDraft"
						:key="d.id"
						size="xs"
						:color="activeDraftId === d.id ? 'primary' : 'neutral'"
						:variant="activeDraftId === d.id ? 'solid' : 'outline'"
						@click="pilihDraft(d.id)"
					>
						{{ d.judul || d.nomorSurat || 'Draft SI' }}
					</UButton>
				</div>
				<UButton
					v-if="daftarDraft.length > 0"
					size="xs"
					color="error"
					variant="ghost"
					icon="i-lucide-trash-2"
					class="ml-auto"
					title="Hapus draft yang sedang aktif"
					@click="hapusDraftAktif"
				>
					Hapus Draft
				</UButton>
			</div>

			<div class="grid grid-cols-1 lg:grid-cols-2 gap-4">
				<!-- KARTU 1: DATA SURAT & BANK TUJUAN -->
				<UCard>
					<template #header>
						<div class="flex items-center gap-2">
							<UIcon name="i-lucide-file-text" class="size-4 text-primary" />
							<p class="font-semibold text-sm text-highlighted">
								Data Surat & Bank Tujuan
							</p>
						</div>
					</template>

					<div class="space-y-3">
						<div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
							<UFormField label="Nomor Surat" required>
								<UInput v-model="form.nomorSurat" placeholder="400.3.5.5/05/IX/SDN.../2026/M" class="w-full" />
							</UFormField>
							<UFormField label="Tanggal Surat" required>
								<UInput v-model="form.tanggal" type="date" class="w-full" />
							</UFormField>
						</div>

						<div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
							<UFormField label="Kota / Kecamatan Penerbitan">
								<UInput v-model="form.kota" placeholder="Rajagaluh" class="w-full" />
							</UFormField>
							<UFormField label="Bank Tujuan Pemindahbukuan" required>
								<UInput v-model="form.bankTujuan" placeholder="Bank BJB Cabang / KCP Rajagaluh" class="w-full" />
							</UFormField>
						</div>

						<div class="grid grid-cols-3 gap-2">
							<UFormField label="Sifat">
								<UInput v-model="form.sifat" placeholder="Biasa" class="w-full" />
							</UFormField>
							<UFormField label="Lampiran">
								<UInput v-model="form.lampiran" placeholder="-" class="w-full" />
							</UFormField>
							<UFormField label="Perihal">
								<UInput v-model="form.perihal" placeholder="Pemindah bukuan/Transfer" class="w-full" />
							</UFormField>
						</div>
					</div>
				</UCard>

				<!-- KARTU 2: REKENING SEKOLAH & PENANDATANGAN -->
				<UCard>
					<template #header>
						<div class="flex items-center gap-2">
							<UIcon name="i-lucide-landmark" class="size-4 text-primary" />
							<p class="font-semibold text-sm text-highlighted">
								Rekening Sumber (Sekolah) & Penandatangan
							</p>
						</div>
					</template>

					<div class="space-y-3">
						<div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
							<UFormField label="Atas Nama Rekening Sekolah" required>
								<UInput v-model="form.namaRekeningSumber" placeholder="20246133 SD NEGERI ..." class="w-full" />
							</UFormField>
							<UFormField label="No. Rekening Sekolah (Giro/BJB)" required>
								<UInput v-model="form.noRekeningSumber" placeholder="0120292021100" class="w-full font-mono" />
							</UFormField>
						</div>

						<UFormField label="Uraian Debet Rekening">
							<UInput v-model="form.uraianSumber" placeholder="Belanja Barang dan Jasa / Makanan dan Minuman" class="w-full" />
						</UFormField>

						<!-- Pihak 1: Kepala Sekolah -->
						<div class="p-2.5 rounded-lg border border-default bg-elevated/40 space-y-2">
							<p class="text-[11px] font-bold text-highlighted uppercase tracking-wider">
								Pihak 1: Kepala Sekolah
							</p>
							<div class="grid grid-cols-1 sm:grid-cols-3 gap-2">
								<UFormField label="Nama Kepala Sekolah">
									<UInput v-model="form.namaKepalaSekolah" placeholder="Nama Kepala Sekolah" class="w-full" />
								</UFormField>
								<UFormField label="NIP Kepala Sekolah">
									<UInput v-model="form.nipKepalaSekolah" placeholder="NIP..." class="w-full font-mono" />
								</UFormField>
								<UFormField label="No. KTP / NIK">
									<UInput v-model="form.ktpKepalaSekolah" placeholder="3210..." class="w-full font-mono" />
								</UFormField>
							</div>
						</div>

						<!-- Pihak 2: Bendahara -->
						<div class="p-2.5 rounded-lg border border-default bg-elevated/40 space-y-2">
							<p class="text-[11px] font-bold text-highlighted uppercase tracking-wider">
								Pihak 2: Bendahara BOS
							</p>
							<div class="grid grid-cols-1 sm:grid-cols-3 gap-2">
								<UFormField label="Nama Bendahara">
									<UInput v-model="form.namaBendahara" placeholder="Nama Bendahara" class="w-full" />
								</UFormField>
								<UFormField label="NIP Bendahara">
									<UInput v-model="form.nipBendahara" placeholder="NIP..." class="w-full font-mono" />
								</UFormField>
								<UFormField label="No. KTP / NIK">
									<UInput v-model="form.ktpBendahara" placeholder="3210..." class="w-full font-mono" />
								</UFormField>
							</div>
						</div>
					</div>
				</UCard>
			</div>

			<!-- KARTU 3: RINCIAN PENERIMA TRANSFER -->
			<UCard>
				<template #header>
					<div class="flex flex-wrap items-center justify-between gap-2">
						<div class="flex items-center gap-2">
							<UIcon name="i-lucide-receipt" class="size-4 text-primary" />
							<div>
								<p class="font-semibold text-sm text-highlighted">
									Rincian Penerima Pemindahbukuan / Transfer
								</p>
								<p class="text-xs text-muted">
									Daftar toko/rekanan belanja non-SIPLah (mamin, fotokopi, penyedia lokal).
								</p>
							</div>
						</div>

						<div class="flex items-center gap-2">
							<UButton
								icon="i-lucide-calendar-check"
								color="neutral"
								variant="outline"
								size="sm"
								@click="bukaModalArkas"
							>
								Pilih dari ARKAS Bulanan
							</UButton>
							<UButton
								icon="i-lucide-plus"
								color="primary"
								variant="solid"
								size="sm"
								@click="tambahBarisManual"
							>
								Tambah Baris Manual
							</UButton>
						</div>
					</div>
				</template>

				<!-- Empty State jika belum ada item -->
				<div v-if="form.items.length === 0" class="py-12 text-center text-muted">
					<UIcon name="i-lucide-file-spreadsheet" class="size-10 mx-auto mb-2 opacity-50" />
					<p class="font-medium text-highlighted">
						Belum Ada Penerima Transfer
					</p>
					<p class="text-xs mt-1 max-w-md mx-auto">
						Klik <strong>"Pilih dari ARKAS Bulanan"</strong> untuk mengambil rencana belanja bulanan dari RKAS atau <strong>"Tambah Baris Manual"</strong> untuk menginput data rekening toko/rekanan secara langsung.
					</p>
				</div>

				<!-- Tabel Item Transfer yang Bisa Diedit Langsung -->
				<div v-else class="space-y-3">
					<div class="border border-default rounded-lg overflow-x-auto">
						<table class="w-full text-xs">
							<thead class="bg-elevated/70 border-b border-default font-semibold text-muted">
								<tr>
									<th class="py-2 px-2.5 text-center w-8">
										No
									</th>
									<th class="py-2 px-2.5 text-left min-w-44">
										Atas Nama Rekening Penerima
									</th>
									<th class="py-2 px-2.5 text-center w-28">
										Bank
									</th>
									<th class="py-2 px-2.5 text-left w-36">
										No. Rekening
									</th>
									<th class="py-2 px-2.5 text-right w-36">
										Jumlah Uang (Rp)
									</th>
									<th class="py-2 px-2.5 text-right w-28">
										Pajak (Rp)
									</th>
									<th class="py-2 px-2.5 text-right w-36">
										Diterima (Rp)
									</th>
									<th class="py-2 px-2.5 text-left min-w-48">
										Keterangan Belanja
									</th>
									<th class="py-2 px-2 text-center w-10" />
								</tr>
							</thead>
							<tbody class="divide-y divide-default">
								<tr v-for="(item, idx) in form.items" :key="item.id" class="hover:bg-neutral-500/5">
									<td class="py-1.5 px-2.5 text-center font-medium text-muted align-top">
										{{ idx + 1 }}
									</td>
									<!-- Atas Nama Rekening -->
									<td class="py-1.5 px-2.5 align-top">
										<UInput
											v-model="item.namaPenerima"
											size="xs"
											placeholder="Contoh: KPRI KPR RAJAGALUH / Toko Mamin"
											class="w-full font-medium"
										/>
									</td>
									<!-- Nama Bank -->
									<td class="py-1.5 px-2.5 align-top">
										<UInput
											v-model="item.namaBank"
											size="xs"
											placeholder="BJB"
											class="w-full text-center font-mono uppercase"
										/>
									</td>
									<!-- No Rekening -->
									<td class="py-1.5 px-2.5 align-top">
										<UInput
											v-model="item.noRekening"
											size="xs"
											placeholder="0101618528100"
											class="w-full font-mono"
										/>
									</td>
									<!-- Jumlah Uang -->
									<td class="py-1.5 px-2.5 align-top">
										<UInput
											:model-value="item.jumlah"
											size="xs"
											type="number"
											placeholder="0"
											class="w-full text-right font-mono"
											@update:model-value="(v) => item.jumlah = Number(v) || 0"
										/>
									</td>
									<!-- Pajak -->
									<td class="py-1.5 px-2.5 align-top">
										<UInput
											:model-value="item.pajak"
											size="xs"
											type="number"
											placeholder="0"
											class="w-full text-right font-mono"
											@update:model-value="(v) => item.pajak = Number(v) || 0"
										/>
									</td>
									<!-- Diterima (Kalkulasi) -->
									<td class="py-1.5 px-2.5 text-right font-mono font-bold text-highlighted align-middle tabular-nums whitespace-nowrap">
										{{ rupiah(item.jumlah - (item.pajak || 0)) }}
									</td>
									<!-- Keterangan -->
									<td class="py-1.5 px-2.5 align-top">
										<UInput
											v-model="item.keterangan"
											size="xs"
											placeholder="Fotocopy & Jilid Modul Ajar / Mamin Rapat"
											class="w-full"
										/>
									</td>
									<!-- Tombol Hapus Baris -->
									<td class="py-1.5 px-2 text-center align-middle">
										<UButton
											icon="i-lucide-trash-2"
											color="error"
											variant="ghost"
											size="xs"
											@click="hapusBaris(idx)"
										/>
									</td>
								</tr>
							</tbody>
						</table>
					</div>

					<!-- Ringkasan Angka Bawah -->
					<div class="grid grid-cols-1 sm:grid-cols-3 gap-3 p-3 rounded-lg bg-elevated/50 border border-default">
						<div>
							<p class="text-xs text-muted">
								Total Kotor
							</p>
							<p class="text-base font-bold text-highlighted font-mono tabular-nums">
								{{ rupiah(totalKotor) }}
							</p>
						</div>
						<div>
							<p class="text-xs text-muted">
								Total Potongan Pajak
							</p>
							<p class="text-base font-bold text-highlighted font-mono tabular-nums">
								{{ rupiah(totalPajak) }}
							</p>
						</div>
						<div>
							<p class="text-xs text-muted">
								Total Bersih Yang Diterima
							</p>
							<p class="text-base font-bold text-primary font-mono tabular-nums">
								{{ rupiah(totalBersih) }}
							</p>
						</div>
					</div>

					<!-- Terbilang -->
					<div class="p-2.5 border border-default rounded-lg bg-card/60 text-xs italic text-muted">
						<strong class="text-highlighted not-italic">Terbilang:</strong> {{ terbilangTeks }}
					</div>
				</div>
			</UCard>
		</div>

		<!-- MODAL: PILIH RENCANA BELANJA ARKAS BULANAN -->
		<UModal v-model:open="modalArkas" title="Pilih Rencana Belanja ARKAS Bulanan" class="max-w-4xl">
			<template #body>
				<div class="space-y-3">
					<div class="flex flex-wrap items-center gap-2">
						<!-- Pemilih Bulan RKAS -->
						<USelect
							v-model="filterBulanArkas"
							:items="bulanOptions"
							icon="i-lucide-calendar"
							class="w-44"
							aria-label="Pilih Bulan Belanja RKAS"
						/>

						<!-- Pemilih Sumber Dana -->
						<USelect
							v-model="filterFundArkas"
							:items="fundOptions"
							icon="i-lucide-wallet"
							class="w-48"
							aria-label="Filter Sumber Dana"
						/>

						<!-- Filter Belum Dibelanjakan -->
						<div class="flex items-center gap-2 px-2.5 py-1.5 border border-default rounded-md bg-neutral-500/5">
							<USwitch
								v-model="filterBelumDibelanjakan"
								size="sm"
							/>
							<span class="text-xs font-medium cursor-pointer" @click="filterBelumDibelanjakan = !filterBelumDibelanjakan">
								Belum Dibelanjakan Saja
							</span>
						</div>

						<!-- Pencarian -->
						<UInput
							v-model="searchArkas"
							icon="i-lucide-search"
							size="sm"
							placeholder="Cari uraian belanja / rekening..."
							class="w-56 ml-auto"
						/>
					</div>

					<div class="flex items-center justify-between text-xs text-muted pt-1">
						<span>
							Ditemukan <strong>{{ filteredArkasItems.length }}</strong> item belanja di bulan {{ BULAN[filterBulanArkas - 1] }}
						</span>
						<div class="flex items-center gap-2">
							<UButton
								size="xs"
								color="neutral"
								variant="ghost"
								@click="pilihSemuaArkas"
							>
								Pilih Semua
							</UButton>
							<span class="text-muted">·</span>
							<UButton
								size="xs"
								color="neutral"
								variant="ghost"
								@click="batalPilihSemuaArkas"
							>
								Batal Pilih
							</UButton>
						</div>
					</div>

					<!-- List Rencana Belanja ARKAS -->
					<div class="max-h-96 overflow-y-auto border border-default rounded-lg divide-y divide-default">
						<div
							v-for="b in filteredArkasItems"
							:key="b.idRapbs"
							class="p-2.5 flex items-start gap-3 hover:bg-neutral-500/5 cursor-pointer text-xs transition-colors"
							@click="toggleArkasRow(b)"
						>
							<UCheckbox
								:model-value="selectedArkasIds.has(b.idRapbs)"
								class="mt-1"
								@click.stop="toggleArkasRow(b)"
							/>
							<div class="min-w-0 flex-1">
								<p class="font-semibold text-highlighted">
									{{ b.uraian }}
								</p>
								<div class="flex flex-wrap items-center gap-2 mt-1 text-[11px] text-muted font-mono">
									<span v-if="b.kodeRekening">{{ b.kodeRekening }}</span>
									<span>·</span>
									<span class="font-sans">Volume: <strong class="text-highlighted">{{ formatVolume(b.volumeBulan[filterBulanArkas - 1] || b.volume) }} {{ b.satuan || 'unit' }}</strong></span>
									<span>·</span>
									<span class="font-sans">Harga Satuan: <strong class="text-highlighted">{{ rupiah(b.hargaSatuan) }}</strong></span>
									<span>·</span>
									<UBadge color="neutral" variant="subtle" size="xs">
										{{ b.fundName }}
									</UBadge>
									<UBadge
										v-if="statusBelanjaMap.get(b.idRapbs)"
										:color="statusBelanjaMap.get(b.idRapbs)?.status === 'lunas' ? 'success' : statusBelanjaMap.get(b.idRapbs)?.status === 'sebagian' ? 'info' : 'warning'"
										variant="subtle"
										size="xs"
									>
										{{ statusBelanjaMap.get(b.idRapbs)?.label }}
									</UBadge>
								</div>
							</div>
							<div class="text-right shrink-0">
								<p class="font-mono font-bold text-highlighted text-sm tabular-nums">
									{{ rupiah(sisaAnggaranMap.get(b.idRapbs) ?? (b.bulan[filterBulanArkas - 1] ?? 0)) }}
								</p>
								<p class="text-[10px] text-muted">
									<span v-if="(sisaAnggaranMap.get(b.idRapbs) ?? 0) < (b.bulan[filterBulanArkas - 1] ?? 0)">Sisa Belum Dibelanjakan</span>
									<span v-else>Pagu Bulan {{ BULAN[filterBulanArkas - 1] }}</span>
								</p>
							</div>
						</div>

						<p v-if="filteredArkasItems.length === 0" class="text-center py-10 text-xs text-muted">
							Tidak ada rencana belanja yang dianggarkan pada bulan {{ BULAN[filterBulanArkas - 1] }} untuk filter ini.
						</p>
					</div>
				</div>
			</template>

			<template #footer>
				<div class="flex items-center justify-between w-full">
					<div class="text-xs">
						<span class="font-medium text-highlighted">{{ selectedArkasIds.size }} item dipilih</span>
						<span v-if="selectedArkasIds.size > 0" class="text-muted ml-2">
							(Total: <strong class="text-highlighted font-mono">{{ rupiah(totalNominalTerpilih) }}</strong>)
						</span>
					</div>
					<div class="flex items-center gap-2">
						<UButton color="neutral" variant="ghost" size="sm" @click="modalArkas = false">
							Batal
						</UButton>
						<UButton
							color="primary"
							variant="solid"
							size="sm"
							:disabled="selectedArkasIds.size === 0"
							@click="imporTransaksiArkas"
						>
							Masukkan ke Rincian SI
						</UButton>
					</div>
				</div>
			</template>
		</UModal>
	</LayoutPageShell>
</template>

<script lang="ts" setup>
	import type { SiData } from "~/components/Print/Si/StandingInstruction.vue";
	import type { RkasItem } from "~/utils/types";
	import StandingInstruction from "~/components/Print/Si/StandingInstruction.vue";

	interface DraftItemRecord {
		id: string
		judul: string
		nomorSurat: string
		updatedAt: string
		data: SiData
	}

	const { connected, year, funds } = useArkas();
	const { open: openPrint } = usePrint();
	const toast = useToast();
	const { pengaturan, sekolah, load: loadDoc } = useDocContext();

	const saving = ref(false);
	const activeDraftId = ref<string>("");
	const daftarDraft = ref<DraftItemRecord[]>([]);

	// Form SI
	const form = ref<SiData>({
		nomorSurat: "",
		sifat: "Biasa",
		lampiran: "-",
		perihal: "Pemindah bukuan/Transfer",
		tanggal: new Date().toISOString().slice(0, 10),
		kota: "",
		bankTujuan: "Bank BJB Cabang / KCP",
		namaRekeningSumber: "",
		noRekeningSumber: "",
		uraianSumber: "Belanja Barang dan Jasa",
		namaKepalaSekolah: "",
		nipKepalaSekolah: "",
		ktpKepalaSekolah: "",
		namaBendahara: "",
		nipBendahara: "",
		ktpBendahara: "",
		items: []
	});

	// State Modal Rencana Belanja ARKAS Bulanan
	const modalArkas = ref(false);
	const filterBulanArkas = ref<number>(new Date().getMonth() + 1);
	const filterFundArkas = ref<number>(0);
	const filterBelumDibelanjakan = ref(true);
	const searchArkas = ref("");
	const rkasItems = ref<RkasItem[]>([]);
	const realisasiData = ref<Realisasi | null>(null);
	const selectedArkasIds = ref<Set<string>>(new Set());

	// Opsi Bulan
	const bulanOptions = BULAN.map((nama, idx) => ({
		label: `${idx + 1} - ${nama}`,
		value: idx + 1
	}));

	// Opsi Sumber Dana
	const fundOptions = computed(() => [
		{ label: "Semua Sumber Dana", value: 0 },
		...funds.value.map((f) => ({
			label: f.name,
			value: f.id
		}))
	]);

	// Format Volume
	const formatVolume = (vol: number): string => {
		if (Number.isInteger(vol)) return String(vol);
		return vol.toLocaleString("id-ID", { maximumFractionDigits: 2 });
	};

	// Perhitungan Total
	const totalKotor = computed(() => form.value.items.reduce((acc, it) => acc + (it.jumlah || 0), 0));
	const totalPajak = computed(() => form.value.items.reduce((acc, it) => acc + (it.pajak || 0), 0));
	const totalBersih = computed(() => totalKotor.value - totalPajak.value);

	const terbilangTeks = computed(() => {
		if (totalBersih.value <= 0) return "Nol rupiah";
		const str = terbilang(totalBersih.value);
		return `${str.charAt(0).toUpperCase() + str.slice(1)} rupiah`;
	});

	// Tambah Baris Manual
	const tambahBarisManual = () => {
		form.value.items.push({
			id: `item-${Date.now()}-${Math.random().toString(36).slice(2, 7)}`,
			namaPenerima: "",
			namaBank: "BJB",
			noRekening: "",
			jumlah: 0,
			pajak: 0,
			keterangan: ""
		});
	};

	// Hapus Baris
	const hapusBaris = (idx: number) => {
		form.value.items.splice(idx, 1);
	};

	// Map Realisasi per item RAPBS dari data realisasi ARKAS
	const realisasiItemMap = computed(() => {
		const map = new Map<string, RealisasiItem>();
		if (realisasiData.value?.items) {
			for (const item of realisasiData.value.items) {
				map.set(item.idRapbs, item);
			}
		}
		return map;
	});

	// Status belanja dan sisa belanja per bulan
	const statusBelanjaMap = computed(() => {
		const map = new Map<string, { status: "belum" | "sebagian" | "lunas", label: string }>();
		const mIdx = filterBulanArkas.value - 1;

		for (const it of rkasItems.value) {
			const rel = realisasiItemMap.value.get(it.idRapbs);
			const paguBulan = it.bulan[mIdx] ?? 0;
			const spentBulan = rel?.realisasi[mIdx] ?? 0;

			if (paguBulan <= 0) continue;

			if (spentBulan >= paguBulan) {
				map.set(it.idRapbs, { status: "lunas", label: "Lunas" });
			} else if (spentBulan > 0) {
				map.set(it.idRapbs, { status: "sebagian", label: `Sebagian (${rupiah(spentBulan)})` });
			} else {
				map.set(it.idRapbs, { status: "belum", label: "Belum Dibelanjakan" });
			}
		}
		return map;
	});

	// Nominal sisa anggaran yang belum dibelanjakan di bulan terpilih
	const sisaAnggaranMap = computed(() => {
		const map = new Map<string, number>();
		const mIdx = filterBulanArkas.value - 1;

		for (const it of rkasItems.value) {
			const rel = realisasiItemMap.value.get(it.idRapbs);
			const paguBulan = it.bulan[mIdx] ?? 0;
			const spentBulan = rel?.realisasi[mIdx] ?? 0;
			const sisa = Math.max(0, paguBulan - spentBulan);
			map.set(it.idRapbs, sisa);
		}
		return map;
	});

	// Filter Item Belanja ARKAS Bulanan
	const filteredArkasItems = computed(() => {
		const mIdx = filterBulanArkas.value - 1;
		const fId = filterFundArkas.value;
		const q = searchArkas.value.trim().toLowerCase();

		return rkasItems.value.filter((it) => {
			// Hanya item yang ada pagu/volume di bulan terpilih
			const paguBulan = it.bulan[mIdx] ?? 0;
			const volBulan = it.volumeBulan[mIdx] ?? 0;
			if (paguBulan <= 0 && volBulan <= 0) return false;

			// Filter sumber dana
			if (fId !== 0 && it.fundId !== fId) return false;

			// Filter status belanja jika toggle "Belum Dibelanjakan Saja" aktif
			if (filterBelumDibelanjakan.value) {
				const statusObj = statusBelanjaMap.value.get(it.idRapbs);
				const sisa = sisaAnggaranMap.value.get(it.idRapbs) ?? paguBulan;
				// Jangan tampilkan jika sudah lunas atau sisa anggarannya 0
				if (statusObj?.status === "lunas" || sisa <= 0) return false;
			}

			// Pencarian
			if (q) {
				const matchUraian = it.uraian.toLowerCase().includes(q);
				const matchRekening = it.kodeRekening?.toLowerCase().includes(q) ?? false;
				if (!matchUraian && !matchRekening) return false;
			}

			return true;
		});
	});

	// Toggle pilih item ARKAS
	const toggleArkasRow = (row: RkasItem) => {
		if (selectedArkasIds.value.has(row.idRapbs)) {
			selectedArkasIds.value.delete(row.idRapbs);
		} else {
			selectedArkasIds.value.add(row.idRapbs);
		}
	};

	// Pilih Semua
	const pilihSemuaArkas = () => {
		for (const it of filteredArkasItems.value) {
			selectedArkasIds.value.add(it.idRapbs);
		}
	};

	// Batal Pilih Semua
	const batalPilihSemuaArkas = () => {
		selectedArkasIds.value.clear();
	};

	// Total Nominal Terpilih
	const totalNominalTerpilih = computed(() => {
		const mIdx = filterBulanArkas.value - 1;
		return rkasItems.value
			.filter((it) => selectedArkasIds.value.has(it.idRapbs))
			.reduce((acc, it) => {
				const sisa = sisaAnggaranMap.value.get(it.idRapbs);
				const nominal = sisa !== undefined ? sisa : (it.bulan[mIdx] || 0);
				return acc + nominal;
			}, 0);
	});

	// Muat Data Kertas Kerja ARKAS & Realisasi
	const muatArkas = async () => {
		if (!connected.value || !year.value) return;
		try {
			const [kk, real] = await Promise.all([
				api.kertasKerja(year.value, null),
				api.realisasi(year.value, 12, null).catch(() => null)
			]);
			rkasItems.value = kk.items;
			realisasiData.value = real;
		} catch (err) {
			console.error("Gagal memuat rencana belanja dan realisasi ARKAS:", err);
		}
	};

	// Buka Modal ARKAS
	const bukaModalArkas = async () => {
		modalArkas.value = true;
		if (rkasItems.value.length === 0 || !realisasiData.value) {
			await muatArkas();
		}
	};

	// Impor dari Rencana Belanja ARKAS yang dipilih
	const imporTransaksiArkas = () => {
		const mIdx = filterBulanArkas.value - 1;
		const bulanAngka = filterBulanArkas.value;
		const itemsToImport = rkasItems.value.filter((it) => selectedArkasIds.value.has(it.idRapbs));

		for (const it of itemsToImport) {
			const paguBulan = it.bulan[mIdx] ?? 0;
			const sisaBulan = sisaAnggaranMap.value.get(it.idRapbs);
			const nominalBelanja = (sisaBulan !== undefined && sisaBulan > 0)
				? sisaBulan
				: (paguBulan > 0 ? paguBulan : (it.hargaSatuan * Math.max(1, Math.round(it.volumeBulan[mIdx] || it.volume))));

			form.value.items.push({
				id: `arkas-${it.idRapbs}-${bulanAngka}-${Date.now()}-${Math.random().toString(36).slice(2, 6)}`,
				namaPenerima: "",
				namaBank: "BJB",
				noRekening: "",
				jumlah: nominalBelanja,
				pajak: 0,
				keterangan: it.uraian
			});
		}

		selectedArkasIds.value.clear();
		modalArkas.value = false;
		toast.add({
			title: "Belanja ARKAS Dimasukkan",
			description: `Berhasil menambahkan ${itemsToImport.length} item rencana belanja bulan ${BULAN[mIdx]} ke rincian SI.`,
			color: "success"
		});
	};

	// Inisialisasi data form default dari sekolah & pejabat
	const inisialisasiFormDefault = () => {
		const sek = sekolah.value;
		const pej = pengaturan.value?.pejabat;

		form.value = {
			nomorSurat: "",
			sifat: "Biasa",
			lampiran: "-",
			perihal: "Pemindah bukuan/Transfer",
			tanggal: new Date().toISOString().slice(0, 10),
			kota: sek?.kecamatan || "Rajagaluh",
			bankTujuan: `Bank BJB Cabang / KCP ${sek?.kecamatan || "Rajagaluh"}`,
			namaRekeningSumber: sek?.nama ? `${sek?.npsn || ""} ${sek.nama}`.trim() : "",
			noRekeningSumber: "",
			uraianSumber: "Belanja Barang dan Jasa",
			namaKepalaSekolah: pej?.kepalaSekolah || sek?.kepalaSekolah || "",
			nipKepalaSekolah: pej?.nipKepalaSekolah || sek?.nipKepalaSekolah || "",
			ktpKepalaSekolah: "",
			namaBendahara: pej?.bendahara || sek?.bendahara || "",
			nipBendahara: pej?.nipBendahara || sek?.nipBendahara || "",
			ktpBendahara: "",
			items: []
		};
		activeDraftId.value = `draft-${Date.now()}`;
	};

	// Pilih Draft
	const pilihDraft = (id: string) => {
		const d = daftarDraft.value.find((item) => item.id === id);
		if (d) {
			activeDraftId.value = d.id;
			form.value = JSON.parse(JSON.stringify(d.data));
		}
	};

	// Buat Draft Baru
	const buatDraftBaru = () => {
		inisialisasiFormDefault();
		toast.add({
			title: "Draft Baru Dibuat",
			description: "Silakan isi rincian instruksi pemindahbukuan.",
			color: "info"
		});
	};

	// Muat Draft Tersimpan dari Settings
	const storageKey = computed(() => `si_generator_drafts_${year.value}`);

	const muatDraft = async () => {
		if (!connected.value || !year.value) return;
		try {
			const list = await api.settingsGet<DraftItemRecord[]>(storageKey.value);
			if (list && Array.isArray(list)) {
				daftarDraft.value = list;
				if (daftarDraft.value.length > 0) {
					const first = daftarDraft.value[0];
					if (first) {
						pilihDraft(first.id);
					}
					return;
				}
			}
		} catch (err) {
			console.error("Gagal memuat riwayat SI:", err);
		}

		// Jika belum ada draft, inisialisasi default
		inisialisasiFormDefault();
	};

	// Simpan Draft
	const simpanDraft = async () => {
		if (!connected.value || !year.value) return;
		saving.value = true;
		try {
			if (!activeDraftId.value) {
				activeDraftId.value = `draft-${Date.now()}`;
			}

			const title = form.value.nomorSurat ? `SI: ${form.value.nomorSurat}` : `SI ${form.value.kota || "Bank"} (${tanggalId(form.value.tanggal)})`;
			const record: DraftItemRecord = {
				id: activeDraftId.value,
				judul: title,
				nomorSurat: form.value.nomorSurat,
				updatedAt: new Date().toISOString(),
				data: JSON.parse(JSON.stringify(form.value))
			};

			const idx = daftarDraft.value.findIndex((d) => d.id === activeDraftId.value);
			if (idx >= 0) {
				daftarDraft.value[idx] = record;
			} else {
				daftarDraft.value.unshift(record);
			}

			await api.settingsSet(storageKey.value, daftarDraft.value);
			toast.add({
				title: "Draft Tersimpan",
				description: "Data Standing Instruction berhasil disimpan ke database lokal SIBOS.",
				color: "success"
			});
		} catch (err) {
			toast.add({
				title: "Gagal Menyimpan",
				description: errorMessage(err),
				color: "error"
			});
		} finally {
			saving.value = false;
		}
	};

	// Hapus Draft Aktif
	const hapusDraftAktif = async () => {
		if (!activeDraftId.value) return;
		daftarDraft.value = daftarDraft.value.filter((d) => d.id !== activeDraftId.value);
		await api.settingsSet(storageKey.value, daftarDraft.value);
		if (daftarDraft.value.length > 0) {
			const first = daftarDraft.value[0];
			if (first) {
				pilihDraft(first.id);
			}
		} else {
			inisialisasiFormDefault();
		}
		toast.add({
			title: "Draft Dihapus",
			description: "Draft SI aktif berhasil dihapus.",
			color: "neutral"
		});
	};

	// Pratinjau & Cetak SI
	const cetakSi = async () => {
		await simpanDraft();
		openPrint({
			title: `Standing Instruction - ${form.value.nomorSurat || "Bank"}`,
			component: StandingInstruction,
			props: {
				si: form.value
			}
		});
	};

	// Lifecycle
	onMounted(async () => {
		await loadDoc();
		await muatDraft();
		await muatArkas();
	});

	watch([year, connected], async () => {
		await loadDoc();
		await muatDraft();
		await muatArkas();
	});
</script>
