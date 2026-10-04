<template>
	<div class="kop-surat mb-4">
		<div class="grid items-center gap-2" style="grid-template-columns: 20mm 1fr 20mm">
			<div class="kop-logo">
				<img v-if="k?.logoKiri" :src="k.logoKiri" alt="">
			</div>
			<div class="text-center leading-[1.15]" style="font-family: 'Times New Roman', Times, serif">
				<p v-if="k?.pemerintah" class="text-[12pt] font-bold uppercase">
					{{ k.pemerintah }}
				</p>
				<p v-if="k?.dinas" class="text-[12pt] font-bold uppercase">
					{{ k.dinas }}
				</p>
				<p class="text-[14pt] font-bold uppercase">
					{{ k?.namaSekolah || sekolah?.nama }}
				</p>
				<p v-if="barisAlamat" class="text-[9pt] italic">
					{{ barisAlamat }}
				</p>
				<p v-if="barisKontak" class="text-[9pt] italic">
					{{ barisKontak }}
				</p>
			</div>
			<div class="kop-logo">
				<img v-if="k?.logoKanan" :src="k.logoKanan" alt="">
			</div>
		</div>
		<!-- Garis ganda: tebal di atas, tipis di bawah -->
		<div class="mt-1.5 border-t-[3px] border-black" />
		<div class="mt-[1.5px] border-t border-black" />
	</div>
</template>

<script lang="ts" setup>
	/** Kop surat sekolah. `kop` dipakai untuk pratinjau di Pengaturan; bila kosong memakai pengaturan tersimpan. */
	const props = defineProps<{ kop?: Pengaturan["kop"] | null }>();
	const { pengaturan, sekolah } = useDocContext();

	const k = computed(() => props.kop ?? pengaturan.value?.kop ?? null);

	const barisAlamat = computed(() => {
		if (k.value?.barisAlamat) return k.value.barisAlamat;
		const parts = [k.value?.alamat || sekolah.value?.alamat, sekolah.value?.kecamatan, sekolah.value?.kabupaten].filter(Boolean);
		return parts.length ? `Alamat ${parts.join(", ")}` : "";
	});

	const barisKontak = computed(() => {
		if (k.value?.barisKontak) return k.value.barisKontak;
		return [
			sekolah.value?.npsn && `NPSN. ${sekolah.value.npsn}`,
			k.value?.email && `E-mail: ${k.value.email}`,
			k.value?.laman && `website: ${k.value.laman}`
		].filter(Boolean).join(" ");
	});
</script>

<style scoped>
	/* Bingkai persegi 2 x 2 cm yang sama untuk kedua logo agar simetris dan tidak gepeng */
	.kop-logo {
		width: 20mm;
		height: 20mm;
		display: flex;
		align-items: center;
		justify-content: center;
	}
	.kop-logo img {
		max-width: 100%;
		max-height: 100%;
		object-fit: contain;
	}
</style>
