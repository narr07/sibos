<template>
	<div class="mt-8 grid text-center text-[10pt] break-inside-avoid" :style="{ gridTemplateColumns: `repeat(${Math.max(2, list.length)}, minmax(0, 1fr))` }">
		<div v-for="(s, i) in list" :key="i" class="flex flex-col items-center px-2" :class="list.length === 1 ? 'col-start-2' : ''">
			<p class="min-h-[1.3em]">
				{{ s.atas ?? "" }}
			</p>
			<p>{{ s.jabatan }}</p>
			<div :style="{ height: space }" />
			<p class="font-bold underline">
				{{ s.nama || "(..............................)" }}
			</p>
			<p v-if="s.nip !== undefined">
				NIP. {{ s.nip || "-" }}
			</p>
		</div>
	</div>
</template>

<script lang="ts" setup>
	const props = withDefaults(defineProps<{
		signers?: Penandatangan[]
		tanggal?: string
		/** Tinggi ruang tanda tangan; untuk nota/kwitansi dibuat cukup untuk meterai. */
		space?: string
	}>(), { space: "20mm" });
	const { pengaturan } = useDocContext();

	const list = computed<Penandatangan[]>(() => {
		if (props.signers?.length) return props.signers;
		const p = pengaturan.value?.pejabat;
		const kota = pengaturan.value?.cetak.kota;
		const tempatTanggal = [kota, props.tanggal].filter(Boolean).join(", ");
		return [
			{ atas: "Mengetahui,", jabatan: "Kepala Sekolah", nama: p?.kepalaSekolah, nip: p?.nipKepalaSekolah },
			{ atas: tempatTanggal, jabatan: "Bendahara", nama: p?.bendahara, nip: p?.nipBendahara }
		];
	});
</script>
