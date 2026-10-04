<template>
	<LayoutPageShell title="Changelog" :picker="false">
		<div class="w-full max-w-5xl mx-auto pb-12 space-y-6">
			<!-- Pengantar tetap sempit; daftar versi memakai lebar penuh agar garis & tanggal di kolom kiri -->
			<div class="w-full max-w-2xl mx-auto space-y-4">
				<div class="flex flex-wrap items-center gap-2 text-muted">
					<span>Riwayat versi dan perubahan SIBOS. Versi terpasang:</span>
					<UBadge color="neutral" variant="subtle">
						{{ versiTerpasang ? `v${versiTerpasang}` : "tidak diketahui" }}
					</UBadge>
				</div>

				<UAlert
					v-if="adaVersiBaru"
					color="primary"
					variant="subtle"
					icon="i-lucide-sparkles"
					:title="`Versi baru tersedia: v${terbaru?.versi}`"
					description="Perbarui SIBOS untuk mendapatkan fitur dan perbaikan terbaru."
					:actions="[{ label: 'Periksa pembaruan', icon: 'i-lucide-refresh-cw', onClick: () => cek() }]"
				/>
			</div>

			<UChangelogVersions :versions="versions">
				<template #date="{ version }">
					{{ tanggalPanjang(version.tanggal) }}
				</template>
				<template #body="{ version }">
					<div class="space-y-4">
						<div v-for="(items, jenis) in kelompok(version.perubahan)" :key="jenis">
							<p class="mb-1 text-xs font-medium uppercase tracking-wide" :class="JENIS[jenis].kelas">
								{{ JENIS[jenis].label }}
							</p>
							<ul class="list-disc space-y-1 pl-5 text-sm">
								<li v-for="(p, j) in items" :key="j">
									{{ p.teks }}
								</li>
							</ul>
						</div>
					</div>
				</template>
			</UChangelogVersions>
		</div>
	</LayoutPageShell>
</template>

<script lang="ts" setup>
	type Jenis = VersiRilis["perubahan"][number]["jenis"];

	const JENIS: Record<Jenis, { label: string, kelas: string }> = {
		baru: { label: "Fitur baru", kelas: "text-success" },
		perbaikan: { label: "Perbaikan", kelas: "text-warning" },
		ubah: { label: "Perubahan", kelas: "text-info" }
	};
	const URUTAN: Jenis[] = ["baru", "perbaikan", "ubah"];

	const { cek } = useUpdater();
	const info = ref<AppInfo | null>(null);
	onMounted(async () => {
		info.value = await api.appInfo().catch(() => null);
	});

	/** Versi terpasang dari aplikasi; kosong bila tidak diketahui (tidak menebak dari changelog). */
	const versiTerpasang = computed(() => info.value?.version ?? "");
	const terbaru = computed(() => CHANGELOG[0]);

	/** "0.10.0" > "0.9.0": bandingkan per angka. */
	const lebihBaru = (a: string, b: string) => {
		const pa = a.split(".").map(Number);
		const pb = b.split(".").map(Number);
		for (let i = 0; i < Math.max(pa.length, pb.length); i++) {
			if ((pa[i] ?? 0) !== (pb[i] ?? 0)) return (pa[i] ?? 0) > (pb[i] ?? 0);
		}
		return false;
	};
	const adaVersiBaru = computed(() => !!versiTerpasang.value && !!terbaru.value && lebihBaru(terbaru.value.versi, versiTerpasang.value));

	const versions = computed(() => CHANGELOG.map((v, i) => ({
		title: `v${v.versi} · ${v.judul}`,
		description: v.ringkasan,
		date: v.tanggal,
		badge: v.versi === versiTerpasang.value
			? { label: "Terpasang", color: "success" as const, variant: "subtle" as const }
			: i === 0 ? { label: "Terbaru", color: "primary" as const, variant: "subtle" as const } : undefined,
		tanggal: v.tanggal,
		perubahan: v.perubahan
	})));

	/** Perubahan dikelompokkan per jenis (fitur baru, perbaikan, perubahan). */
	const kelompok = (list: VersiRilis["perubahan"]) => {
		const out: Partial<Record<Jenis, VersiRilis["perubahan"]>> = {};
		for (const jenis of URUTAN) {
			const items = list.filter((p) => p.jenis === jenis);
			if (items.length) out[jenis] = items;
		}
		return out as Record<Jenis, VersiRilis["perubahan"]>;
	};
</script>
