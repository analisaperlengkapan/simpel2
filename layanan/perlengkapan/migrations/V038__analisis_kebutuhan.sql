-- V038: Promote `analisis_kebutuhan` from the legacy hand-coded bootstrap
-- (Database::create_tables, now removed) to a refinery migration so that
-- refinery is the single source of schema truth.
--
-- Backs the standalone Analisis Kebutuhan CRUD (`/analisis`) used by the
-- `/analitik/roadmap` UI. (The predictive `roadmap_sarpras` forecast is a
-- separate, UI-less feature — unifying them is deferred to a feature phase.)

CREATE SCHEMA IF NOT EXISTS perlengkapan;

CREATE TABLE IF NOT EXISTS perlengkapan.analisis_kebutuhan (
    id             UUID PRIMARY KEY,
    judul          VARCHAR NOT NULL,
    kategori       VARCHAR NOT NULL,
    deskripsi      TEXT,
    prioritas      VARCHAR NOT NULL DEFAULT 'sedang',
    status         VARCHAR NOT NULL DEFAULT 'draft',
    estimasi_biaya DECIMAL(15, 2),
    justifikasi    TEXT,
    created_at     TIMESTAMPTZ DEFAULT NOW(),
    updated_at     TIMESTAMPTZ DEFAULT NOW(),
    created_by     UUID,
    updated_by     UUID
);

CREATE INDEX IF NOT EXISTS idx_analisis_kebutuhan_status
    ON perlengkapan.analisis_kebutuhan (status);
CREATE INDEX IF NOT EXISTS idx_analisis_kebutuhan_created_by
    ON perlengkapan.analisis_kebutuhan (created_by);
