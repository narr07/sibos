<template>
	<div class="space-y-2">
		<!-- Toolbar -->
		<div class="flex flex-wrap items-center gap-1.5">
			<UButton size="sm" color="neutral" variant="outline" icon="i-lucide-type" @click="tambah('teks')">
				Teks
			</UButton>
			<UDropdownMenu :items="isianMenu" :content="{ align: 'start' }" :ui="{ content: 'max-h-80' }">
				<UButton size="sm" color="neutral" variant="outline" icon="i-lucide-braces" trailing-icon="i-lucide-chevron-down">
					Isian otomatis
				</UButton>
			</UDropdownMenu>
			<UButton size="sm" color="neutral" variant="outline" icon="i-lucide-table" @click="tambah('tabel')">
				Tabel barang
			</UButton>
			<UButton size="sm" color="neutral" variant="outline" icon="i-lucide-image" @click="tambah('gambar')">
				Gambar
			</UButton>
			<UButton size="sm" color="neutral" variant="outline" icon="i-lucide-minus" @click="tambah('garis')">
				Garis
			</UButton>
			<UButton size="sm" color="neutral" variant="outline" icon="i-lucide-square" @click="tambah('kotak')">
				Kotak
			</UButton>
			<USeparator orientation="vertical" class="h-6 mx-1" />
			<UFileUpload v-slot="{ open }" accept="image/*" reset :preview="false" @update:model-value="unggahLatar">
				<UButton size="sm" color="primary" variant="soft" icon="i-lucide-scan" @click="open()">
					{{ kanvas.latar ? "Ganti foto/scan" : "Unggah foto/scan kwitansi" }}
				</UButton>
			</UFileUpload>
			<div class="ml-auto flex items-center gap-2 text-xs text-muted">
				<span>Zoom</span>
				<USlider v-model="zoom" :min="0.4" :max="1.6" :step="0.05" class="w-28" />
				<span class="tabular w-9">{{ Math.round(zoom * 100) }}%</span>
			</div>
		</div>

		<div class="grid xl:grid-cols-[minmax(0,1fr)_18rem] gap-3">
			<!-- Kertas -->
			<div
				ref="area"
				tabindex="0"
				class="overflow-auto rounded-md bg-elevated p-4 max-h-[calc(100vh-14rem)] outline-none focus-visible:ring-2 ring-primary"
				@keydown="onKey"
				@pointerdown.self="pilih(null)"
			>
				<div :style="{ width: `${pageW * MM * zoom}px`, height: `${pageH * MM * zoom}px` }" @pointerdown.self="pilih(null)">
					<div
						class="kanvas-kertas"
						:style="{ width: `${pageW}mm`, height: `${pageH}mm`, transform: `scale(${zoom})` }"
						@pointerdown.self="pilih(null)"
					>
						<img
							v-if="kanvas.latar?.src"
							:src="kanvas.latar.src"
							alt=""
							class="kanvas-latar"
							:class="{ 'cursor-move': geserLatar }"
							:style="{ left: `${kanvas.latar.x}mm`, top: `${kanvas.latar.y}mm`, width: `${kanvas.latar.w}mm`, opacity: kanvas.latar.opasitas, pointerEvents: geserLatar ? 'auto' : 'none' }"
							draggable="false"
							@pointerdown.stop="mulaiGeserLatar"
						>
						<PrintKanvasElemen v-for="el in kanvas.elemen" :key="el.id" :el="el" :vars="vars" :items="items" />
						<!-- Lapisan interaksi -->
						<div
							v-for="el in kanvas.elemen"
							:key="`h${el.id}`"
							class="kanvas-pegangan"
							:class="el.id === selectedId ? 'is-aktif' : ''"
							:style="{ left: `${el.x}mm`, top: `${el.type === 'garis' ? el.y - 1.5 : el.y}mm`, width: `${el.w}mm`, height: `${el.type === 'garis' ? 3 : el.h}mm` }"
							@pointerdown.stop="(e) => mulaiGeser(e, el, 'pindah')"
						>
							<span v-if="el.id === selectedId" class="kanvas-ubah" @pointerdown.stop="(e) => mulaiGeser(e, el, 'ubah')" />
						</div>
					</div>
				</div>
			</div>

			<!-- Panel properti -->
			<div class="space-y-3 text-sm">
				<UCard v-if="kanvas.latar" :ui="{ body: 'p-3 sm:p-3 space-y-2' }">
					<p class="font-medium text-xs uppercase text-muted">
						Foto/scan latar
					</p>
					<USwitch v-model="kanvas.latar.cetak" label="Ikut dicetak" description="Matikan bila mencetak di kertas kwitansi asli" />
					<USwitch v-model="geserLatar" label="Geser latar dengan mouse" />
					<UFormField label="Transparansi">
						<USlider v-model="kanvas.latar.opasitas" :min="0.1" :max="1" :step="0.05" />
					</UFormField>
					<div class="grid grid-cols-3 gap-2">
						<UFormField label="X (mm)">
							<UInputNumber v-model="kanvas.latar.x" :step="0.5" size="xs" />
						</UFormField>
						<UFormField label="Y (mm)">
							<UInputNumber v-model="kanvas.latar.y" :step="0.5" size="xs" />
						</UFormField>
						<UFormField label="Lebar">
							<UInputNumber v-model="kanvas.latar.w" :min="10" :step="0.5" size="xs" />
						</UFormField>
					</div>
					<div class="flex gap-1">
						<UButton size="xs" color="neutral" variant="outline" @click="latarPenuh">
							Selebar kertas
						</UButton>
						<UButton size="xs" color="error" variant="ghost" icon="i-lucide-trash-2" class="ml-auto" @click="kanvas.latar = null">
							Hapus latar
						</UButton>
					</div>
				</UCard>

				<UCard v-if="sel" :ui="{ body: 'p-3 sm:p-3 space-y-2' }">
					<div class="flex items-center gap-1">
						<p class="font-medium text-xs uppercase text-muted flex-1">
							{{ ELEMEN_LABEL[sel.type] }}
						</p>
						<UButton size="xs" color="neutral" variant="ghost" icon="i-lucide-copy" aria-label="Duplikat" @click="duplikat" />
						<UButton size="xs" color="error" variant="ghost" icon="i-lucide-trash-2" aria-label="Hapus" @click="hapus" />
					</div>

					<template v-if="sel.type === 'teks'">
						<UTextarea v-model="sel.teks" :rows="3" autoresize class="w-full" placeholder="Teks atau {isian}" />
						<UDropdownMenu :items="sisipMenu" :ui="{ content: 'max-h-72' }">
							<UButton size="xs" color="neutral" variant="link" icon="i-lucide-braces" class="p-0">
								Sisipkan isian
							</UButton>
						</UDropdownMenu>
					</template>

					<div class="grid grid-cols-2 gap-2">
						<UFormField label="X (mm)">
							<UInputNumber v-model="sel.x" :step="0.5" size="xs" />
						</UFormField>
						<UFormField label="Y (mm)">
							<UInputNumber v-model="sel.y" :step="0.5" size="xs" />
						</UFormField>
						<UFormField label="Lebar (mm)">
							<UInputNumber v-model="sel.w" :min="1" :step="0.5" size="xs" />
						</UFormField>
						<UFormField v-if="sel.type !== 'garis'" label="Tinggi (mm)">
							<UInputNumber v-model="sel.h" :min="1" :step="0.5" size="xs" />
						</UFormField>
					</div>

					<template v-if="sel.type === 'teks' || sel.type === 'tabel'">
						<div class="grid grid-cols-2 gap-2">
							<UFormField label="Huruf (pt)">
								<UInputNumber v-model="sel.ukuran" :min="5" :max="40" :step="0.5" size="xs" />
							</UFormField>
							<UFormField label="Jenis huruf">
								<USelect v-model="sel.font" :items="[{ label: 'Times', value: 'serif' }, { label: 'Arial', value: 'sans' }]" size="xs" />
							</UFormField>
						</div>
						<div v-if="sel.type === 'teks'" class="flex flex-wrap items-center gap-1">
							<UButton size="xs" :variant="sel.tebal ? 'solid' : 'outline'" color="neutral" icon="i-lucide-bold" aria-label="Tebal" @click="sel.tebal = !sel.tebal" />
							<UButton size="xs" :variant="sel.miring ? 'solid' : 'outline'" color="neutral" icon="i-lucide-italic" aria-label="Miring" @click="sel.miring = !sel.miring" />
							<UFieldGroup size="xs">
								<UButton v-for="r in RATA" :key="r.value" :icon="r.icon" :variant="sel.rata === r.value ? 'solid' : 'outline'" color="neutral" :aria-label="r.label" @click="sel.rata = r.value" />
							</UFieldGroup>
							<UFieldGroup size="xs">
								<UButton v-for="t in TEGAK" :key="t.value" :icon="t.icon" :variant="sel.tegak === t.value ? 'solid' : 'outline'" color="neutral" :aria-label="t.label" @click="sel.tegak = t.value" />
							</UFieldGroup>
						</div>
					</template>

					<UFormField v-if="sel.type === 'teks' || sel.type === 'garis' || sel.type === 'kotak'" label="Warna">
						<UPopover>
							<UButton color="neutral" variant="outline" size="xs" block class="justify-start">
								<span class="size-4 rounded-sm ring ring-default" :style="{ backgroundColor: sel.warna }" />
								<span class="font-mono">{{ sel.warna }}</span>
							</UButton>
							<template #content>
								<UColorPicker v-model="sel.warna" class="p-2" />
							</template>
						</UPopover>
					</UFormField>

					<UFormField v-if="sel.type === 'garis' || sel.type === 'kotak'" label="Tebal garis (pt)">
						<UInputNumber v-model="sel.tebalGaris" :min="0.25" :max="6" :step="0.25" size="xs" />
					</UFormField>

					<UFileUpload v-if="sel.type === 'gambar'" v-slot="{ open }" accept="image/*" reset :preview="false" @update:model-value="unggahGambar">
						<UButton size="xs" color="neutral" variant="outline" icon="i-lucide-image-up" block @click="open()">
							Pilih gambar
						</UButton>
					</UFileUpload>

					<template v-if="sel.type === 'tabel'">
						<div class="grid grid-cols-2 gap-2">
							<UFormField label="Tinggi baris (mm)">
								<UInputNumber v-model="sel.tinggiBaris" :min="3" :max="20" :step="0.5" size="xs" />
							</UFormField>
							<div class="flex flex-col justify-end gap-1">
								<UCheckbox v-model="sel.garisSel" label="Garis sel" />
								<UCheckbox v-model="sel.bersihkanNama" label="Nama ringkas" />
							</div>
						</div>
						<p class="text-xs text-muted">
							Kolom (lebar mm). Untuk kwitansi bergaris, matikan "Garis sel" lalu sesuaikan lebar dengan kolom di foto.
						</p>
						<div v-for="(k, i) in sel.kolom" :key="i" class="flex items-center gap-1">
							<USelect v-model="k.kunci" :items="kolomItems" size="xs" class="flex-1 min-w-0" />
							<UInputNumber v-model="k.lebar" :min="3" :step="0.5" size="xs" class="w-20" />
							<UButton size="xs" color="error" variant="ghost" icon="i-lucide-x" aria-label="Hapus kolom" @click="sel.kolom!.splice(i, 1)" />
						</div>
						<UButton size="xs" color="neutral" variant="link" icon="i-lucide-plus" class="p-0" @click="sel.kolom!.push({ kunci: 'nama', lebar: 20 })">
							Tambah kolom
						</UButton>
					</template>
				</UCard>

				<p v-else class="text-xs text-muted">
					Klik elemen di kertas untuk mengatur. Geser dengan mouse, ubah ukuran dari pojok kanan bawah,
					atau gunakan tombol panah (Shift = 5 mm). Tekan Delete untuk menghapus.
				</p>
			</div>
		</div>
	</div>
