-- ============================================================================
-- Migration V003: identity-derived satker_code on workflow tables (#66 part 2)
-- ============================================================================
-- izin_pemakaian_bmn + penghapusan_bmn previously keyed satker only by a bare
-- UUID (pegawai_satker_id / satker_id) with NO foreign key, supplied by the
-- CLIENT at create time. That is not a trustworthy RBAC boundary: the caller's
-- identity is a MySIMKARI `kode_satker` (Claims.satker_code) which had no
-- reliable link to those UUIDs, so list endpoints could not be satker-scoped.
--
-- This adds an authoritative `satker_code` (MySIMKARI kode_satker, TEXT) that
-- the service derives from the caller's claims at create — NOT from client input
-- (also closing the client-spoofed-satker gap). It is the column used for tiered
-- RBAC data scoping (operator=own satker, validator_wilayah=wilayah via
-- integrasi.mysimkari_satker.wilayah, pusat/admin=all). Mirrors the bank_aset
-- AsetScope model (#66 part 1) but keyed directly on the MySIMKARI code.
--
-- Expand/contract: additive + idempotent. Nullable — pre-existing rows (mock/
-- seed) keep NULL and are visible only to cross-satker roles (fail-closed for
-- satker/wilayah tiers), which is the safe default. No backfill of legacy UUIDs
-- (their provenance is unverified); real values flow in from create going fwd.
-- ============================================================================

ALTER TABLE perlengkapan.izin_pemakaian_bmn
    ADD COLUMN IF NOT EXISTS satker_code TEXT;

COMMENT ON COLUMN perlengkapan.izin_pemakaian_bmn.satker_code IS
    'MySIMKARI kode_satker of the creating operator (derived from JWT claims at '
    'create, #66). Authoritative satker for RBAC list scoping; NOT the legacy '
    'client-supplied pegawai_satker_id UUID.';

ALTER TABLE perlengkapan.penghapusan_bmn
    ADD COLUMN IF NOT EXISTS satker_code TEXT;

COMMENT ON COLUMN perlengkapan.penghapusan_bmn.satker_code IS
    'MySIMKARI kode_satker of the creating operator (derived from JWT claims at '
    'create, #66). Authoritative satker for RBAC list scoping; NOT the legacy '
    'client-supplied satker_id UUID.';

CREATE INDEX IF NOT EXISTS idx_izin_pemakaian_bmn_satker_code
    ON perlengkapan.izin_pemakaian_bmn (satker_code);

CREATE INDEX IF NOT EXISTS idx_penghapusan_bmn_satker_code
    ON perlengkapan.penghapusan_bmn (satker_code);
