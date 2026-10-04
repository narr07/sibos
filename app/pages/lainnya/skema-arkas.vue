<template>
	<LayoutPageShell title="Skema ARKAS" :picker="false">
		<div class="space-y-4">
			<UInput v-model="search" icon="i-lucide-search" placeholder="Cari tabel atau kolom..." class="max-w-md" />

			<UAlert v-if="error" color="error" variant="subtle" icon="i-lucide-triangle-alert" :title="error" />

			<div class="grid lg:grid-cols-2 gap-3">
				<UCard v-for="table in filtered" :key="table.name" :ui="{ body: 'p-3 sm:p-3' }">
					<div class="flex items-baseline justify-between gap-2">
						<p class="font-mono font-medium">
							{{ table.name }}
						</p>
						<span class="text-xs text-muted tabular">{{ table.rowCount.toLocaleString("id-ID") }} baris</span>
					</div>
					<p class="font-mono text-xs text-muted mt-1 break-words">
						{{ table.columns.join(", ") }}
					</p>
				</UCard>
			</div>
		</div>
	</LayoutPageShell>
</template>

<script lang="ts" setup>
	const tables = ref<TableInfo[]>([]);
	const search = ref("");
	const error = ref("");

	const filtered = computed(() => {
		const q = search.value.trim().toLowerCase();
		if (!q) return tables.value;
		return tables.value.filter((t) => t.name.includes(q) || t.columns.some((c) => c.toLowerCase().includes(q)));
	});

	onMounted(async () => {
		try {
			tables.value = await api.arkasSchema();
		} catch (err) {
			error.value = errorMessage(err);
		}
	});
</script>
