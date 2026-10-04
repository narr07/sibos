<template>
	<UBadge v-if="sumber === 'arkas'" color="success" variant="subtle" size="sm" icon="i-lucide-database">
		ARKAS
	</UBadge>
	<UButton
		v-else-if="sumber === 'manual'"
		size="xs"
		color="warning"
		variant="link"
		icon="i-lucide-rotate-ccw"
		class="p-0"
		@click="emit('reset')"
	>
		Diubah · pakai data ARKAS
	</UButton>
</template>

<script lang="ts" setup>
	/** Penanda asal isian Pengaturan: dari ARKAS atau diubah manual (dengan tombol kembali ke ARKAS). */
	defineProps<{ sumber: "arkas" | "manual" | null }>();
	const emit = defineEmits<{ reset: [] }>();
</script>
