-- Grup ukuran menjadi DATA REFERENSI, dan profil ukuran pegawai disimpan per spesifikasi.
--
-- Kenapa: sampai V018 dua hal membuat jenis pakaian baru tak bisa ditambah sebagai data.
--   1. Grup ukuran hanya boleh BAJU/CELANA/SEPATU (dua CHECK: `ms_ukuran` dan
--      `ms_spesifikasi_pakaian_dinas`), jadi topi atau isian teks seperti nama dada
--      tak punya tempat, dan menambahkannya selalu berarti migrasi + rilis.
--   2. Profil pegawai (`pegawai_pakaian_dinas`) hanya punya SATU kolom per grup, dan
--      saran pengisian dicocokkan per grup. Dua kolom kampanye dalam grup yang sama
--      (sepatu PDH dan sepatu PDL, kaos dan baju) mendapat saran yang sama dan
--      saling menimpa.
--
-- Bentuk baru, semuanya ADITIF (expand): `helm rollback` ke rilis lama tetap aman,
-- kolom profil lama tidak disentuh.
--   * `ms_ukuran_group`: kode, nama, tipe (PILIHAN = daftar ukuran di `ms_ukuran`,
--     TEKS = isian bebas), urutan, aktif. Kedua CHECK diganti FK ke tabel ini.
--   * `pegawai_pakaian_dinas_ukuran`: satu baris per (nip, spesifikasi).
--
-- Idempoten dengan sengaja (dijalankan ulang tidak menggandakan apa pun dan tidak
-- menimpa yang sudah diubah admin), dan setiap pengubah data dijaga WHERE NOT EXISTS
-- atau ON CONFLICT.

-- 1. Grup ukuran.
CREATE TABLE IF NOT EXISTS perlengkapan.ms_ukuran_group (
    kode        character varying(50)  NOT NULL,
    nama        character varying(100) NOT NULL,
    tipe        character varying(10)  NOT NULL DEFAULT 'PILIHAN',
    urutan      integer                NOT NULL DEFAULT 0,
    is_active   boolean                NOT NULL DEFAULT true,
    created_at  timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at  timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT ms_ukuran_group_pkey PRIMARY KEY (kode),
    CONSTRAINT ms_ukuran_group_tipe_check CHECK (tipe IN ('PILIHAN', 'TEKS')),
    CONSTRAINT ms_ukuran_group_kode_check CHECK (kode = upper(btrim(kode)) AND kode <> '')
);

INSERT INTO perlengkapan.ms_ukuran_group (kode, nama, tipe, urutan) VALUES
    ('BAJU',      'Baju',                          'PILIHAN', 1),
    ('CELANA',    'Celana',                        'PILIHAN', 2),
    ('SEPATU',    'Sepatu',                        'PILIHAN', 3),
    ('TOPI',      'Topi (lingkar kepala, cm)',     'PILIHAN', 4),
    ('NAMA_DADA', 'Nama Dada',                     'TEKS',    5)
ON CONFLICT (kode) DO NOTHING;

-- 2. CHECK tiga-nama diganti FK ke tabel referensi. Baris yang ada seluruhnya sudah
--    BAJU/CELANA/SEPATU (itu yang dijamin CHECK lama), jadi FK pasti terpenuhi.
ALTER TABLE perlengkapan.ms_ukuran
    DROP CONSTRAINT IF EXISTS ms_ukuran_group_check;
ALTER TABLE perlengkapan.ms_spesifikasi_pakaian_dinas
    DROP CONSTRAINT IF EXISTS ms_spesifikasi_pakaian_dinas_ukuran_group_check;

DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'ms_ukuran_group_fk') THEN
        ALTER TABLE perlengkapan.ms_ukuran
            ADD CONSTRAINT ms_ukuran_group_fk FOREIGN KEY ("group")
            REFERENCES perlengkapan.ms_ukuran_group (kode) ON UPDATE CASCADE;
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'ms_spesifikasi_ukuran_group_fk') THEN
        ALTER TABLE perlengkapan.ms_spesifikasi_pakaian_dinas
            ADD CONSTRAINT ms_spesifikasi_ukuran_group_fk FOREIGN KEY (ukuran_group)
            REFERENCES perlengkapan.ms_ukuran_group (kode) ON UPDATE CASCADE;
    END IF;
