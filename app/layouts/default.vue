<template>
	<UDashboardGroup storage="local" storage-key="sibos-sidebar">
		<UDashboardSidebar collapsible resizable :default-size="20" :min-size="16" :max-size="28" :ui="{ body: 'no-scrollbar', footer: 'flex-col items-stretch gap-1' }">
			<template #header="{ collapsed }">
				<NuxtLink to="/" class="flex items-center gap-2 min-w-0">
					<LayoutAppLogo class="size-7 shrink-0" />
					<div v-if="!collapsed" class="min-w-0">
						<p class="font-semibold leading-tight">
							{{ app.name }}
						</p>
						<p class="text-xs text-muted truncate">
							{{ app.tagline }}
						</p>
					</div>
				</NuxtLink>
			</template>

			<template #default="{ collapsed }">
				<UNavigationMenu
					v-for="(group, i) in items"
					:key="i"
					:collapsed="collapsed"
					:items="group"
					orientation="vertical"
					tooltip
					:ui="{ linkLabel: 'whitespace-normal overflow-visible text-left leading-snug' }"
				/>
			</template>

			<template #footer="{ collapsed }">
				<LayoutSettingsMenu :collapsed="collapsed" />
				<LayoutConnectionBadge :collapsed="collapsed" />
			</template>
		</UDashboardSidebar>

		<slot />
	</UDashboardGroup>
</template>

<script lang="ts" setup>
	const { app } = useAppConfig();
	const items = navigationItems();
</script>
