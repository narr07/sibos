<template>
	<div class="space-y-2">
		<div class="flex gap-2">
			<USelect v-model="month" :items="monthItems" icon="i-lucide-calendar-days" class="w-40" aria-label="Bulan" />
			<UInput v-model="search" icon="i-lucide-search" placeholder="Cari uraian, toko..." class="flex-1" />
		</div>
		<USwitch v-model="hanya" :label="labelSaring" />
		<UAlert v-if="error" color="error" variant="subtle" icon="i-lucide-triangle-alert" :title="error" />
		<p class="text-xs text-muted">
			{{ groups.length }} belanja dari BKU ARKAS{{ loading ? " · memuat..." : "" }}
		</p>
		<div class="max-h-[calc(100vh-16rem)] overflow-auto space-y-1">
			<UButton
				v-for="g in groups"
				:key="g.key"
				block
				:color="g.key === selectedKey ? 'primary' : 'neutral'"
				:variant="g.key === selectedKey ? 'soft' : 'outline'"
				class="justify-start text-left"
				@click="emit('pick', g)"
			>
				<div class="min-w-0 w-full">
					<p class="text-sm font-medium line-clamp-2">
						{{ g.uraian }}
					</p>
					<p class="text-xs text-muted">
						{{ tanggalId(g.tanggal.slice(0, 10)) }} · {{ rupiah(g.total) }}{{ g.key in tersimpan ? " · tersimpan" : "" }}
					</p>
				</div>
			</UButton>
			<p v-if="!groups.length && !loading" class="py-8 text-center text-sm text-muted">
				{{ hanya ? "Tidak ada belanja yang cocok. Matikan saringan untuk melihat semua belanja." : "Tidak ada belanja pada periode ini." }}
			</p>
		</div>
	</div>
</template>

<script lang="ts" setup>
	/** Daftar belanja BKU di sisi kiri halaman generator: pilih bulan, cari, dan saring. */
	defineProps<{
		groups: NotaGroup[]
		selectedKey: string | null
		/** Isian tersimpan per key nota (hanya dipakai untuk tanda "tersimpan"). */
		tersimpan: Record<string, unknown>
		loading: boolean
		error: string
		labelSaring: string
	}>();
	const emit = defineEmits<{ pick: [g: NotaGroup] }>();

	const month = defineModel<number>("month", { required: true });
	const search = defineModel<string>("search", { required: true });
	const hanya = defineModel<boolean>("hanya", { required: true });

	const monthItems = [{ label: "Semua bulan", value: 0 }, ...BULAN.map((label, i) => ({ label, value: i + 1 }))];
</script>
