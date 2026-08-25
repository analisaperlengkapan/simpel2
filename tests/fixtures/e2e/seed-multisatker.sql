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
--
-- SCOPE: authenc + integrasi ONLY. This file is applied by EVERY e2e job, and
-- several of them (e2e-portal, e2e-integrasi-authenc, e2e-simpelv1-integration)
-- bring up no layanan-perlengkapan — so the `perlengkapan` schema does not
-- exist there. Business-workflow preconditions therefore live in the companion
-- seed-perlengkapan-workflow.sql, applied only by jobs whose stack includes
-- layanan-perlengkapan. Do NOT add perlengkapan.* rows here.
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
-- 2b. integrasi: MySIMKARI pegawai — one per DKI satker + one Bandung. Makes the
--     gRPC GetMysimkariPegawai read path (nip_filter / kode_satker filter, the
--     one perlengkapan's IntegrasiClient exercises) assert against REAL rows
--     instead of an always-empty result. satker_id is the MySIMKARI code (TEXT).
-- ----------------------------------------------------------------------------
INSERT INTO integrasi.mysimkari_pegawai
  (nip, nama, satker_id, nama_satker, jabatan, golpang, gol_kd, jk, email, no_hp, status_pegawai)
VALUES
  ('200000000000000001', 'E2E Operator Jakpus',  '0200010', 'KEJAKSAAN NEGERI JAKARTA PUSAT',   'Operator Satker',   'Penata Muda', 'III/a', 'L', '200000000000000001@kejaksaan.go.id', '081200000001', 'aktif'),
  ('200000000000000002', 'E2E Operator Jaksel',  '0200020', 'KEJAKSAAN NEGERI JAKARTA SELATAN', 'Operator Satker',   'Penata Muda', 'III/a', 'P', '200000000000000002@kejaksaan.go.id', '081200000002', 'aktif'),
  ('200000000000000009', 'E2E Pegawai Bandung',  '0300010', 'KEJAKSAAN NEGERI BANDUNG',         'Operator Satker',   'Penata',      'III/c', 'L', '200000000000000009@kejaksaan.go.id', '081200000009', 'aktif')
ON CONFLICT (nip) DO NOTHING;

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

-- SHAPE NOTE (do not "tidy" this back). The ingest builds its INSERT column
-- list from the SIMAN payload's own JSON keys (layanan/integrasi/src/db.rs), so
-- a schema column stays empty forever unless SIMAN sends a field of that exact
-- name. SIMAN sends `no_aset`, `kd_brg`, `ur_sskel`, `ur_kondisi`; it sends
-- neither `nup` nor `kategori_aset`. Measured on the staging snapshot: `nup`
-- and `kategori_aset` are populated in 5 of 624 533 rows — and those five were
-- THIS SEED's own rows.
--
-- That is precisely how two production defects survived a green suite: the
-- dashboard read `kategori_aset` (NULL in real data → panic under
-- panic="abort", killing the backend) and the list/lookup read `nup` (empty in
-- real data → a "-" column and a 404 for every real asset). The seed populated
-- both columns, so the tests certified queries that could never work.
--
-- So this fixture now carries PRODUCTION'S SHAPE: `nup` and `kategori_aset` are
-- left NULL, the E2E markers live in `no_aset` (where a real NUP lives), and
-- `jenis_aset` uses the real SIMAN taxonomy rather than invented labels.
-- Keeping the marker STRINGS unchanged means specs that assert on "E2E-A-1"
-- keep working — they now just read it from the column production fills.
INSERT INTO integrasi.siman_aset
  (jenis_aset, kategori_aset, no_aset, ur_sskel, nama, kd_brg, merk, tipe, ur_kondisi, alamat, nama_satker, kdsatker_keu, nup, rph_aset, tgl_perlh)
VALUES
  ('Alat Angkutan Bermotor',      NULL, 'E2E-A-1', 'Kendaraan Dinas Roda 4', 'Toyota Avanza', '3.05.01.04.001', 'Toyota', 'Avanza 1.3', 'BAIK', 'Jl. Sunda Kelapa No.1', 'KEJAKSAAN NEGERI JAKARTA PUSAT', '006019999010001KD', NULL, '250000000', '2020-01-15'),
  ('Peralatan Mesin Khusus TIK',  NULL, 'E2E-A-2', 'Personal Computer Unit',  'Laptop Dell',   '3.10.01.02.003', 'Dell',   'Latitude',   'BAIK', 'Jl. Sunda Kelapa No.1', 'KEJAKSAAN NEGERI JAKARTA PUSAT', '006019999010001KD', NULL, '15000000',  '2021-03-10'),
  ('Alat Angkutan Bermotor',      NULL, 'E2E-B-1', 'Kendaraan Dinas Roda 2', 'Honda Vario',   '3.05.02.01.002', 'Honda',  'Vario 125',  'BAIK', 'Jl. Ampera Raya No.2',  'KEJAKSAAN NEGERI JAKARTA SELATAN', '006019999020001KD', NULL, '22000000',  '2019-07-01'),
  ('Tanah',                       NULL, 'E2E-B-2', 'Tanah Bangunan Kantor',  'Tanah Kantor',  '2.01.01.01.001', NULL,     NULL,         'BAIK', 'Jl. Ampera Raya No.2',  'KEJAKSAAN NEGERI JAKARTA SELATAN', '006019999020001KD', NULL, '5000000000','2010-01-01'),
  ('Peralatan Mesin Khusus TIK',  NULL, 'E2E-C-1', 'Personal Computer Unit',  'Printer Epson', '3.10.01.05.010', 'Epson',  'L3210',      'RUSAK RINGAN', 'Jl. Asia Afrika No.3', 'KEJAKSAAN NEGERI BANDUNG', '006018888010001KD', NULL, '4000000', '2022-11-20');

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
  ('44444444-4444-4444-8444-444444444444', '200000000000000004', '200000000000000004@kejaksaan.go.id', '$argon2id$v=19$m=65536,t=3,p=4$TG0TRGGPnVrMiDnG2RfqeQ$wwhai83/MyAlKcB8W4XLHj5iSa5ATcB/DJ/a6/5zg6M', true, true, '00000000-0000-0000-0000-000000000000', false, now(), now(), 0, false, '0100000', false, false, 0, '200000000000000004', 'E2E Validator Pusat', 'Validator Pusat', false, false, false, 0),
  -- Login-able ADMIN (the baseline `admin` user carries a placeholder hash and
  -- require_password_change; portal admin/IAM e2e needs a real admin login).
  ('55555555-5555-4555-8555-555555555555', '200000000000000005', '200000000000000005@kejaksaan.go.id', '$argon2id$v=19$m=65536,t=3,p=4$TG0TRGGPnVrMiDnG2RfqeQ$wwhai83/MyAlKcB8W4XLHj5iSa5ATcB/DJ/a6/5zg6M', true, true, '00000000-0000-0000-0000-000000000000', false, now(), now(), 0, false, '0100000', false, false, 0, '200000000000000005', 'E2E Admin Pusat', 'Administrator', false, false, false, 0),
  -- Satker-internal approval chain (#96). Both sit in 0200010 alongside
  -- operator_a, because the Pemakaian chain is satker-internal by design: the
  -- validator and the approver are the operator's own satker colleagues.
  ('66666666-6666-4666-8666-666666666666', '200000000000000006', '200000000000000006@kejaksaan.go.id', '$argon2id$v=19$m=65536,t=3,p=4$TG0TRGGPnVrMiDnG2RfqeQ$wwhai83/MyAlKcB8W4XLHj5iSa5ATcB/DJ/a6/5zg6M', true, true, '00000000-0000-0000-0000-000000000000', false, now(), now(), 0, false, '0200010', false, false, 0, '200000000000000006', 'E2E Validator Satker Jakpus', 'Validator Satker', false, false, false, 0),
  ('77777777-7777-4777-8777-777777777777', '200000000000000007', '200000000000000007@kejaksaan.go.id', '$argon2id$v=19$m=65536,t=3,p=4$TG0TRGGPnVrMiDnG2RfqeQ$wwhai83/MyAlKcB8W4XLHj5iSa5ATcB/DJ/a6/5zg6M', true, true, '00000000-0000-0000-0000-000000000000', false, now(), now(), 0, false, '0200010', false, false, 0, '200000000000000007', 'E2E Approver Satker Jakpus', 'Pengguna Barang Satker', false, false, false, 0)
ON CONFLICT (id) DO NOTHING;

-- role assignments (single role per user)
INSERT INTO authenc.user_roles (id, user_id, role_id, created_at)
VALUES
  ('1111aaaa-0000-4000-8000-000000000001', '11111111-1111-4111-8111-111111111111', 'f646c0e4-5cb4-4008-8e3b-118af6c22ede', now()), -- operator_satker
  ('2222aaaa-0000-4000-8000-000000000002', '22222222-2222-4222-8222-222222222222', 'f646c0e4-5cb4-4008-8e3b-118af6c22ede', now()), -- operator_satker
  ('3333aaaa-0000-4000-8000-000000000003', '33333333-3333-4333-8333-333333333333', '54b5d1ae-8aaa-4e31-b4b2-a77a49cfd06b', now()), -- validator_wilayah
  ('4444aaaa-0000-4000-8000-000000000004', '44444444-4444-4444-8444-444444444444', '9377d14c-22b6-4674-b080-a6ba43969eb1', now()), -- validator_pusat
  ('5555aaaa-0000-4000-8000-000000000005', '55555555-5555-4555-8555-555555555555', '00000000-0000-0000-0000-000000000002', now()), -- admin
  -- Role ids come from authenc migration 004, which seeds these two roles; they
  -- did not exist before, which is why nobody could hold them (#96).
  ('6666aaaa-0000-4000-8000-000000000006', '66666666-6666-4666-8666-666666666666', '7c2f1a90-6d3e-4b52-9a41-2f8e5c7b1d04', now()), -- validator_satker
  ('7777aaaa-0000-4000-8000-000000000007', '77777777-7777-4777-8777-777777777777', '9e4b3c17-8a05-4d66-b3f2-6c1a9d0e5f38', now())  -- approver_satker
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
  ('4444bbbb-0000-4000-8000-000000000044', '44444444-4444-4444-8444-444444444444', 'satker_code', '0100000',          now()),
  ('5555bbbb-0000-4000-8000-000000000005', '55555555-5555-4555-8555-555555555555', 'active_role', 'admin',            now()),
  ('5555bbbb-0000-4000-8000-000000000055', '55555555-5555-4555-8555-555555555555', 'satker_code', '0100000',          now()),
  ('6666bbbb-0000-4000-8000-000000000006', '66666666-6666-4666-8666-666666666666', 'active_role', 'validator_satker', now()),
  ('6666bbbb-0000-4000-8000-000000000066', '66666666-6666-4666-8666-666666666666', 'satker_code', '0200010',          now()),
  ('7777bbbb-0000-4000-8000-000000000007', '77777777-7777-4777-8777-777777777777', 'active_role', 'approver_satker',  now()),
  ('7777bbbb-0000-4000-8000-000000000077', '77777777-7777-4777-8777-777777777777', 'satker_code', '0200010',          now())
ON CONFLICT (id) DO NOTHING;

COMMIT;
