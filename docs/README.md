# Situs SIBOS

Situs dokumentasi dan unduhan SIBOS di **https://sibos.permadi.dev**, dibuat dengan [Docus](https://docus.dev).

## Menjalankan

```bash
cd docs
bun install
bun run dev        # http://localhost:3000
bun run generate   # situs statis di .output/public
```

## Isi

- `content/` — halaman dokumentasi (Markdown). Angka di depan nama file/folder menentukan urutan menu.
- `app/components/content/DownloadButton.vue` — tombol unduh; mengambil installer `-setup.exe` dari rilis terbaru di GitHub Releases, atau ke halaman rilis bila API GitHub tidak bisa diakses.
- `app/app.config.ts` — judul, logo, warna, dan tautan GitHub.

## Deploy

Situs statis, bisa di-host di Cloudflare Pages, Netlify, atau Vercel:

| Pengaturan | Nilai |
| --- | --- |
| Root directory | `docs` |
| Build command | `bun run generate` |
| Output directory | `.output/public` |

Lalu tambahkan domain `sibos.permadi.dev` di layanan hosting dan buat record DNS `CNAME sibos` ke alamat yang diberikan layanan tersebut.
