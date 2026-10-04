<template>
	<div v-if="connected" class="flex items-center gap-2">
		<USelect
			:model-value="year ?? undefined"
			:items="yearItems"
			placeholder="Tahun"
			icon="i-lucide-calendar"
			class="w-32"
			aria-label="Tahun anggaran"
			@update:model-value="(v) => v && setYear(Number(v))"
		/>
		<USelect
			:model-value="fund"
			:items="fundItems"
			icon="i-lucide-wallet-cards"
			class="w-56"
			aria-label="Sumber dana"
			@update:model-value="(v) => setFund(Number(v))"
		/>
	</div>
</template>

<script lang="ts" setup>
	const { connected, years, year, funds, fund, setYear, setFund } = useArkas();

	const yearItems = computed(() => years.value.map((y) => ({ label: `TA ${y}`, value: y })));
	const fundItems = computed(() => [
		{ label: "Semua sumber dana", value: ALL_FUNDS },
		...funds.value.map((f) => ({ label: f.name, value: f.id }))
	]);
</script>
