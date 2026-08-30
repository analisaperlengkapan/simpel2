-- Two UUID columns on a usulan penghapusan that reference nothing.
--
-- The form asked an operator to TYPE both of them ("Asset ID (UUID)",
-- "Satker ID"), and neither has a source:
--
--   * `asset_id` — SIMAN assets live in `integrasi.siman_aset`, whose `id` is
--     BIGSERIAL. There is no UUID anywhere to copy. Measured on staging: 0 of
--     the 3 stored rows match any `siman_aset.id`, all three are e2e fixtures
--     with invented values, and the column has no foreign key and is never
--     JOINed. An asset's identity here is satker + kode barang + NUP — that is
--     what `verify_asset_siman` looks it up by.
--   * `satker_id` — shadowed by `satker_code`, which is derived from the
--     creator's JWT claims and is the column the RBAC predicate actually
--     filters on. The only list filter that read `satker_id` was never given a
--     value by any caller (the one construction site passes `None`).
--
-- Both carried an index, so the database was maintaining B-trees over columns
-- that pointed at nothing.
--
-- Dropping rather than relaxing to NULL: a nullable column nothing resolves is
-- still a field the next form will offer to fill. `IF EXISTS` because a
-- migration that has to be re-runnable is cheaper than one that is not.
ALTER TABLE perlengkapan.penghapusan_bmn DROP COLUMN IF EXISTS asset_id;
ALTER TABLE perlengkapan.penghapusan_bmn DROP COLUMN IF EXISTS satker_id;
ALTER TABLE perlengkapan.penghapusan_bmn_item DROP COLUMN IF EXISTS asset_id;
