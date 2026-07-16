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

-- ----------------------------------------------------------------------------
-- 5. perlengkapan: Kebutuhan BMN workflow preconditions (F-E2E E-1).
--    ONE open campaign (periode, status 2000) + per-satker response rows seeded
--    at DISTINCT workflow statuses so each role's transition is exercised
--    independently & deterministically (rather than one brittle long chain).
--    The satker-detail component (/kebutuhan-bmn/satker/:id) gates actions by
--    status_kode: operator submits at 2000/2001; validator_wilayah forwards at
--    2002; validator_pusat decides at 2005.
--      S1 0200010 @2001 Input Barang  -> operator_a: Tambah Barang + Submit ke Wilayah
--      S2 0200020 @2002 →Wilayah      -> validator_wilayah (DKI): Teruskan ke Pusat
--      S3 0300010 @2005 Analisis      -> validator_pusat: Setujui
--    ms_workflow_status(kebutuhan_bmn): 2000 Draft, 2001 Input, 2002 →Wilayah,
--    2003 Revisi, 2004 →Pusat, 2005 Analisis, 2006 Disetujui, 2007 Ditolak.
--    Known UUIDs so specs can deep-link to the satker detail without list-click.
-- ----------------------------------------------------------------------------
INSERT INTO perlengkapan.pengajuan_kebutuhan_bmn
  (id, nama, deskripsi, tahun, tgl_mulai, tgl_selesai, pilihan_satker, status_kode, created_by, scope_satker)
VALUES
  ('c1000000-0000-4c00-8c00-000000000001', 'E2E Periode Kebutuhan BMN 2026', 'Seed F-E2E kebutuhan workflow', 2026, '2026-01-01', '2026-12-31', 'semua', 2000, '44444444-4444-4444-8444-444444444444', 'semua')
ON CONFLICT (id) DO NOTHING;

INSERT INTO perlengkapan.pengajuan_kebutuhan_bmn_satker
  (id, pengajuan_id, satker_id, satker_nama, status_kode, created_by)
VALUES
  ('c1000000-0000-4c00-8c00-0000000a0001', 'c1000000-0000-4c00-8c00-000000000001', '0200010', 'KEJAKSAAN NEGERI JAKARTA PUSAT',   2001, '11111111-1111-4111-8111-111111111111'),
  ('c1000000-0000-4c00-8c00-0000000a0002', 'c1000000-0000-4c00-8c00-000000000001', '0200020', 'KEJAKSAAN NEGERI JAKARTA SELATAN', 2002, '22222222-2222-4222-8222-222222222222'),
  ('c1000000-0000-4c00-8c00-0000000a0003', 'c1000000-0000-4c00-8c00-000000000001', '0300010', 'KEJAKSAAN NEGERI BANDUNG',         2005, '44444444-4444-4444-8444-444444444444')
ON CONFLICT (id) DO NOTHING;

-- One pre-existing barang per satker row so (a) the operator's "Submit ke
-- Wilayah" is valid without depending on the add-barang modal, and (b) the
-- wilayah/pusat detail views are non-empty. Operator ALSO adds one via the UI
-- modal during the test (asserting the count goes 1 -> 2).
INSERT INTO perlengkapan.pengajuan_kebutuhan_bmn_satker_barang
  (id, pengajuan_satker_id, nama, kode_barang, jumlah, satuan, alasan)
VALUES
  ('c1000000-0000-4c00-8c00-00000000b001', 'c1000000-0000-4c00-8c00-0000000a0001', 'E2E Barang Seed A', '3.10.01.02.003', 3, 'Unit', 'Seed justifikasi kebutuhan'),
  ('c1000000-0000-4c00-8c00-00000000b002', 'c1000000-0000-4c00-8c00-0000000a0002', 'E2E Barang Seed B', '3.05.02.01.002', 2, 'Unit', 'Seed justifikasi kebutuhan'),
  ('c1000000-0000-4c00-8c00-00000000b003', 'c1000000-0000-4c00-8c00-0000000a0003', 'E2E Barang Seed C', '3.10.01.05.010', 1, 'Unit', 'Seed justifikasi kebutuhan')
ON CONFLICT (id) DO NOTHING;

