<template>
	<UModal
		v-model:open="open"
		:dismissible="!installing"
		:close="!installing"
		:title="update ? `SIBOS ${update.version} tersedia` : 'Pembaruan SIBOS'"
		:description="update ? `Versi terpasang ${update.currentVersion}${update.date ? ` · dirilis ${tanggalRilis}` : ''}` : ''"
	>
		<template #body>
			<div class="space-y-3">
				<div v-if="update?.body" class="max-h-64 overflow-auto rounded-md bg-elevated p-3 text-sm whitespace-pre-wrap">
					{{ update.body }}
				</div>
				<p v-else class="text-sm text-muted">
					Versi baru berisi perbaikan dan fitur terbaru.
				</p>
				<template v-if="installing">
					<UProgress :model-value="progress || null" />
					<p class="text-xs text-muted">
						{{ progress < 100 ? `Mengunduh pembaruan... ${progress}%` : "Memasang pembaruan, SIBOS akan dibuka kembali." }}
					</p>
				</template>
				<UAlert v-if="error" color="error" variant="subtle" icon="i-lucide-triangle-alert" title="Pembaruan gagal" :description="error" />
				<p class="text-xs text-muted">
					Data SIBOS dan koneksi ARKAS tetap aman; pembaruan hanya mengganti program.
				</p>
			</div>
		</template>
		<template #footer>
			<div class="flex w-full justify-end gap-2">
				<UButton color="neutral" variant="outline" :disabled="installing" @click="nanti">
					Nanti
				</UButton>
				<UButton icon="i-lucide-download" :loading="installing" @click="pasang">
					Update sekarang
				</UButton>
			</div>
		</template>
	</UModal>
</template>

<script lang="ts" setup>
	/** Cek pembaruan diam-diam saat aplikasi dibuka, hanya bila ada internet. */
	const { update, open, installing, progress, error, cek, pasang, nanti } = useUpdater();

	const tanggalRilis = computed(() => (update.value?.date ? tanggalPanjang(update.value.date.slice(0, 10)) : ""));

	let tunggu: ReturnType<typeof setTimeout> | undefined;
	const cekSekali = () => {
		window.removeEventListener("online", cekSekali);
		cek(true);
	};

	onMounted(() => {
		if (!inTauri()) return;
		if (navigator.onLine) {
			// Beri waktu aplikasi selesai memuat data dulu.
			tunggu = setTimeout(cek, 5000, true);
		} else {
			// Offline: cek begitu internet tersambung.
			window.addEventListener("online", cekSekali);
		}
	});

	onBeforeUnmount(() => {
		clearTimeout(tunggu);
		window.removeEventListener("online", cekSekali);
	});
</script>
