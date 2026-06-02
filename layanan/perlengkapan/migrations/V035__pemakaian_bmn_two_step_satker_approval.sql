-- ============================================================================
-- V035 — Pemakaian BMN: workflow internal satker 3-step (Fase 1.5)
-- ============================================================================
--
-- Sebelum: Operator Satker → Pimpinan Satker langsung.
-- Sesudah: Operator → Validator Satker → Approver Satker (semua internal
--          satker). Validator Wilayah / Validator Pusat HANYA monitor.
--
-- Stakeholder note: model approval lama salah secara organisatoris. Validator
-- Satker = peninjau pertama (kelengkapan data, kelayakan BMN); Approver
-- Satker = pengambil keputusan (Pengguna Barang Satker, mis. Kajari/Kacab).
--
-- Backward-compat: record yg sudah APPROVED via alur lama ditandai
-- `approved_via_legacy_flow = true` agar audit tidak bingung. Record yg
-- sedang SUBMITTED saat migration berjalan tidak diutak-atik — Operator
-- harus re-submit jika ingin masuk ke alur baru.
-- ============================================================================

-- ── Kolom Validator Satker ──────────────────────────────────────────────
ALTER TABLE perlengkapan.izin_pemakaian_bmn
    ADD COLUMN IF NOT EXISTS validator_satker_id        UUID,
    ADD COLUMN IF NOT EXISTS validator_satker_nama      VARCHAR(255),
    ADD COLUMN IF NOT EXISTS tanggal_validasi_satker    TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS catatan_validator_satker   TEXT;

-- ── Kolom Approver Satker ───────────────────────────────────────────────
-- Kolom `approved_by` lama tetap dipertahankan (legacy approver pimpinan).
-- Untuk alur baru gunakan `approver_satker_*` agar pemisahan terlihat di
-- query audit & dashboard.
ALTER TABLE perlengkapan.izin_pemakaian_bmn
    ADD COLUMN IF NOT EXISTS approver_satker_id         UUID,
    ADD COLUMN IF NOT EXISTS approver_satker_nama       VARCHAR(255),
    ADD COLUMN IF NOT EXISTS tanggal_approval_satker    TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS catatan_approver_satker    TEXT;

-- ── Flag legacy + optimistic locking ────────────────────────────────────
ALTER TABLE perlengkapan.izin_pemakaian_bmn
    ADD COLUMN IF NOT EXISTS approved_via_legacy_flow   BOOLEAN NOT NULL DEFAULT FALSE,
    ADD COLUMN IF NOT EXISTS version                    INTEGER NOT NULL DEFAULT 1;

-- ── Backfill: tandai record APPROVED/ACTIVE/EXPIRED/REVOKED sbg legacy ──
-- Record dgn status pasca-approval (kode ≥ 3002) yg sudah ada sebelum
-- migration ini dianggap berasal dari alur lama. Kita tidak menyentuh
-- kolom approver_satker_* — biar tetap NULL agar mudah dibedakan.
UPDATE perlengkapan.izin_pemakaian_bmn
SET approved_via_legacy_flow = TRUE
WHERE status_kode IN (3002, 3004, 3005, 3006)
  AND approved_via_legacy_flow = FALSE;

-- ── Workflow state baru di ms_aktivitas_bmn ─────────────────────────────
-- 3010 SUBMITTED_APPROVER_SATKER — diteruskan Validator Satker; menunggu
--      keputusan Approver Satker.
-- 3011 REVISI_OPERATOR          — dikembalikan ke Operator (oleh Validator
--      atau Approver Satker) dgn catatan revisi wajib.
INSERT INTO perlengkapan.ms_aktivitas_bmn (kode, nama, deskripsi, urutan, is_terminal)
VALUES
    (3010, 'SUBMITTED_APPROVER_SATKER',
     'Menunggu keputusan Approver Satker (setelah validasi internal)', 10, FALSE),
    (3011, 'REVISI_OPERATOR',
     'Dikembalikan ke Operator Satker untuk revisi (catatan wajib)', 11, FALSE)
ON CONFLICT (kode) DO UPDATE
SET nama         = EXCLUDED.nama,
    deskripsi    = EXCLUDED.deskripsi,
    urutan       = EXCLUDED.urutan,
    is_terminal  = EXCLUDED.is_terminal;

-- ── Index utk dashboard monitoring (Validator Wilayah/Pusat) ────────────
-- Read-only dashboard akan filter by status_kode → satker_id, jadi index
-- composite mengoptimalkan query agregat per wilayah.
CREATE INDEX IF NOT EXISTS idx_izin_pemakaian_status_satker
    ON perlengkapan.izin_pemakaian_bmn (status_kode, pegawai_satker_id);

-- ── Index utk audit trail per approver satker ───────────────────────────
CREATE INDEX IF NOT EXISTS idx_izin_pemakaian_approver_satker
    ON perlengkapan.izin_pemakaian_bmn (approver_satker_id)
    WHERE approver_satker_id IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_izin_pemakaian_validator_satker
    ON perlengkapan.izin_pemakaian_bmn (validator_satker_id)
    WHERE validator_satker_id IS NOT NULL;

COMMENT ON COLUMN perlengkapan.izin_pemakaian_bmn.validator_satker_id IS
    'V035 (Fase 1.5): Validator internal satker yg meneruskan/menolak. NULL utk record legacy.';
COMMENT ON COLUMN perlengkapan.izin_pemakaian_bmn.approver_satker_id IS
    'V035 (Fase 1.5): Approver internal satker (Pengguna Barang Satker). NULL utk record legacy.';
COMMENT ON COLUMN perlengkapan.izin_pemakaian_bmn.approved_via_legacy_flow IS
    'V035 (Fase 1.5): TRUE jika record di-approve via alur lama Operator → Pimpinan langsung.';
COMMENT ON COLUMN perlengkapan.izin_pemakaian_bmn.version IS
    'V035 (Fase 1.5): Optimistic lock counter — increment tiap UPDATE workflow.';
