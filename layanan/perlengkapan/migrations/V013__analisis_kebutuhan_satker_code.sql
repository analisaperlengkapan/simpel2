-- ============================================================================
-- Migration V013: identity-derived satker_code on analisis_kebutuhan
-- ============================================================================
-- `analisis_kebutuhan` (the Analitik > Roadmap list) had no owner beyond a
-- nullable `created_by`, so every authenticated caller listed and read every
-- satker's analyses: an operator in one Kejari saw another's cost estimates and
-- justifications. Same fix as V003 for the workflow tables: an authoritative
-- MySIMKARI `satker_code`, derived from the caller's claims at create (never from
-- the request body), used for tiered data scoping.
--
-- Expand/contract: additive + idempotent. Nullable — pre-existing rows keep NULL
-- and are visible only to cross-satker roles (fail-closed for the satker and
-- wilayah tiers), which is the safe default. No backfill: their provenance is
-- unknown. `helm rollback` of the app needs no DB restore — the previous version
-- simply ignores the column.
-- ============================================================================

ALTER TABLE perlengkapan.analisis_kebutuhan
    ADD COLUMN IF NOT EXISTS satker_code TEXT;

COMMENT ON COLUMN perlengkapan.analisis_kebutuhan.satker_code IS
    'MySIMKARI kode_satker of the creating operator (derived from JWT claims at '
    'create). Authoritative satker for RBAC list scoping. NULL on rows that '
    'predate V013: visible to cross-satker roles only.';

CREATE INDEX IF NOT EXISTS idx_analisis_kebutuhan_satker_code
    ON perlengkapan.analisis_kebutuhan (satker_code);
