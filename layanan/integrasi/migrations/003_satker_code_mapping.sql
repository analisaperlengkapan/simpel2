-- ============================================================================
-- Migration: Satker code cross-reference (task #43)
-- Description: canonical mapping between MySIMKARI `kode_satker` (Kejaksaan-
--   internal, used for org/RBAC/pegawai) and SIMAN `kdsatker_keu` (Kemenkeu/SAKTI
--   finance code, used for BMN assets). The two code systems DIFFER — only
--   `nama_satker` overlaps. integrasi owns BOTH sources (mysimkari_satker +
--   siman_aset), so the cross-ref SoT belongs here. Consumers (perlengkapan
--   bank_aset BMN-per-satker scoping) JOIN via this mapping instead of the
--   current fragile string match on `nama_satker`.
-- Author: SIMPEL Team
-- Created: 2026-06-10
--
-- BEST-EFFORT / STAGING-VALIDATED (see memory project_p1a_satker_readmodel_
-- assumptions): the auto-derive view seeds candidate pairs by NORMALIZED
-- nama_satker. Real values (do MySIMKARI kode_satker & SIMAN kdsatker_keu truly
-- differ? does name-match resolve cleanly, or are there dupes/typos requiring
-- manual/official-reference overrides?) MUST be validated against the real
-- staging data pull (P2/F5-E). The canonical table below carries a `verified`
-- flag + `match_method` so staging review can promote/override auto matches
-- WITHOUT touching this migration (expand/contract: additive only).
-- Idempotent (IF NOT EXISTS / CREATE OR REPLACE) — safe to re-run.
-- ============================================================================

CREATE SCHEMA IF NOT EXISTS integrasi;
SET search_path TO integrasi, public;

-- ----------------------------------------------------------------------------
-- Canonical cross-ref table (manual/verified overrides live here).
-- One MySIMKARI satker maps to at most one SIMAN finance code (1:1 expected;
-- staging may reveal exceptions → revisit). Until a row is `verified`, treat the
-- auto-derived view as the source of candidates.
-- ----------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS integrasi.satker_code_map (
    kode_satker  TEXT PRIMARY KEY,          -- MySIMKARI (mysimkari_satker.kode_satker)
    kdsatker_keu TEXT,                       -- SIMAN finance code (siman_aset.kdsatker_keu)
    nama_satker  TEXT,                       -- reference name at time of mapping
    match_method TEXT NOT NULL DEFAULT 'manual',  -- 'manual' | 'official_ref' | 'nama_satker_exact'
    verified     BOOLEAN NOT NULL DEFAULT FALSE,  -- confirmed by staging review / official reference
    notes        TEXT,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- A finance code should not map to two MySIMKARI satkers (partial: ignore NULLs).
CREATE UNIQUE INDEX IF NOT EXISTS uq_satker_code_map_kdkeu
    ON integrasi.satker_code_map (kdsatker_keu)
    WHERE kdsatker_keu IS NOT NULL;

-- ----------------------------------------------------------------------------
-- Auto-derive view: best-effort candidate pairs by NORMALIZED nama_satker
-- (case/whitespace-insensitive). NOT authoritative — review at staging, then
-- INSERT confirmed rows into satker_code_map with verified=TRUE.
-- ----------------------------------------------------------------------------
CREATE OR REPLACE VIEW integrasi.v_satker_code_map_auto AS
WITH siman_satker AS (
    SELECT DISTINCT
        kdsatker_keu,
        nama_satker,
        upper(regexp_replace(btrim(nama_satker), '\s+', ' ', 'g')) AS nama_norm
    FROM integrasi.siman_aset
    WHERE kdsatker_keu IS NOT NULL AND nama_satker IS NOT NULL
),
mysimkari AS (
    SELECT
        kode_satker,
        nama_satker,
        upper(regexp_replace(btrim(nama_satker), '\s+', ' ', 'g')) AS nama_norm
    FROM integrasi.mysimkari_satker
    WHERE nama_satker IS NOT NULL
)
SELECT
    m.kode_satker,
    s.kdsatker_keu,
    m.nama_satker            AS nama_satker_mysimkari,
    s.nama_satker            AS nama_satker_siman,
    'nama_satker_exact'::TEXT AS match_method
FROM mysimkari m
JOIN siman_satker s ON s.nama_norm = m.nama_norm;

-- ----------------------------------------------------------------------------
-- Resolved view: verified manual mappings take precedence over auto-derived
-- candidates. Consumers query THIS view for kode_satker ↔ kdsatker_keu.
-- ----------------------------------------------------------------------------
CREATE OR REPLACE VIEW integrasi.v_satker_code_map AS
SELECT
    kode_satker,
    kdsatker_keu,
    nama_satker AS nama_satker,
    match_method,
    TRUE AS verified
FROM integrasi.satker_code_map
WHERE kdsatker_keu IS NOT NULL
UNION
SELECT
    a.kode_satker,
    a.kdsatker_keu,
    a.nama_satker_mysimkari AS nama_satker,
    a.match_method,
    FALSE AS verified
FROM integrasi.v_satker_code_map_auto a
WHERE a.kode_satker NOT IN (SELECT kode_satker FROM integrasi.satker_code_map);

COMMENT ON TABLE integrasi.satker_code_map IS
    'Canonical MySIMKARI kode_satker <-> SIMAN kdsatker_keu cross-ref (task #43). '
    'integrasi = owner/SoT of the mapping. verified rows override auto-derive. '
    'Validate values at staging (P2/F5-E) — codes differ; name-match is a seed only.';
COMMENT ON VIEW integrasi.v_satker_code_map IS
    'Resolved satker code mapping for consumers: verified manual rows + '
    'best-effort name-matched candidates. Join here instead of matching nama_satker.';
