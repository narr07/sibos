# Situs Lajur

Situs dokumentasi dan unduhan Lajur di **https://lajur.web.id**, dibuat dengan [Docus](https://docus.dev).

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

Lalu tambahkan domain `lajur.web.id` (dan `www.lajur.web.id`) di layanan hosting. Karena `lajur.web.id` adalah domain utama (apex), record DNS-nya mengikuti petunjuk layanan hosting: A/ALIAS ke IP yang diberikan, atau `CNAME` dengan *CNAME flattening* bila DNS-nya di Cloudflare. Arahkan `www` ke domain utama.

Domain lama `sibos.permadi.dev` sebaiknya dialihkan (redirect 301) ke `https://lajur.web.id` agar tautan lama tetap jalan.
