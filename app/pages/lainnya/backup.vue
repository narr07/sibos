<template>
	<LayoutPageShell title="Backup & Restore" :picker="false">
		<div class="max-w-4xl mx-auto space-y-5 pb-12">
			<div class="grid md:grid-cols-2 gap-4">
				<UCard>
					<template #header>
						<p class="font-medium flex items-center gap-2">
							<UIcon name="i-lucide-database-backup" class="size-4" /> Backup data SIBOS
						</p>
					</template>
					<p class="text-sm text-muted mb-3">
						Satu file <code>.sibos</code> berisi semua data SIBOS: pengaturan, uraian yang diubah, gabungan bukti,
						pajak manual, register kas, rekening koran, draft RKAS, dan foto nota.
					</p>
					<UButton icon="i-lucide-save" :loading="busy" @click="backup">
						Buat backup
					</UButton>
				</UCard>

				<UCard>
					<template #header>
						<p class="font-medium flex items-center gap-2">
							<UIcon name="i-lucide-archive-restore" class="size-4" /> Pulihkan data
						</p>
					</template>
					<p class="text-sm text-muted mb-3">
						Data SIBOS sekarang ditimpa isi backup. Sebelum itu SIBOS otomatis membuat backup data saat ini.
					</p>
					<UButton color="warning" variant="outline" icon="i-lucide-folder-open" :loading="busy" @click="pickRestore">
						Pilih file backup
					</UButton>
				</UCard>

				<UCard>
					<template #header>
						<p class="font-medium flex items-center gap-2">
							<UIcon name="i-lucide-copy" class="size-4" /> Salin database ARKAS
						</p>
					</template>
					<p class="text-sm text-muted mb-3">
						Salinan mentah <code>arkas.db</code> ke folder pilihan. File ARKAS asli hanya dibaca.
					</p>
					<UButton color="neutral" variant="outline" icon="i-lucide-download" :loading="busy" @click="copyArkas">
						Salin arkas.db
					</UButton>
				</UCard>

				<UCard>
					<template #header>
						<p class="font-medium flex items-center gap-2">
							<UIcon name="i-lucide-file-spreadsheet" class="size-4" /> Export semua laporan
						</p>
					</template>
					<p class="text-sm text-muted mb-3">
						Satu file Excel: BKU, buku bank, tunai, pajak, realisasi, dan kertas kerja tahun {{ year ?? "-" }}.
					</p>
					<UButton color="neutral" variant="outline" icon="i-lucide-sheet" :loading="saver.busy.value" :disabled="!connected || !year" @click="exportAll">
						Export semua
					</UButton>
				</UCard>
			</div>

			<UCard>
				<template #header>
					<p class="font-medium flex items-center gap-2">
						<UIcon name="i-lucide-timer" class="size-4" /> Backup otomatis
					</p>
				</template>
				<div class="space-y-3">
					<USwitch v-model="auto.enabled" label="Buat backup setiap kali aplikasi dibuka" />
					<div class="grid sm:grid-cols-[1fr_10rem] gap-3">
						<UFormField label="Folder tujuan">
							<UFieldGroup class="w-full">
								<UInput v-model="auto.folder" placeholder="Pilih folder..." />
								<UButton color="neutral" variant="outline" icon="i-lucide-folder-open" @click="pickFolder">
									Pilih
								</UButton>
							</UFieldGroup>
						</UFormField>
						<UFormField label="Simpan berapa file">
							<UInputNumber v-model="auto.keep" :min="1" :max="100" />
						</UFormField>
					</div>
					<p v-if="auto.last" class="text-xs text-muted">
						Backup otomatis terakhir: {{ auto.last }}
					</p>
					<UButton icon="i-lucide-check" :loading="busy" @click="saveAuto">
						Simpan pengaturan
					</UButton>
				</div>
			</UCard>
		</div>

		<UModal v-model:open="restoreOpen" title="Pulihkan backup?" :description="restoreInfo ? `Backup ${restoreInfo.manifest.sekolah ?? ''} tanggal ${restoreInfo.manifest.createdAt}, ${restoreInfo.manifest.notaFiles} foto nota.` : ''">
			<template #body>
				<p class="text-sm">
					Semua data SIBOS saat ini akan diganti isi backup ini. Data saat ini dibackup otomatis ke folder <code>backups</code> sebelum dipulihkan.
					Database ARKAS tidak terpengaruh.
				</p>
			</template>
			<template #footer>
				<div class="flex w-full justify-end gap-2">
					<UButton color="neutral" variant="outline" @click="restoreOpen = false">
						Batal
					</UButton>
					<UButton color="warning" icon="i-lucide-archive-restore" :loading="busy" @click="restore">
						Pulihkan
					</UButton>
				</div>
			</template>
		</UModal>
	</LayoutPageShell>
