<template>
	<LayoutPageShell title="Tentang Aplikasi" :picker="false">
		<div class="w-full max-w-5xl mx-auto space-y-8 pb-12">
			<!-- Hero -->
			<UCard :ui="{ body: 'p-6 sm:p-8' }">
				<div class="flex flex-col sm:flex-row sm:items-center gap-6">
					<LayoutAppLogo class="size-20 shrink-0" />
					<div class="flex-1 min-w-0">
						<div class="flex flex-wrap items-center gap-2">
							<h1 class="text-3xl font-bold text-highlighted">
								{{ app.name }}
							</h1>
							<UBadge color="primary" variant="subtle">
								v{{ info?.version ?? "-" }}
							</UBadge>
							<UBadge color="neutral" variant="outline">
								Open source · MIT
							</UBadge>
						</div>
						<p class="text-muted mt-1">
							Sistem Informasi BOS: pendamping ARKAS untuk menyusun SPJ BOSP dengan cepat, rapi, dan aman.
						</p>
						<div class="flex flex-wrap gap-2 mt-4">
							<UButton icon="i-lucide-refresh-cw" :loading="checking" @click="periksa">
								Periksa pembaruan
							</UButton>
							<UButton to="/lainnya/changelog" icon="i-lucide-history" color="neutral" variant="outline">
								Changelog
							</UButton>
							<UButton icon="i-lucide-github" color="neutral" variant="ghost" @click="buka('https://github.com/narr07/sibos')">
								Kode sumber
							</UButton>
						</div>
						<p v-if="pesanUpdate" class="text-sm mt-2" :class="error ? 'text-error' : 'text-success'">
							{{ pesanUpdate }}
						</p>
					</div>
				</div>
			</UCard>

			<!-- Pembuat -->
			<section>
				<h2 class="text-lg font-semibold text-highlighted mb-3">
					Dibuat oleh
				</h2>
				<UCard>
					<div class="flex flex-col sm:flex-row sm:items-center gap-4">
						<UUser
							name="Dinar Permadi"
							description="@narr07 · Bendahara BOSP & pengembang SIBOS"
							:avatar="{ alt: 'Dinar Permadi', icon: 'i-lucide-user' }"
							size="xl"
							class="flex-1"
						/>
						<div class="flex flex-wrap gap-2">
							<UButton icon="i-lucide-globe" color="neutral" variant="outline" @click="buka('https://permadi.dev')">
								permadi.dev
							</UButton>
							<UButton icon="i-lucide-github" color="neutral" variant="outline" @click="buka('https://github.com/narr07')">
								@narr07
							</UButton>
						</div>
					</div>
				</UCard>
			</section>

			<!-- Fitur -->
			<section>
				<h2 class="text-lg font-semibold text-highlighted">
					Fitur utama
				</h2>
				<p class="text-sm text-muted mb-4">
					Semua data dibaca langsung dari ARKAS; yang dibuat di SIBOS tersimpan terpisah sehingga data ARKAS tidak pernah berubah.
				</p>
				<UPageGrid class="lg:grid-cols-3 gap-4">
					<UPageCard
						v-for="f in FITUR"
						:key="f.title"
						:icon="f.icon"
						:title="f.title"
						:description="f.description"
						:to="f.to"
						variant="subtle"
						spotlight
						:ui="{ leadingIcon: 'text-primary' }"
					/>
				</UPageGrid>
			</section>

			<!-- Prinsip -->
			<section class="grid md:grid-cols-3 gap-4">
				<UPageFeature
					v-for="p in PRINSIP"
					:key="p.title"
					:icon="p.icon"
					:title="p.title"
					:description="p.description"
					orientation="vertical"
				/>
			</section>

			<!-- Info teknis -->
			<UAccordion
				:items="[{ label: 'Informasi teknis', icon: 'i-lucide-info', slot: 'teknis' }]"
				:ui="{ item: 'border border-default rounded-lg px-4' }"
			>
				<template #teknis>
					<dl class="grid grid-cols-[9rem_1fr] gap-y-2 text-sm pb-4">
						<dt class="text-muted">
							Versi aplikasi
						</dt>
						<dd>{{ info?.version ?? "-" }}</dd>
						<dt class="text-muted">
							Versi data
						</dt>
						<dd>{{ info?.schemaVersion ?? "-" }}</dd>
						<dt class="text-muted">
							Folder data
						</dt>
						<dd class="break-all font-mono text-xs">
							{{ info?.dataDir ?? "-" }}
						</dd>
						<dt class="text-muted">
							Teknologi
						</dt>
						<dd>Tauri 2, Nuxt 4, Nuxt UI, Rust</dd>
					</dl>
					<UButton to="/lainnya/skema-arkas" color="neutral" variant="outline" size="sm" icon="i-lucide-table-properties" :disabled="!connected" class="mb-4">
						Lihat skema ARKAS (alat pengembang)
					</UButton>
				</template>
			</UAccordion>

			<p class="text-center text-xs text-muted">
				© 2026 Dinar Permadi · Dirilis dengan lisensi MIT · SIBOS bukan aplikasi resmi Kemendikdasmen
			</p>
		</div>
	</LayoutPageShell>
