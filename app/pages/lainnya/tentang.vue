<template>
	<LayoutPageShell title="Tentang Aplikasi" :picker="false">
		<div class="max-w-2xl mx-auto space-y-6">
			<UCard>
				<div class="flex items-center gap-4">
					<LayoutAppLogo class="size-14" />
					<div>
						<p class="text-xl font-semibold">
							{{ app.name }}
						</p>
						<p class="text-muted">
							{{ app.tagline }}
						</p>
					</div>
				</div>
				<dl class="grid grid-cols-[10rem_1fr] gap-y-2 text-sm mt-6">
					<dt class="text-muted">
						Versi
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
				</dl>
			</UCard>

			<UCard>
				<p class="font-medium mb-2">
					Alat pengembang
				</p>
				<p class="text-sm text-muted mb-4">
					Lihat daftar tabel dan kolom database ARKAS (hanya baca) untuk memeriksa struktur data.
				</p>
				<UButton to="/lainnya/skema-arkas" color="neutral" variant="outline" icon="i-lucide-table-properties" :disabled="!connected">
					Lihat skema ARKAS
				</UButton>
			</UCard>
		</div>
	</LayoutPageShell>
</template>

<script lang="ts" setup>
	const { app } = useAppConfig();
	const { connected } = useArkas();
	const info = ref<AppInfo | null>(null);

	onMounted(async () => {
		info.value = await api.appInfo().catch(() => null);
	});
</script>
