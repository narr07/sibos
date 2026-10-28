<template>
	<LayoutPageShell title="Belanja & Pencocokan Harga SIPLah">
		<template #actions>
			<UButton icon="i-lucide-plus" color="primary" @click="modalTambah = true">
				Tambah Toko SIPLah
			</UButton>
			<UButton
				v-if="selectedToko && matchReport"
				color="neutral"
				variant="outline"
				icon="i-lucide-file-spreadsheet"
				:loading="busyExport"
				@click="exportExcel"
			>
				Export Hasil (.xlsx)
			</UButton>
		</template>

		<UEmpty
			v-if="!connected"
			icon="i-lucide-plug-zap"
			title="ARKAS belum terhubung"
			description="Hubungkan database ARKAS untuk mencocokkan harga toko SIPLah dengan Kertas Kerja RKAS."
			:actions="[{ label: 'Hubungkan', icon: 'i-lucide-plug', to: '/setup' }]"
			class="py-16"
		/>

		<div v-else class="grid lg:grid-cols-[19rem_minmax(0,1fr)] gap-4 min-w-0 w-full flex-1 min-h-0">
			<!-- Panel Kiri: Daftar Toko Tersimpan -->
			<div class="flex flex-col w-full min-w-0 shrink-0 gap-3 border border-default rounded-lg p-3 bg-elevated/40">
				<div class="flex items-center justify-between">
					<span class="text-sm font-semibold text-highlighted">Toko SIPLah Tersimpan</span>
					<UBadge color="neutral" variant="subtle">
						{{ tokoList.length }} toko
					</UBadge>
				</div>

				<UInput
					v-model="searchToko"
					icon="i-lucide-search"
					placeholder="Cari toko..."
					size="sm"
					class="w-full"
				/>

				<div v-if="loadingToko" class="space-y-2 py-4">
					<USkeleton class="h-16 w-full rounded-md" />
					<USkeleton class="h-16 w-full rounded-md" />
				</div>

				<div v-else-if="filteredTokoList.length === 0" class="py-8 text-center text-xs text-muted">
					<UIcon name="i-lucide-store" class="size-8 mx-auto mb-2 text-muted/60" />
					<p v-if="searchToko">
						Tidak ada toko dengan nama "{{ searchToko }}".
					</p>
					<p v-else>
						Belum ada toko yang disimpan. Klik "Tambah Toko SIPLah" untuk menambahkan toko dari link.
					</p>
				</div>

				<div v-else class="flex-1 min-h-0 overflow-y-auto space-y-2 pr-1">
					<div
						v-for="t in filteredTokoList"
						:key="t.id"
						class="p-2.5 rounded-md border transition-colors cursor-pointer text-left"
						:class="selectedTokoId === t.id ? 'border-primary bg-primary/5 shadow-xs' : 'border-default bg-elevated hover:bg-neutral-500/5'"
						@click="pilihToko(t)"
					>
						<div class="flex items-start gap-2.5">
							<div class="size-9 rounded-md bg-neutral-500/10 flex items-center justify-center shrink-0 overflow-hidden">
								<img
									v-if="t.gambar"
									:src="t.gambar"
									:alt="t.nama"
									class="size-full object-cover"
									loading="lazy"
								>
								<UIcon v-else name="i-lucide-store" class="size-5 text-muted" />
							</div>
							<div class="min-w-0 flex-1">
								<p class="text-sm font-medium text-highlighted truncate" :title="t.nama">
									{{ t.nama }}
								</p>
								<p class="text-xs text-muted truncate">
									{{ t.kabupaten || t.provinsi || "SIPLah Toko Ladang" }}
								</p>
								<div class="flex items-center gap-2 mt-1.5 text-[11px] text-muted">
									<UBadge color="neutral" variant="subtle" size="xs">
										{{ t.totalProduk }} produk
									</UBadge>
									<span v-if="t.terakhirSync" class="truncate" :title="`Sinkron: ${t.terakhirSync}`">
										{{ formatWaktuRelatif(t.terakhirSync) }}
									</span>
								</div>
							</div>
						</div>
					</div>
				</div>
			</div>

			<!-- Panel Kanan: Rincian Toko & Pencocokan Harga -->
			<div class="min-w-0 flex flex-col gap-4 w-full">
				<UEmpty
					v-if="!selectedToko"
					icon="i-lucide-store"
					title="Pilih Toko SIPLah"
					description="Pilih toko dari daftar di sebelah kiri atau tambah toko baru menggunakan link toko SIPLah."
					class="py-20 my-auto border border-default rounded-lg"
				/>

				<div v-else class="flex min-h-0 flex-1 flex-col gap-4 min-w-0 w-full">
					<!-- Kartu Profil Toko Aktif -->
					<div class="p-4 border border-default rounded-lg bg-elevated/40 flex flex-col sm:flex-row sm:items-center justify-between gap-4">
						<div class="flex items-start gap-3.5 min-w-0">
							<div class="size-14 rounded-lg bg-neutral-500/10 flex items-center justify-center shrink-0 overflow-hidden border border-default">
								<img
									v-if="selectedToko.gambar"
									:src="selectedToko.gambar"
									:alt="selectedToko.nama"
									class="size-full object-cover"
								>
								<UIcon v-else name="i-lucide-store" class="size-7 text-muted" />
							</div>
							<div class="min-w-0 flex-1">
								<div class="flex flex-wrap items-center gap-2">
									<h2 class="text-lg font-bold text-highlighted truncate">
										{{ selectedToko.nama }}
									</h2>
									<UBadge v-if="selectedToko.tipe" color="neutral" variant="subtle" size="xs">
										{{ selectedToko.tipe }}
									</UBadge>
								</div>
								<p v-if="selectedToko.slogan" class="text-xs text-muted italic mt-0.5">
									"{{ selectedToko.slogan }}"
								</p>
								<p class="text-xs text-muted mt-1 truncate">
									{{ [selectedToko.alamat, selectedToko.kecamatan, selectedToko.kabupaten, selectedToko.provinsi].filter(Boolean).join(", ") }}
								</p>
								<p class="text-[11px] text-muted mt-1">
									Total produk tersimpan: <strong class="text-highlighted">{{ selectedToko.totalProduk }}</strong>
									<span v-if="selectedToko.terakhirSync"> · Sinkronisasi terakhir: {{ selectedToko.terakhirSync }}</span>
								</p>
							</div>
						</div>

						<div class="flex flex-wrap items-center gap-2 shrink-0">
							<UButton
								icon="i-lucide-refresh-cw"
								color="primary"
								variant="soft"
								size="sm"
								:loading="syncing"
								@click="sinkronkanProduk(selectedToko.id)"
							>
								Sinkronkan Produk
							</UButton>
							<UButton
								icon="i-lucide-external-link"
								color="neutral"
								variant="outline"
								size="sm"
								@click="bukaLinkToko(selectedToko.url)"
							>
								Buka di Web
							</UButton>
							<UButton
								icon="i-lucide-trash-2"
								color="error"
								variant="ghost"
								size="sm"
								aria-label="Hapus toko"
								title="Hapus toko"
								@click="konfirmasiHapus(selectedToko.id)"
							/>
						</div>
					</div>

					<!-- Tab Switcher: Pencocokan vs Katalog Produk -->
					<div class="flex items-center justify-between border-b border-default pb-2">
						<div class="flex items-center gap-2">
							<UButton
								size="sm"
								:color="tabAktif === 'match' ? 'primary' : 'neutral'"
								:variant="tabAktif === 'match' ? 'solid' : 'ghost'"
								icon="i-lucide-scale"
								@click="tabAktif = 'match'"
							>
								Pencocokan Harga RKAS
							</UButton>
							<UButton
								size="sm"
								:color="tabAktif === 'katalog' ? 'primary' : 'neutral'"
								:variant="tabAktif === 'katalog' ? 'solid' : 'ghost'"
								icon="i-lucide-shopping-cart"
								@click="tabAktif = 'katalog'"
							>
								Katalog Produk Toko ({{ selectedToko.totalProduk }})
							</UButton>
						</div>

						<!-- Filter Mode Sumber Data RKAS / Standar Harga -->
						<div v-if="tabAktif === 'match'" class="flex items-center gap-2">
							<span class="text-xs text-muted hidden sm:inline">Bandingkan dengan:</span>
							<USelect
								v-model="sourceMode"
								:items="sourceModeOptions"
								size="xs"
								class="w-48"
							/>
						</div>
					</div>

					<!-- TAB 1: PENCOCOKAN HARGA -->
					<div v-if="tabAktif === 'match'" class="flex min-h-0 flex-1 flex-col gap-3">
						<div v-if="loadingMatch" class="py-12 flex flex-col items-center justify-center gap-2">
							<UIcon name="i-lucide-loader" class="size-8 animate-spin text-primary" />
							<p class="text-sm text-muted">
								Mencocokkan barang RKAS dengan produk toko SIPLah...
							</p>
						</div>

						<template v-else-if="matchReport">
							<!-- Kartu Ringkasan Metrik -->
							<div class="grid grid-cols-2 md:grid-cols-4 gap-3 min-w-0 w-full">
								<div class="p-3 border border-default rounded-md bg-elevated/40">
									<p class="text-xs text-muted">
										Item Dibandingkan
									</p>
									<p class="text-xl font-bold text-highlighted mt-1">
										{{ matchReport.summary.totalItems }}
									</p>
									<p class="text-xs text-muted mt-0.5">
										{{ matchReport.summary.totalCocok }} ditemukan di toko
									</p>
								</div>
								<div class="p-3 border border-default rounded-md bg-elevated/40">
									<p class="text-xs text-muted">
										Lebih Murah (Hemat)
									</p>
									<p class="text-xl font-bold text-success mt-1">
										{{ matchReport.summary.totalLebihMurah }} item
									</p>
									<p class="text-xs text-muted mt-0.5">
										{{ matchReport.summary.totalSesuai }} harga pas
									</p>
								</div>
								<div class="p-3 border border-default rounded-md bg-elevated/40">
									<p class="text-xs text-muted">
										Lebih Mahal
									</p>
									<p class="text-xl font-bold text-error mt-1">
										{{ matchReport.summary.totalLebihMahal }} item
									</p>
									<p class="text-xs text-muted mt-0.5">
										{{ matchReport.summary.totalTidakCocok }} belum cocok
									</p>
								</div>
								<div class="p-3 border border-default rounded-md bg-elevated/40">
									<p class="text-xs text-muted">
										Selisih Total Belanja
									</p>
									<p class="text-xl font-bold mt-1" :class="matchReport.summary.totalHemat >= 0 ? 'text-success' : 'text-error'">
										{{ matchReport.summary.totalHemat >= 0 ? `Hemat ${rupiah(matchReport.summary.totalHemat)}` : `Lebih ${rupiah(-matchReport.summary.totalHemat)}` }}
									</p>
									<p class="text-xs text-muted mt-0.5">
										Pagu: {{ rupiah(matchReport.summary.totalAnggaranRkas) }}
									</p>
								</div>
							</div>

							<!-- Filter Status & Pencarian Baris -->
							<div class="flex flex-wrap items-center gap-2 min-w-0 w-full">
								<UInput
									v-model="filterItemSearch"
									icon="i-lucide-search"
									size="sm"
									placeholder="Cari barang atau produk..."
									class="w-full sm:w-64"
								/>
								<div class="flex flex-wrap items-center gap-1">
									<UButton
										v-for="st in statusFilters"
										:key="st.value"
										size="xs"
										:color="filterStatus === st.value ? 'primary' : 'neutral'"
										:variant="filterStatus === st.value ? 'solid' : 'outline'"
										@click="filterStatus = st.value"
									>
										{{ st.label }} ({{ countPerStatus(st.value) }})
									</UButton>
								</div>
							</div>

							<!-- Tabel Pencocokan -->
							<div class="flex-1 min-h-0 min-w-0 w-full border border-default rounded-md overflow-x-auto">
								<UTable
									:data="filteredMatchItems"
									:columns="matchColumns"
									sticky
									class="min-h-60"
									:ui="{ td: 'py-2 text-xs', th: 'py-2.5 text-xs font-semibold' }"
								>
									<template #index-cell="{ row }">
										<span class="text-muted tabular-nums">{{ row.index + 1 }}</span>
									</template>

									<template #namaRkas-cell="{ row }">
										<div class="max-w-xs">
											<p class="font-medium text-highlighted text-sm">
												{{ row.original.namaRkas }}
											</p>
											<div class="flex flex-wrap items-center gap-1.5 mt-1 text-[11px] text-muted">
												<span v-if="row.original.volume && row.original.satuan">
													{{ row.original.volume }} {{ row.original.satuan }}
												</span>
												<span v-if="row.original.kodeRekening" class="font-mono">
													· {{ row.original.kodeRekening }}
												</span>
												<UBadge v-if="row.original.sumberDana" color="neutral" variant="subtle" size="xs">
													{{ row.original.sumberDana }}
												</UBadge>
											</div>
										</div>
									</template>

									<template #hargaRkas-cell="{ row }">
										<div class="text-right tabular-nums">
											<p class="font-medium">
												Rp {{ angka(row.original.hargaRkas) }}
											</p>
											<p v-if="row.original.totalRkas > row.original.hargaRkas" class="text-[11px] text-muted">
												Tot: Rp {{ angka(row.original.totalRkas) }}
											</p>
										</div>
									</template>

									<template #matchedProduct-cell="{ row }">
										<div v-if="row.original.matchedProduct" class="max-w-sm">
											<div class="flex items-start gap-2">
												<div class="size-8 rounded bg-neutral-500/10 shrink-0 overflow-hidden border border-default mt-0.5">
													<img
														v-if="row.original.matchedProduct.gambar"
														:src="row.original.matchedProduct.gambar"
														:alt="row.original.matchedProduct.nama"
														class="size-full object-cover"
														loading="lazy"
													>
													<UIcon v-else name="i-lucide-box" class="size-4 m-2 text-muted" />
												</div>
												<div class="min-w-0 flex-1">
													<p class="font-medium text-highlighted line-clamp-2" :title="row.original.matchedProduct.nama">
														{{ row.original.matchedProduct.nama }}
													</p>
													<div class="flex items-center gap-1.5 mt-1">
														<UBadge :color="row.original.score >= 90 ? 'primary' : 'neutral'" variant="subtle" size="xs">
															{{ row.original.score >= 100 ? 'Persis' : (row.original.score >= 90 ? 'Mirip' : 'Kata Sama') }}
														</UBadge>
														<UButton
															v-if="row.original.candidates.length > 1"
															size="xs"
															color="neutral"
															variant="link"
															class="p-0 text-[11px]"
															@click="bukaModalPilihKandidat(row.original)"
														>
															Ganti ({{ row.original.candidates.length }} opsi)
														</UButton>
													</div>
												</div>
											</div>
										</div>
										<div v-else class="text-muted italic flex items-center gap-2">
											<span>Belum ada yang cocok</span>
											<UButton
												size="xs"
												color="neutral"
												variant="ghost"
												icon="i-lucide-search"
												@click="bukaModalPilihKandidat(row.original)"
											>
												Cari
											</UButton>
										</div>
									</template>

									<template #hargaToko-cell="{ row }">
										<div v-if="row.original.matchedProduct" class="text-right tabular-nums">
											<p class="font-medium text-highlighted">
												Rp {{ angka(row.original.matchedProduct.harga) }}
											</p>
											<p class="text-[11px] text-muted">
												Stok: {{ row.original.matchedProduct.stok }}
											</p>
										</div>
										<span v-else class="text-muted text-right block">-</span>
									</template>

									<template #selisih-cell="{ row }">
										<div v-if="row.original.matchedProduct" class="text-right tabular-nums">
											<UBadge
												v-if="row.original.status === 'sesuai'"
												color="neutral"
												variant="subtle"
												size="xs"
											>
												Sesuai
											</UBadge>
											<UBadge
												v-else-if="row.original.status === 'lebih_murah'"
												color="success"
												variant="subtle"
												size="xs"
											>
												- Rp {{ angka(-row.original.selisih) }}
											</UBadge>
											<UBadge
												v-else
												color="error"
												variant="subtle"
												size="xs"
											>
												+ Rp {{ angka(row.original.selisih) }}
											</UBadge>
										</div>
										<span v-else class="text-muted text-right block">-</span>
									</template>
								</UTable>
							</div>
						</template>
					</div>

					<!-- TAB 2: KATALOG PRODUK TOKO -->
					<div v-else-if="tabAktif === 'katalog'" class="flex min-h-0 flex-1 flex-col gap-3">
						<div class="flex items-center gap-2">
							<UInput
								v-model="searchProdukKatalog"
								icon="i-lucide-search"
								size="sm"
								placeholder="Cari produk toko yang tersimpan..."
								class="w-full sm:w-80"
							/>
							<span class="text-xs text-muted ml-auto">
								{{ filteredKatalogProduk.length }} produk ditemukan
							</span>
						</div>

						<div class="flex-1 min-h-0 min-w-0 w-full border border-default rounded-md overflow-x-auto">
							<UTable
								:data="filteredKatalogProduk"
								:columns="katalogColumns"
								sticky
								class="min-h-60"
								:ui="{ td: 'py-2 text-xs', th: 'py-2.5 text-xs font-semibold' }"
							>
								<template #gambar-cell="{ row }">
									<div class="size-10 rounded bg-neutral-500/10 shrink-0 overflow-hidden border border-default">
										<img
											v-if="row.original.gambar"
											:src="row.original.gambar"
											:alt="row.original.nama"
											class="size-full object-cover"
											loading="lazy"
										>
										<UIcon v-else name="i-lucide-box" class="size-5 m-2.5 text-muted" />
									</div>
								</template>

								<template #nama-cell="{ row }">
									<div>
										<p class="font-medium text-highlighted text-sm">
											{{ row.original.nama }}
										</p>
										<p class="text-[11px] text-muted font-mono mt-0.5">
											ID: {{ row.original.id }}
										</p>
									</div>
								</template>

								<template #harga-cell="{ row }">
									<div class="text-right tabular-nums">
										<p class="font-bold text-highlighted">
											Rp {{ angka(row.original.harga) }}
										</p>
										<UBadge v-if="row.original.kenaPajak" color="neutral" variant="subtle" size="xs">
											PPN
										</UBadge>
									</div>
								</template>

								<template #stok-cell="{ row }">
									<div class="text-right tabular-nums text-muted">
										{{ row.original.stok }}
									</div>
								</template>

								<template #terjual-cell="{ row }">
									<div class="text-right tabular-nums text-muted">
										{{ row.original.terjual }}
									</div>
								</template>
							</UTable>
						</div>
					</div>
				</div>
			</div>
		</div>

		<!-- MODAL: TAMBAH TOKO SIPLAH DARI LINK -->
		<UModal v-model:open="modalTambah" title="Tambah Toko SIPLah Baru">
			<template #body>
				<div class="space-y-4">
					<p class="text-xs text-muted">
						Masukkan link atau ID toko SIPLah (contoh dari Toko Ladang). Semua produk dari toko akan langsung diambil dan disimpan ke database lokal SIBOS.
					</p>

					<UFormField label="Link Toko SIPLah" hint="URL Toko Ladang">
						<UInput
							v-model="inputUrl"
							placeholder="https://siplah.tokoladang.co.id/m/kpr-rajagaluh.92186"
							icon="i-lucide-link"
							class="w-full"
							@keydown.enter="periksaLink"
						/>
					</UFormField>

					<div class="flex justify-end">
						<UButton
							icon="i-lucide-search"
							color="neutral"
							variant="outline"
							size="sm"
							:loading="checkingUrl"
							:disabled="!inputUrl.trim()"
							@click="periksaLink"
						>
							Periksa Link Toko
						</UButton>
					</div>

					<UAlert
						v-if="checkError"
						color="error"
						variant="subtle"
						icon="i-lucide-triangle-alert"
						title="Gagal memeriksa link"
						:description="checkError"
					/>

					<!-- Preview Toko yang Ditemukan -->
					<div v-if="previewToko" class="p-3 border border-default rounded-lg bg-elevated/60 space-y-2">
						<div class="flex items-center gap-3">
							<div class="size-12 rounded-lg bg-neutral-500/10 flex items-center justify-center shrink-0 overflow-hidden border border-default">
								<img
									v-if="previewToko.gambar"
									:src="previewToko.gambar"
									:alt="previewToko.nama"
									class="size-full object-cover"
								>
								<UIcon v-else name="i-lucide-store" class="size-6 text-muted" />
							</div>
							<div>
								<p class="font-bold text-highlighted text-base">
									{{ previewToko.nama }}
								</p>
								<p class="text-xs text-muted">
									{{ [previewToko.kecamatan, previewToko.kabupaten, previewToko.provinsi].filter(Boolean).join(", ") }}
								</p>
								<UBadge v-if="previewToko.tipe" color="neutral" variant="subtle" size="xs" class="mt-1">
									{{ previewToko.tipe }}
								</UBadge>
							</div>
						</div>
						<p v-if="previewToko.slogan" class="text-xs italic text-muted">
							"{{ previewToko.slogan }}"
						</p>
					</div>
				</div>
			</template>

			<template #footer>
				<div class="flex justify-end gap-2 w-full">
					<UButton color="neutral" variant="ghost" @click="modalTambah = false">
						Batal
					</UButton>
					<UButton
						color="primary"
						icon="i-lucide-download"
						:loading="savingToko"
						:disabled="!previewToko"
						@click="simpanDanSyncToko"
					>
						Simpan & Ambil Semua Produk
					</UButton>
				</div>
			</template>
		</UModal>

		<!-- MODAL: GANTI PRODUK / PILIH KANDIDAT LAIN -->
		<UModal v-model:open="modalPilihKandidat" :title="`Pilih Produk untuk '${activeMatchItem?.namaRkas}'`">
			<template #body>
				<div class="space-y-3">
					<p class="text-xs text-muted">
						Harga RKAS: <strong class="text-highlighted">Rp {{ angka(activeMatchItem?.hargaRkas ?? 0) }}</strong>.
						Pilih salah satu produk toko di bawah ini untuk mencocokkan harga.
					</p>

					<!-- Kandidat otomatis dengan kata yang sama -->
					<div v-if="activeMatchItem && activeMatchItem.candidates.length" class="space-y-1.5">
						<p class="text-xs font-semibold text-highlighted">
							Kandidat Produk dengan Kata yang Sama:
						</p>
						<div
							v-for="c in activeMatchItem.candidates"
							:key="c.produk.id"
							class="p-2 border rounded-md transition-colors cursor-pointer flex items-center justify-between gap-2"
							:class="activeMatchItem.matchedProduct?.id === c.produk.id ? 'border-primary bg-primary/5' : 'border-default hover:bg-neutral-500/5'"
							@click="tetapkanProduk(c.produk)"
						>
							<div class="min-w-0 flex-1">
								<p class="text-xs font-medium text-highlighted">
									{{ c.produk.nama }}
								</p>
								<div class="flex items-center gap-2 mt-0.5 text-[11px] text-muted">
									<span>Skor: {{ c.score }}%</span>
									<span>·</span>
									<span>Kata cocok: {{ c.matchedWords.join(", ") }}</span>
								</div>
							</div>
							<div class="text-right shrink-0">
								<p class="text-xs font-bold text-highlighted">
									Rp {{ angka(c.produk.harga) }}
								</p>
								<p class="text-[11px]" :class="c.produk.harga <= activeMatchItem.hargaRkas ? 'text-success' : 'text-error'">
									{{ c.produk.harga <= activeMatchItem.hargaRkas ? `Hemat Rp ${angka(activeMatchItem.hargaRkas - c.produk.harga)}` : `+ Rp ${angka(c.produk.harga - activeMatchItem.hargaRkas)}` }}
								</p>
							</div>
						</div>
					</div>

					<!-- Cari manual dari katalog toko -->
					<div class="pt-2 border-t border-default space-y-2">
						<p class="text-xs font-semibold text-highlighted">
							Atau cari produk lain di toko ini:
						</p>
						<UInput
							v-model="searchManualCandidate"
							icon="i-lucide-search"
							size="sm"
							placeholder="Ketik nama produk..."
							class="w-full"
						/>
						<div class="max-h-48 overflow-y-auto space-y-1 pr-1">
							<div
								v-for="p in filteredManualCandidates"
								:key="p.id"
								class="p-2 border border-default rounded-md hover:bg-neutral-500/5 cursor-pointer flex items-center justify-between text-xs"
								@click="tetapkanProduk(p)"
							>
								<span class="truncate font-medium text-highlighted">{{ p.nama }}</span>
								<span class="font-bold ml-2 shrink-0">Rp {{ angka(p.harga) }}</span>
							</div>
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

		<!-- MODAL: KONFIRMASI HAPUS TOKO -->
		<UModal v-model:open="modalHapus" title="Hapus Toko SIPLah?">
			<template #body>
				<p class="text-sm text-muted">
					Toko dan semua produk yang tersimpan untuk toko ini akan dihapus dari database lokal SIBOS. Aksi ini tidak dapat dibatalkan.
				</p>
			</template>
			<template #footer>
				<div class="flex justify-end gap-2 w-full">
					<UButton color="neutral" variant="ghost" @click="modalHapus = false">
						Batal
					</UButton>
					<UButton color="error" icon="i-lucide-trash-2" @click="eksekusiHapus">
						Hapus Toko
					</UButton>
				</div>
			</template>
		</UModal>
	</LayoutPageShell>
