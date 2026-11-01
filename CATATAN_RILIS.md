### Lajur 0.4.0 - Nama baru & banding harga draft RKAS

SIBOS berganti nama menjadi Lajur. Draft RKAS kini bisa dibandingkan dengan harga toko SIPLah, dan aplikasi lebih responsif serta aman saat backup dan restore.

#### Fitur Baru:
- Bandingkan harga draft RKAS dengan toko SIPLah (Toko Ladang): perkiraan biaya, selisih per item, ganti produk pasangan, dan tombol Pakai harga untuk memperbarui harga satuan draft.
- Sinkronisasi katalog SIPLah menampilkan progres per halaman dan bisa dibatalkan; katalog tidak berubah bila dibatalkan.

#### Perubahan:
- SIBOS kini bernama Lajur, dengan logo baru. Data, pengaturan, dan koneksi ARKAS tetap sama; tidak perlu mengatur ulang apa pun.
- Halaman Penganggaran RKAS lebih lega: daftar draft pindah ke pemilih di atas sehingga tabel memakai lebar penuh.
- Tampilan bersudut siku di seluruh aplikasi.

#### Perbaikan:
- Aplikasi tidak lagi membeku saat memuat buku kas, export Excel, atau backup; semua proses berat berjalan di latar belakang.
- Salin database ARKAS kini menyertakan transaksi terakhir yang belum tersimpan permanen oleh ARKAS (file -wal).
- Restore backup aman bila gagal di tengah jalan: data lama dikembalikan otomatis.
- Angka Hemat di SIPLah dan Belanja hanya membandingkan barang yang ditemukan di toko, tidak lagi terlalu besar.
- Total terpilih di Generator SI sama dengan nominal yang dimasukkan ke rincian.
- Pesan gagal SIPLah lebih jelas: toko tidak ditemukan, tidak ada internet, atau server tidak menjawab.
- Muat ulang koneksi ARKAS tidak lagi menghapus salinan sementara yang sedang dipakai.

Pengguna SIBOS versi lama akan menerima pembaruan ini seperti biasa. Nama installer dan shortcut masih SIBOS untuk sementara agar pembaruan berjalan mulus.