</template>

<script lang="ts" setup>
	const { connected, year, fund, refresh } = useArkas();
	const toast = useToast();
	const saver = useSaveFile();

	const busy = ref(false);
	const auto = reactive<AutoBackup>({ enabled: false, folder: "", keep: 10, last: null });
	const restoreOpen = ref(false);
	const restorePath = ref("");
	const restoreInfo = ref<BackupInfo | null>(null);

	onMounted(async () => {
		Object.assign(auto, await api.autoBackupGet().catch(() => ({})));
	});

	const stamp = () => {
		const d = new Date();
		return `${d.getFullYear()}${String(d.getMonth() + 1).padStart(2, "0")}${String(d.getDate()).padStart(2, "0")}`;
	};

	const backup = () => saver.save({
		defaultName: `SIBOS_Backup_${stamp()}.sibos`,
		filters: [{ name: "Backup SIBOS", extensions: ["sibos"] }],
		run: async (path) => (await api.backupCreate(path)).path,
		success: "Backup tersimpan"
	});

	const copyArkas = () => saver.save({
		defaultName: `arkas_salinan_${stamp()}.db`,
		filters: [{ name: "Database", extensions: ["db"] }],
		run: (path) => api.backupArkasDb(path),
		success: "Salinan arkas.db tersimpan"
	});

	const exportAll = () => {
		if (!year.value) return;
		const y = year.value;
		saver.xlsx(`Laporan_Lengkap_${y}`, (path) => api.exportAllXlsx(y, fund.value === ALL_FUNDS ? null : fund.value, path));
	};

	const pickRestore = async () => {
		const picked = await useTauriDialogOpen({ multiple: false, filters: [{ name: "Backup SIBOS", extensions: ["sibos"] }] });
		if (typeof picked !== "string") return;
		try {
			restoreInfo.value = await api.backupInspect(picked);
			restorePath.value = picked;
			restoreOpen.value = true;
		} catch (err) {
			toast.add({ title: "File tidak bisa dibaca", description: errorMessage(err), color: "error" });
		}
	};

	const restore = async () => {
		busy.value = true;
		try {
			await api.backupRestore(restorePath.value);
			restoreOpen.value = false;
			toast.add({ title: "Data berhasil dipulihkan", color: "success" });
			await refresh();
		} catch (err) {
			toast.add({ title: "Gagal memulihkan", description: errorMessage(err), color: "error" });
		} finally {
			busy.value = false;
		}
	};

	const pickFolder = async () => {
		const picked = await useTauriDialogOpen({ directory: true, multiple: false });
		if (typeof picked === "string") auto.folder = picked;
	};

	const saveAuto = async () => {
		busy.value = true;
		try {
			await api.autoBackupSet({ ...auto });
			toast.add({ title: "Pengaturan backup otomatis tersimpan", color: "success" });
		} catch (err) {
			toast.add({ title: "Gagal menyimpan", description: errorMessage(err), color: "error" });
		} finally {
			busy.value = false;
		}
	};
</script>
