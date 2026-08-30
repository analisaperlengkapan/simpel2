-- ============================================================================
-- 006 — Kodefikasi barang: snapshot of the barang codification SIMAN publishes.
--
-- WHY THIS EXISTS
--
-- The kebutuhan-BMN form asks an operator to TYPE a nama barang and a kode
-- barang. Both already exist in this schema, so typing them invites exactly the
-- two failures the reference rule is meant to prevent: a wrong code, and two
-- spellings of one item that no longer aggregate together.
--
-- The codification is a FUNCTION, measured over the 624 533 real assets in the
-- staging snapshot 2026-08-30: 2 033 distinct `kd_brg` map to 2 033 names — not
-- one code carries two. The reverse is NOT a function: seven names ("Genset", "Helmet",
-- "Voice Recorder", "Alat Ukur Lainnya", "Blender", "Digital Thermometer",
-- "Hematology Analyzer") are each shared by two codes. That asymmetry is the
-- whole reason a picker must return a CODE and never a name.
--
-- WHY MATERIALIZED
--
-- Deriving it live costs a full scan. Measured on staging, the search a
-- typeahead issues —
--
--   SELECT … FROM integrasi.siman_aset WHERE <name> ILIKE '%station wagon%'
--     GROUP BY kode, nama
--
-- — is a parallel seq scan over 624 533 rows at **1 285 ms**, per keystroke
-- after debounce. The same search over this snapshot reads 2 038 rows.
--
-- The snapshot is derived data owned by its source schema, not a second copy of
-- someone else's master: `integrasi` owns `siman_aset`, so it owns the
-- aggregate. Consumers read it cross-schema (perlengkapan `bank_aset`), which
-- is the SSoT rule, not an exception to it.
--
-- SHAPE: grouped by (kode, nama) rather than by code with `min(nama)`.
-- Collapsing to one row per code would HIDE a source that has stopped being a
-- function, where two rows show the operator the ambiguity and let them pick
-- the code they mean.
--
-- That is not hypothetical. Run against staging as it stands, this definition
-- returns 2 038 rows for 2 034 distinct codes: FOUR codes carry two names. All
-- four are the e2e fixture rows that gave a real code another item's name —
-- one seeded row each against 7 428, 43 791, 2 141 and 166 real ones. A UNIQUE
-- index on the code alone would therefore have failed the very first REFRESH
-- on staging, taking the picker down to report a fixture problem. Correcting
-- the fixture collapses all four; the definition does not depend on that
-- having happened.
--
-- Expand/contract: additive only. Idempotent — this runner re-applies every
-- migration on every invocation, so the file must CONVERGE, not merely succeed
-- once (see AGENTS.md §0 and infra/scripts/check-migration-replay.sh).
-- ============================================================================

SET search_path TO integrasi, public;

CREATE MATERIALIZED VIEW IF NOT EXISTS integrasi.mv_kodefikasi_barang AS
WITH norm AS (
    SELECT
        -- Both halves of every later comparison are reduced the same way.
        -- SIMAN stores ten undotted digits (3050201002) while the rest of the
        -- system writes the dotted presentation form (3.05.02.01.002);
        -- comparing the two spellings is an equality that can never hold, and
        -- it fails as an empty result rather than as an error.
        replace(
            COALESCE(NULLIF(btrim(kode_barang), ''), NULLIF(btrim(kd_brg), ''), ''),
            '.', ''
        )                                                                 AS kode_barang,
        -- `ur_sskel` is the STANDARD name belonging to the code. `nama` is the
        -- SIMAN operator's own label for one item: 64 220 distinct values
        -- across those same codes, blank in 22% of rows, and a copy of `merk`
        -- in 77%. Reading `nama` here would turn a taxonomy into free text.
        COALESCE(NULLIF(btrim(nama_barang), ''), NULLIF(btrim(ur_sskel), ''), '')
                                                                          AS nama_barang,
        NULLIF(btrim(kdsatker_keu), '')                                   AS kdsatker_keu
    FROM integrasi.siman_aset
)
SELECT
    kode_barang,
    nama_barang,
    COUNT(*)::BIGINT                      AS jumlah_aset,
    COUNT(DISTINCT kdsatker_keu)::BIGINT  AS jumlah_satker
FROM norm
WHERE kode_barang <> ''
  AND nama_barang <> ''
GROUP BY kode_barang, nama_barang;

-- UNIQUE on the pair, not on the code: it is the grouping key, and a unique
-- constraint on the code alone would turn a source that stopped being a
-- function into a FAILED REFRESH — taking the picker down to report a data
-- problem the picker itself is able to display.
CREATE UNIQUE INDEX IF NOT EXISTS idx_mv_kodefikasi_barang_pair
    ON integrasi.mv_kodefikasi_barang (kode_barang, nama_barang);
CREATE INDEX IF NOT EXISTS idx_mv_kodefikasi_barang_nama_lower
    ON integrasi.mv_kodefikasi_barang (lower(nama_barang));

-- Plain REFRESH, deliberately not CONCURRENTLY: this runner applies each file
-- in ONE implicit transaction and CONCURRENTLY cannot run inside one. It holds
-- ACCESS EXCLUSIVE for the ~1.3 s the aggregate takes, at deploy and at the
-- refresh cadence — the same trade 004 already makes.
--
-- On a fresh install `siman_aset` is empty, so the snapshot is empty. That is
-- correct and harmless: the picker reports "no match" and the first post-sync
-- refresh fills it.
REFRESH MATERIALIZED VIEW integrasi.mv_kodefikasi_barang;

ANALYZE integrasi.mv_kodefikasi_barang;

COMMENT ON MATERIALIZED VIEW integrasi.mv_kodefikasi_barang IS
    'Barang codification (kode barang -> nama barang) derived from siman_aset. '
    'Exists so a form can OFFER the codification instead of asking an operator '
    'to type it: deriving the same search live is a 1 285 ms scan of 624 533 '
    'rows per keystroke. Refreshed by integrasi-migrate (deploy) and by the '
    'integrasi-snapshot-refresh CronJob. NOT a second master — integrasi owns '
    'siman_aset, so it owns this aggregate of it.';
