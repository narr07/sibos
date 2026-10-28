<template>
	<LayoutPageShell title="Belanja">
		<template #actions>
			<UButton
				color="neutral"
				variant="outline"
				icon="i-lucide-store"
				to="/penganggaran/siplah"
			>
				Kelola Toko SIPLah
			</UButton>
			<UButton
				color="primary"
				variant="solid"
				icon="i-lucide-file-spreadsheet"
				:loading="busyExport"
				:disabled="!matchReport || rows.length === 0"
				@click="eksporExcel"
			>
				Export Excel
			</UButton>
		</template>

		<!-- Empty state: ARKAS belum terhubung -->
		<div v-if="!connected" class="py-16 text-center text-muted">
			<UIcon name="i-lucide-database" class="size-10 mx-auto mb-2 opacity-50" />
			<p class="font-medium text-highlighted">
				Database ARKAS Belum Terhubung
			</p>
			<p class="text-sm mt-1">
				Hubungkan ke file database ARKAS terlebih dahulu untuk melihat daftar rencana belanja RKAS.
			</p>
		</div>

		<!-- Empty state: Belum ada toko tersimpan -->
		<div v-else-if="!loadingToko && tokoList.length === 0" class="py-16 text-center text-muted">
			<UIcon name="i-lucide-store" class="size-10 mx-auto mb-2 opacity-50" />
			<p class="font-medium text-highlighted">
				Belum Ada Toko SIPLah Tersimpan
			</p>
			<p class="text-sm mt-1 max-w-md mx-auto">
				Simpan toko SIPLah (contoh: Toko Ladang) terlebih dahulu agar sistem dapat mencocokkan barang belanja bulanan Anda.
			</p>
			<div class="mt-4">
				<UButton
					to="/penganggaran/siplah"
					color="primary"
					variant="solid"
					icon="i-lucide-plus"
				>
					Tambah Toko SIPLah
				</UButton>
			</div>
		</div>

		<!-- Main Content -->
		<div v-else class="flex min-h-0 flex-1 flex-col gap-4">
			<!-- Header Info Toko Terpilih & Link Toko -->
			<div v-if="selectedToko" class="p-3.5 border border-default rounded-xl bg-elevated/40 flex flex-wrap items-center justify-between gap-3">
				<div class="flex items-center gap-3 min-w-0">
					<div class="size-12 rounded-lg bg-neutral-500/10 flex items-center justify-center shrink-0 overflow-hidden border border-default">
						<img
							v-if="selectedToko.gambar"
							:src="selectedToko.gambar"
							:alt="selectedToko.nama"
							class="size-full object-cover"
							@error="handleImgError"
						>
						<UIcon v-else name="i-lucide-store" class="size-6 text-muted" />
					</div>
					<div class="min-w-0">
						<div class="flex flex-wrap items-center gap-2">
							<h2 class="text-base font-bold text-highlighted truncate">
								{{ selectedToko.nama }}
							</h2>
							<UBadge v-if="selectedToko.tipe" color="neutral" variant="subtle" size="xs">
								{{ selectedToko.tipe }}
							</UBadge>
						</div>
						<p class="text-xs text-muted mt-0.5 truncate">
							{{ [selectedToko.kecamatan, selectedToko.kabupaten, selectedToko.provinsi].filter(Boolean).join(", ") || 'Lokasi toko tidak tercantum' }}
							· Total produk tersimpan: <strong class="text-highlighted">{{ selectedToko.totalProduk }}</strong>
						</p>
					</div>
				</div>

				<div class="flex flex-wrap items-center gap-2">
					<!-- LINK TOKO KE SIPLAH -->
					<UButton
						icon="i-lucide-external-link"
						color="primary"
						variant="soft"
						size="sm"
						:to="selectedToko.url"
						target="_blank"
						title="Buka halaman toko di SIPLah"
					>
						Kunjungi Toko di SIPLah
					</UButton>
					<UButton
						icon="i-lucide-refresh-cw"
						color="neutral"
						variant="outline"
						size="sm"
						:loading="syncing || loadingMatch"
						title="Sinkronkan ulang katalog produk toko dari SIPLah"
						@click="sinkronkanToko"
					>
						Sinkronkan Produk
					</UButton>
				</div>
			</div>

			<!-- Toolbar Filter -->
			<div class="flex flex-wrap items-center gap-2.5 bg-card/60 p-3 rounded-lg border border-default">
				<!-- Pemilih Bulan -->
				<USelect
					v-model="selectedMonth"
					:items="bulanOptions"
					icon="i-lucide-calendar"
					class="w-44"
					aria-label="Pilih Bulan Belanja"
				/>

				<!-- Pemilih Toko -->
				<USelect
					v-model="selectedTokoId"
					:items="tokoOptions"
					icon="i-lucide-store"
					class="w-56"
					aria-label="Pilih Toko SIPLah"
				/>

				<!-- Pemilih Sumber Dana -->
				<USelect
					v-model="selectedFund"
					:items="fundOptions"
					icon="i-lucide-wallet"
					class="w-48"
					aria-label="Filter Sumber Dana"
				/>

				<!-- Filter Status Kecocokan -->
				<USelect
					v-model="filterStatus"
					:items="statusOptions"
					class="w-44"
					aria-label="Filter Status Kecocokan"
				/>

				<!-- Toggle Hanya yang Belum Dibelanjakan -->
				<div class="flex items-center gap-2 px-2 py-1.5 rounded border border-default bg-elevated/40">
					<USwitch
						v-model="onlyUnspent"
						size="sm"
						aria-label="Hanya belum dibelanjakan"
					/>
					<span class="text-xs font-medium cursor-pointer select-none text-highlighted" @click="onlyUnspent = !onlyUnspent">
						Belum dibelanjakan saja
					</span>
				</div>

				<!-- Pencarian -->
				<UInput
					v-model="searchQuery"
					icon="i-lucide-search"
					placeholder="Cari uraian atau produk..."
					class="w-56 ml-auto"
				/>
			</div>

			<!-- Alert Error -->
			<UAlert
				v-if="errorMessageText"
				color="error"
				variant="subtle"
				icon="i-lucide-triangle-alert"
				title="Terjadi kesalahan"
				:description="errorMessageText"
			/>

			<!-- Metric Cards -->
			<div v-if="matchReport" class="grid grid-cols-2 lg:grid-cols-4 gap-3">
				<UCard :ui="{ body: 'p-3 sm:p-3' }">
					<p class="text-xs text-muted">
						Rencana Belanja Bulan {{ BULAN[selectedMonth - 1] }}
					</p>
					<p class="text-lg font-bold tabular text-highlighted">
						{{ matchReport.summary.totalItems }} <span class="text-xs font-normal text-muted">item</span>
					</p>
					<p class="text-[11px] text-muted mt-0.5">
						{{ matchReport.summary.totalCocok }} cocok · {{ matchReport.summary.totalTidakCocok }} belum cocok
					</p>
				</UCard>

				<UCard :ui="{ body: 'p-3 sm:p-3' }">
					<p class="text-xs text-muted">
						Pagu Anggaran RKAS
					</p>
					<p class="text-lg font-bold tabular text-highlighted">
						{{ rupiah(matchReport.summary.totalAnggaranRkas) }}
					</p>
					<p class="text-[11px] text-muted mt-0.5">
						{{ onlyUnspent ? 'Sisa yang belum dibelanjakan' : 'Total rencana bulan ini' }}
					</p>
				</UCard>

				<UCard :ui="{ body: 'p-3 sm:p-3' }">
					<p class="text-xs text-muted">
						Estimasi Belanja di Toko
					</p>
					<p class="text-lg font-bold tabular text-highlighted">
						{{ rupiah(matchReport.summary.totalEstimasiToko) }}
					</p>
					<p class="text-[11px] text-muted mt-0.5 truncate" :title="selectedToko?.nama">
						{{ selectedToko?.nama ?? 'Toko SIPLah' }}
					</p>
				</UCard>

				<UCard :ui="{ body: 'p-3 sm:p-3' }">
					<p class="text-xs text-muted">
						Selisih / Penghematan
					</p>
					<p
						class="text-lg font-bold tabular"
						:class="matchReport.summary.totalHemat >= 0 ? 'text-success' : 'text-error'"
					>
						{{ matchReport.summary.totalHemat >= 0 ? `Hemat ${rupiah(matchReport.summary.totalHemat)}` : `Lebih ${rupiah(Math.abs(matchReport.summary.totalHemat))}` }}
					</p>
					<p class="text-[11px] text-muted mt-0.5">
						{{ matchReport.summary.totalLebihMurah }} lebih murah · {{ matchReport.summary.totalLebihMahal }} lebih mahal
					</p>
				</UCard>
			</div>

			<!-- Info banner jika toko belum memiliki produk tersimpan -->
			<div
				v-if="selectedToko && selectedToko.totalProduk === 0"
				class="p-4 border border-warning/40 bg-warning/5 rounded-lg flex items-center justify-between gap-3 text-xs"
			>
				<div class="flex items-center gap-2 text-warning">
					<UIcon name="i-lucide-info" class="size-5 shrink-0" />
					<span>
						Toko <strong>{{ selectedToko.nama }}</strong> belum memiliki produk yang disinkronkan. Klik tombol sinkronisasi untuk mengimpor produk toko secara otomatis.
					</span>
				</div>
				<UButton
					size="xs"
					color="warning"
					variant="soft"
					icon="i-lucide-refresh-cw"
					:loading="syncing"
					@click="sinkronkanToko"
				>
					Sinkronkan Sekarang
				</UButton>
			</div>

			<!-- LOADING STATE -->
			<div v-if="loadingMatch" class="py-16 text-center text-muted space-y-2">
				<UIcon name="i-lucide-loader-2" class="size-8 mx-auto animate-spin text-primary" />
				<p class="text-sm">
					Mencocokkan rencana belanja dengan katalog toko SIPLah...
				</p>
			</div>

			<!-- EMPTY ROWS -->
			<div
				v-else-if="rows.length === 0"
				class="py-16 text-center text-muted border border-dashed border-default rounded-xl bg-card/40"
			>
				<UIcon name="i-lucide-shopping-bag" class="size-10 mx-auto mb-2 opacity-50" />
				<p class="font-medium text-highlighted">
					Tidak Ada Rencana Belanja
				</p>
				<p class="text-xs text-muted mt-1 max-w-sm mx-auto">
					{{ onlyUnspent ? 'Semua rencana belanja bulan ini sudah dibelanjakan atau tidak ada item belanja pada filter ini.' : 'Tidak ada rencana belanja RKAS pada bulan dan filter ini.' }}
				</p>
			</div>

			<!-- TAMPILAN KOMPARASI KARTU SIDE-BY-SIDE (BEBAS SCROLL SAMPING) -->
			<div v-else class="space-y-3 pb-8">
				<div
					v-for="item in rows"
					:key="item.sourceId"
					class="p-4 rounded-xl border border-default bg-card shadow-xs hover:border-primary/40 transition-colors"
				>
					<div class="grid grid-cols-1 lg:grid-cols-2 gap-4 lg:divide-x divide-default">
						<!-- KOLOM KIRI: Rencana Belanja RKAS -->
						<div class="space-y-2.5 pr-0 lg:pr-4">
							<div class="flex items-start justify-between gap-2">
								<div class="min-w-0 flex-1">
									<h3 class="font-semibold text-highlighted text-sm leading-snug">
										{{ item.namaRkas }}
									</h3>
									<div class="flex flex-wrap items-center gap-1.5 mt-1 text-[11px] text-muted font-mono">
										<span v-if="item.kodeRekening">{{ item.kodeRekening }}</span>
										<span v-if="item.sumberDana" class="font-sans">
											· <UBadge color="neutral" variant="subtle" size="xs">{{ item.sumberDana }}</UBadge>
										</span>
									</div>
								</div>
								<UBadge
									:color="item.statusBelanja === 'lunas' ? 'success' : (item.statusBelanja === 'sebagian' ? 'warning' : 'neutral')"
									variant="subtle"
									size="xs"
									class="shrink-0"
								>
									{{ item.statusBelanja === 'lunas' ? 'Sudah Dibelanjakan' : (item.statusBelanja === 'sebagian' ? 'Belanja Sebagian' : 'Belum Dibelanjakan') }}
								</UBadge>
							</div>

							<!-- Rincian Angka Belanja RKAS -->
							<div class="grid grid-cols-3 gap-2 p-2.5 rounded-lg bg-elevated/60 text-center border border-default/70">
								<div>
									<p class="text-[10px] text-muted">
										Volume Bulan Ini
									</p>
									<p class="text-xs font-bold text-highlighted tabular mt-0.5">
										{{ formatVolume(item.volume) }} <span class="font-normal text-[11px] text-muted">{{ item.satuan || 'unit' }}</span>
									</p>
								</div>
								<div>
									<p class="text-[10px] text-muted">
										Harga Anggaran
									</p>
									<p class="text-xs font-bold text-highlighted font-mono mt-0.5">
										{{ rupiah(item.hargaRkas) }}
									</p>
								</div>
								<div>
									<p class="text-[10px] text-muted">
										Total Pagu Bulan Ini
									</p>
									<p class="text-xs font-bold text-highlighted font-mono mt-0.5">
										{{ rupiah(item.totalRkas) }}
									</p>
								</div>
							</div>
						</div>

						<!-- KOLOM KANAN: Produk Toko SIPLah yang Cocok -->
						<div class="pl-0 lg:pl-4 flex flex-col justify-between pt-3 lg:pt-0 border-t lg:border-t-0 border-default">
							<!-- Jika ada produk cocok -->
							<div v-if="item.matchedProduct" class="space-y-2.5">
								<div class="flex items-start gap-3">
									<!-- Foto Produk (URL siplah-oss yang valid) -->
									<div class="size-14 rounded-lg border border-default bg-neutral-500/10 flex items-center justify-center shrink-0 overflow-hidden">
										<img
											v-if="item.matchedProduct.gambar"
											:src="item.matchedProduct.gambar"
											:alt="item.matchedProduct.nama"
											class="size-full object-cover"
											@error="handleImgError"
										>
										<UIcon v-else name="i-lucide-package" class="size-6 text-muted" />
									</div>

									<div class="min-w-0 flex-1">
										<div class="flex items-center gap-1.5 flex-wrap">
											<UBadge :color="item.score >= 90 ? 'primary' : 'neutral'" variant="subtle" size="xs">
												{{ item.score >= 100 ? 'Persis' : (item.score >= 90 ? 'Mirip' : '1 Kata Sama') }}
											</UBadge>
											<span class="text-[11px] text-muted font-medium">Skor: {{ item.score }}%</span>
										</div>
										<p class="font-semibold text-highlighted text-xs mt-1 line-clamp-2" :title="item.matchedProduct.nama">
											{{ item.matchedProduct.nama }}
										</p>
									</div>
								</div>

								<!-- Rincian Harga Toko & Perbandingan -->
								<div class="grid grid-cols-3 gap-2 p-2.5 rounded-lg bg-elevated/60 text-center border border-default/70">
									<div>
										<p class="text-[10px] text-muted">
											Harga Toko Satuan
										</p>
										<p class="text-xs font-bold text-highlighted font-mono mt-0.5">
											{{ rupiah(item.matchedProduct.harga) }}
										</p>
										<p class="text-[10px] text-muted">
											Stok: {{ item.matchedProduct.stok }}
										</p>
									</div>
									<div>
										<p class="text-[10px] text-muted">
											Estimasi Belanja Toko
										</p>
										<p class="text-xs font-bold text-highlighted font-mono mt-0.5">
											{{ rupiah(item.matchedProduct.harga * Math.max(1, Math.round(item.volume))) }}
										</p>
									</div>
									<div>
										<p class="text-[10px] text-muted">
											Status Selisih
										</p>
										<div class="mt-0.5">
											<UBadge
												v-if="item.status === 'sesuai'"
												color="neutral"
												variant="subtle"
												size="xs"
											>
												Sesuai
											</UBadge>
											<UBadge
												v-else-if="item.status === 'lebih_murah'"
												color="success"
												variant="subtle"
												size="xs"
											>
												Hemat {{ rupiah(-item.selisih) }}
											</UBadge>
											<UBadge
												v-else
												color="error"
												variant="subtle"
												size="xs"
											>
												+ {{ rupiah(item.selisih) }}
											</UBadge>
										</div>
									</div>
								</div>

								<!-- Tombol Aksi Produk Toko -->
								<div class="flex items-center justify-between gap-2 pt-1">
									<!-- LINK LANGSUNG KE PRODUK TOKO SIPLAH -->
									<UButton
										icon="i-lucide-external-link"
										color="neutral"
										variant="ghost"
										size="xs"
										:to="productUrl(item.matchedProduct.id)"
										target="_blank"
										title="Buka halaman produk ini di SIPLah"
									>
										Buka Produk di SIPLah
									</UButton>

									<UButton
										icon="i-lucide-arrow-left-right"
										color="neutral"
										variant="outline"
										size="xs"
										@click="bukaModalPilihKandidat(item)"
									>
										Ganti Produk ({{ item.candidates.length }} opsi)
									</UButton>
								</div>
							</div>

							<!-- Jika belum ada produk cocok -->
							<div v-else class="h-full flex flex-col items-center justify-center text-center p-4 rounded-lg border border-dashed border-default bg-elevated/30">
								<UIcon name="i-lucide-package-search" class="size-7 text-muted mb-1 opacity-60" />
								<p class="text-xs font-medium text-muted">
									Belum ada produk yang cocok di toko ini
								</p>
								<div class="mt-2.5">
									<UButton
										icon="i-lucide-search"
										color="primary"
										variant="soft"
										size="xs"
										@click="bukaModalPilihKandidat(item)"
									>
										Cari Produk di Katalog Toko
									</UButton>
								</div>
							</div>
						</div>
					</div>
				</div>
			</div>
		</div>

		<!-- MODAL: GANTI PRODUK / PILIH KANDIDAT DARI KATALOG -->
		<UModal v-model:open="modalPilihKandidat" :title="`Pilih Produk Toko: ${activeMatchItem?.namaRkas ?? ''}`">
			<template #body>
				<div v-if="activeMatchItem" class="space-y-4">
					<div class="p-3 border border-default rounded-lg bg-elevated/50 text-xs space-y-1">
						<p class="text-muted">
							Item RKAS: <strong class="text-highlighted">{{ activeMatchItem.namaRkas }}</strong>
						</p>
						<p class="text-muted">
							Volume Bulan Ini: <strong class="text-highlighted">{{ formatVolume(activeMatchItem.volume) }} {{ activeMatchItem.satuan || 'unit' }}</strong>
							· Harga Satuan: <strong class="text-highlighted">{{ rupiah(activeMatchItem.hargaRkas) }}</strong>
						</p>
					</div>

					<!-- Rekomendasi Kecocokan Sistem -->
					<div v-if="activeMatchItem.candidates.length > 0" class="space-y-2">
						<p class="text-xs font-semibold text-highlighted">
							Rekomendasi Kecocokan Sistem:
						</p>
						<div class="space-y-1.5 max-h-52 overflow-y-auto pr-1">
							<div
								v-for="c in activeMatchItem.candidates"
								:key="c.produk.id"
								class="p-2.5 border rounded-lg flex items-center justify-between gap-3 cursor-pointer transition-colors"
								:class="activeMatchItem.matchedProduct?.id === c.produk.id ? 'border-primary bg-primary/10' : 'border-default hover:bg-neutral-500/5'"
								@click="tetapkanProduk(c.produk)"
							>
								<div class="size-10 rounded border border-default bg-neutral-500/10 flex items-center justify-center shrink-0 overflow-hidden">
									<img
										v-if="c.produk.gambar"
										:src="c.produk.gambar"
										:alt="c.produk.nama"
										class="size-full object-cover"
										@error="handleImgError"
									>
									<UIcon v-else name="i-lucide-package" class="size-5 text-muted" />
								</div>
								<div class="min-w-0 flex-1">
									<p class="text-xs font-medium text-highlighted line-clamp-1">
										{{ c.produk.nama }}
									</p>
									<div class="flex items-center gap-2 mt-0.5 text-[11px] text-muted">
										<span>Skor: {{ c.score }}%</span>
										<span>·</span>
										<span>Kata cocok: {{ c.matchedWords.join(", ") }}</span>
									</div>
								</div>
								<div class="text-right shrink-0 tabular">
									<p class="text-xs font-bold text-highlighted font-mono">
										{{ rupiah(c.produk.harga) }}
									</p>
									<p class="text-[11px]" :class="c.produk.harga <= activeMatchItem.hargaRkas ? 'text-success' : 'text-error'">
										{{ c.produk.harga <= activeMatchItem.hargaRkas ? `Hemat ${rupiah(activeMatchItem.hargaRkas - c.produk.harga)}` : `+ ${rupiah(c.produk.harga - activeMatchItem.hargaRkas)}` }}
									</p>
								</div>
							</div>
						</div>
					</div>

					<!-- Cari Manual dari Katalog Toko -->
					<div class="pt-2 border-t border-default space-y-2">
						<p class="text-xs font-semibold text-highlighted">
							Atau cari produk lain di toko ini:
						</p>
						<UInput
							v-model="searchManualCandidate"
							icon="i-lucide-search"
							size="sm"
							placeholder="Ketik nama produk di toko..."
							class="w-full"
						/>
						<div class="max-h-48 overflow-y-auto space-y-1 pr-1">
							<div
								v-for="p in filteredManualCandidates"
								:key="p.id"
								class="p-2 border border-default rounded-md hover:bg-neutral-500/5 cursor-pointer flex items-center justify-between text-xs"
								@click="tetapkanProduk(p)"
							>
								<div class="flex items-center gap-2 min-w-0">
									<div class="size-7 rounded border border-default bg-neutral-500/10 flex items-center justify-center shrink-0 overflow-hidden">
										<img
											v-if="p.gambar"
											:src="p.gambar"
											:alt="p.nama"
											class="size-full object-cover"
											@error="handleImgError"
										>
										<UIcon v-else name="i-lucide-package" class="size-4 text-muted" />
									</div>
									<span class="truncate font-medium text-highlighted">{{ p.nama }}</span>
								</div>
								<span class="font-bold ml-2 shrink-0 tabular font-mono">{{ rupiah(p.harga) }}</span>
							</div>
							<p v-if="filteredManualCandidates.length === 0" class="text-center text-xs text-muted py-4">
								Tidak ada produk toko yang cocok dengan pencarian.
							</p>
						</div>
					</div>
				</div>
			</template>

			<template #footer>
				<div class="flex justify-end w-full">
					<UButton color="neutral" variant="ghost" size="sm" @click="modalPilihKandidat = false">
						Tutup
					</UButton>
				</div>
			</template>
		</UModal>
	</LayoutPageShell>
