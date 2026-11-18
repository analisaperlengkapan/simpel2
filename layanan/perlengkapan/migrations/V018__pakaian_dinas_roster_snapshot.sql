-- ============================================================================
-- Migration V018: snapshot daftar pegawai per kampanye pakaian dinas
-- ============================================================================
-- Kebutuhan: daftar pegawai pakaian dinas harus DIKUNCI per kampanye pada saat
-- validator pusat membuat pengajuan, lalu ditampilkan ke semua role beserta
-- tanggal pembekuannya ("data pegawai per tanggal kapan").
--
-- Sebelum ini roster dibaca LANGSUNG dari `integrasi.mysimkari_pegawai` setiap
-- halaman dibuka (`repository/roster.rs::get_roster_pengisian`). Konsekuensinya
-- keanggotaan (siapa saja yang harus mengisi) TIDAK PERNAH dibekukan: pegawai
-- yang mutasi masuk/keluar setelah kampanye dibuat ikut berubah pada kampanye
-- lama, sehingga rekap yang sudah disetujui bisa menunjuk orang yang berbeda
-- dari yang diisi operator. Identitas memang sudah dibekukan saat operator
-- MENYIMPAN ukuran, tetapi daftar orangnya sendiri tidak.
--
-- Rancangan:
--   * `pengajuan_pakaian_dinas.roster_snapshot_at` — kapan daftar dibekukan.
--     NULL untuk kampanye yang dibuat sebelum migrasi ini; pembacaan jatuh
--     kembali ke sumber live supaya kampanye lama tetap berfungsi.
--   * `pengajuan_pakaian_dinas_pegawai_snapshot` — baris pegawai yang dibekukan
--     per (kampanye, satker), ON DELETE CASCADE dari kampanye seperti tabel
--     anak lainnya.
--
-- Additif (expand): kolom + tabel baru, tanpa mengubah tabel yang ada, sehingga
-- rollback ke binary lama aman — binary lama mengabaikan keduanya.

ALTER TABLE perlengkapan.pengajuan_pakaian_dinas
    ADD COLUMN IF NOT EXISTS roster_snapshot_at timestamp with time zone;

COMMENT ON COLUMN perlengkapan.pengajuan_pakaian_dinas.roster_snapshot_at IS
    'Kapan daftar pegawai kampanye ini dibekukan (saat dibuat validator pusat). NULL = kampanye lama / belum dibekukan.';

CREATE TABLE IF NOT EXISTS perlengkapan.pengajuan_pakaian_dinas_pegawai_snapshot (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    pengajuan_id uuid NOT NULL
        REFERENCES perlengkapan.pengajuan_pakaian_dinas(id) ON DELETE CASCADE,
    -- MySIMKARI kode_satker (SoT: integrasi.mysimkari_satker.kode_satker), sama
    -- seperti `_satker.satker_id` setelah V006. BUKAN surrogate bigint.
    satker_id character varying(20) NOT NULL,
    nip character varying(30) NOT NULL,
    -- `nama_pegawai`, bukan `nama` generik: standar penamaan kolom F5-B
    -- (layanan/AGENTS.md) meminta kualifikasi domainnya.
    --
    -- NOT NULL, tetapi penulisnya menyalin `COALESCE(p.nama, p.nip)`:
    -- MySIMKARI mengizinkan nama NULL, dan satu pegawai tanpa nama tidak boleh
    -- menggagalkan pembuatan periode untuk SELURUH satker (transaksi snapshot
    -- adalah satu INSERT..SELECT untuk semua satker target).
    nama_pegawai text NOT NULL,
    pangkat text,
    jabatan text,
    eselon text,
    -- Nilai mentah dari MySIMKARI (kolom sumbernya TEXT); CHECK L/P tidak
    -- dipasang di sini karena baris sumber bisa kosong, dan snapshot harus
    -- setia pada apa yang ada saat pembekuan.
    jenis_kelamin text,
    gol_kd text,
    jenis text,
    foto text,
    snapshot_at timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT pengajuan_pakaian_dinas_pegawai_snapshot_unique
        UNIQUE (pengajuan_id, satker_id, nip)
);

COMMENT ON TABLE perlengkapan.pengajuan_pakaian_dinas_pegawai_snapshot IS
    'Daftar pegawai yang DIBEKUKAN per kampanye pakaian dinas saat validator pusat membuat pengajuan. Sumber tampilan roster ke semua role; tidak berubah walau MySIMKARI berubah.';

COMMENT ON COLUMN perlengkapan.pengajuan_pakaian_dinas_pegawai_snapshot.snapshot_at IS
    'Kapan baris ini dibekukan; sama untuk seluruh baris satu kampanye.';

-- Pembacaan selalu per (kampanye, satker).
CREATE INDEX IF NOT EXISTS idx_ppd_pegawai_snapshot_pengajuan_satker
    ON perlengkapan.pengajuan_pakaian_dinas_pegawai_snapshot (pengajuan_id, satker_id);
