<template>
	<LayoutPageShell :title="title">
		<template #actions>
			<slot name="actions" />
			<UButton
				color="neutral"
				variant="outline"
				icon="i-lucide-file-spreadsheet"
				:disabled="!book || loading"
				:loading="exporting"
				@click="exportExcel"
			>
				Export Excel
			</UButton>
			<UButton icon="i-lucide-printer" :disabled="!book || loading" @click="printPdf">
				Cetak PDF
			</UButton>
		</template>

		<div v-if="!connected" class="py-16 text-center text-muted">
			Hubungkan ARKAS terlebih dahulu.
			<UButton to="/setup" variant="link">
				Buka koneksi
			</UButton>
		</div>

		<div v-else class="flex min-h-0 flex-1 flex-col gap-4">
			<div class="flex flex-wrap items-center gap-2">
				<USelect v-model="month" :items="monthItems" icon="i-lucide-calendar-days" class="w-44" aria-label="Bulan" />
				<UInput v-model="search" icon="i-lucide-search" placeholder="Cari uraian, no bukti, kode..." class="w-80" />
				<USelect v-if="kind !== 'pajak'" v-model="metode" :items="metodeItems" class="w-48" aria-label="Metode belanja" />
				<span class="text-sm text-muted ml-auto">{{ filtered.length }} baris</span>
			</div>

			<UAlert v-if="error" color="error" variant="subtle" icon="i-lucide-triangle-alert" :title="error" />

			<div v-if="book" class="grid grid-cols-2 lg:grid-cols-4 gap-3">
				<UCard :ui="{ body: 'p-3 sm:p-3' }">
					<p class="text-xs text-muted">
						Saldo awal
					</p>
					<p class="text-lg font-semibold tabular">
						{{ rupiah(book.opening) }}
					</p>
				</UCard>
				<UCard :ui="{ body: 'p-3 sm:p-3' }">
					<p class="text-xs text-muted">
						{{ kind === 'pajak' ? 'Pungut' : 'Penerimaan' }}
					</p>
					<p class="text-lg font-semibold tabular text-success">
						{{ rupiah(book.totalPenerimaan) }}
					</p>
				</UCard>
				<UCard :ui="{ body: 'p-3 sm:p-3' }">
					<p class="text-xs text-muted">
						{{ kind === 'pajak' ? 'Setor' : 'Pengeluaran' }}
					</p>
					<p class="text-lg font-semibold tabular text-error">
						{{ rupiah(book.totalPengeluaran) }}
					</p>
				</UCard>
				<UCard :ui="{ body: 'p-3 sm:p-3' }">
					<p class="text-xs text-muted">
						{{ kind === 'pajak' ? 'Belum disetor' : 'Saldo akhir' }}
					</p>
					<p class="text-lg font-semibold tabular">
						{{ rupiah(book.closing) }}
					</p>
					<p v-if="kind === 'umum'" class="text-xs text-muted tabular">
						Bank {{ rupiah(book.closingBank) }} · Tunai {{ rupiah(book.closingTunai) }}
					</p>
				</UCard>
			</div>

			<template v-if="book && kind !== 'pajak'">
				<UAlert
					v-if="mismatches.length"
					color="error"
					variant="subtle"
					icon="i-lucide-circle-x"
					title="Saldo tidak cocok dengan ARKAS"
				>
					<template #description>
						<ul class="list-disc pl-4">
							<li v-for="c in mismatches" :key="`${c.month}-${c.fundId}`">
								{{ BULAN[c.month - 1] }} ({{ c.fundName }}): SIBOS bank {{ rupiah(c.computedBank) }}, tunai {{ rupiah(c.computedTunai) }} · ARKAS bank {{ rupiah(c.arkasBank) }}, tunai {{ rupiah(c.arkasTunai) }}
							</li>
						</ul>
					</template>
				</UAlert>
				<p v-else-if="book.checks.length" class="text-sm text-success flex items-center gap-1.5">
					<UIcon name="i-lucide-badge-check" class="size-4" />
					Saldo akhir {{ book.checks.length }} bulan cocok dengan ARKAS
				</p>
			</template>

			<UAlert
				v-for="(w, i) in book?.warnings ?? []"
				:key="i"
				color="warning"
				variant="subtle"
				icon="i-lucide-triangle-alert"
				:title="w"
			/>

			<UTable
				:data="filtered"
				:columns="columns"
				:loading="loading"
				sticky
				class="min-h-60 flex-1 border border-default rounded-md"
				:ui="{ td: 'py-1.5 text-sm align-top', th: 'py-2 text-xs' }"
			>
				<template #noBukti-cell="{ row }">
					<div class="flex items-center gap-1 whitespace-nowrap">
						<span>{{ row.original.noBukti }}</span>
						<UBadge v-if="row.original.siplah" color="info" variant="subtle" size="sm">
							SIPLah
						</UBadge>
					</div>
				</template>
				<template #uraian-cell="{ row }">
					<div class="group flex items-start gap-1 min-w-64 whitespace-normal">
						<span :class="row.original.isTax ? 'text-muted' : ''">
							{{ row.original.uraian }}
						</span>
						<UTooltip v-if="row.original.overridden" :text="`Asli: ${row.original.uraianAsli}`">
							<UBadge color="info" variant="subtle" size="sm">
								diubah
							</UBadge>
						</UTooltip>
						<UButton
							v-if="kind !== 'pajak'"
							icon="i-lucide-pencil"
							size="xs"
							color="neutral"
							variant="ghost"
							class="opacity-0 group-hover:opacity-100 ml-auto shrink-0"
							aria-label="Ubah uraian"
							@click="openEdit(row.original)"
						/>
					</div>
				</template>
				<template #empty>
					<p class="py-8 text-center text-muted">
						Tidak ada transaksi pada periode ini.
					</p>
				</template>
			</UTable>
		</div>

		<UModal v-model:open="editOpen" title="Ubah uraian tampilan" description="Hanya mengubah tampilan dan cetakan SIBOS. Data ARKAS tidak berubah.">
			<template #body>
				<div class="space-y-3">
					<p class="text-xs text-muted">
						Uraian di ARKAS: <span class="text-default">{{ editing?.uraianAsli }}</span>
					</p>
					<UTextarea v-model="editText" :rows="3" autofocus class="w-full" />
				</div>
			</template>
			<template #footer>
				<div class="flex w-full gap-2">
					<UButton v-if="editing?.overridden" color="neutral" variant="ghost" icon="i-lucide-undo-2" @click="saveEdit(true)">
						Kembalikan ke ARKAS
					</UButton>
					<UButton class="ml-auto" color="neutral" variant="outline" @click="editOpen = false">
						Batal
					</UButton>
					<UButton icon="i-lucide-check" :loading="saving" @click="saveEdit(false)">
						Simpan
					</UButton>
				</div>
			</template>
		</UModal>
	</LayoutPageShell>
