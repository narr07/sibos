<template>
	<!-- Satu elemen kanvas pada posisi mm (dipakai saat cetak dan di editor) -->
	<div class="kel" :style="boxStyle">
		<div v-if="el.type === 'teks'" class="kel-teks" :style="teksStyle">
			<span>{{ isi }}</span>
		</div>

		<img v-else-if="el.type === 'gambar' && el.src" :src="el.src" alt="" class="w-full h-full object-contain">
		<div v-else-if="el.type === 'gambar'" class="kel-kosong">
			Gambar
		</div>

		<div v-else-if="el.type === 'garis'" class="w-full" :style="{ borderTop: `${el.tebalGaris ?? 1}pt solid ${el.warna ?? '#000'}` }" />

		<div v-else-if="el.type === 'kotak'" class="w-full h-full" :style="{ border: `${el.tebalGaris ?? 1}pt solid ${el.warna ?? '#000'}` }" />

		<table v-else-if="el.type === 'tabel'" class="kel-tabel" :class="{ 'kel-tabel-garis': el.garisSel }" :style="teksStyle">
			<colgroup>
				<col v-for="(k, i) in el.kolom" :key="i" :style="{ width: `${k.lebar}mm` }">
			</colgroup>
			<tbody>
				<tr v-for="(it, r) in barisTabel" :key="r" :style="{ height: `${el.tinggiBaris ?? 6}mm` }">
					<td v-for="(k, c) in el.kolom" :key="c" :class="kolomAngka(k.kunci) ? 'text-right' : k.kunci === 'no' ? 'text-center' : ''">
						{{ it ? isiSelTabel(k.kunci, it, r, el.bersihkanNama ?? true) : "" }}
					</td>
				</tr>
			</tbody>
		</table>
	</div>
</template>

<script lang="ts" setup>
	const props = defineProps<{
		el: ElemenKanvas
		vars: Record<string, string> | null
		items?: NotaItem[]
	}>();

	const isi = computed(() => (props.vars ? fillVars(props.el.teks ?? "", props.vars) : props.el.teks ?? ""));

	const boxStyle = computed(() => ({
		left: `${props.el.x}mm`,
		top: `${props.el.y}mm`,
		width: `${props.el.w}mm`,
		height: props.el.type === "garis" ? "0" : `${props.el.h}mm`
	}));

	const teksStyle = computed(() => ({
		fontSize: `${props.el.ukuran ?? 11}pt`,
		fontWeight: props.el.tebal ? "bold" : "normal",
		fontStyle: props.el.miring ? "italic" : "normal",
		fontFamily: props.el.font === "sans" ? "Arial, Helvetica, sans-serif" : "'Times New Roman', Times, serif",
		color: props.el.warna ?? "#000",
		textAlign: props.el.rata ?? "left",
		justifyContent: { top: "flex-start", middle: "center", bottom: "flex-end" }[props.el.tegak ?? "middle"]
	}));

	/** Baris tabel: item nota; di editor tanpa data, tampil baris contoh kosong. */
	const barisTabel = computed<(NotaItem | null)[]>(() => {
		if (props.items?.length) return props.items;
		return [null, null, null];
	});
</script>

<style scoped>
	.kel {
		position: absolute;
		box-sizing: border-box;
	}
	.kel-teks {
		width: 100%;
		height: 100%;
		display: flex;
		flex-direction: column;
		line-height: 1.2;
		white-space: pre-wrap;
		overflow: visible;
	}
	.kel-kosong {
		width: 100%;
		height: 100%;
		border: 1px dashed #999;
		color: #999;
		font-size: 8pt;
		display: flex;
		align-items: center;
		justify-content: center;
	}
	.kel-tabel {
		width: 100%;
		border-collapse: collapse;
		table-layout: fixed;
		line-height: 1.15;
	}
	.kel-tabel td {
		padding: 0 1.5mm;
		vertical-align: middle;
		overflow: hidden;
		white-space: nowrap;
		text-overflow: ellipsis;
	}
	.kel-tabel-garis td {
		border: 0.75pt solid #000;
	}
</style>