</template>

<script lang="ts" setup>
	import type { DropdownMenuItem } from "@nuxt/ui";

	defineProps<{ vars: Record<string, string> | null, items?: NotaItem[] }>();
	const data = defineModel<TemplateData>({ required: true });
	const toast = useToast();

	/** 1 mm dalam piksel CSS. */
	const MM = 96 / 25.4;
	const SNAP = 0.5;
	const snap = (v: number) => Math.round(v / SNAP) * SNAP;

	const RATA = [
		{ value: "left", icon: "i-lucide-align-left", label: "Rata kiri" },
		{ value: "center", icon: "i-lucide-align-center", label: "Rata tengah" },
		{ value: "right", icon: "i-lucide-align-right", label: "Rata kanan" }
	] as const;
	const TEGAK = [
		{ value: "top", icon: "i-lucide-align-vertical-justify-start", label: "Atas" },
		{ value: "middle", icon: "i-lucide-align-vertical-justify-center", label: "Tengah" },
		{ value: "bottom", icon: "i-lucide-align-vertical-justify-end", label: "Bawah" }
	] as const;
	const kolomItems = Object.entries(KOLOM_LABEL).map(([value, label]) => ({ label, value }));

	if (!data.value.kanvas) data.value.kanvas = kanvasKosong();
	const kanvas = computed(() => data.value.kanvas!);
	const size = computed(() => ukuranHalaman(data.value));
	const pageW = computed(() => size.value[0]);
	const pageH = computed(() => size.value[1]);

	const zoom = ref(0.8);
	const geserLatar = ref(false);
	const selectedId = ref<string | null>(null);
	const sel = computed(() => kanvas.value.elemen.find((e) => e.id === selectedId.value) ?? null);
	const area = useTemplateRef<HTMLElement>("area");

	const pilih = (id: string | null) => {
		selectedId.value = id;
	};

	/** Elemen baru muncul di tengah area yang sedang terlihat. */
	const posisiBaru = (): [number, number] => {
		const el = area.value;
		if (!el) return [20, 20];
		const x = (el.scrollLeft + el.clientWidth / 3) / (MM * zoom.value);
		const y = (el.scrollTop + el.clientHeight / 3) / (MM * zoom.value);
		return [snap(Math.min(Math.max(5, x), pageW.value - 30)), snap(Math.min(Math.max(5, y), pageH.value - 20))];
	};

	const tambah = (type: ElemenKanvas["type"], teks?: string) => {
		const [x, y] = posisiBaru();
		const el = elemenBaru(type, x, y, teks);
		kanvas.value.elemen.push(el);
		pilih(el.id);
		area.value?.focus();
	};

	const isianMenu = computed<DropdownMenuItem[]>(() => DAFTAR_ISIAN.map((v) => ({
		label: `{${v.kunci}}`,
		description: v.ket,
		onSelect: () => tambah("teks", `{${v.kunci}}`)
	})));
	const sisipMenu = computed<DropdownMenuItem[]>(() => DAFTAR_ISIAN.map((v) => ({
		label: `{${v.kunci}}`,
		description: v.ket,
		onSelect: () => {
			if (sel.value) sel.value.teks = `${sel.value.teks ?? ""}{${v.kunci}}`;
		}
	})));

	const hapus = () => {
		if (!sel.value) return;
		kanvas.value.elemen = kanvas.value.elemen.filter((e) => e.id !== sel.value!.id);
		pilih(null);
	};
	const duplikat = () => {
		if (!sel.value) return;
		const copy: ElemenKanvas = { ...structuredClone(toRaw(sel.value)), id: Math.random().toString(36).slice(2, 10), x: sel.value.x + 3, y: sel.value.y + 3 };
		kanvas.value.elemen.push(copy);
		pilih(copy.id);
	};

	// Geser & ubah ukuran dengan mouse (posisi disimpan dalam mm, dibulatkan 0,5 mm; tahan Alt = tanpa pembulatan).
	const mulaiGeser = (e: PointerEvent, el: ElemenKanvas, mode: "pindah" | "ubah") => {
		if (e.button !== 0) return;
		pilih(el.id);
		area.value?.focus({ preventScroll: true });
		const start = { px: e.clientX, py: e.clientY, x: el.x, y: el.y, w: el.w, h: el.h };
		const move = (ev: PointerEvent) => {
			const dx = (ev.clientX - start.px) / (MM * zoom.value);
			const dy = (ev.clientY - start.py) / (MM * zoom.value);
			const r = ev.altKey ? (v: number) => Math.round(v * 10) / 10 : snap;
			if (mode === "pindah") {
				el.x = r(start.x + dx);
				el.y = r(start.y + dy);
			} else {
				el.w = Math.max(2, r(start.w + dx));
				if (el.type !== "garis") el.h = Math.max(2, r(start.h + dy));
			}
		};
		const up = () => {
			window.removeEventListener("pointermove", move);
			window.removeEventListener("pointerup", up);
		};
		window.addEventListener("pointermove", move);
		window.addEventListener("pointerup", up);
	};

	const mulaiGeserLatar = (e: PointerEvent) => {
		const latar = kanvas.value.latar;
		if (!latar || !geserLatar.value || e.button !== 0) return;
		pilih(null);
		const start = { px: e.clientX, py: e.clientY, x: latar.x, y: latar.y };
		const move = (ev: PointerEvent) => {
			latar.x = snap(start.x + (ev.clientX - start.px) / (MM * zoom.value));
			latar.y = snap(start.y + (ev.clientY - start.py) / (MM * zoom.value));
		};
		const up = () => {
			window.removeEventListener("pointermove", move);
			window.removeEventListener("pointerup", up);
		};
		window.addEventListener("pointermove", move);
		window.addEventListener("pointerup", up);
	};

	const onKey = (e: KeyboardEvent) => {
		const target = e.target as HTMLElement;
		if (!sel.value || target !== area.value) return;
		const step = e.shiftKey ? 5 : SNAP;
		const moves: Record<string, [number, number]> = { ArrowLeft: [-step, 0], ArrowRight: [step, 0], ArrowUp: [0, -step], ArrowDown: [0, step] };
		const d = moves[e.key];
		if (d) {
			e.preventDefault();
			sel.value.x = snap(sel.value.x + d[0]);
			sel.value.y = snap(sel.value.y + d[1]);
		} else if (e.key === "Delete" || e.key === "Backspace") {
			e.preventDefault();
			hapus();
		} else if (e.key === "Escape") {
			pilih(null);
		}
	};

	const unggahLatar = async (file: File | null | undefined) => {
		if (!file) return;
		try {
			const { src, rasio } = await fileToJpeg(file);
			const lama = kanvas.value.latar;
			kanvas.value.latar = { src, rasio, x: lama?.x ?? 0, y: lama?.y ?? 0, w: lama?.w ?? pageW.value, opasitas: lama?.opasitas ?? 1, cetak: lama?.cetak ?? true };
		} catch (err) {
			toast.add({ title: "Gagal memuat gambar", description: errorMessage(err), color: "error" });
		}
	};
	const latarPenuh = () => {
		const latar = kanvas.value.latar;
		if (!latar) return;
		latar.x = 0;
		latar.y = 0;
		latar.w = pageW.value;
	};

	const unggahGambar = async (file: File | null | undefined) => {
		if (!file || !sel.value) return;
		try {
			sel.value.src = await fileToDataUrl(file, 800);
		} catch (err) {
			toast.add({ title: "Gagal memuat gambar", description: errorMessage(err), color: "error" });
		}
	};
</script>

<style scoped>
	.kanvas-kertas {
		position: relative;
		transform-origin: top left;
		background: #fff;
		color: #000;
		box-shadow: 0 1px 6px rgb(0 0 0 / 0.2);
		overflow: hidden;
	}
	.kanvas-latar {
		position: absolute;
		height: auto;
		max-width: none;
		user-select: none;
	}
	.kanvas-pegangan {
		position: absolute;
		cursor: move;
		outline: 1px dashed transparent;
		touch-action: none;
	}
	.kanvas-pegangan:hover {
		outline-color: rgb(59 130 246 / 0.6);
	}
	.kanvas-pegangan.is-aktif {
		outline: 1.5px solid rgb(59 130 246);
		background: rgb(59 130 246 / 0.06);
	}
	.kanvas-ubah {
		position: absolute;
		right: -2mm;
		bottom: -2mm;
		width: 3.5mm;
		height: 3.5mm;
		background: rgb(59 130 246);
		border: 1px solid #fff;
		border-radius: 1px;
		cursor: nwse-resize;
	}
</style>
