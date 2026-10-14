<template>
	<UButton
		:to="rilis?.url || HALAMAN_RILIS"
		:size="size"
		icon="i-lucide-download"
		external
	>
		Unduh SIBOS{{ rilis ? ` ${rilis.versi}` : "" }}
		<span v-if="rilis?.ukuran && !ringkas" class="font-normal opacity-80">· Windows, {{ mb(rilis.ukuran) }}</span>
	</UButton>
</template>

<script lang="ts" setup>
	/** Tombol unduh installer Windows versi terbaru. */
	withDefaults(defineProps<{ size?: "sm" | "md" | "lg" | "xl", ringkas?: boolean }>(), { size: "xl", ringkas: false });

	const { rilis } = useRilisTerbaru();
	const mb = (byte: number) => `${(byte / 1024 / 1024).toFixed(1).replace(".", ",")} MB`;
</script>