</template>

<script lang="ts" setup>
	import type { SiplahMatchItem, SiplahMatchReport, SiplahProduk, SiplahToko } from "~/utils/api";

	const FILENAME_SANITIZE_REGEX = /[^\w-]/g;

	const { connected, year, funds } = useArkas();
	const toast = useToast();
	const { xlsx, busy: busyExport } = useSaveFile();

	// State toko
	const tokoList = ref<SiplahToko[]>([]);
	const loadingToko = ref(false);
	const selectedTokoId = ref<number | undefined>(undefined);
	const selectedToko = computed(() => tokoList.value.find((t) => t.id === selectedTokoId.value) ?? null);

	// Filter
	const selectedMonth = ref<number>(new Date().getMonth() + 1);
	const selectedFund = ref<number>(0);
	const filterStatus = ref<string>("semua");
	const onlyUnspent = ref<boolean>(true);
	const searchQuery = ref<string>("");

	// Data matching
	const matchReport = ref<SiplahMatchReport | null>(null);
	const loadingMatch = ref(false);
	const syncing = ref(false);
	const errorMessageText = ref("");

	// Modal kandidat
	const modalPilihKandidat = ref(false);
	const activeMatchItem = ref<SiplahMatchItem | null>(null);
	const searchManualCandidate = ref("");
	const allTokoProducts = ref<SiplahProduk[]>([]);

	// Opsi Bulan
	const bulanOptions = BULAN.map((nama, idx) => ({
		label: `${idx + 1} - ${nama}`,
		value: idx + 1
	}));

	// Opsi Toko
	const tokoOptions = computed(() =>
		tokoList.value.map((t) => ({
			label: t.nama,
			value: t.id
		}))
	);

	// Opsi Sumber Dana
	const fundOptions = computed(() => [
		{ label: "Semua Sumber Dana", value: 0 },
		...funds.value.map((f) => ({
			label: f.name,
			value: f.id
		}))
	]);

	// Opsi Status Kecocokan
	const statusOptions = [
		{ label: "Semua Status Cocok", value: "semua" },
		{ label: "Cocok (Semua)", value: "cocok" },
		{ label: "Lebih Murah", value: "lebih_murah" },
		{ label: "Harga Sesuai", value: "sesuai" },
		{ label: "Lebih Mahal", value: "lebih_mahal" },
		{ label: "Belum Ada Cocok", value: "tidak_cocok" }
	];

	// Helper URL produk SIPLah
	const productUrl = (productId: number): string => `https://siplah.tokoladang.co.id/p/produk.${productId}`;

	// Helper fallback gambar jika gagal memuat
	const handleImgError = (event: Event) => {
		const target = event.target as HTMLImageElement;
		if (target) {
			target.style.display = "none";
		}
	};

	// Helper format volume
	const formatVolume = (vol: number): string => {
		if (Number.isInteger(vol)) return String(vol);
		return vol.toLocaleString("id-ID", { maximumFractionDigits: 2 });
	};

	// Muat daftar toko
	const muatToko = async () => {
		loadingToko.value = true;
		try {
			const list = await api.siplahTokoList();
			tokoList.value = list;
			if (list.length > 0 && (selectedTokoId.value === undefined || !list.some((t) => t.id === selectedTokoId.value))) {
				const first = list[0];
				if (first) {
					selectedTokoId.value = first.id;
				}
			}
		} catch (err) {
			console.error("Gagal memuat daftar toko:", err);
		} finally {
			loadingToko.value = false;
		}
	};

	// Muat pencocokan harga
	const muatMatch = async () => {
		if (!connected.value || !year.value || selectedTokoId.value === undefined) {
			matchReport.value = null;
			return;
		}

		loadingMatch.value = true;
		errorMessageText.value = "";
		try {
			const fundArg = selectedFund.value === 0 ? null : selectedFund.value;
			matchReport.value = await api.siplahMatchPrices(
				year.value,
				selectedTokoId.value,
				"rkas",
				selectedMonth.value,
				fundArg,
				onlyUnspent.value
			);
		} catch (err) {
			errorMessageText.value = errorMessage(err);
			matchReport.value = null;
		} finally {
			loadingMatch.value = false;
		}
	};

	// Sinkronkan produk toko aktif
	const sinkronkanToko = async () => {
		if (selectedTokoId.value === undefined) return;
		syncing.value = true;
		try {
			const res = await api.siplahSyncProducts(selectedTokoId.value);
			toast.add({
				title: "Sinkronisasi Berhasil",
				description: `Berhasil mengimpor ${res.total} produk dari SIPLah.`,
				color: "success"
			});
			await muatToko();
			await muatMatch();
		} catch (err) {
			toast.add({
				title: "Sinkronisasi Gagal",
				description: errorMessage(err),
				color: "error"
			});
		} finally {
			syncing.value = false;
		}
	};

	// Buka modal pilih kandidat
	const bukaModalPilihKandidat = async (item: SiplahMatchItem) => {
		activeMatchItem.value = item;
		searchManualCandidate.value = "";
		modalPilihKandidat.value = true;

		if (selectedTokoId.value !== undefined) {
			try {
				allTokoProducts.value = await api.siplahProdukList(selectedTokoId.value);
			} catch (err) {
				console.error("Gagal memuat katalog toko:", err);
			}
		}
	};

	// Tetapkan produk pilihan untuk item
	const tetapkanProduk = (produk: SiplahProduk) => {
		if (!activeMatchItem.value || !matchReport.value) return;

		const item = activeMatchItem.value;
		item.matchedProduct = produk;
		item.selisih = produk.harga - item.hargaRkas;
		item.score = 100;

		if (item.selisih === 0) {
			item.status = "sesuai";
		} else if (item.selisih < 0) {
			item.status = "lebih_murah";
		} else {
			item.status = "lebih_mahal";
		}

		// Hitung ulang summary
		let totalEstimasi = 0;
		let totalSesuai = 0;
		let totalMurah = 0;
		let totalMahal = 0;
		let totalCocok = 0;

		for (const it of matchReport.value.items) {
			if (it.matchedProduct) {
				totalCocok += 1;
				const vol = Math.max(1, Math.round(it.volume));
				totalEstimasi += it.matchedProduct.harga * vol;
				if (it.status === "sesuai") totalSesuai += 1;
				else if (it.status === "lebih_murah") totalMurah += 1;
				else if (it.status === "lebih_mahal") totalMahal += 1;
			}
		}

		matchReport.value.summary.totalCocok = totalCocok;
		matchReport.value.summary.totalTidakCocok = matchReport.value.items.length - totalCocok;
		matchReport.value.summary.totalSesuai = totalSesuai;
		matchReport.value.summary.totalLebihMurah = totalMurah;
		matchReport.value.summary.totalLebihMahal = totalMahal;
		matchReport.value.summary.totalEstimasiToko = totalEstimasi;
		matchReport.value.summary.totalHemat = matchReport.value.summary.totalAnggaranRkas - totalEstimasi;

		modalPilihKandidat.value = false;
		toast.add({
			title: "Produk Ditetapkan",
			description: `Produk "${produk.nama}" dipilih untuk item ${item.namaRkas}.`,
			color: "success"
		});
	};

	// Kandidat manual yang difilter
	const filteredManualCandidates = computed(() => {
		const q = searchManualCandidate.value.trim().toLowerCase();
		if (!q) return allTokoProducts.value.slice(0, 30);
		return allTokoProducts.value.filter((p) => p.nama.toLowerCase().includes(q)).slice(0, 40);
	});

	// Ekspor Excel
	const eksporExcel = () => {
		if (!matchReport.value || selectedTokoId.value === undefined || !year.value) return;
		const tokoNama = selectedToko.value?.nama.replace(FILENAME_SANITIZE_REGEX, "_") || "siplah";
		const bulanNama = BULAN[selectedMonth.value - 1];
		const filename = `Belanja_RKAS_${bulanNama}_${year.value}_${tokoNama}.xlsx`;
		const y = year.value;
		const tId = selectedTokoId.value;
		const m = selectedMonth.value;
		const f = selectedFund.value === 0 ? null : selectedFund.value;
		const unspent = onlyUnspent.value;

		xlsx(filename, (path) => api.siplahExportMatchXlsx(y, tId, "rkas", path, m, f, unspent));
	};

	// Filtered Rows
	const rows = computed(() => {
		if (!matchReport.value) return [];
		const q = searchQuery.value.trim().toLowerCase();
		const st = filterStatus.value;

		return matchReport.value.items.filter((item) => {
			// Filter status
			if (st === "cocok" && !item.matchedProduct) return false;
			if (st === "tidak_cocok" && item.matchedProduct) return false;
			if (st === "lebih_murah" && item.status !== "lebih_murah") return false;
			if (st === "sesuai" && item.status !== "sesuai") return false;
			if (st === "lebih_mahal" && item.status !== "lebih_mahal") return false;

			// Filter query
			if (q) {
				const matchUraian = item.namaRkas.toLowerCase().includes(q);
				const matchRekening = item.kodeRekening?.toLowerCase().includes(q) ?? false;
				const matchProd = item.matchedProduct?.nama.toLowerCase().includes(q) ?? false;
				return matchUraian || matchRekening || matchProd;
			}

			return true;
		});
	});

	// Watchers
	watch([year, connected], () => {
		muatToko();
	}, { immediate: true });

	watch([selectedTokoId, selectedMonth, selectedFund, onlyUnspent], () => {
		muatMatch();
	});

	watch(tokoList, (list) => {
		if (list.length > 0 && selectedTokoId.value === undefined) {
			const first = list[0];
			if (first) {
				selectedTokoId.value = first.id;
			}
		}
	});
</script>
