/** Pilih lokasi simpan lewat dialog, jalankan export, lalu tampilkan notifikasi "Buka folder". */
export const useSaveFile = () => {
	const toast = useToast();
	const busy = ref(false);

	const save = async (opts: {
		defaultName: string
		filters: { name: string, extensions: string[] }[]
		run: (path: string) => Promise<string>
		success?: string
	}) => {
		try {
			const path = await useTauriDialogSave({ defaultPath: fileSafe(opts.defaultName), filters: opts.filters });
			if (!path) return null;
			busy.value = true;
			const saved = await opts.run(path);
			toast.add({
				title: opts.success ?? "File tersimpan",
				description: saved,
				color: "success",
				actions: [{ label: "Buka folder", onClick: () => {
					useTauriOpenerRevealItemInDir(saved);
				} }]
			});
			return saved;
		} catch (err) {
			toast.add({ title: "Gagal menyimpan file", description: errorMessage(err), color: "error" });
			return null;
		} finally {
			busy.value = false;
		}
	};

	const xlsx = (defaultName: string, run: (path: string) => Promise<string>) =>
		save({ defaultName: defaultName.endsWith(".xlsx") ? defaultName : `${defaultName}.xlsx`, filters: [{ name: "Excel", extensions: ["xlsx"] }], run, success: "File Excel tersimpan" });

	return { busy, save, xlsx };
};
