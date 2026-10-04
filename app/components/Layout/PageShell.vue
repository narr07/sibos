<template>
	<UDashboardPanel :id="panelId">
		<template #header>
			<UDashboardNavbar :title="title">
				<template #leading>
					<UDashboardSidebarCollapse />
				</template>
				<template #right>
					<slot name="actions" />
					<LayoutYearFundPicker v-if="picker" />
				</template>
			</UDashboardNavbar>
		</template>

		<template #body>
			<slot />
		</template>
	</UDashboardPanel>
</template>

<script lang="ts" setup>
	const props = withDefaults(defineProps<{ title: string, picker?: boolean }>(), { picker: true });
	const panelId = computed(() => `panel-${slugify(props.title)}`);
</script>
