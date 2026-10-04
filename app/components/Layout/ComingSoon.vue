<template>
	<LayoutPageShell :title="page?.label ?? 'Halaman'">
		<div class="max-w-2xl mx-auto py-12 text-center space-y-4">
			<UIcon :name="page?.icon ?? 'i-lucide-construction'" class="size-12 text-dimmed mx-auto" />
			<h2 class="text-xl font-semibold">
				{{ page?.label }}
			</h2>
			<p class="text-muted">
				{{ page?.description }}
			</p>
			<div class="flex justify-center gap-2 flex-wrap">
				<UBadge v-if="page && page.phase < 99" color="neutral" variant="subtle">
					Dikerjakan di Fase {{ page.phase }}
				</UBadge>
				<UBadge v-else color="warning" variant="subtle">
					Menunggu keputusan
				</UBadge>
				<UBadge v-if="page?.redesign" color="info" variant="subtle">
					Desain ulang di Fase 6
				</UBadge>
			</div>
		</div>
	</LayoutPageShell>
</template>

<script lang="ts" setup>
	const route = useRoute();
	const page = computed(() => findPage(route.path));
</script>
