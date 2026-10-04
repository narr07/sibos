<template>
	<div class="viz-root">
		<div class="flex items-center gap-4 mb-2 text-xs text-muted">
			<span v-for="(s, i) in series" :key="s.label" class="flex items-center gap-1.5">
				<span class="size-2.5 rounded-sm" :style="{ background: `var(--series-${i + 1})` }" />
				<span class="text-default">{{ s.label }}</span>
			</span>
			<UButton size="xs" color="neutral" variant="ghost" class="ml-auto" :icon="showTable ? 'i-lucide-chart-column' : 'i-lucide-table'" @click="showTable = !showTable">
				{{ showTable ? "Grafik" : "Tabel" }}
			</UButton>
		</div>

		<table v-if="showTable" class="w-full text-xs">
			<thead class="text-muted">
				<tr>
					<th class="text-left py-1">
						Bulan
					</th>
					<th v-for="s in series" :key="s.label" class="text-right py-1">
						{{ s.label }}
					</th>
				</tr>
			</thead>
			<tbody>
				<tr v-for="(m, i) in BULAN" :key="m" class="border-t border-default">
					<td class="py-1">
						{{ m }}
					</td>
					<td v-for="s in series" :key="s.label" class="text-right tabular py-1">
						{{ angka(s.values[i] ?? 0) }}
					</td>
				</tr>
			</tbody>
		</table>

		<div v-else class="relative">
			<svg :viewBox="`0 0 ${W} ${H}`" class="w-full h-auto" role="img" :aria-label="ariaLabel" @mouseleave="hover = null">
				<!-- grid & sumbu: tipis dan samar -->
				<g>
					<line
						v-for="t in ticks"
						:key="t"
						:x1="PAD_L"
						:x2="W - PAD_R"
						:y1="y(t)"
						:y2="y(t)"
						class="grid-line"
					/>
					<text v-for="t in ticks" :key="`l${t}`" :x="PAD_L - 6" :y="y(t) + 3" text-anchor="end" class="axis-label">{{ short(t) }}</text>
				</g>
				<g v-for="(_, m) in 12" :key="m">
					<rect
						:x="colX(m)"
						:y="PAD_T"
						:width="colW"
						:height="plotH"
						:class="hover === m ? 'hover-band' : 'fill-transparent'"
						@mouseenter="hover = m"
					/>
					<path
						v-for="(s, si) in series"
						:key="si"
						:d="barPath(barX(m, si), y(s.values[m] ?? 0), barW, y(0) - y(s.values[m] ?? 0))"
						:style="{ fill: `var(--series-${si + 1})` }"
						class="pointer-events-none"
					/>
					<text :x="colX(m) + colW / 2" :y="H - 6" text-anchor="middle" class="axis-label">{{ BULAN_PENDEK[m] }}</text>
				</g>
			</svg>
			<div
				v-if="hover !== null"
				class="absolute top-0 pointer-events-none rounded-md border border-default bg-default px-2.5 py-1.5 text-xs shadow-md"
				:style="tooltipStyle"
			>
				<p class="font-medium mb-0.5">
					{{ BULAN[hover] }}
				</p>
				<p v-for="(s, si) in series" :key="si" class="flex items-center gap-1.5 tabular">
					<span class="size-2 rounded-sm" :style="{ background: `var(--series-${si + 1})` }" />
					<span class="text-muted">{{ s.label }}</span>
					<span class="ml-auto pl-3">{{ rupiah(s.values[hover] ?? 0) }}</span>
				</p>
			</div>
		</div>
	</div>
</template>

<script lang="ts" setup>
	const props = defineProps<{ series: { label: string, values: number[] }[], ariaLabel?: string }>();

	const BULAN_PENDEK = ["Jan", "Feb", "Mar", "Apr", "Mei", "Jun", "Jul", "Agu", "Sep", "Okt", "Nov", "Des"];
	const W = 720;
	const H = 220;
	const PAD_L = 52;
	const PAD_R = 8;
	const PAD_T = 10;
	const PAD_B = 22;
	const plotH = H - PAD_T - PAD_B;
	const colW = (W - PAD_L - PAD_R) / 12;
	const GAP = 2;

	const hover = ref<number | null>(null);
	const showTable = ref(false);

	const max = computed(() => {
		const m = Math.max(1, ...props.series.flatMap((s) => s.values));
		const step = 10 ** Math.floor(Math.log10(m));
		return Math.ceil(m / step) * step;
	});
	const ticks = computed(() => [0, max.value / 2, max.value]);
	const y = (v: number) => PAD_T + plotH - (v / max.value) * plotH;
	const colX = (m: number) => PAD_L + m * colW;
	const barW = computed(() => Math.max(4, (colW * 0.7 - GAP * (props.series.length - 1)) / props.series.length));
	const barX = (m: number, si: number) => colX(m) + (colW - (barW.value * props.series.length + GAP * (props.series.length - 1))) / 2 + si * (barW.value + GAP);

	/** Batang dengan ujung atas membulat 4px, menempel ke garis dasar. */
	const barPath = (x: number, top: number, w: number, h: number) => {
		if (h <= 0) return "";
		const r = Math.min(4, w / 2, h);
		return `M${x},${top + h}V${top + r}Q${x},${top} ${x + r},${top}H${x + w - r}Q${x + w},${top} ${x + w},${top + r}V${top + h}Z`;
	};

	const short = (v: number) => {
		if (v >= 1e9) return `${(v / 1e9).toLocaleString("id-ID", { maximumFractionDigits: 1 })} M`;
		if (v >= 1e6) return `${(v / 1e6).toLocaleString("id-ID", { maximumFractionDigits: 1 })} jt`;
		if (v >= 1e3) return `${(v / 1e3).toLocaleString("id-ID", { maximumFractionDigits: 0 })} rb`;
		return String(v);
	};

	const tooltipStyle = computed(() => {
		const m = hover.value ?? 0;
		const pct = ((colX(m) + colW / 2) / W) * 100;
		return m > 7 ? { right: `${100 - pct + 2}%` } : { left: `${pct + 2}%` };
	});
</script>

<style scoped>
	.viz-root {
		--series-1: #2a78d6;
		--series-2: #eb6834;
		--grid: #e7e6e2;
		--axis-text: #52514e;
		--hover: rgba(0, 0, 0, 0.04);
	}
	:global(.dark) .viz-root {
		--series-1: #3987e5;
		--series-2: #d95926;
		--grid: #2e2e2c;
		--axis-text: #c3c2b7;
		--hover: rgba(255, 255, 255, 0.05);
	}
	.grid-line {
		stroke: var(--grid);
		stroke-width: 1;
	}
	.axis-label {
		fill: var(--axis-text);
		font-size: 10px;
	}
	.hover-band {
		fill: var(--hover);
	}
</style>