END $$;

-- 3. Penamaan ukuran besar: 2XL, 3XL (bukan XXL, XXXL). Nilai yang sudah tersimpan
--    ikut diganti, di semua tempat yang menyimpannya: kolom kampanye lewat grup
--    yang DISALIN ke `pengajuan_pakaian_dinas_pakaian`, dan profil pegawai.
UPDATE perlengkapan.ms_ukuran u SET ukuran = n.baru
FROM (VALUES ('XXL', '2XL'), ('XXXL', '3XL')) AS n(lama, baru)
WHERE u."group" = 'BAJU' AND u.ukuran = n.lama
  AND NOT EXISTS (SELECT 1 FROM perlengkapan.ms_ukuran x WHERE x."group" = 'BAJU' AND x.ukuran = n.baru);

-- Dicocokkan lewat btrim: nilai lama bisa berspasi (' XXL '), dan isi-balik di bawah memangkasnya;
-- tanpa ini ukuran itu dipangkas menjadi XXL yang sudah tak ada di daftar, lalu tak pernah menjadi saran.
UPDATE perlengkapan.pengajuan_pakaian_dinas_satker_pegawai_ukuran su
SET ukuran = CASE btrim(su.ukuran) WHEN 'XXL' THEN '2XL' WHEN 'XXXL' THEN '3XL' END
FROM perlengkapan.pengajuan_pakaian_dinas_pakaian pk
WHERE pk.id = su.pakaian_id AND pk.spesifikasi_ukuran_group = 'BAJU'
  AND btrim(su.ukuran) IN ('XXL', 'XXXL');

UPDATE perlengkapan.pegawai_pakaian_dinas
SET ukuran_baju = CASE btrim(ukuran_baju) WHEN 'XXL' THEN '2XL' WHEN 'XXXL' THEN '3XL' END
WHERE btrim(ukuran_baju) IN ('XXL', 'XXXL');

-- Isian TEKS (nama dada) bisa sampai 100 karakter, sedangkan kolom ukuran kampanye hanya 20.
-- Melebarkan varchar adalah perubahan metadata saja dan kode lama (menulis paling banyak 20)
-- tak terpengaruh.
ALTER TABLE perlengkapan.pengajuan_pakaian_dinas_satker_pegawai_ukuran
    ALTER COLUMN ukuran TYPE character varying(100);

-- 4. Ukuran yang dipakai pegawai nyata tetapi belum ada di daftar. `urutan` untuk
--    ukuran bernomor = angkanya, supaya daftar tetap terurut bila ditambah lagi.
UPDATE perlengkapan.ms_ukuran SET urutan = ukuran::integer
WHERE "group" IN ('CELANA', 'SEPATU') AND ukuran ~ '^[0-9]+$';

INSERT INTO perlengkapan.ms_ukuran (ukuran, "group", urutan)
SELECT v.ukuran, v.grp, v.urutan
FROM (VALUES
    ('5XL', 'BAJU', 9), ('6XL', 'BAJU', 10), ('7XL', 'BAJU', 11), ('8XL', 'BAJU', 12)
) AS v(ukuran, grp, urutan)
WHERE NOT EXISTS (
    SELECT 1 FROM perlengkapan.ms_ukuran u WHERE u."group" = v.grp AND u.ukuran = v.ukuran
);

-- Bernomor: celana 26-52, sepatu 34-49, topi 50-67 cm (lingkar kepala). Yang sudah ada dilewati.
INSERT INTO perlengkapan.ms_ukuran (ukuran, "group", urutan)
SELECT n::text, v.grp, n
FROM (VALUES ('CELANA', 26, 52), ('SEPATU', 34, 49), ('TOPI', 50, 67)) AS v(grp, lo, hi),
     LATERAL generate_series(v.lo, v.hi) AS n
WHERE NOT EXISTS (
    SELECT 1 FROM perlengkapan.ms_ukuran u WHERE u."group" = v.grp AND u.ukuran = n::text
);