</template>

<script lang="ts" setup>
	import type { TableColumn } from "@nuxt/ui";
	import type { SiplahMatchItem, SiplahMatchReport, SiplahProduk, SiplahToko } from "~/utils/api";

	const { connected, year } = useArkas();
	const toast = useToast();
	const { xlsx, busy: busyExport } = useSaveFile();

	const tokoList = ref<SiplahToko[]>([]);
	const loadingToko = ref(false);
	const searchToko = ref("");
	const selectedTokoId = ref<number | null>(null);
	const selectedToko = computed(() => tokoList.value.find((t) => t.id === selectedTokoId.value) ?? null);

	const tabAktif = ref<"match" | "katalog">("match");
	const sourceMode = ref<"rkas" | "standar_harga">("rkas");
	const sourceModeOptions = [
		{ label: "Kertas Kerja RKAS (Tahun Ini)", value: "rkas" },
		{ label: "Katalog Standar Harga", value: "standar_harga" }
	];

	const matchReport = ref<SiplahMatchReport | null>(null);
	const loadingMatch = ref(false);
	const syncing = ref(false);

	const filterItemSearch = ref("");
	const filterStatus = ref<string>("semua");
	const statusFilters = [
		{ label: "Semua", value: "semua" },
		{ label: "Cocok", value: "cocok" },
		{ label: "Lebih Murah", value: "lebih_murah" },
		{ label: "Sesuai", value: "sesuai" },
		{ label: "Lebih Mahal", value: "lebih_mahal" },
		{ label: "Belum Cocok", value: "tidak_cocok" }
	];

	// Dialog tambah toko
	const modalTambah = ref(false);
	const inputUrl = ref("");
	const checkingUrl = ref(false);
	const checkError = ref("");
	const previewToko = ref<SiplahToko | null>(null);
	const savingToko = ref(false);

	// Dialog kandidat pengganti
	const modalPilihKandidat = ref(false);
	const activeMatchItem = ref<SiplahMatchItem | null>(null);
	const searchManualCandidate = ref("");
	const katalogProduk = ref<SiplahProduk[]>([]);
	const searchProdukKatalog = ref("");

	const filteredTokoList = computed(() => {
		const q = searchToko.value.trim().toLowerCase();
		if (!q) return tokoList.value;
		return tokoList.value.filter((t) => t.nama.toLowerCase().includes(q) || (t.kabupaten && t.kabupaten.toLowerCase().includes(q)));
	});

	const loadToko = async () => {
		loadingToko.value = true;
		try {
			tokoList.value = await api.siplahTokoList();
			if (!selectedTokoId.value && tokoList.value.length > 0 && tokoList.value[0]) {
				selectedTokoId.value = tokoList.value[0].id;
			}
		} catch (err) {
			toast.add({ title: "Gagal memuat toko", description: errorMessage(err), color: "error" });
		} finally {
			loadingToko.value = false;
		}
	};

	const pilihToko = (t: SiplahToko) => {
		selectedTokoId.value = t.id;
	};

	const loadMatch = async () => {
		if (!selectedTokoId.value || !year.value) return;
		loadingMatch.value = true;
		try {
			matchReport.value = await api.siplahMatchPrices(year.value, selectedTokoId.value, sourceMode.value);
		} catch (err) {
			toast.add({ title: "Pencocokan gagal", description: errorMessage(err), color: "error" });
		} finally {
			loadingMatch.value = false;
		}
	};

	const loadKatalog = async () => {
		if (!selectedTokoId.value) return;
		try {
			katalogProduk.value = await api.siplahProdukList(selectedTokoId.value);
		} catch (err) {
			console.error(err);
		}
	};

	watch([selectedTokoId, sourceMode, year], () => {
		loadMatch();
		loadKatalog();
	});

	onMounted(async () => {
		await loadToko();
		if (selectedTokoId.value) {
			await loadMatch();
			await loadKatalog();
		}
	});

	// Aksi sinkronisasi produk dari SIPLah
	const sinkronkanProduk = async (tokoId: number) => {
		syncing.value = true;
		try {
			const res = await api.siplahSyncProducts(tokoId);
			toast.add({
				title: "Sinkronisasi Berhasil",
				description: `Berhasil mengambil ${res.total} produk dari SIPLah.`,
				color: "success"
			});
			await loadToko();
			await loadMatch();
			await loadKatalog();
		} catch (err) {
			toast.add({ title: "Sinkronisasi gagal", description: errorMessage(err), color: "error" });
		} finally {
			syncing.value = false;
		}
	};

	const tokoIdHapus = ref<number | null>(null);
	const modalHapus = ref(false);

	const konfirmasiHapus = (id: number) => {
		tokoIdHapus.value = id;
		modalHapus.value = true;
	};

	const eksekusiHapus = async () => {
		if (!tokoIdHapus.value) return;
		try {
			await api.siplahTokoDelete(tokoIdHapus.value);
			toast.add({ title: "Toko dihapus", color: "neutral" });
			modalHapus.value = false;
			selectedTokoId.value = null;
			await loadToko();
		} catch (err) {
			toast.add({ title: "Gagal menghapus toko", description: errorMessage(err), color: "error" });
		}
	};

	const periksaLink = async () => {
		const q = inputUrl.value.trim();
		if (!q) return;
		checkingUrl.value = true;
		checkError.value = "";
		previewToko.value = null;
		try {
			previewToko.value = await api.siplahInspectUrl(q);
		} catch (err) {
			checkError.value = errorMessage(err);
		} finally {
			checkingUrl.value = false;
		}
	};

	const simpanDanSyncToko = async () => {
		if (!previewToko.value) return;
		savingToko.value = true;
		try {
			await api.siplahTokoSave(previewToko.value);
			toast.add({
				title: "Toko Disimpan",
				description: "Mengambil semua produk dari toko...",
				color: "primary"
			});
			const res = await api.siplahSyncProducts(previewToko.value.id);
			toast.add({
				title: "Selesai",
				description: `${res.total} produk berhasil disimpan.`,
				color: "success"
			});
			modalTambah.value = false;
			selectedTokoId.value = previewToko.value.id;
			inputUrl.value = "";
			previewToko.value = null;
			await loadToko();
			await loadMatch();
		} catch (err) {
			toast.add({ title: "Gagal menyimpan toko", description: errorMessage(err), color: "error" });
		} finally {
			savingToko.value = false;
		}
	};

	const bukaLinkToko = async (url: string) => {
		try {
			if (inTauri()) await useTauriOpenerOpenUrl(url);
			else window.open(url, "_blank");
		} catch (err) {
			toast.add({ title: "Gagal membuka link", description: errorMessage(err), color: "error" });
		}
	};

	const bukaModalPilihKandidat = (item: SiplahMatchItem) => {
		activeMatchItem.value = item;
		searchManualCandidate.value = "";
		modalPilihKandidat.value = true;
	};

	const tetapkanProduk = (prod: SiplahProduk) => {
		if (!activeMatchItem.value) return;
		activeMatchItem.value.matchedProduct = prod;
		const diff = prod.harga - activeMatchItem.value.hargaRkas;
		activeMatchItem.value.selisih = diff;
		activeMatchItem.value.status = diff === 0 ? "sesuai" : (diff < 0 ? "lebih_murah" : "lebih_mahal");
		activeMatchItem.value.persenSelisih = activeMatchItem.value.hargaRkas > 0
			? (diff / activeMatchItem.value.hargaRkas) * 100
			: 0;
		modalPilihKandidat.value = false;
		toast.add({ title: "Produk dicocokkan", description: prod.nama, color: "success" });
	};

	const filteredManualCandidates = computed(() => {
		const q = searchManualCandidate.value.trim().toLowerCase();
		if (!q) return katalogProduk.value.slice(0, 30);
		return katalogProduk.value.filter((p) => p.nama.toLowerCase().includes(q)).slice(0, 30);
	});

	const filteredKatalogProduk = computed(() => {
		const q = searchProdukKatalog.value.trim().toLowerCase();
		if (!q) return katalogProduk.value;
		return katalogProduk.value.filter((p) => p.nama.toLowerCase().includes(q));
	});

	const countPerStatus = (status: string) => {
		if (!matchReport.value) return 0;
		if (status === "semua") return matchReport.value.items.length;
		if (status === "cocok") return matchReport.value.summary.totalCocok;
		return matchReport.value.items.filter((i) => i.status === status).length;
	};

	const filteredMatchItems = computed(() => {
		if (!matchReport.value) return [];
		let list = matchReport.value.items;
		if (filterStatus.value === "cocok") {
			list = list.filter((i) => i.matchedProduct !== null);
		} else if (filterStatus.value !== "semua") {
			list = list.filter((i) => i.status === filterStatus.value);
		}
		const q = filterItemSearch.value.trim().toLowerCase();
		if (q) {
			list = list.filter((i) =>
				i.namaRkas.toLowerCase().includes(q)
				|| (i.matchedProduct && i.matchedProduct.nama.toLowerCase().includes(q))
				|| (i.kodeRekening && i.kodeRekening.includes(q))
			);
		}
		return list;
	});

	const exportExcel = () => {
		if (!selectedTokoId.value || !year.value || !selectedToko.value) return;
		const y = year.value;
		const tId = selectedTokoId.value;
		const sMode = sourceMode.value;
		const safeToko = fileSafe(selectedToko.value.nama);
		const filename = `Pencocokan_SIPLah_${safeToko}_${y}`;
		xlsx(filename, (path) => api.siplahExportMatchXlsx(y, tId, sMode, path));
	};

	const formatWaktuRelatif = (dt: string) => {
		return dt.slice(0, 16);
	};

	const nominal = { th: "text-right", td: "text-right tabular-nums whitespace-nowrap" };
	const matchColumns: TableColumn<SiplahMatchItem>[] = [
		{ accessorKey: "index", header: "No", meta: { class: { th: "w-10 text-center", td: "w-10 text-center" } } },
		{ accessorKey: "namaRkas", header: "Uraian Belanja RKAS", meta: { class: { td: "min-w-40 max-w-xs" } } },
		{ accessorKey: "hargaRkas", header: "Harga RKAS", meta: { class: nominal } },
		{ accessorKey: "matchedProduct", header: "Produk Toko SIPLah", meta: { class: { td: "min-w-48 max-w-sm" } } },
		{ accessorKey: "hargaToko", header: "Harga SIPLah", meta: { class: nominal } },
		{ accessorKey: "selisih", header: "Selisih Satuan", meta: { class: nominal } }
	];

	const katalogColumns: TableColumn<SiplahProduk>[] = [
		{ accessorKey: "gambar", header: "Foto", meta: { class: { td: "w-12 text-center" } } },
		{ accessorKey: "nama", header: "Nama Produk" },
		{ accessorKey: "harga", header: "Harga", meta: { class: nominal } },
		{ accessorKey: "stok", header: "Stok", meta: { class: nominal } },
		{ accessorKey: "terjual", header: "Terjual", meta: { class: nominal } }
	];
</script>