-- ----------------------------------------------------------------------------
-- 6. perlengkapan: Pakaian Dinas workflow preconditions (F-E2E E-2).
--    One jenis (master) + one open campaign targeting the DKI + Bandung satkers,
--    with per-satker rows at DISTINCT aktivitas so each validator tier's action
--    is exercised independently. `satker_id` here is the integrasi.mysimkari_satker
--    UUID (auto-generated) → resolved by subquery, NOT hardcoded.
--    aktivitas (ms_aktivitas_bmn 1000-series): 1000 INPUT, 1001 SUBMIT_TO_VALIDATOR
--    (validator_wilayah acts), 1004 SUBMIT_TO_PUSAT (validator_pusat acts), 1008 SELESAI.
--      P1 0200010 @1001 -> validator_wilayah (DKI): Teruskan ke Pusat
--      P2 0200020 @1004 -> validator_pusat: decide
-- ----------------------------------------------------------------------------
INSERT INTO perlengkapan.ms_jenis_pakaian_dinas (id, nama, deskripsi, is_active)
VALUES ('d1000000-0000-4d00-8d00-000000000001', 'PDH E2E', 'Seed F-E2E pakaian dinas', true)
ON CONFLICT (id) DO NOTHING;

INSERT INTO perlengkapan.pengajuan_pakaian_dinas
  (id, nama, tahun, pilihan_satker, scope_satker, jenis_pakaian_dinas_id, aktivitas_id, created_by, tgl_mulai, tgl_selesai)
VALUES
  ('d1000000-0000-4d00-8d00-0000000000c1', 'E2E Pengajuan Pakaian Dinas 2026', 2026, 'semua', 'semua', 'd1000000-0000-4d00-8d00-000000000001', 1000, '44444444-4444-4444-8444-444444444444', '2026-01-01', '2026-12-31')
ON CONFLICT (id) DO NOTHING;

