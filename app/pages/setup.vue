<template>
	<LayoutPageShell title="Koneksi ARKAS" :picker="false">
		<div class="max-w-2xl mx-auto space-y-6">
			<UAlert
				icon="i-lucide-shield-check"
				color="info"
				variant="subtle"
				title="SIBOS hanya membaca database ARKAS"
				description="Data ARKAS tidak pernah diubah. Semua catatan buatan SIBOS disimpan terpisah di data aplikasi."
			/>

			<UAlert
				v-if="status && status.state !== 'connected' && status.state !== 'disconnected'"
				icon="i-lucide-triangle-alert"
				color="error"
				variant="subtle"
				:title="status.message ?? 'Gagal terhubung'"
			/>

			<UCard v-if="connected && school">
				<template #header>
					<div class="flex items-center gap-2">
						<span class="size-2.5 rounded-full bg-success" />
						<span class="font-medium">Terhubung</span>
						<UBadge v-if="status?.usingSnapshot" color="warning" variant="subtle" class="ml-auto">
							Membaca salinan sementara
						</UBadge>
					</div>
				</template>
				<dl class="grid grid-cols-[10rem_1fr] gap-y-2 text-sm">
					<dt class="text-muted">
						Sekolah
					</dt>
					<dd>{{ school.nama ?? "-" }}</dd>
					<dt class="text-muted">
						NPSN
					</dt>
					<dd>{{ school.npsn ?? "-" }}</dd>
					<dt class="text-muted">
						Kepala sekolah
					</dt>
					<dd>{{ school.kepalaSekolah ?? "-" }}</dd>
					<dt class="text-muted">
						Bendahara
					</dt>
					<dd>{{ school.bendahara ?? "-" }}</dd>
					<dt class="text-muted">
						File
					</dt>
					<dd class="break-all font-mono text-xs">
						{{ status?.path }}
					</dd>
				</dl>
				<template #footer>
					<div class="flex gap-2 justify-end">
						<UButton color="neutral" variant="outline" icon="i-lucide-refresh-cw" :loading="loading" @click="reconnect">
							Muat ulang
						</UButton>
						<UButton color="neutral" variant="ghost" icon="i-lucide-unplug" @click="disconnect(false)">
							Putuskan
						</UButton>
					</div>
				</template>
			</UCard>

			<UCard v-else>
				<template #header>
					<p class="font-medium">
						Hubungkan database ARKAS
					</p>
				</template>
				<UForm :state="form" class="space-y-4" @submit="submit">
					<UFormField label="File database" :hint="status?.defaultPath ? 'Default terdeteksi otomatis' : undefined">
						<UFieldGroup class="w-full">
							<UInput v-model="form.path" :placeholder="status?.defaultPath ?? 'C:\\Users\\...\\AppData\\Roaming\\Arkas\\arkas.db'" class="font-mono" />
							<UButton color="neutral" variant="outline" icon="i-lucide-folder-open" @click="pickFile">
								Pilih
							</UButton>
						</UFieldGroup>
					</UFormField>

					<UFormField label="Kunci database" :description="status?.hasStoredKey ? 'Kunci tersimpan sudah ada. Kosongkan untuk memakai kunci tersimpan.' : 'Kunci enkripsi database ARKAS.'">
						<UInput v-model="form.key" type="password" autocomplete="off" />
					</UFormField>

					<UCheckbox v-model="form.rememberKey" label="Simpan kunci di Windows Credential Manager" />

					<div class="flex justify-end">
						<UButton type="submit" icon="i-lucide-plug" :loading="loading">
							Hubungkan
						</UButton>
					</div>
				</UForm>
			</UCard>

			<p v-if="!inTauri()" class="text-sm text-muted text-center">
				Mode browser: fitur ARKAS hanya tersedia lewat aplikasi desktop.
			</p>
		</div>
	</LayoutPageShell>
</template>

<script lang="ts" setup>
	const { status, school, connected, loading, connect, disconnect } = useArkas();
	const toast = useToast();

	const form = reactive({ path: "", key: "", rememberKey: true });

	watchEffect(() => {
		if (!form.path && status.value?.path) form.path = status.value.path;
	});

	const pickFile = async () => {
		try {
			const picked = await useTauriDialogOpen({
				multiple: false,
				directory: false,
				filters: [{ name: "Database ARKAS", extensions: ["db"] }]
			});
			if (typeof picked === "string") form.path = picked;
		} catch (err) {
			toast.add({ title: "Gagal memilih file", description: errorMessage(err), color: "error" });
		}
	};

	const submit = async () => {
		try {
			const result = await connect({ path: form.path || undefined, key: form.key || undefined, rememberKey: form.rememberKey });
			form.key = "";
			if (result.state === "connected") {
				toast.add({ title: "Terhubung ke ARKAS", description: result.school?.nama ?? undefined, color: "success" });
				await navigateTo("/");
			}
		} catch (err) {
			toast.add({ title: "Gagal terhubung", description: errorMessage(err), color: "error" });
		}
	};

	const reconnect = async () => {
		try {
			await connect();
		} catch (err) {
			toast.add({ title: "Gagal memuat ulang", description: errorMessage(err), color: "error" });
		}
	};
</script>
