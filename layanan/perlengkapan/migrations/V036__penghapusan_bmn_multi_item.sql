-- V036: Multi-item Usulan SK Penghapusan BMN (Fase 2.8)
--
-- Sebelumnya satu record `perlengkapan.penghapusan_bmn` hanya memuat SATU
-- aset (kolom kode_barang / nama_barang / nup / asset_id / nilai_perolehan
-- di tabel utama). Operator harus membuat banyak record untuk satu usulan SK
-- yang sebenarnya mencakup beberapa BMN — tidak realistis.
--
-- V036 menambahkan tabel anak `penghapusan_bmn_item` (1 usulan → N item).
--
-- Backward compat (R2/§9): kolom item di tabel utama TIDAK di-drop. Tabel
-- utama tetap menyimpan "item utama" (item pertama) agar kode & laporan lama
-- yang membaca kolom tunggal tetap berfungsi. Migrasi mem-backfill satu baris
-- item dari setiap record lama, sehingga seluruh usulan eksisting otomatis
-- memiliki representasi multi-item (berisi 1 item).

CREATE TABLE IF NOT EXISTS perlengkapan.penghapusan_bmn_item (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    penghapusan_id UUID NOT NULL
        REFERENCES perlengkapan.penghapusan_bmn(id) ON DELETE CASCADE,
    asset_id UUID,
    kode_barang VARCHAR(50) NOT NULL,
    nama_barang VARCHAR(255) NOT NULL,
    nup VARCHAR(50) NOT NULL,
    nilai_perolehan DECIMAL(15, 2),
    -- TRUE bila nilai berasal dari backfill (perlu verifikasi), selaras V029.
    nilai_perolehan_dari_backfill BOOLEAN NOT NULL DEFAULT FALSE,
    -- Kondisi aset menurut SIMAN saat usulan dibuat (BAIK/RR/RB), opsional.
    kondisi VARCHAR(50),
    -- Urutan tampil/cetak dalam SK (1-based).
    urutan INT NOT NULL DEFAULT 1,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_penghapusan_bmn_item_penghapusan_id
    ON perlengkapan.penghapusan_bmn_item(penghapusan_id);

-- Backfill: setiap usulan lama → satu item dari kolom tunggal tabel utama.
-- Idempoten: hanya menyisipkan bila usulan tsb belum punya item.
INSERT INTO perlengkapan.penghapusan_bmn_item
    (penghapusan_id, asset_id, kode_barang, nama_barang, nup,
     nilai_perolehan, nilai_perolehan_dari_backfill, urutan)
SELECT
    p.id, p.asset_id, p.kode_barang, p.nama_barang, p.nup,
    p.nilai_perolehan, COALESCE(p.nilai_perolehan_dari_backfill, FALSE), 1
FROM perlengkapan.penghapusan_bmn p
WHERE NOT EXISTS (
    SELECT 1 FROM perlengkapan.penghapusan_bmn_item i
    WHERE i.penghapusan_id = p.id
);

COMMENT ON TABLE perlengkapan.penghapusan_bmn_item IS
    'Item BMN dalam satu Usulan SK Penghapusan (Fase 2.8). 1 usulan → N item. '
    'Tabel utama tetap menyimpan item pertama untuk backward compatibility.';
COMMENT ON COLUMN perlengkapan.penghapusan_bmn_item.urutan IS
    'Urutan tampil/cetak item dalam SK (1-based).';
