# Runbook: data awal ukuran pakaian dinas

Berkas satker (Kejagung: sepatu PDH; Kejati: kaos, baju, celana, topi, sepatu PDL, sepatu PDH, nama dada)
dimuat menjadi **profil ukuran per spesifikasi**. Berkasnya ditulis tangan dan tidak bersih, jadi
pemuatan dibagi dua: alat penyiapan memilah **di luar aplikasi**, endpoint admin memuat dan **memvalidasi
ulang**.

> **Data nyata tidak boleh masuk git, staging, atau rehearsal.** Staging berisi data tiruan; berkas ini
> berisi NIP dan nama pegawai sungguhan. Impor hanya dijalankan terhadap **produksi**, setelah produksi
> terpasang. Keluaran alat penyiapan berisi NIP dan nama: simpan di luar repo, jangan ditempel ke tiket
> atau obrolan, hapus dengan `shred -u` setelah selesai.

## 1. Siapkan (di komputer yang aman, tanpa jaringan ke produksi)

```bash
python3 infra/scripts/siapkan-impor-pakaian-dinas.py \
  --kejagung /aman/00_2026_pdh_kejagung.xlsx \
  --kejati   /aman/00_2026_pdhpdl_kejati.xlsx \
  --keluaran /aman/impor-2026        # DITOLAK bila di dalam pohon repo
```

Mencetak **jumlah** saja (tanpa NIP/nama) dan menulis, dengan izin 0600:

| Berkas | Isi |
|---|---|
| `lot-0001.json` ... | baris yang **jelas**, 1000 per lot (permintaan siap kirim) |
| `karantina.csv` | baris yang **diragukan**, dengan alasan (dan usulan NIP bila ada rujukan) |
| `laporan.json` | hitungan per alasan |

Baris masuk karantina (tidak dimuat) bila: NIP rusak (notasi ilmiah `1.99E+17`, terlalu pendek/panjang,
struktur tak sah), NIP kosong, **digit jenis kelamin NIP beda dengan kolom gender**, NIP dipakai **dua
orang berbeda**, atau NIP ganda dengan **data berbeda**. NIP ganda yang identik atau saling melengkapi
digabung. "NIP + nomor lama" dipotong menjadi 18 digit.

**Asumsi yang perlu dikonfirmasi pemilik data** (tabel `PETA` di alat): kolom `baju` dan `celana`
berkas Kejati diterapkan ke **PDH dan PDL** sekaligus (hanya sepatu yang dipisah di berkasnya).

## 2. Periksa tanpa menulis

```bash
export SIMPEL_TOKEN=...   # token admin; JANGAN lewat argumen perintah
python3 infra/scripts/siapkan-impor-pakaian-dinas.py ... --kirim https://simpel.kejaksaan.go.id/api/v1/perlengkapan
```

Tanpa `--muat` ini hanya **pemeriksaan** (`dry_run`): server memvalidasi dengan aturan yang sama dengan
setiap jalur penyimpan ukuran dan menghitung apa yang akan terjadi. Baca angkanya (`baris_dimuat`,
`profil_baru`, `ukuran_ditulis`, dan alasan penolakan server) sebelum lanjut. `ditolak-server.csv` memuat
rujukan baris, NIP yang **disamarkan**, dan alasannya.

## 3. Muat

Ulangi perintah di atas dengan `--muat`. Aturan server:

- profil yang **sudah ada** tidak diubah (nama dan hijab milik orangnya atau sinkronisasi);
- ukuran yang **diisi orangnya sendiri** lewat aplikasi tidak ditimpa (dihitung `ukuran_dilewati_sudah_diisi`);
- hanya ukuran hasil impor sebelumnya (`sumber = IMPOR`) yang diganti, jadi mengulang impor **aman**;
- baris yang gagal validasi ditolak sendiri-sendiri; sisanya satu transaksi per lot;
- jejak audit mencatat **jumlah** (tak ada NIP atau nama pada log maupun audit).

## 4. Baris yang tertahan (karantina)

Diperiksa ke MySIMKARI setelah token tersedia. Ekspor `nip,nama` dari MySIMKARI, lalu jalankan ulang
alat penyiapan dengan `--rujukan-pegawai mysimkari.csv`: NIP yang tak ada di rujukan, atau namanya beda,
tertahan; NIP rusak mendapat **usulan** bila ada satu kandidat dengan nama yang sama. Usulan tidak
dimuat otomatis kecuali `--muat-pemulihan` (hanya yang unik dan sama namanya). Setelah itu `--kirim`
lagi: impor yang diulang tidak mengganggu yang sudah termuat.

## 5. Selesai

`shred -u` untuk `lot-*.json`, `karantina.csv`, `ditolak-server.csv`, dan berkas xlsx salinan kerja.