-- 5. Profil per spesifikasi: satu baris per pegawai per kolom pakaian, diisi dari tiga
--    kolom lama SEKALI SAJA, yaitu pada saat tabelnya dibuat.
--
--    Perilaku hari ini: saran dicocokkan per grup, jadi ukuran baju profil menjadi saran
--    untuk SETIAP kolom ber-grup BAJU. Mengisi untuk setiap spesifikasi yang ADA sekarang
--    mempertahankan perilaku itu persis; spesifikasi yang ditambahkan di langkah 7 (kaos,
--    topi, ...) tidak mendapat tebakan dari kolom yang bukan miliknya.
--
--    Isi-balik dijaga oleh "tabelnya baru dibuat di sini", bukan oleh ON CONFLICT saja:
--    dijalankan ulang SETELAH langkah 7, `ON CONFLICT` tidak akan melindungi spesifikasi
--    baru itu dari tebakan (terukur: penerapan kedua mengisi Kaos dan Celana PDL).
DO $$
DECLARE
    tabel_baru boolean := to_regclass('perlengkapan.pegawai_pakaian_dinas_ukuran') IS NULL;
BEGIN
    CREATE TABLE IF NOT EXISTS perlengkapan.pegawai_pakaian_dinas_ukuran (
        nip            character varying(30)  NOT NULL,
        spesifikasi_id uuid                   NOT NULL,
        ukuran         character varying(100) NOT NULL,
        sumber         character varying(30)  NOT NULL DEFAULT 'APLIKASI',
        updated_at     timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
        CONSTRAINT pegawai_pakaian_dinas_ukuran_pkey PRIMARY KEY (nip, spesifikasi_id),
        CONSTRAINT pegawai_pakaian_dinas_ukuran_nip_fk FOREIGN KEY (nip)
            REFERENCES perlengkapan.pegawai_pakaian_dinas (nip) ON DELETE CASCADE ON UPDATE CASCADE,
        CONSTRAINT pegawai_pakaian_dinas_ukuran_spesifikasi_fk FOREIGN KEY (spesifikasi_id)
            REFERENCES perlengkapan.ms_spesifikasi_pakaian_dinas (id) ON DELETE CASCADE,
        CONSTRAINT pegawai_pakaian_dinas_ukuran_ukuran_check CHECK (btrim(ukuran) <> '')
    );
    CREATE INDEX IF NOT EXISTS idx_pegawai_pakaian_dinas_ukuran_spesifikasi
        ON perlengkapan.pegawai_pakaian_dinas_ukuran (spesifikasi_id);

    IF tabel_baru THEN
        INSERT INTO perlengkapan.pegawai_pakaian_dinas_ukuran (nip, spesifikasi_id, ukuran, sumber)
        SELECT p.nip, s.id,
               btrim(CASE s.ukuran_group
                    WHEN 'BAJU'   THEN p.ukuran_baju
                    WHEN 'CELANA' THEN p.ukuran_celana
                    WHEN 'SEPATU' THEN p.ukuran_sepatu
               END),
               'MIGRASI'
        FROM perlengkapan.pegawai_pakaian_dinas p
        CROSS JOIN perlengkapan.ms_spesifikasi_pakaian_dinas s
        WHERE s.ukuran_group IN ('BAJU', 'CELANA', 'SEPATU')
          AND btrim(COALESCE(CASE s.ukuran_group
                    WHEN 'BAJU'   THEN p.ukuran_baju
                    WHEN 'CELANA' THEN p.ukuran_celana
                    WHEN 'SEPATU' THEN p.ukuran_sepatu
               END, '')) <> ''
        ON CONFLICT (nip, spesifikasi_id) DO NOTHING;
    END IF;
END $$;

-- 5b. Jembatan untuk penulis LAMA. Selama rolling upgrade, atau sesudah rollback lalu upgrade
--     ulang, pod versi lama masih menulis HANYA tiga kolom profil, sedangkan versi baru membaca
--     saran dari tabel per spesifikasi: perubahan yang berhasil disimpan versi lama tak akan
--     pernah menjadi saran, dan isi-balik di atas hanya berjalan satu kali. Trigger ini
--     menerjemahkan penulisan tiga kolom itu ke tabel per spesifikasi di transaksi yang sama,
--     untuk SETIAP spesifikasi dalam grupnya (makna kolom itu sejak awal), dan hanya bila kolom
--     itu benar-benar berubah supaya isian per spesifikasi yang ditulis kode baru tidak tertimpa.
--     Penulis baru menulis tabel per spesifikasi langsung dan tidak menyentuh tiga kolom ini.
CREATE OR REPLACE FUNCTION perlengkapan.sinkron_ukuran_profil_lama() RETURNS trigger
LANGUAGE plpgsql AS $fn$
DECLARE
    baju_berubah   boolean;
    celana_berubah boolean;
    sepatu_berubah boolean;
