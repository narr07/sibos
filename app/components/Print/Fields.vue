<template>
	<table class="fields" :class="$attrs.class">
		<tbody>
			<tr v-for="(row, i) in visible" :key="i">
				<td class="fields-label" :style="{ width: labelWidth }">
					{{ row[0] }}
				</td>
				<td class="fields-colon">
					:
				</td>
				<td class="fields-value">
					<slot :name="`value-${i}`" :value="row[1]">
						{{ row[1] }}
					</slot>
				</td>
			</tr>
		</tbody>
	</table>
</template>

<script lang="ts" setup>
	defineOptions({ inheritAttrs: false });

	/** Daftar "Label : Nilai" dengan titik dua di kolom sendiri, sehingga nilai yang
	 *  turun ke baris berikutnya tetap sejajar dengan awal nilai di baris pertama. */
	const props = withDefaults(defineProps<{
		rows: [string, string | number | null | undefined][]
		labelWidth?: string
		/** Sembunyikan baris yang nilainya kosong. */
		hideEmpty?: boolean
	}>(), { labelWidth: "11rem", hideEmpty: false });

	const visible = computed(() =>
		props.hideEmpty ? props.rows.filter(([, v]) => v !== null && v !== undefined && v !== "") : props.rows);
</script>

<style scoped>
	.fields {
		border-collapse: collapse;
	}
	.fields td {
		vertical-align: top;
		padding: 1px 0;
	}
	.fields-colon {
		width: 1.1em;
		padding-right: 0.35em !important;
	}
</style>
