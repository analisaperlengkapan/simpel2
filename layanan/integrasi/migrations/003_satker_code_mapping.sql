-- ============================================================================
-- Migration: Satker code cross-reference (task #43 / #65)
-- Description: canonical mapping between MySIMKARI `kode_satker` (Kejaksaan-
--   internal, used for org/RBAC/pegawai) and SIMAN `kdsatker_keu` (Kemenkeu/SAKTI
--   finance code, used for BMN assets). The two code systems DIFFER — only
--   `nama_satker` overlaps. integrasi owns BOTH sources (mysimkari_satker +
--   siman_aset), so the cross-ref SoT belongs here. Consumers (perlengkapan
--   bank_aset BMN-per-satker scoping) JOIN via this mapping instead of the
--   current fragile string match on `nama_satker`.
-- Author: SIMPEL Team
-- Created: 2026-06-10 · Revised: 2026-06-17 (SIMAN-first; real-data-validated)
--
-- REAL-DATA VALIDATION (2026-06-17, controlled MySIMKARI+SIMAN pull — see memory
-- project_p1a_satker_readmodel_assumptions):
--   * The codes are DISJOINT: MySIMKARI `kode_satker` is dotted ("02.28"); SIMAN
--     `kdsatker_keu` is a 20-char finance code "006 01 WWWW SSSSSS AAA KP/KD"
--     (KL · eselon1 · WILAYAH(6-9) · satker · anak(000=induk) · KP=pusat/KD=daerah).
--     0 exact code matches → name is the only join key.
--   * SIMAN has ~16 satkers with NO MySIMKARI counterpart (have BMN, not in the
--     org/personnel hierarchy): 7 Jaksa Agung Muda, Badiklat, Badan Pemulihan
--     Aset, 4 overseas perwakilan, 3 RS Adhyaksa — all wilayah `0199` (pusat) / RS.
--     The reverse (MySIMKARI satker with no SIMAN asset) is effectively empty.
--   * Name-match is unreliable (~12-15% noise: abbreviations KEJARI/KEJATI/CABJARI,
--     province abbrev SUMUT/KALTENG, KAB./KOTA, hyphens, typos). Exact-normalized
--     match (below) is SAFE (no wrong matches) but partial; the remaining ones need
--     `verified` overrides. A `pg_trgm` fuzzy pass raises coverage but can mis-match
--     (e.g. "KEJATI SUMUT" → "...ACEH") so it is a REVIEW/seeding tool, not a
--     runtime authority — promote only confirmed pairs to `verified=TRUE`.
--
-- The view is SIMAN-FIRST (anchored on the asset-bearing SIMAN satker list, per
-- user direction): every SIMAN satker appears, with its MySIMKARI counterpart or
-- NULL (= SIMAN-only), plus the decoded `wilayah_kode` for wilayah-tier scoping.
--
-- NOTE (#17): `siman_aset.kdsatker_keu` must be populated by the SIMAN ingest. The
-- production scheduler currently persists via the dynamic `db::save_to_database`
-- path (schema-on-write, writes the raw `kode_satker` field) and does NOT set the
-- curated `kdsatker_keu` — reconciling the two ingest paths is the integrasi
-- rebuild (#17). Until then, `kdsatker_keu` is populated out-of-band (staging seed).
--
-- Expand/contract: additive only; idempotent (IF NOT EXISTS / CREATE OR REPLACE).
-- ============================================================================

CREATE SCHEMA IF NOT EXISTS integrasi;
CREATE EXTENSION IF NOT EXISTS pg_trgm WITH SCHEMA public;
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
    match_method TEXT NOT NULL DEFAULT 'manual',  -- 'manual' | 'official_ref' | 'nama_norm_exact' | 'nama_trgm'
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
-- Name normalizer (IMMUTABLE): case/whitespace-fold + fix the recurring real-data
-- variants (triple-A typo "KEJAKSAAAN"; abbreviations KEJARI/KEJATI/CABJARI).
-- Province abbreviations (SUMUT, KALTENG, …) and place-name spelling are NOT
-- expanded here (long, error-prone) — those resolve via `verified` overrides.
-- ----------------------------------------------------------------------------
CREATE OR REPLACE FUNCTION integrasi.satker_nama_norm(p TEXT)
RETURNS TEXT LANGUAGE sql IMMUTABLE AS $$
    SELECT upper(
        regexp_replace(
          regexp_replace(
            regexp_replace(
              regexp_replace(
                regexp_replace(btrim(coalesce(p, '')), '\s+', ' ', 'g'),
              'KEJAKSA+N', 'KEJAKSAAN', 'gi'),
            '^KEJARI ', 'KEJAKSAAN NEGERI ', 'i'),
          '^KEJATI ', 'KEJAKSAAN TINGGI ', 'i'),
        '^CABJARI ', 'CABANG KEJAKSAAN NEGERI ', 'i')
    )
$$;

-- Re-shaping a view changes its column set, which CREATE OR REPLACE VIEW cannot
-- do — so a reshape needs a real DROP. This one is unconditional and safe:
-- nothing depends on the resolved view (004 only CREATE-OR-REPLACEs it).
DROP VIEW IF EXISTS integrasi.v_satker_code_map;

-- ----------------------------------------------------------------------------
-- Auto-derive view (SIMAN-FIRST): every distinct SIMAN satker, LEFT JOIN'd to its
-- MySIMKARI counterpart by NORMALIZED nama_satker. `kode_satker` IS NULL marks a
-- SIMAN-only satker (has BMN, no org/personnel entry — review needed). Decodes
-- `wilayah_kode` (digits 6-9) + `is_pusat` (KP suffix) from `kdsatker_keu`.
-- Exact-normalized match only (safe, fast, no false positives) — NOT authoritative;
-- promote confirmed rows into satker_code_map with verified=TRUE.
-- ----------------------------------------------------------------------------
-- `v_satker_code_map_auto` is NOT dropped unconditionally, and that is
-- load-bearing: 004 materializes this very view
-- (`CREATE MATERIALIZED VIEW mv_satker_code_map_auto AS SELECT * FROM` it), so
-- once 004 has run anywhere, an unconditional DROP fails with
--
--   cannot drop view v_satker_code_map_auto because other objects depend on it
--   DETAIL: materialized view mv_satker_code_map_auto depends on it
--
-- and since this runner re-applies EVERY migration on EVERY invocation, that
-- turned integrasi-migrate into something that could only ever succeed once.
-- It blocked the rc28 staging upgrade outright.
--
-- So: try CREATE OR REPLACE first, which is what a re-run actually needs and
-- which touches no dependent. Only a genuine reshape raises
-- `invalid_table_definition`, and only then do we DROP ... CASCADE and rebuild —
-- 004 recreates the snapshot and its indexes right after, in the same run.
-- The DDL is held in one variable so the two paths can never drift apart.
DO $do$
DECLARE
    ddl CONSTANT text := $ddl$
CREATE OR REPLACE VIEW integrasi.v_satker_code_map_auto AS
WITH siman_satker AS (
    SELECT DISTINCT
        kdsatker_keu,
        nama_satker,
        integrasi.satker_nama_norm(nama_satker) AS nama_norm,
        substring(kdsatker_keu FROM 6 FOR 4)    AS wilayah_kode,
        (right(kdsatker_keu, 2) = 'KP')         AS is_pusat
    FROM integrasi.siman_aset
    WHERE kdsatker_keu IS NOT NULL AND nama_satker IS NOT NULL
),
mysimkari AS (
    SELECT
        kode_satker,
        nama_satker,
        integrasi.satker_nama_norm(nama_satker) AS nama_norm
    FROM integrasi.mysimkari_satker
    WHERE nama_satker IS NOT NULL
)
SELECT
    m.kode_satker,                                   -- NULL = SIMAN-only
    s.kdsatker_keu,
    s.wilayah_kode,
    s.is_pusat,
    m.nama_satker AS nama_satker_mysimkari,
    s.nama_satker AS nama_satker_siman,
    CASE WHEN m.kode_satker IS NULL THEN 'siman_only' ELSE 'nama_norm_exact' END AS match_method
FROM siman_satker s
LEFT JOIN mysimkari m ON m.nama_norm = s.nama_norm
    $ddl$;
BEGIN
    EXECUTE ddl;
EXCEPTION WHEN invalid_table_definition THEN
    DROP VIEW IF EXISTS integrasi.v_satker_code_map_auto CASCADE;
    EXECUTE ddl;
END
$do$;

-- ----------------------------------------------------------------------------
-- Resolved view: verified manual mappings take precedence; everything else falls
-- back to the SIMAN-first auto candidates (incl. SIMAN-only rows so consumers see
-- the FULL asset-bearing satker set). Consumers JOIN here for kode_satker ↔
-- kdsatker_keu and may read `wilayah_kode` for wilayah-tier scoping.
-- ----------------------------------------------------------------------------
CREATE OR REPLACE VIEW integrasi.v_satker_code_map AS
SELECT
    kode_satker,
    kdsatker_keu,
    substring(kdsatker_keu FROM 6 FOR 4) AS wilayah_kode,
    (right(kdsatker_keu, 2) = 'KP')      AS is_pusat,
    nama_satker,
    match_method,
    TRUE AS verified
FROM integrasi.satker_code_map
WHERE kdsatker_keu IS NOT NULL
UNION ALL
SELECT
    a.kode_satker,
    a.kdsatker_keu,
    a.wilayah_kode,
    a.is_pusat,
    COALESCE(a.nama_satker_mysimkari, a.nama_satker_siman) AS nama_satker,
    a.match_method,
    FALSE AS verified
FROM integrasi.v_satker_code_map_auto a
WHERE a.kdsatker_keu NOT IN (
    SELECT kdsatker_keu FROM integrasi.satker_code_map WHERE kdsatker_keu IS NOT NULL
);

COMMENT ON FUNCTION integrasi.satker_nama_norm(TEXT) IS
    'Normalize a satker name for cross-source matching: trim/ws-fold + fix '
    'KEJAKSAAAN typo + expand KEJARI/KEJATI/CABJARI. IMMUTABLE.';
COMMENT ON TABLE integrasi.satker_code_map IS
    'Canonical MySIMKARI kode_satker <-> SIMAN kdsatker_keu cross-ref (task #43/#65). '
    'integrasi = owner/SoT. verified rows override auto-derive. Codes are disjoint; '
    'name-match is a seed — promote confirmed pairs to verified=TRUE.';
COMMENT ON VIEW integrasi.v_satker_code_map IS
    'Resolved SIMAN-first satker map for consumers: verified manual rows + '
    'exact-normalized name candidates (incl. SIMAN-only, kode_satker NULL). '
    'wilayah_kode = digits 6-9 of kdsatker_keu (Kejaksaan wilayah for RBAC scoping).';
