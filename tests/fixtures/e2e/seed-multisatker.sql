-- ============================================================================
-- E2E multi-satker fixture (#33 / F5-C) — CI-safe RBAC data-scoping validation
-- ============================================================================
-- Loaded into dbsimpelv2 by the `e2e-seed` service (docker-compose.e2e.yml)
-- AFTER integrasi-migrate + authenc-migrate complete. SYNTHETIC, deterministic
-- data only — no real PII. Gives the Playwright suite multiple satkers across
-- two wilayah + per-role users so tiered scoping (AsetScope #565 /
-- SatkerScope #566 / kebutuhan #578) is OBSERVABLE on CI without staging.
--
-- Auth: JWT satker_code comes from authenc.users.satker_code (stored column);
-- role from authenc.user_roles + the active_role attribute. All test users
-- share the seed password "199203142014031001" (reuses the existing seed
-- Argon2id hash — verifies that exact password regardless of username).
--
-- Idempotent: PK/unique rows use ON CONFLICT DO NOTHING; siman_aset (BIGSERIAL,
-- no natural key) is delete-by-marker then re-inserted.
-- ============================================================================

BEGIN;

-- Resolve unqualified names against the service schemas. authenc/integrasi
-- objects live in their own schemas; triggers on authenc.users (e.g. the MFA
-- stats maintainer referencing `mfa_statistics`) resolve unqualified relations
-- via the session search_path, which defaults to `public` under a plain
-- `psql -d dbsimpelv2` — without this the first authenc.users INSERT errors
-- with `relation "mfa_statistics" does not exist`.
SET search_path TO authenc, integrasi, public;

-- ----------------------------------------------------------------------------
-- 1. integrasi: satker master (MySIMKARI). Two wilayah: DKI JAKARTA (2 satkers)
--    + JAWA BARAT (1). PUSAT entry backs the validator_pusat / existing seed user.
-- ----------------------------------------------------------------------------
INSERT INTO integrasi.mysimkari_satker (kode_satker, nama_satker, wilayah, tipe_satker, kategori_satker)
VALUES
  ('0100000', 'KEJAKSAAN AGUNG REPUBLIK INDONESIA', 'PUSAT',        'Kejaksaan Agung',  'pusat'),
  ('0200010', 'KEJAKSAAN NEGERI JAKARTA PUSAT',     'DKI JAKARTA',  'Kejaksaan Negeri', 'daerah'),
  ('0200020', 'KEJAKSAAN NEGERI JAKARTA SELATAN',   'DKI JAKARTA',  'Kejaksaan Negeri', 'daerah'),
  ('0300010', 'KEJAKSAAN NEGERI BANDUNG',           'JAWA BARAT',   'Kejaksaan Negeri', 'daerah')
ON CONFLICT (kode_satker) DO NOTHING;

-- ----------------------------------------------------------------------------
-- 2. integrasi: canonical satker code map (kode_satker ↔ SIMAN kdsatker_keu).
--    kdsatker_keu = 20-char finance code; v_satker_code_map derives
--    wilayah_kode = substring(kdsatker_keu,6,4): DKI satkers share '9999',
--    Bandung '8888' — so the wilayah tier groups the two DKI satkers together.
-- ----------------------------------------------------------------------------
INSERT INTO integrasi.satker_code_map (kode_satker, kdsatker_keu, nama_satker, match_method, verified)
VALUES
  ('0200010', '006019999010001KD', 'KEJAKSAAN NEGERI JAKARTA PUSAT',   'manual', TRUE),
  ('0200020', '006019999020001KD', 'KEJAKSAAN NEGERI JAKARTA SELATAN', 'manual', TRUE),
  ('0300010', '006018888010001KD', 'KEJAKSAAN NEGERI BANDUNG',         'manual', TRUE)
ON CONFLICT (kode_satker) DO NOTHING;

-- ----------------------------------------------------------------------------
-- 3. integrasi: SIMAN assets — 2 for satker A, 2 for B (both DKI), 1 for C
--    (Bandung). Total 5. Expected scoped counts:
--      operator_a (0200010)        -> 2   (own satker)
--      operator_b (0200020)        -> 2   (own satker)
--      validator_wilayah (DKI)     -> 4   (both DKI satkers)
--      validator_pusat / admin     -> 5   (all)
-- ----------------------------------------------------------------------------
DELETE FROM integrasi.siman_aset
 WHERE kdsatker_keu IN ('006019999010001KD', '006019999020001KD', '006018888010001KD');

INSERT INTO integrasi.siman_aset
  (jenis_aset, kategori_aset, no_aset, ur_sskel, nama, kd_brg, merk, tipe, ur_kondisi, alamat, nama_satker, kdsatker_keu, nup, rph_aset, tgl_perlh)
VALUES
  ('Peralatan dan Mesin', 'Alat Angkutan', 'A-001', 'Kendaraan Dinas Roda 4', 'Toyota Avanza', '3.05.01.04.001', 'Toyota', 'Avanza 1.3', 'BAIK', 'Jl. Sunda Kelapa No.1', 'KEJAKSAAN NEGERI JAKARTA PUSAT', '006019999010001KD', 'E2E-A-1', '250000000', '2020-01-15'),
  ('Peralatan dan Mesin', 'Alat Kantor',   'A-002', 'Personal Computer Unit',  'Laptop Dell',   '3.10.01.02.003', 'Dell',   'Latitude',   'BAIK', 'Jl. Sunda Kelapa No.1', 'KEJAKSAAN NEGERI JAKARTA PUSAT', '006019999010001KD', 'E2E-A-2', '15000000',  '2021-03-10'),
  ('Peralatan dan Mesin', 'Alat Angkutan', 'B-001', 'Kendaraan Dinas Roda 2', 'Honda Vario',   '3.05.02.01.002', 'Honda',  'Vario 125',  'BAIK', 'Jl. Ampera Raya No.2',  'KEJAKSAAN NEGERI JAKARTA SELATAN', '006019999020001KD', 'E2E-B-1', '22000000',  '2019-07-01'),
  ('Tanah',               'Tanah',         'B-002', 'Tanah Bangunan Kantor',  'Tanah Kantor',  '2.01.01.01.001', NULL,     NULL,         'BAIK', 'Jl. Ampera Raya No.2',  'KEJAKSAAN NEGERI JAKARTA SELATAN', '006019999020001KD', 'E2E-B-2', '5000000000','2010-01-01'),
  ('Peralatan dan Mesin', 'Alat Kantor',   'C-001', 'Personal Computer Unit',  'Printer Epson', '3.10.01.05.010', 'Epson',  'L3210',      'RUSAK RINGAN', 'Jl. Asia Afrika No.3', 'KEJAKSAAN NEGERI BANDUNG', '006018888010001KD', 'E2E-C-1', '4000000', '2022-11-20');

-- ----------------------------------------------------------------------------
-- 4. authenc: per-role test users (single role each so the JWT role is
--    unambiguous). Password = "199203142014031001" (reused Argon2id hash).
--    satker_code drives SatkerScope/AsetScope; validator_pusat = cross-satker.
-- ----------------------------------------------------------------------------
INSERT INTO authenc.users
  (id, username, email, password_hash, email_verified, enabled, realm_id, federated, created_at, updated_at, login_count, mfa_enabled, satker_code, require_mfa_setup, require_password_change, password_history_count, nip, nama, jabatan, phone_verified, webauthn_enabled, account_locked, failed_login_attempts)
VALUES
  ('11111111-1111-4111-8111-111111111111', '200000000000000001', '200000000000000001@kejaksaan.go.id', '$argon2id$v=19$m=65536,t=3,p=4$TG0TRGGPnVrMiDnG2RfqeQ$wwhai83/MyAlKcB8W4XLHj5iSa5ATcB/DJ/a6/5zg6M', true, true, '00000000-0000-0000-0000-000000000000', false, now(), now(), 0, false, '0200010', false, false, 0, '200000000000000001', 'E2E Operator Jakpus', 'Operator Satker', false, false, false, 0),
  ('22222222-2222-4222-8222-222222222222', '200000000000000002', '200000000000000002@kejaksaan.go.id', '$argon2id$v=19$m=65536,t=3,p=4$TG0TRGGPnVrMiDnG2RfqeQ$wwhai83/MyAlKcB8W4XLHj5iSa5ATcB/DJ/a6/5zg6M', true, true, '00000000-0000-0000-0000-000000000000', false, now(), now(), 0, false, '0200020', false, false, 0, '200000000000000002', 'E2E Operator Jaksel', 'Operator Satker', false, false, false, 0),
  ('33333333-3333-4333-8333-333333333333', '200000000000000003', '200000000000000003@kejaksaan.go.id', '$argon2id$v=19$m=65536,t=3,p=4$TG0TRGGPnVrMiDnG2RfqeQ$wwhai83/MyAlKcB8W4XLHj5iSa5ATcB/DJ/a6/5zg6M', true, true, '00000000-0000-0000-0000-000000000000', false, now(), now(), 0, false, '0200010', false, false, 0, '200000000000000003', 'E2E Validator Wilayah DKI', 'Validator Wilayah', false, false, false, 0),
  ('44444444-4444-4444-8444-444444444444', '200000000000000004', '200000000000000004@kejaksaan.go.id', '$argon2id$v=19$m=65536,t=3,p=4$TG0TRGGPnVrMiDnG2RfqeQ$wwhai83/MyAlKcB8W4XLHj5iSa5ATcB/DJ/a6/5zg6M', true, true, '00000000-0000-0000-0000-000000000000', false, now(), now(), 0, false, '0100000', false, false, 0, '200000000000000004', 'E2E Validator Pusat', 'Validator Pusat', false, false, false, 0)
ON CONFLICT (id) DO NOTHING;

-- role assignments (single role per user)
INSERT INTO authenc.user_roles (id, user_id, role_id, created_at)
VALUES
  ('1111aaaa-0000-4000-8000-000000000001', '11111111-1111-4111-8111-111111111111', 'f646c0e4-5cb4-4008-8e3b-118af6c22ede', now()), -- operator_satker
  ('2222aaaa-0000-4000-8000-000000000002', '22222222-2222-4222-8222-222222222222', 'f646c0e4-5cb4-4008-8e3b-118af6c22ede', now()), -- operator_satker
  ('3333aaaa-0000-4000-8000-000000000003', '33333333-3333-4333-8333-333333333333', '54b5d1ae-8aaa-4e31-b4b2-a77a49cfd06b', now()), -- validator_wilayah
  ('4444aaaa-0000-4000-8000-000000000004', '44444444-4444-4444-8444-444444444444', '9377d14c-22b6-4674-b080-a6ba43969eb1', now())  -- validator_pusat
ON CONFLICT (id) DO NOTHING;

-- active_role + satker_code attributes (mirror the base seed user shape)
INSERT INTO authenc.user_attributes (id, user_id, name, value, created_at)
VALUES
  ('1111bbbb-0000-4000-8000-000000000001', '11111111-1111-4111-8111-111111111111', 'active_role', 'operator_satker',  now()),
  ('1111bbbb-0000-4000-8000-000000000011', '11111111-1111-4111-8111-111111111111', 'satker_code', '0200010',          now()),
  ('2222bbbb-0000-4000-8000-000000000002', '22222222-2222-4222-8222-222222222222', 'active_role', 'operator_satker',  now()),
  ('2222bbbb-0000-4000-8000-000000000022', '22222222-2222-4222-8222-222222222222', 'satker_code', '0200020',          now()),
  ('3333bbbb-0000-4000-8000-000000000003', '33333333-3333-4333-8333-333333333333', 'active_role', 'validator_wilayah', now()),
  ('3333bbbb-0000-4000-8000-000000000033', '33333333-3333-4333-8333-333333333333', 'satker_code', '0200010',          now()),
  ('4444bbbb-0000-4000-8000-000000000004', '44444444-4444-4444-8444-444444444444', 'active_role', 'validator_pusat',  now()),
  ('4444bbbb-0000-4000-8000-000000000044', '44444444-4444-4444-8444-444444444444', 'satker_code', '0100000',          now())
ON CONFLICT (id) DO NOTHING;

COMMIT;
