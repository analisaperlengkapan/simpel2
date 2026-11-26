-- ============================================================================
-- Migration V020: pegawai yang dikeluarkan dari pengajuan pakaian dinas
-- ============================================================================
-- Kebutuhan: "Keluarkan dari Pengajuan" menghapus baris partisipasi pegawai
-- beserta ukurannya, tetapi roster beku kampanye (V018) tetap memuat orang itu.
-- Hitungan kesiapan membaca roster, bukan tabel partisipasi, sehingga pegawai
-- yang sudah dikeluarkan tetap masuk penyebut selamanya: modal operator
-- memperingatkan orang yang tidak memerlukan pakaian dinas, dan validator
-- menghitung satker itu belum lengkap.
--
-- Rancangan:
--   * Tabel pengecualian eksplisit per (kampanye, satker, NIP). Snapshot historis
--     TIDAK diubah — yang dikecualikan hanya hitungan "siapa yang masih wajib
--     mengisi", bukan catatan siapa yang pernah ada pada kampanye itu.
--   * Operator yang menyimpan ukuran lagi untuk NIP tersebut menghapus
--     pengecualiannya (pegawai kembali masuk pengajuan).
--
-- Additif: tabel baru saja, sehingga rollback ke binary lama aman — binary lama
-- mengabaikannya.

CREATE TABLE IF NOT EXISTS perlengkapan.pengajuan_pakaian_dinas_pegawai_dikeluarkan (
    pengajuan_id uuid NOT NULL
        REFERENCES perlengkapan.pengajuan_pakaian_dinas(id) ON DELETE CASCADE,
    -- MySIMKARI kode_satker, sama dengan `_satker.satker_id` dan `_pegawai_snapshot.satker_id`.
    satker_id character varying(20) NOT NULL,
    nip character varying(30) NOT NULL,
    dikeluarkan_at timestamp with time zone NOT NULL DEFAULT NOW(),
    PRIMARY KEY (pengajuan_id, satker_id, nip)
);

COMMENT ON TABLE perlengkapan.pengajuan_pakaian_dinas_pegawai_dikeluarkan IS
    'Pegawai yang sengaja dikeluarkan operator dari pengajuan ini (tidak memerlukan pakaian dinas). Tidak mengubah snapshot; hanya dikecualikan dari hitungan kesiapan.';
