/** Baca file gambar lalu perkecil (sisi terpanjang maks. `maxSide` px) menjadi data URL PNG. */
export const fileToDataUrl = (file: File, maxSide = 400): Promise<string> => new Promise((resolve, reject) => {
	const reader = new FileReader();
	reader.onerror = () => reject(new Error("Gagal membaca file"));
	reader.onload = () => {
		const img = new Image();
		img.onerror = () => reject(new Error("File bukan gambar yang valid"));
		img.onload = () => {
			const scale = Math.min(1, maxSide / Math.max(img.width, img.height));
			const canvas = document.createElement("canvas");
			canvas.width = Math.round(img.width * scale);
			canvas.height = Math.round(img.height * scale);
			const ctx = canvas.getContext("2d");
			if (!ctx) return reject(new Error("Canvas tidak tersedia"));
			ctx.drawImage(img, 0, 0, canvas.width, canvas.height);
			resolve(canvas.toDataURL("image/png"));
		};
		img.src = String(reader.result);
	};
	reader.readAsDataURL(file);
});

/**
 * Logo kop: gambar ditempatkan di tengah kanvas persegi transparan tanpa ditarik,
 * sehingga logo kiri & kanan berukuran sama, simetris, dan tidak gepeng.
 */
export const fileToSquareLogo = (file: File, size = 400): Promise<string> => new Promise((resolve, reject) => {
	const reader = new FileReader();
	reader.onerror = () => reject(new Error("Gagal membaca file"));
	reader.onload = () => {
		const img = new Image();
		img.onerror = () => reject(new Error("File bukan gambar yang valid"));
		img.onload = () => {
			const canvas = document.createElement("canvas");
			canvas.width = size;
			canvas.height = size;
			const ctx = canvas.getContext("2d");
			if (!ctx) return reject(new Error("Canvas tidak tersedia"));
			const scale = Math.min(size / img.width, size / img.height);
			const w = img.width * scale;
			const h = img.height * scale;
			ctx.drawImage(img, (size - w) / 2, (size - h) / 2, w, h);
			resolve(canvas.toDataURL("image/png"));
		};
		img.src = String(reader.result);
	};
	reader.readAsDataURL(file);
});
