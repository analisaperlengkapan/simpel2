-- Tarik semua satker
-- name: GetAllSatker :many
SELECT * FROM mysimkari_satker ORDER BY nama_satker;

-- Hapus semua data satker
-- name: DeleteAllSatker :exec
DELETE FROM mysimkari_satker;

-- Masukkan satu data satker
-- name: InsertSatker :exec
INSERT INTO mysimkari_satker (
    id, nama_satker, provinsi, wilayah, kategori_satker
) VALUES (
    $1, $2, $3, $4, $5
) ON CONFLICT (id) DO UPDATE SET
    nama_satker = EXCLUDED.nama_satker,
    provinsi = EXCLUDED.provinsi,
    wilayah = EXCLUDED.wilayah,
    kategori_satker = EXCLUDED.kategori_satker,
    updated_at = now();

-- Truncate pegawai berdasarkan ID satker
-- name: TruncatePegawaiBySatker :exec
DELETE FROM mysimkari_pegawai WHERE satker_id = $1;

-- Masukkan banyak pegawai sekaligus
-- name: InsertPegawai :copyfrom
INSERT INTO mysimkari_pegawai (
    nip, nama, email_dinas, golpang, jabatan, jenis_jabatan_terakhir,
    agama, jenis_kelamin, gol_kd, eselon, foto, satker_id
) VALUES (
    $1, $2, $3, $4, $5, $6,
    $7, $8, $9, $10, $11, $12
);
