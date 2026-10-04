# SIBOS

Aplikasi desktop pendamping ARKAS untuk menyusun SPJ BOSP: buku kas, buku pembantu, laporan, dan bukti cetak. SIBOS **hanya membaca** database ARKAS; data buatan SIBOS disimpan terpisah.

Dibangun dengan Tauri 2, Nuxt 4, Nuxt UI, dan Rust.

## Prasyarat (Windows)

| Alat | Catatan |
|---|---|
| [Bun](https://bun.sh) 1.3+ | Package manager & runner |
| [Rust](https://rustup.rs) stable (MSVC) | Butuh Visual Studio Build Tools (C++) |
| [Strawberry Perl](https://strawberryperl.com) | Untuk mengompilasi OpenSSL yang dipakai SQLCipher. Perl bawaan Git tidak cukup |
| WebView2 | Sudah ada di Windows 10/11 terbaru |

Arahkan build OpenSSL ke Strawberry Perl lewat file lokal `src-tauri/.cargo/config.toml` (tidak ikut repo):

```toml
[env]
OPENSSL_SRC_PERL = 'C:/Strawberry/perl/bin/perl.exe'
```

## Menjalankan

```bash
bun install
bun run tauri:dev      # mode pengembangan
bun run tauri:build    # installer NSIS di src-tauri/target/release/bundle/nsis
```

Build pertama memakan waktu lebih lama (±10 menit) karena OpenSSL dan SQLCipher dikompilasi dari sumber.

## Pemeriksaan

```bash
bun run lint
bun run typecheck
cd src-tauri && cargo test
```

`cargo test` memuat test keamanan: database ARKAS sintetis terenkripsi dibuka, semua query dijalankan, percobaan menulis harus ditolak, dan isi file harus tetap sama persis.

## Koneksi ARKAS

Saat pertama dibuka, SIBOS mencari `%APPDATA%\Arkas\arkas.db`. Database ARKAS terenkripsi; kuncinya diisi di halaman **Koneksi ARKAS** dan bisa disimpan di Windows Credential Manager. Untuk pengembangan, kunci juga bisa diberikan lewat variabel lingkungan `SIBOS_ARKAS_KEY`.

Data SIBOS sendiri disimpan di `%APPDATA%\id.sibos.app\sibos.db`.

## Struktur

```
app/            Nuxt (halaman, komponen, composable)
src-tauri/src/
  commands/     command yang dipanggil frontend
  repo/arkas/   akses ARKAS, hanya baca
  repo/app.rs   sibos.db + migrasi
```

## Lisensi

MIT, lihat [LICENSE](LICENSE). Kerangka awal proyek berasal dari template [Nuxtor](https://github.com/NicolaSpadari/nuxtor) karya Nicola Spadari (MIT, lihat [LICENSE.nuxtor](LICENSE.nuxtor)).