</template>

<script lang="ts" setup>
	import type { TableColumn } from "@nuxt/ui";
	import BukuKas from "~/components/Print/Laporan/BukuKas.vue";

	const props = defineProps<{ kind: BookKind, title: string }>();

	const { connected, year, fund, funds } = useArkas();
	const toast = useToast();

	const book = ref<Book | null>(null);
	const loading = ref(false);
	const error = ref("");
	const month = ref<number>(0);
	const search = ref("");
	const metode = ref<"semua" | "siplah" | "langsung">("semua");
	const metodeItems = [
		{ label: "Semua belanja", value: "semua" },
		{ label: "Lewat SIPLah", value: "siplah" },
		{ label: "Transfer langsung / tunai", value: "langsung" }
	];

	const monthItems = [
		{ label: "Semua bulan", value: 0 },
		...BULAN.map((label, i) => ({ label, value: i + 1 }))
	];

	const mismatches = computed(() => book.value?.checks.filter((c) => !c.ok) ?? []);

	const filtered = computed(() => {
		const byMetode = (book.value?.lines ?? []).filter((l) => {
			if (metode.value === "siplah") return l.siplah;
			// "Transfer langsung": belanja (pengeluaran) yang bukan SIPLah.
			if (metode.value === "langsung") return !l.siplah && l.pengeluaran > 0 && !l.isTax;
			return true;
		});
		const q = search.value.trim().toLowerCase();
		if (!q) return byMetode;
		return byMetode.filter((l) =>
			[l.uraian, l.uraianAsli, l.noBukti, l.kodeRekening, l.kodeKegiatan, l.jenisPajak]
				.some((v) => v?.toLowerCase().includes(q))
		);
	});

	const money = (key: "penerimaan" | "pengeluaran" | "saldo"): TableColumn<BookLine> => ({
		accessorKey: key,
		header: key === "saldo"
			? "Saldo"
			: props.kind === "pajak"
				? (key === "penerimaan" ? "Pungut" : "Setor")
				: (key === "penerimaan" ? "Penerimaan" : "Pengeluaran"),
		cell: ({ row }) => {
			const v = row.original[key];
			return key !== "saldo" && v === 0 ? "" : angka(v);
		},
		meta: { class: { th: "text-right", td: "text-right tabular whitespace-nowrap" } }
	});

	const columns = computed<TableColumn<BookLine>[]>(() => {
		const tanggal: TableColumn<BookLine> = {
			accessorKey: "tanggal",
			header: "Tanggal",
			cell: ({ row }) => tanggalId(row.original.tanggal),
			meta: { class: { td: "whitespace-nowrap tabular" } }
		};
		const bukti: TableColumn<BookLine> = { accessorKey: "noBukti", header: "No. Bukti", meta: { class: { td: "whitespace-nowrap" } } };
		const uraian: TableColumn<BookLine> = { id: "uraian", accessorKey: "uraian", header: "Uraian" };
		if (props.kind === "pajak") {
			return [tanggal, bukti, uraian, { accessorKey: "jenisPajak", header: "Jenis Pajak" }, money("penerimaan"), money("pengeluaran"), money("saldo")];
		}
		return [
			tanggal,
			{ accessorKey: "kodeKegiatan", header: "Kode Kegiatan", meta: { class: { td: "whitespace-nowrap font-mono text-xs" } } },
			{ accessorKey: "kodeRekening", header: "Kode Rekening", meta: { class: { td: "whitespace-nowrap font-mono text-xs" } } },
			bukti,
			uraian,
			money("penerimaan"),
			money("pengeluaran"),
			money("saldo")
		];
	});

	const fundArg = computed(() => (fund.value === ALL_FUNDS ? null : fund.value));
	const monthArg = computed(() => (month.value === 0 ? null : month.value));

	const load = async () => {
		if (!connected.value || !year.value) {
			book.value = null;
			return;
		}
		loading.value = true;
		error.value = "";
		try {
			book.value = await api.book({ kind: props.kind, year: year.value, month: monthArg.value, fund: fundArg.value });
		} catch (err) {
			error.value = errorMessage(err);
			book.value = null;
		} finally {
			loading.value = false;
		}
	};

	// Saat tahun berganti, buka bulan terakhir yang ada transaksinya.
	watch(year, async (y) => {
		if (!y || !connected.value) return;
		const last = await api.lastActiveMonth(y).catch(() => null);
		const next = last ?? 0;
		if (next === month.value) await load();
		else month.value = next;
	}, { immediate: true });

	watch([month, fund, connected], load);

	defineExpose({ reload: load });

	// Ubah uraian
	const editOpen = ref(false);
	const editing = ref<BookLine | null>(null);
	const editText = ref("");
	const saving = ref(false);

	const openEdit = (line: BookLine) => {
		editing.value = line;
		editText.value = line.uraian;
		editOpen.value = true;
	};

	const saveEdit = async (reset: boolean) => {
		if (!editing.value || !year.value) return;
		saving.value = true;
		try {
			const text = reset || editText.value.trim() === editing.value.uraianAsli ? "" : editText.value;
			await api.uraianOverrideSet(year.value, editing.value.id, text);
			editOpen.value = false;
			await load();
		} catch (err) {
			toast.add({ title: "Gagal menyimpan uraian", description: errorMessage(err), color: "error" });
		} finally {
			saving.value = false;
		}
	};

	// Export Excel
	const exporting = ref(false);
	const fileLabel: Record<BookKind, string> = { umum: "BKU", bank: "Buku_Bank", tunai: "Buku_Tunai", pajak: "Buku_Pajak" };

	const { open: openPrint } = usePrint();
	const printPdf = () => {
		if (!book.value || !year.value) return;
		const fundLabel = funds.value.find((f) => f.id === fund.value)?.name ?? "Semua sumber dana";
		const period = monthArg.value ? BULAN[monthArg.value - 1] : "Setahun";
		openPrint({ title: `${props.title} ${period} ${year.value}`, landscape: true, component: BukuKas, props: { book: book.value, fundLabel } });
	};

	const exportExcel = async () => {
		if (!year.value) return;
		const fundName = funds.value.find((f) => f.id === fund.value)?.name;
		const period = monthArg.value ? `${String(monthArg.value).padStart(2, "0")}_${BULAN[monthArg.value - 1]}` : "Setahun";
		const defaultPath = `${fileSafe([fileLabel[props.kind], year.value, period, fundName].filter(Boolean).join("_"))}.xlsx`;
		try {
			const path = await useTauriDialogSave({ defaultPath, filters: [{ name: "Excel", extensions: ["xlsx"] }] });
			if (!path) return;
			exporting.value = true;
			const saved = await api.exportBookXlsx({ kind: props.kind, year: year.value, month: monthArg.value, fund: fundArg.value, path });
			toast.add({
				title: "File Excel tersimpan",
				description: saved,
				color: "success",
				actions: [{ label: "Buka folder", onClick: () => {
					useTauriOpenerRevealItemInDir(saved);
				} }]
			});
		} catch (err) {
			toast.add({ title: "Gagal export Excel", description: errorMessage(err), color: "error" });
		} finally {
			exporting.value = false;
		}
	};
</script>
