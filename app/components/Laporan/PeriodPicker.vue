<template>
	<div class="flex flex-wrap items-center gap-2">
		<USelect v-model="mode" :items="modeItems" class="w-36" aria-label="Jenis periode" />
		<USelect v-if="mode !== 'tahun'" v-model="index" :items="indexItems" class="w-44" aria-label="Periode" />
	</div>
</template>

<script lang="ts" setup>
	type Mode = "bulan" | "triwulan" | "semester" | "tahun";

	const props = withDefaults(defineProps<{ modes?: Mode[], initial?: Mode }>(), {
		modes: () => ["bulan", "triwulan", "semester", "tahun"],
		initial: "semester"
	});
	const range = defineModel<{ start: number, end: number }>({ required: true });

	const labels: Record<Mode, string> = { bulan: "Bulanan", triwulan: "Triwulan", semester: "Semester", tahun: "Setahun" };
	const modeItems = computed(() => props.modes.map((m) => ({ label: labels[m], value: m })));

	const initialIndex = () => {
		const m = new Date().getMonth() + 1;
		return { bulan: m, triwulan: Math.ceil(m / 3), semester: m <= 6 ? 1 : 2, tahun: 1 };
	};
	const mode = ref<Mode>(props.initial);
	const index = ref<number>(initialIndex()[props.initial]);

	const indexItems = computed(() => {
		switch (mode.value) {
		case "bulan": return BULAN.map((label, i) => ({ label, value: i + 1 }));
		case "triwulan": return ["I (Jan-Mar)", "II (Apr-Jun)", "III (Jul-Sep)", "IV (Okt-Des)"].map((label, i) => ({ label: `Triwulan ${label}`, value: i + 1 }));
		case "semester": return [{ label: "Semester 1 (Jan-Jun)", value: 1 }, { label: "Semester 2 (Jul-Des)", value: 2 }];
		default: return [];
		}
	});

	watch(mode, (m) => {
		index.value = initialIndex()[m];
	});

	const toRange = (m: Mode, i: number) => {
		switch (m) {
		case "bulan": return { start: i, end: i };
		case "triwulan": return { start: (i - 1) * 3 + 1, end: i * 3 };
		case "semester": return i === 1 ? { start: 1, end: 6 } : { start: 7, end: 12 };
		default: return { start: 1, end: 12 };
		}
	};

	watch([mode, index], () => {
		range.value = toRange(mode.value, index.value);
	}, { immediate: true });
</script>