BEGIN
    IF TG_OP = 'UPDATE' THEN
        baju_berubah   := NEW.ukuran_baju   IS DISTINCT FROM OLD.ukuran_baju;
        celana_berubah := NEW.ukuran_celana IS DISTINCT FROM OLD.ukuran_celana;
        sepatu_berubah := NEW.ukuran_sepatu IS DISTINCT FROM OLD.ukuran_sepatu;
    ELSE
        baju_berubah := true; celana_berubah := true; sepatu_berubah := true;
    END IF;

    INSERT INTO perlengkapan.pegawai_pakaian_dinas_ukuran (nip, spesifikasi_id, ukuran, sumber)
    -- Pod lama (sesudah rollback) masih menulis XXL/XXXL; daftar baru menamainya 2XL/3XL.
    SELECT NEW.nip, s.id,
           CASE WHEN v.grup = 'BAJU' THEN
                    CASE upper(btrim(v.ukuran)) WHEN 'XXL' THEN '2XL' WHEN 'XXXL' THEN '3XL' ELSE btrim(v.ukuran) END
                ELSE btrim(v.ukuran) END,
           'PROFIL'
    FROM perlengkapan.ms_spesifikasi_pakaian_dinas s
    JOIN (VALUES ('BAJU',   NEW.ukuran_baju,   baju_berubah),
                 ('CELANA', NEW.ukuran_celana, celana_berubah),
                 ('SEPATU', NEW.ukuran_sepatu, sepatu_berubah)) AS v(grup, ukuran, berubah)
      ON v.grup = s.ukuran_group
    WHERE v.berubah AND btrim(COALESCE(v.ukuran, '')) <> ''
    ON CONFLICT (nip, spesifikasi_id) DO UPDATE
       SET ukuran = EXCLUDED.ukuran, sumber = EXCLUDED.sumber, updated_at = now();
    RETURN NULL;
END
$fn$;

DROP TRIGGER IF EXISTS trg_sinkron_ukuran_profil_lama ON perlengkapan.pegawai_pakaian_dinas;
CREATE TRIGGER trg_sinkron_ukuran_profil_lama
    AFTER INSERT OR UPDATE OF ukuran_baju, ukuran_celana, ukuran_sepatu
    ON perlengkapan.pegawai_pakaian_dinas
    FOR EACH ROW EXECUTE FUNCTION perlengkapan.sinkron_ukuran_profil_lama();

-- 6. Spesifikasi pakaian lapangan yang selama ini tak punya tempat. PDL sudah ada
--    (V011); hanya yang belum ada yang ditambahkan, dicocokkan lewat NAMA seperti V011.
INSERT INTO perlengkapan.ms_spesifikasi_pakaian_dinas
    (jenis_pakaian_dinas_id, nama, gender, ukuran_group, deskripsi, is_active)
SELECT j.id, v.nama, 'SEMUA', v.ukuran_group, v.deskripsi, true
FROM (VALUES
    ('Kaos',      'BAJU',      'Kaos PDL'),
    ('Celana',    'CELANA',    'Celana PDL'),
    ('Topi',      'TOPI',      'Topi PDL'),
    ('Nama Dada', 'NAMA_DADA', 'Nama yang disulam pada dada PDL')
) AS v(nama, ukuran_group, deskripsi)
JOIN perlengkapan.ms_jenis_pakaian_dinas j ON upper(btrim(j.nama)) = 'PDL'
WHERE NOT EXISTS (
    SELECT 1 FROM perlengkapan.ms_spesifikasi_pakaian_dinas s
    WHERE s.jenis_pakaian_dinas_id = j.id AND upper(btrim(s.nama)) = upper(v.nama)
);
