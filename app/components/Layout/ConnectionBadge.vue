<template>
	<UTooltip :text="tooltip" :disabled="!collapsed">
		<UButton
			to="/setup"
			color="neutral"
			variant="ghost"
			block
			:square="collapsed"
			class="justify-start"
		>
			<span class="size-2.5 rounded-full shrink-0" :class="dotClass" />
			<span v-if="!collapsed" class="min-w-0 text-left">
				<span class="block text-sm font-medium truncate">{{ label }}</span>
				<span v-if="school?.nama" class="block text-xs text-muted truncate">{{ school.nama }}</span>
			</span>
		</UButton>
	</UTooltip>
</template>

<script lang="ts" setup>
	defineProps<{ collapsed?: boolean }>();

	const { status, school } = useArkas();

	const labels: Record<string, string> = {
		connected: "Terhubung ke ARKAS",
		not_found: "Database tidak ditemukan",
		no_key: "Kunci belum diisi",
		bad_key: "Kunci salah",
		busy: "ARKAS sedang sibuk",
		error: "Gagal terhubung",
		disconnected: "Belum terhubung"
	};

	const label = computed(() => labels[status.value?.state ?? "disconnected"] ?? "Belum terhubung");
	const tooltip = computed(() => (school.value?.nama ? `${label.value} · ${school.value.nama}` : label.value));
	const dotClass = computed(() => {
		switch (status.value?.state) {
		case "connected": return "bg-success";
		case "busy": return "bg-warning";
		case undefined:
		case "disconnected": return "bg-dimmed";
		default: return "bg-error";
		}
	});
</script>