</template>

<script lang="ts" setup>
	const { app } = useAppConfig();
	const { connected } = useArkas();
	const { checking, error, cek } = useUpdater();
	const toast = useToast();

	const info = ref<AppInfo | null>(null);
	const pesanUpdate = ref("");

	onMounted(async () => {
		info.value = await api.appInfo().catch(() => null);
	});

	const periksa = async () => {
		pesanUpdate.value = "";
		const ada = await cek();
		pesanUpdate.value = error.value || (ada ? "" : "SIBOS sudah versi terbaru.");
	};

	/** Tautan luar dibuka di browser bawaan. */
	const buka = async (url: string) => {
		try {
			if (inTauri()) await useTauriOpenerOpenUrl(url);
			else window.open(url, "_blank");
		} catch (err) {
			toast.add({ title: "Gagal membuka tautan", description: errorMessage(err), color: "error" });
		}
	};

	const FITUR = [
		{ icon: "i-lucide-book-open", title: "Buku Kas & Pembantu", description: "BKU, Buku Pembantu Tunai, Bank, dan Pajak per bulan atau setahun, saldo cocok dengan ARKAS.", to: "/penatausahaan/bku" },
		{ icon: "i-lucide-file-spreadsheet", title: "Kertas Kerja RKAS", description: "Format Tahunan, Triwulan, Bulanan, dan Lembar Kertas Kerja dengan export Excel & PDF.", to: "/penganggaran/kertas-kerja" },
		{ icon: "i-lucide-chart-column", title: "Realisasi Belanja", description: "Bandingkan rencana RKAS dengan realisasi BKU per item, lengkap dengan status jatuh tempo.", to: "/penganggaran/realisasi" },
		{ icon: "i-lucide-printer", title: "Kwitansi & Bukti", description: "Cetak Kwitansi A2, Bukti Pengeluaran, Nota Toko, SP, dan BA Serah Terima langsung dari BKU.", to: "/penatausahaan/kwitansi" },
		{ icon: "i-lucide-layout-template", title: "Template Dokumen", description: "Susun template sendiri atau dari foto/scan kwitansi kosong, lalu bagikan lewat export/import.", to: "/lainnya/template" },
		{ icon: "i-lucide-receipt", title: "Pajak Manual", description: "Catat pajak per nota sesuai bukti setor; otomatis masuk Buku Pembantu Pajak.", to: "/penatausahaan/pajak" },
		{ icon: "i-lucide-file-check", title: "Laporan Dinas", description: "BA Rekonsiliasi, Rekonsiliasi Bank, SPTJM, K7/K7a, dan Register Penutupan Kas.", to: "/laporan/ba-rekon" },
		{ icon: "i-lucide-pencil-ruler", title: "Penyusun RKAS", description: "Buat draft Perubahan atau Pergeseran untuk simulasi sebelum diinput di ARKAS.", to: "/penganggaran/rkas" },
		{ icon: "i-lucide-database-backup", title: "Backup & Restore", description: "Cadangkan data SIBOS, backup otomatis, dan salin arkas.db dengan aman.", to: "/lainnya/backup" }
	];

	const PRINSIP = [
		{ icon: "i-lucide-shield-check", title: "ARKAS hanya dibaca", description: "SIBOS tidak pernah mengubah database ARKAS. Semua catatan SIBOS disimpan terpisah." },
		{ icon: "i-lucide-wifi-off", title: "Jalan tanpa internet", description: "Semua fitur bekerja offline. Internet hanya dipakai untuk mengecek pembaruan." },
		{ icon: "i-lucide-feather", title: "Ringan & cepat", description: "Installer hanya beberapa MB dan langsung memakai data ARKAS di komputer." }
	];
</script>
