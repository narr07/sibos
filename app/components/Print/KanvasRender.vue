<template>
	<PrintSheet :paper="template.kertas" :landscape="template.orientasi === 'landscape'" :margin="0" exact>
		<!-- Tinggi dikurangi 1 mm agar pembulatan tidak memunculkan lembar kosong saat dicetak -->
		<div class="kanvas" :style="{ width: `${pageW}mm`, height: `${pageH - 1}mm` }">
			<img
				v-if="kanvas.latar?.src"
				:src="kanvas.latar.src"
				alt=""
				class="kanvas-latar"
				:class="{ 'kanvas-latar-layar': !kanvas.latar.cetak }"
				:style="{ left: `${kanvas.latar.x}mm`, top: `${kanvas.latar.y}mm`, width: `${kanvas.latar.w}mm`, opacity: kanvas.latar.opasitas }"
			>
			<PrintKanvasElemen v-for="el in kanvas.elemen" :key="el.id" :el="el" :vars="vars" :items="g.items" />
		</div>
	</PrintSheet>
</template>

<script lang="ts" setup>
	/** Cetak template kanvas: gambar latar (opsional ikut dicetak) + elemen pada posisi mm. */
	const props = defineProps<{
		template: TemplateData
		g: NotaGroup
		penyedia: PenyediaData | null
		urut: number
	}>();
	const { pengaturan, sekolah } = useDocContext();

	const kanvas = computed(() => props.template.kanvas ?? kanvasKosong());
	const size = computed(() => ukuranHalaman(props.template));
	const pageW = computed(() => size.value[0]);
	const pageH = computed(() => size.value[1]);

	const vars = computed(() => buildVars({
		g: props.g,
		sekolah: sekolah.value,
		pengaturan: pengaturan.value,
		penyedia: props.penyedia,
		urut: props.urut,
		template: props.template
	}));
</script>

<style scoped>
	.kanvas {
		position: relative;
		overflow: hidden;
		color: #000;
		-webkit-print-color-adjust: exact;
		print-color-adjust: exact;
	}
	.kanvas-latar {
		position: absolute;
		height: auto;
		max-width: none;
		pointer-events: none;
	}
	@media print {
		.kanvas-latar-layar {
			display: none;
		}
	}
</style>
