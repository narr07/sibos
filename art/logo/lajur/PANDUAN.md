# Logo Lajur: panduan pemakaian

## Ide

Huruf **L** berdiri di atas garis kedua. Kaki L dan garis itu membentuk **garis ganda**, tanda dalam pembukuan bahwa sebuah buku sudah ditutup dan saldonya seimbang. Wordmark mengulang motif yang sama: ekor huruf **j** memanjang ke kiri menjadi garis jumlah di bawah "la".

## File master (`final/`)

| File | Pakai untuk |
|---|---|
| `lajur-symbol.svg` | Simbol, ukuran 48 px ke atas |
| `lajur-symbol-small.svg` | Simbol ukuran kecil (≤ 32 px): batang lebih tebal, celah lebih lebar |
| `lajur-wordmark.svg` | Tulisan "lajur" saja |
| `lajur-horizontal.svg` | Lockup utama: simbol + tulisan sejajar |
| `lajur-stacked.svg` | Lockup bertumpuk untuk ruang sempit-tinggi |

Varian siap pakai (hitam, putih, satu warna, ikon aplikasi, favicon, ikon web) ada di `export/`.

## Warna

| Nama | HEX | RGB | Catatan |
|---|---|---|---|
| Emerald Lajur | `#047857` | 4, 120, 87 | Warna utama. Kontras dengan putih 5,5 : 1 (lolos WCAG AA) |
| Putih | `#FFFFFF` | 255, 255, 255 | Logo di atas latar emerald atau gelap |
| Hitam | `#000000` | 0, 0, 0 | Cetak satu warna, stempel, fotokopi |

Logo selalu satu warna. Tidak memakai gradasi, bayangan, atau efek.

## Bentuk

- **Semua sudut siku.** Tidak ada sudut membulat, baik di simbol, tile ikon aplikasi, maupun UI aplikasi (radius 0).
- Ujung garis wordmark persegi (`stroke-linecap="square"`, `stroke-linejoin="miter"`).
- Wordmark disimpan sebagai garis (stroke) tebal 24 unit. Untuk keperluan potong vinil atau bordir, ubah garis menjadi bentuk isi (outline) di Inkscape/Illustrator.

## Ruang kosong dan ukuran minimum

- Ruang kosong di sekeliling logo minimal setinggi **satu batang garis** (tebal kaki L).
- Ukuran minimum simbol: 16 px layar / 5 mm cetak (pakai versi kecil).
- Ukuran minimum lockup horizontal: lebar 96 px layar / 25 mm cetak.

## Latar yang diizinkan

- Emerald di atas putih atau abu sangat terang.
- Putih di atas emerald `#047857` atau latar gelap.
- Hitam untuk cetak satu warna.

## Jangan

- Membuang garis kedua di bawah L (tanpa garis itu, logonya hanya huruf L biasa).
- Membulatkan sudut, memiringkan, meregangkan, atau memberi kontur.
- Mengganti tulisan "lajur" dengan font lain.
- Menaruh logo emerald di atas foto atau latar yang ramai.

## Catatan

Merek belum dicek di PDKI (DJKI). Periksa kelas 9 dan 42 sebelum dipakai resmi.
