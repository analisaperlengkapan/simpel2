-- Referensi spesifikasi & subspesifikasi pakaian dinas — data awal dari SIMPel v1.
--
-- Kenapa migrasi, bukan diisi lewat aplikasi: sampai perubahan ini frontend
-- tidak punya satu pun pemanggil untuk `POST /pakaian-dinas/spesifikasi`, jadi
-- tabel ini KOSONG di mana pun. Dan tanpa minimal satu spesifikasi, pengajuan
-- pakaian dinas tidak dapat dibuat sama sekali (frontend menolak pilihan
-- kosong, dan kampanye tanpa pakaian tidak punya kolom ukuran). Jadi baris di
-- bawah bukan contoh — ia prasyarat agar alurnya bisa dijalankan.
--
-- Sumbernya bukan karangan: `monolith/simpelv1/database/dbsimpelv1.sql.gz`,
-- tabel `public.ms_spesifikasi_pakaian_dinas` (7 baris) dan
-- `public.ms_subspesifikasi_pakaian_dinas` (3 baris), dibuat "Superadmin"
-- 2023-07-31 s/d 2023-08-31 dan masih dipakai sampai pembaruan terakhir
-- 2024-06-24. Id v1 bertipe teks (`20230811003`) sedangkan v2 memakai uuid,
-- jadi pemetaannya lewat NAMA jenis + NAMA spesifikasi, bukan id.
--
-- Idempoten dengan sengaja. Tabel-tabel ini tidak punya unique constraint pada
-- (jenis, nama), sehingga `ON CONFLICT` tidak bisa dipakai; penjagaan dilakukan
-- dengan `WHERE NOT EXISTS`. Menjalankan ulang tidak menggandakan baris, dan
-- baris yang sudah diubah admin tidak ditimpa.

-- 1. Jenis yang dirujuk harus ada. PDH dan PDL sudah ada di v2; PDU-I 2023
--    hanya ada di v1 dan dikembalikan di sini. Jenis lain yang sudah terlanjur
--    ada di v2 (Batik, Toga, dsb.) sengaja TIDAK disentuh — menghapus data
--    yang mungkin dimasukkan orang bukan tugas migrasi seed.
INSERT INTO perlengkapan.ms_jenis_pakaian_dinas (nama, deskripsi)
SELECT v.nama, v.deskripsi
FROM (VALUES
    ('PDH',         'Pakaian Dinas Harian'),
    ('PDL',         'Pakaian Dinas Lapangan'),
    ('PDU-I 2023',  'Pakaian Dinas Upacara I (2023)')
) AS v(nama, deskripsi)
WHERE NOT EXISTS (
    SELECT 1 FROM perlengkapan.ms_jenis_pakaian_dinas j
    WHERE upper(btrim(j.nama)) = upper(v.nama)
);

-- 2. Spesifikasi. `ukuran_group` menentukan daftar ukuran yang ditawarkan saat
--    pengisian; `gender` = SEMUA berarti berlaku untuk seluruh pegawai.
INSERT INTO perlengkapan.ms_spesifikasi_pakaian_dinas
    (jenis_pakaian_dinas_id, nama, gender, ukuran_group, deskripsi, is_active)
SELECT j.id, v.nama, v.gender, v.ukuran_group, v.deskripsi, true
FROM (VALUES
    ('PDH',        'Pakaian Dinas',  'SEMUA', 'BAJU',   'Baju PDH'),
    ('PDH',        'Celana',         'SEMUA', 'CELANA', 'Celana PDH'),
    ('PDH',        'Sepatu Dinas',   'SEMUA', 'SEPATU', 'Sepatu PDH'),
    ('PDL',        'Baju',           'SEMUA', 'BAJU',   'Baju PDL'),
    ('PDL',        'Sepatu',         'SEMUA', 'SEPATU', 'Sepatu PDL'),
    ('PDU-I 2023', 'Kemeja PDU-I',   'SEMUA', 'BAJU',   'Kemeja PDU-I'),
    ('PDU-I 2023', 'Sepatu',         'SEMUA', 'SEPATU', 'Sepatu PDU-I')
) AS v(jenis_nama, nama, gender, ukuran_group, deskripsi)
JOIN perlengkapan.ms_jenis_pakaian_dinas j
  ON upper(btrim(j.nama)) = upper(v.jenis_nama)
WHERE NOT EXISTS (
    SELECT 1 FROM perlengkapan.ms_spesifikasi_pakaian_dinas s
    WHERE s.jenis_pakaian_dinas_id = j.id
      AND upper(btrim(s.nama)) = upper(v.nama)
);

-- 3. Subspesifikasi — varian ber-gender di bawah satu spesifikasi. v1 hanya
--    punya tiga, semuanya di bawah PDH; disalin apa adanya.
INSERT INTO perlengkapan.ms_subspesifikasi_pakaian_dinas
    (spesifikasi_id, nama, gender, is_active)
SELECT s.id, v.nama, v.gender, true
FROM (VALUES
    ('PDH', 'Pakaian Dinas', 'Lengan Pendek - L', 'L'),
    ('PDH', 'Pakaian Dinas', 'Lengan Pendek - P', 'P'),
    ('PDH', 'Celana',        'Celana Wanita',     'P')
) AS v(jenis_nama, spesifikasi_nama, nama, gender)
JOIN perlengkapan.ms_jenis_pakaian_dinas j
  ON upper(btrim(j.nama)) = upper(v.jenis_nama)
JOIN perlengkapan.ms_spesifikasi_pakaian_dinas s
  ON s.jenis_pakaian_dinas_id = j.id
 AND upper(btrim(s.nama)) = upper(v.spesifikasi_nama)
WHERE NOT EXISTS (
    SELECT 1 FROM perlengkapan.ms_subspesifikasi_pakaian_dinas sub
    WHERE sub.spesifikasi_id = s.id
      AND upper(btrim(sub.nama)) = upper(v.nama)
);
