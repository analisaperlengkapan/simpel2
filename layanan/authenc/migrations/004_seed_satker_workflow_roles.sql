-- Seed the two satker-internal workflow roles the Pemakaian BMN policy requires.
--
-- `PemakaianBmnPolicy` (layanan/perlengkapan/src/shared/policy.rs) has always
-- gated the satker approval chain on `validator_satker` and `approver_satker`:
--
--   SUBMITTED                 --ValidatorSatkerForward--> SUBMITTED_APPROVER_SATKER
--   SUBMITTED                 --ValidatorSatkerReturn---> REVISI_OPERATOR
--   SUBMITTED_APPROVER_SATKER --ApproverSatkerApprove---> APPROVED
--   SUBMITTED_APPROVER_SATKER --ApproverSatkerReturn----> REVISI_OPERATOR
--   ACTIVE                    --Revoke------------------> REVOKED
--
-- but neither role has ever existed in the IAM seed. `002_seed.sql` creates only
-- admin / operator_satker / validator_wilayah / validator_pusat. So no account
-- could hold them, and every one of those transitions was unreachable by anyone
-- except an admin travelling through the `authorize()` bypass. The e2e fixture
-- recorded this as a known gap rather than a failing test
-- (tests/fixtures/e2e/seed-perlengkapan-workflow.sql, section 8).
--
-- Expand-only and idempotent: two INSERTs guarded by ON CONFLICT, plus their
-- permissions. Nothing existing is altered, so this is safe to re-run and safe
-- to roll back (the app tolerates roles nobody holds — that was the status quo).
--
-- The permission rows mirror the operator_satker grants for pemakaian_bmn,
-- narrowed to the actions each role actually performs. Scope stays own_satker:
-- both roles are satker-internal by definition (validator = the satker's own
-- checker, approver = Pengguna Barang Satker).

INSERT INTO authenc.roles
  (id, name, description, realm_id, composite, client_role, created_at, updated_at, deleted_at)
VALUES
  ('7c2f1a90-6d3e-4b52-9a41-2f8e5c7b1d04', 'validator_satker',
   'Validator Satker Perlengkapan - Verifikasi usulan pemakaian BMN di dalam satuan kerja sebelum diteruskan ke Approver Satker',
   '00000000-0000-0000-0000-000000000000', false, true, now(), now(), NULL),
  ('9e4b3c17-8a05-4d66-b3f2-6c1a9d0e5f38', 'approver_satker',
   'Approver Satker Perlengkapan (Pengguna Barang Satker) - Menyetujui atau mencabut izin pemakaian BMN di satuan kerja',
   '00000000-0000-0000-0000-000000000000', false, true, now(), now(), NULL)
ON CONFLICT (id) DO NOTHING;

INSERT INTO authenc.role_permissions
  (id, role_id, permission, resource, actions, conditions, created_at)
VALUES
  ('3a5d8e21-4f7b-4c09-8d16-b2e7a4c93f50',
   '7c2f1a90-6d3e-4b52-9a41-2f8e5c7b1d04',
   'manage', 'pemakaian_bmn', '{read,forward,return}', '{"scope": "own_satker"}', now()),
  ('6b1c4f83-2d09-4e75-a8c3-1f5b7e2d906a',
   '9e4b3c17-8a05-4d66-b3f2-6c1a9d0e5f38',
   'manage', 'pemakaian_bmn', '{read,approve,return,revoke}', '{"scope": "own_satker"}', now())
ON CONFLICT (id) DO NOTHING;
