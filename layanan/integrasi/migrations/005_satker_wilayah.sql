-- ============================================================================
-- Migration: one definition of "wilayah" — the Kejaksaan Tinggi above a satker
-- Description: RBAC tier `validator_wilayah` had no shared definition of what a
--   wilayah IS, so two grew independently and neither matched the domain:
--
--     consumer                                   key                              n grup  terbesar
--     ----------------------------------------   ------------------------------   ------  --------
--     SatkerScope / kebutuhan_bmn / pakaian_dinas mysimkari_satker.wilayah              5       217
--     AsetScope (bank aset)                       v_satker_code_map.wilayah_kode       36        39
--
--   `mysimkari_satker.wilayah` does NOT hold a Kejati name, which is what every
--   docstring reading it claimed. MEASURED on staging (539 satkers from the real
--   2026-06-17 MySIMKARI snapshot) it holds `I`, `II`, `III` — the JAM-level
--   supervision grouping. `wilayah = 'I'` spans **15 Kejaksaan Tinggi** (Aceh,
--   Bengkulu, Jambi, Kalbar, Kalsel, …) = 238 satkers, so a validator at Kejati
--   Kepulauan Riau was reading rows from Sumatera Utara and Kalimantan Selatan.
--   Confirmed with the stakeholder 2026-08-26: wilayah means the Kejati.
-- Author: SIMPEL Team
-- Created: 2026-08-26
--
-- WHY A VIEW AND NOT A MATERIALIZED ONE — the opposite call from 004, on
-- purpose. 004 materialized because its source was a DISTINCT over all 624 533
-- `siman_aset` rows. This one climbs `mysimkari_satker`: 543 rows, depth ≤ 3.
-- MEASURED: a wilayah-scoped COUNT over `izin_pemakaian_bmn` through this
-- recursion runs in 4.3 ms and still uses `idx_izin_pemakaian_bmn_satker_code`.
-- A snapshot would buy nothing and would inherit the failure mode 004 documents
-- at length: stale ⇒ satker-scoped users fail closed to ZERO rows. Freshness
-- for free beats 4 ms.
--
-- Expand/contract: additive only, idempotent (CREATE OR REPLACE). No table is
-- touched and no consumer is required to migrate in the same release.
-- ============================================================================

SET search_path TO integrasi, public;

-- ----------------------------------------------------------------------------
-- Which Kejaksaan Tinggi is a satker under?
--
-- The hierarchy is three levels deep (Kejagung → Kejati → Kejari → Cabjari) and
-- `parent_id` references the parent's `api_id`, not its `kode_satker` — the
-- assumption authenc's read-model already bakes in, now confirmed against real
-- data: 538 of 543 rows resolve, and the 34 resulting groups average 15.8
-- satkers (largest: Kejati Jawa Timur, 40).
--
-- Climb until the node IS a Kejati, so a Kejati maps to itself (depth 0), a
-- Kejari to its parent (depth 1) and a Cabjari to its grandparent (depth 2).
-- Kejaksaan Agung and its pusat units never reach a Kejati and so get NO row —
-- correct, because those are cross-satker roles that carry no wilayah tier.
--
-- Matching is on `tipe_satker`, a closed four-value vocabulary in the source
-- ('Kejaksaan Agung' / 'Kejaksaan Tinggi' / 'Kejaksaan Negeri' / 'Cabang
-- Kejaksaan Negeri'), never on `nama_satker`. COALESCE keeps a NULL
-- `tipe_satker` from silently halting the climb, and `depth < 4` bounds it
-- against a cycle in upstream data.
-- ----------------------------------------------------------------------------
CREATE OR REPLACE VIEW integrasi.v_satker_wilayah AS
WITH RECURSIVE climb AS (
    SELECT
        s.kode_satker AS kode_satker,
        s.kode_satker AS node_code,
        s.nama_satker AS node_nama,
        s.api_id,
        s.parent_id,
        s.tipe_satker,
        0 AS depth
    FROM integrasi.mysimkari_satker s
    UNION ALL
    SELECT
        c.kode_satker,
        p.kode_satker,
        p.nama_satker,
        p.api_id,
        p.parent_id,
        p.tipe_satker,
        c.depth + 1
    FROM climb c
    JOIN integrasi.mysimkari_satker p ON p.api_id = c.parent_id
    WHERE COALESCE(lower(c.tipe_satker), '') NOT LIKE '%tinggi%'
      AND c.depth < 4
)
SELECT DISTINCT ON (kode_satker)
    kode_satker,
    node_code AS wilayah_code,
    node_nama AS wilayah_nama
FROM climb
WHERE COALESCE(lower(tipe_satker), '') LIKE '%tinggi%'
ORDER BY kode_satker, depth;

COMMENT ON VIEW integrasi.v_satker_wilayah IS
    'kode_satker -> the Kejaksaan Tinggi that supervises it. THE definition of '
    'the RBAC wilayah tier; every scope must read it rather than grow a second '
    'one. Derived from the MySIMKARI hierarchy (parent_id -> api_id). Satkers '
    'with no Kejati above them (Kejagung and its pusat units) are absent by '
    'design. Do NOT use mysimkari_satker.wilayah for this: that column holds '
    'the JAM supervision grouping I/II/III, 15 Kejati per value.';