-- NOTE (#94): per-satker rows are NOT seeded. pengajuan_pakaian_dinas_satker.satker_id
-- is `uuid`, but integrasi.mysimkari_satker.id is `bigint` (BIGSERIAL) — the BE join
-- `ON ps.satker_id = s.id` (laporan.rs) is uuid=bigint = invalid SQL, so the pakaian
-- satker workflow (list/forward/rekap) is BROKEN against the real integrasi schema.
-- The E-2 satker workflow e2e is therefore descoped until #94 reconciles the key type.

-- ----------------------------------------------------------------------------
-- 7. perlengkapan: Penghapusan BMN workflow preconditions (F-E2E E-3).
--    Usulan rows at DISTINCT statuses (known UUIDs, deep-linked by the spec) so
--    each role's transition is exercised independently. RBAC visibility is the
--    authoritative MySIMKARI `satker_code` (V003, SatkerScope) — NO mysimkari
--    join, so this module is #94-safe. `satker_id`/`asset_id` are free uuids
--    (no FK).
--    Status model (4000-series): 4000 DRAFT → 4001 SUBMIT_WILAYAH →
--    4003 SUBMIT_PUSAT → 4004 VERIFIKASI_PUSAT → 4005 KONSEP_SK_GENERATED →
--    4006 SK_SIGNED → 4007 COMPLETED.
--      H1 0200010 @4000 -> operator_a: "Ajukan ke Validator Wilayah"
--      H2 0200010 @4001 -> validator_wilayah (DKI): "Teruskan ke Validator Pusat"
--      H3 0200020 @4003 -> validator_pusat: verifikasi (API; FE gap) + Generate SK (UI)
-- ----------------------------------------------------------------------------
INSERT INTO perlengkapan.penghapusan_bmn
  (id, satker_id, asset_id, kode_barang, nama_barang, nup,
   tanggal_penghapusan, alasan, metode_penghapusan, nilai_perolehan,
   status, status_kode, catatan_operator, kewenangan_penetap_sk,
   created_by, satker_code)
VALUES
  ('e1000000-0000-4e00-8e00-0000000a0001', 'e1a00000-0000-4e00-8e00-000000000001', 'e1b00000-0000-4e00-8e00-000000000001', '3.10.01.02.003', 'E2E Laptop Hapus A', 'E2E-H-1',
   '2026-06-01', 'Rusak berat, tidak ekonomis diperbaiki', 'DIMUSNAHKAN', 15000000,
   'DRAFT', 4000, 'Seed F-E2E penghapusan (operator step)', 'PUSAT',
   '11111111-1111-4111-8111-111111111111', '0200010'),
  ('e1000000-0000-4e00-8e00-0000000a0002', 'e1a00000-0000-4e00-8e00-000000000002', 'e1b00000-0000-4e00-8e00-000000000002', '3.10.01.05.010', 'E2E Printer Hapus B', 'E2E-H-2',
   '2026-06-01', 'Rusak berat, biaya perbaikan melebihi nilai', 'DIMUSNAHKAN', 4000000,
   'SUBMIT_WILAYAH', 4001, 'Seed F-E2E penghapusan (wilayah step)', 'PUSAT',
   '11111111-1111-4111-8111-111111111111', '0200010'),
  ('e1000000-0000-4e00-8e00-0000000a0003', 'e1a00000-0000-4e00-8e00-000000000003', 'e1b00000-0000-4e00-8e00-000000000003', '3.05.02.01.002', 'E2E Motor Hapus C', 'E2E-H-3',
   '2026-06-01', 'Usia teknis terlampaui, akan dilelang', 'DIJUAL', 22000000,
   'SUBMIT_PUSAT', 4003, 'Seed F-E2E penghapusan (pusat step)', 'PUSAT',
   '22222222-2222-4222-8222-222222222222', '0200020')
ON CONFLICT (id) DO NOTHING;

-- One item per usulan (Fase 2.8 multi-item; the seed bypasses the create
-- endpoint that would normally insert these, and the detail + generated SK
-- lampiran render from this table).
INSERT INTO perlengkapan.penghapusan_bmn_item
  (id, penghapusan_id, asset_id, kode_barang, nama_barang, nup, nilai_perolehan, kondisi, urutan)
VALUES
  ('e1c00000-0000-4e00-8e00-000000000001', 'e1000000-0000-4e00-8e00-0000000a0001', 'e1b00000-0000-4e00-8e00-000000000001', '3.10.01.02.003', 'E2E Laptop Hapus A', 'E2E-H-1', 15000000, 'RUSAK BERAT', 1),
  ('e1c00000-0000-4e00-8e00-000000000002', 'e1000000-0000-4e00-8e00-0000000a0002', 'e1b00000-0000-4e00-8e00-000000000002', '3.10.01.05.010', 'E2E Printer Hapus B', 'E2E-H-2', 4000000, 'RUSAK BERAT', 1),
  ('e1c00000-0000-4e00-8e00-000000000003', 'e1000000-0000-4e00-8e00-0000000a0003', 'e1b00000-0000-4e00-8e00-000000000003', '3.05.02.01.002', 'E2E Motor Hapus C', 'E2E-H-3', 22000000, 'RUSAK RINGAN', 1)
ON CONFLICT (id) DO NOTHING;

-- ----------------------------------------------------------------------------
-- 8. perlengkapan: Izin Pemakaian BMN precondition (F-E2E E-3).
--    ONE ACTIVE (3004) izin at satker 0200010. Also satker_code-scoped (V003,
--    #94-safe). The FE surface for the satker-internal approval chain
--    (Submitted→SubmittedApproverSatker→Approved: validator-satker-action /
--    approver-satker-action) is NOT wired, and the `validator_satker` /
--    `approver_satker` roles the policy requires don't exist in the authenc
--    seed — so E-3 asserts the LIST/DETAIL render + scoping + the revoke
--    policy rejections (approver-only, admin explicitly blocked), and the
--    missing-role/missing-UI gap is tracked as a finding.
--    jenis_bmn LAPTOP avoids the vehicle/housing CHECK constraints.
-- ----------------------------------------------------------------------------
INSERT INTO perlengkapan.izin_pemakaian_bmn
  (id, nomor_izin, jenis_bmn, bmn_nup, bmn_kode_barang, bmn_nama_barang,
   serial_number, pegawai_nip, pegawai_nama, pegawai_jabatan,
   pegawai_satker_id, pegawai_satker_nama, tanggal_mulai, tanggal_selesai,
   status, status_kode, keperluan, satker_code)
VALUES
  ('e2000000-0000-4e00-8e00-0000000b0001', 'E2E-IZIN-001', 'LAPTOP', 'E2E-A-2', '3.10.01.02.003', 'E2E Laptop Dinas Pinjam',
   'SN-E2E-0001', '200000000000000001', 'E2E Operator Jakpus', 'Operator Satker',
   'e2a00000-0000-4e00-8e00-000000000001', 'KEJAKSAAN NEGERI JAKARTA PUSAT', '2026-01-01', '2026-12-31',
   'ACTIVE', 3004, 'Penunjang tugas kedinasan harian', '0200010')
ON CONFLICT (id) DO NOTHING;

COMMIT;
