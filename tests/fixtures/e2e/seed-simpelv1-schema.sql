-- ============================================================================
-- simpelv1 (legacy Laravel monolith) schema + minimal seed
-- ============================================================================
-- WHY THIS EXISTS
--
-- `dbsimpelv1` is created EMPTY by `infra/helm/simpel/templates/backend/
-- db-create-job.yaml`. Nothing in this repo ever built its schema: the v1
-- schema only ever existed as a production dump (`dbsimpelv1.sql.gz`) that the
-- chart expects as a ConfigMap and that is not in version control. Consequence:
-- every v1 request 500s in every environment built from this repo, so the
-- portal → v1 handoff cannot be exercised at all.
--
-- This file is the version-controlled replacement for that dump. It is NOT the
-- production schema — it is the schema the v1 code paths actually read on the
-- way to a rendered dashboard, which is what the portal↔v1 e2e needs. Tables
-- for modules that stay on v1 and are not on this path (pengadaan, wasdal,
-- pakaian dinas workflow, …) are deliberately absent; add them here when a test
-- actually reaches them.
--
-- SCOPE + IDEMPOTENCY
--
-- Applied by the `e2e-portal-cross-app` job (the "Seed e2e fixtures
-- (dbsimpelv2 + dbsimpelv1)" step), and available to anyone bringing v1 up
-- locally, via `psql --single-transaction`. Every statement is
-- `CREATE ... IF NOT EXISTS` / `INSERT ... ON CONFLICT DO NOTHING` so it is
-- re-runnable against an existing database. No top-level BEGIN/COMMIT — the
-- caller owns the transaction.
--
-- The `siman` schema: v1's Dashboard model reads `siman.siman_aset_*_kl` and
-- `siman.vw_all_asset_siman`, but in the F5-B topology SIMAN data lives in
-- `dbsimpelv2.integrasi.siman_aset`. Those tables are created here as VIEWS
-- over the shared `dbsimpelv2` database via postgres_fdw, so v1 reads real
-- SIMAN data instead of a divergent copy. See section 3.
-- ============================================================================

-- ----------------------------------------------------------------------------
-- 0. Cross-database access to dbsimpelv2 (integrasi = SIMAN SoT)
-- ----------------------------------------------------------------------------
-- Invoke with `-v pw=<postgres password>`; the foreign server needs credentials
-- because the compose/host connection uses scram, not trust.
--
-- `:USER` / `:'pw'` are psql variables, and psql does NOT substitute them
-- inside dollar-quoted bodies — hence the bare statements here rather than the
-- guarded DO block the rest of this file uses. `DROP ... IF EXISTS` provides
-- the idempotency instead.
CREATE EXTENSION IF NOT EXISTS postgres_fdw;

DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_foreign_server WHERE srvname = 'dbsimpelv2_srv') THEN
        EXECUTE 'CREATE SERVER dbsimpelv2_srv FOREIGN DATA WRAPPER postgres_fdw '
             || 'OPTIONS (host ''postgres'', dbname ''dbsimpelv2'', port ''5432'')';
    END IF;
END $$;

DROP USER MAPPING IF EXISTS FOR CURRENT_USER SERVER dbsimpelv2_srv;
CREATE USER MAPPING FOR CURRENT_USER SERVER dbsimpelv2_srv
    OPTIONS (user :'USER', password :'pw');

-- ----------------------------------------------------------------------------
-- 1. Core reference tables
-- ----------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS ms_satker (
    inst_satkerkd   varchar(20) PRIMARY KEY,
    inst_nama       varchar(255),
    inst_satkerinduk varchar(20),
    kdsatker_keu    varchar(30)
);

CREATE TABLE IF NOT EXISTS unit_kerja (
    id            varchar(20) PRIMARY KEY,
    nama          varchar(255),
    akronim       varchar(50),
    eselon_simple varchar(10),
    eselon2       varchar(20)
);

CREATE TABLE IF NOT EXISTS ms_role (
    id   integer PRIMARY KEY,
    name varchar(100)
);

CREATE TABLE IF NOT EXISTS ms_menu (
    id          integer PRIMARY KEY,
    name        varchar(150),
    route       varchar(255),
    -- Remixicon suffix rendered as `<i class="ri-{{ $lv1['icon'] }}">` in
    -- layout/sidebar.blade.php. Only top-level (level 1) menus read it, but the
    -- column is NOT NULL-able in the legacy schema, so every row carries one.
    icon        varchar(100),
    parent      integer,
    level       integer,
    urutan      integer,
    is_active   smallint DEFAULT 1,
    is_active_mobile smallint DEFAULT 1
);

CREATE TABLE IF NOT EXISTS ms_role_menu (
    id      serial PRIMARY KEY,
    role_id integer,
    menu_id integer
);

-- v1's ViewServiceProvider runs on EVERY view render and dereferences
-- `$logo->logo_aplikasi` etc. unconditionally, so a row must exist or every
-- page dies with "Attempt to read property on null".
CREATE TABLE IF NOT EXISTS ms_setting_qr (
    id                 integer PRIMARY KEY,
    logo_aplikasi      text,
    logo_aplikasi_dark text,
    logo_kecil         text,
    nama_aplikasi      varchar(150),
    copyright          varchar(255)
);

-- ----------------------------------------------------------------------------
-- 2. Identity
-- ----------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS users (
    id                 bigserial PRIMARY KEY,
    name               varchar(255),
    username           varchar(100) UNIQUE,
    email              varchar(255),
    password           varchar(255),
    ms_satker_id       varchar(20),
    satker             varchar(255),
    ms_satker_pusat_id varchar(20),
    satker_pusat       varchar(255),
    pangkat            varchar(100),
    jabatan            varchar(255),
    foto               text,
    is_superadmin      smallint DEFAULT 0,
    firebase_token     text,
    has_user           smallint DEFAULT 0,
    has_review         smallint DEFAULT 0,
    google2fa_secret   varchar(255),
    remember_token     varchar(100),
    created_at         timestamp,
    updated_at         timestamp
);

CREATE TABLE IF NOT EXISTS user_role (
    id                 serial PRIMARY KEY,
    user_id            bigint,
    ms_role_id         integer,
    ms_satker_id       varchar(20),
    ms_satker_pusat_id varchar(20),
    ms_satker_id_keu   varchar(30)
);

-- Login writes a row here (AuthController + oauthCallback, both LOGIN and
-- LOGIN_GAGAL paths) and the write is not optional: without the table, login
-- fails after the credentials already checked out.
CREATE TABLE IF NOT EXISTS user_log (
    id                 bigserial PRIMARY KEY,
    username           varchar(100),
    operation          varchar(50),
    "table"            varchar(100),
    ms_satker_id       varchar(20),
    ms_satker_pusat_id varchar(20),
    pkey               bigint,
    keterangan         text,
    ip_address         varchar(64),
    user_agent         text,
    created_at         timestamp,
    updated_at         timestamp
);

-- ----------------------------------------------------------------------------
-- 3. SIMAN asset views (foreign tables over dbsimpelv2.integrasi)
-- ----------------------------------------------------------------------------
-- v1's Dashboard model asks `siman.siman_aset_<jenis>_kl` for columns the
-- unified `integrasi.siman_aset` does not carry: `kelompok`, `sub_kelompok`,
-- `status_penggunaan`, `luas_tanah_*`, and `nilai_buku` in the *_kl variants.
-- The unified table supplies `jenis_aset`, `kategori_aset`, `ur_sskel`,
-- `ur_kondisi`, `rph_aset`, `tgl_perlh`, `kdsatker_keu`. Map what exists and
-- derive the rest so the dashboard renders real numbers.
--
-- `jenis_aset` values in integrasi.siman_aset follow the SIMAN category names
-- ("Tanah", "Gedung Bangunan", "Alat Angkutan Bermotor", …); see
-- tests/fixtures/e2e/seed-multisatker.sql.

CREATE SCHEMA IF NOT EXISTS siman;

-- `integrasi.siman_aset` lives in dbsimpelv2, so it must be reached through a
-- foreign table — plain SQL cannot cross databases. Import its columns
-- explicitly (rather than IMPORT FOREIGN SCHEMA) so this fixture keeps working
-- when integrasi adds columns.
-- CASCADE, not a bare DROP: the `siman_aset_<jenis>_kl` views and
-- `vw_all_asset_siman` below depend on this table, so re-running the fixture
-- would otherwise abort here with "other objects depend on it" — and aborting
-- is invisible to a fresh database, which is why the first apply always passed.
-- The dependent views are recreated (CREATE OR REPLACE) further down, so
-- dropping them here loses nothing.
DROP FOREIGN TABLE IF EXISTS siman.siman_aset CASCADE;
CREATE FOREIGN TABLE siman.siman_aset (
    id           bigint,
    kdsatker_keu varchar(30),
    kd_brg       varchar(50),
    ur_sskel     varchar(255),
    kategori_aset varchar(255),
    ur_kondisi   varchar(100),
    tgl_perlh    date,
    rph_aset     numeric,
    jenis_aset   varchar(100)
) SERVER dbsimpelv2_srv OPTIONS (schema_name 'integrasi', table_name 'siman_aset');

-- One view per jenis so each `siman_aset_<x>_kl` below is a thin filter
-- rather than a repeated CASE ladder.
DO $$
DECLARE
    spec record;
BEGIN
    FOR spec IN
        SELECT * FROM (VALUES
            ('siman_aset_tanah_kl',            'Tanah'),
            ('siman_aset_gedung_bangunan_kl',  'Gedung Bangunan'),
            ('siman_aset_alat_angkutan_kl',    'Alat Angkutan Bermotor'),
            ('siman_aset_alat_berat_kl',       'Alat Besar'),
            ('siman_aset_pm_tik_kl',           'Peralatan Mesin Khusus TIK'),
            ('siman_aset_pm_nontik_kl',        'Peralatan Mesin Non TIK'),
            ('siman_aset_jalan_jembatan_kl',   'Jalan dan Jembatan'),
            ('siman_aset_bangunan_air_kl',     'Bangunan Air'),
            ('siman_aset_instalasi_jaringan_kl','Instalasi Jaringan'),
            ('siman_aset_kdp_kl',              'Konstruksi Dalam Pengerjaan'),
            ('siman_aset_rumah_negara_kl',     'Rumah Negara'),
            ('siman_aset_tak_berwujud_kl',     'Aset Tak Berwujud'),
            ('siman_aset_tetap_lainnya_kl',    'Aset Tetap Lainnya'),
            ('siman_aset_renovasi_kl',         'Aset Tetap Renovasi'),
            ('siman_aset_persediaan_kl',       'Persediaan'),
            ('siman_aset_rusak_berat_kl',      'Aset Rusak Berat')
        ) AS v(view_name, jenis)
    LOOP
        EXECUTE format($f$
            CREATE OR REPLACE VIEW siman.%I AS
            SELECT
                a.id::bigint                                   AS id,
                a.kdsatker_keu                                 AS id_satker_keu,
                a.kd_brg                                       AS kode_barang,
                a.ur_sskel                                     AS sub_kelompok,
                a.kategori_aset                                AS kelompok,
                a.ur_kondisi                                   AS kondisi,
                a.tgl_perlh                                    AS tanggal_perolehan,
                a.tgl_perlh                                    AS tgl_perolehan,
                COALESCE(a.rph_aset, 0)::numeric               AS nilai_perolehan,
                COALESCE(a.rph_aset, 0)::numeric               AS nilai_buku,
                NULL::varchar                                  AS no_psp,
                NULL::varchar                                  AS status_penggunaan,
                0::numeric                                     AS luas_bangunan,
                0::numeric                                     AS luas_tanah_kosong,
                0::numeric                                     AS luas_tanah_seluruhnya,
                0::numeric                                     AS luas_tanah_untuk_bangunan
            FROM siman.siman_aset a
            WHERE a.jenis_aset = %L
        $f$, spec.view_name, spec.jenis);
    END LOOP;
END $$;

-- `vw_all_asset_siman` — v1 reads PSP + acquisition-year charts from it.
CREATE OR REPLACE VIEW siman.vw_all_asset_siman AS
SELECT
    a.id::bigint                     AS id,
    a.kdsatker_keu                   AS id_satker_keu,
    a.kd_brg                         AS kode_barang,
    a.tgl_perlh                      AS tanggal_perolehan,
    NULL::varchar                    AS no_psp
FROM siman.siman_aset a;

-- ----------------------------------------------------------------------------
-- 4. Dashboard support tables
-- ----------------------------------------------------------------------------
-- v1 aggregates the whole personnel roster; the per-satker detail comes from
-- integrasi. Empty is correct here — the dashboard renders a zeroed chart,
-- which is what an un-synced environment should show.
CREATE TABLE IF NOT EXISTS vw_pegawai_dashboard (
    inst_satkerkd   varchar(20),
    inst_satkerkd_keu varchar(30),
    jml_laki        integer DEFAULT 0,
    jml_perempuan   integer DEFAULT 0,
    jml_jaksa       integer DEFAULT 0,
    jml_tu          integer DEFAULT 0,
    total           integer DEFAULT 0
);

-- Pegawai lookup used by both the password login (`checkPegawaiExist`) and the
-- OAuth callback (`Master::getPegawaiByNip`). v1 reads `mv_curr_pegawai_all`
-- with columns that do not exist in integrasi.mysimkari_pegawai, so this is a
-- local table rather than a foreign table.
CREATE TABLE IF NOT EXISTS mv_curr_pegawai_all (
    peg_nip_baru        varchar(30) PRIMARY KEY,
    nama                varchar(255),
    pns_mail            varchar(255),
    pangkat             varchar(100),
    jabatan             varchar(255),
    inst_satkerkd       varchar(20),
    satker              varchar(255),
    unitkerja_kd        varchar(20),
    foto                text
);

CREATE TABLE IF NOT EXISTS vw_notifikasi (
    id              bigserial PRIMARY KEY,
    target_username varchar(100),
    is_read         smallint DEFAULT 0,
    judul           varchar(255),
    pesan           text,
    created_at      timestamp
);

CREATE TABLE IF NOT EXISTS notifikasi_manual (
    id        bigserial PRIMARY KEY,
    judul     varchar(255),
    pesan     text,
    is_active smallint DEFAULT 1
);

CREATE TABLE IF NOT EXISTS notifikasi_manual_target (
    id                   bigserial PRIMARY KEY,
    notifikasi_manual_id bigint,
    role_id              integer
);

-- ----------------------------------------------------------------------------
-- 5. Seed
-- ----------------------------------------------------------------------------
-- Satker: a Kejagung entry (drives `getWilayahDashboard`), two Kejati (length-2
-- codes) and three Kejari, each carrying the `kdsatker_keu` the dashboard
-- scopes on. These mirror the codes used by tests/fixtures/e2e/seed-multisatker.sql
-- so a v1 dashboard scoped to a satker shows that satker's assets.
INSERT INTO ms_satker (inst_satkerkd, inst_nama, inst_satkerinduk, kdsatker_keu) VALUES
    ('00',      'KEJAKSAAN AGUNG REPUBLIK INDONESIA', NULL,   '006010000000000KP'),
    ('01',      'KEJAKSAAN TINGGI DKI JAKARTA',       '00',   '006010100000000KP'),
    ('02',      'KEJAKSAAN TINGGI JAWA BARAT',        '00',   '006010200000000KP'),
    ('0100000', 'KEJAKSAAN AGUNG',                    '00',   '006010000000000KP'),
    ('0200010', 'KEJAKSAAN NEGERI JAKARTA PUSAT',     '01',   '006019999010001KD'),
    ('0200020', 'KEJAKSAAN NEGERI JAKARTA SELATAN',   '01',   '006019999020001KD'),
    ('0300010', 'KEJAKSAAN NEGERI BANDUNG',           '02',   '006018888010001KD')
ON CONFLICT (inst_satkerkd) DO NOTHING;

INSERT INTO unit_kerja (id, nama, akronim, eselon_simple, eselon2) VALUES
    ('00', 'KEJAKSAAN AGUNG', 'KEJAGUNG', '1', NULL),
    ('01', 'BIRO UMUM',       'BIROUM',   '3', '00')
ON CONFLICT (id) DO NOTHING;

-- Role 1 = pelaksana satker (`config('constants.pelaksana_satker_role_id')`),
-- the role every auto-provisioned user gets.
INSERT INTO ms_role (id, name) VALUES
    (1, 'Pelaksana Satker'),
    (2, 'Pegawai'),
    (3, 'Superadmin'),
    (20, 'Validator Wilayah'),
    (22, 'Validator Pusat'),
    (27, 'Admin Biro Lengkap')
ON CONFLICT (id) DO NOTHING;

-- One menu per role so `Master::getMenus()` returns something for the seeded
-- user and the layout renders a sidebar.
INSERT INTO ms_menu (id, name, route, icon, parent, level, urutan) VALUES
    (1, 'Dashboard', '/dashboard', 'dashboard-2-line', NULL, 1, 1)
ON CONFLICT (id) DO NOTHING;

INSERT INTO ms_role_menu (role_id, menu_id) VALUES (1, 1)
ON CONFLICT DO NOTHING;

INSERT INTO ms_setting_qr (id, logo_aplikasi, logo_aplikasi_dark, logo_kecil, nama_aplikasi, copyright)
VALUES (1, '/images/logo.png', '/images/logo-dark.png', '/images/logo-small.png', 'SIMPEL', 'Kejaksaan RI')
ON CONFLICT (id) DO NOTHING;

-- Personnel roster. `200000000000000001` is the same NIP the authenc fixture
-- seeds; the OAuth callback keys the v1 row on the token's `nip` claim (see
-- AuthController::oauthCallback), so this is the row a fixture login resolves to.
INSERT INTO mv_curr_pegawai_all (peg_nip_baru, nama, pns_mail, pangkat, jabatan, inst_satkerkd, satker, unitkerja_kd, foto) VALUES
    ('200000000000000001', 'E2E Operator Jakpus',  'op.jakpus@kejaksaan.go.id',  'Penata Muda',      'Operator Satker',     '0200010', 'KEJAKSAAN NEGERI JAKARTA PUSAT',   '01', NULL),
    ('200000000000000002', 'E2E Operator Jaksel',  'op.jaksel@kejaksaan.go.id',  'Penata Muda',      'Operator Satker',     '0200020', 'KEJAKSAAN NEGERI JAKARTA SELATAN', '01', NULL),
    ('200000000000000005', 'E2E Admin Pusat',      'admin@kejaksaan.go.id',      'Pembina',          'Administrator',       '0100000', 'KEJAKSAAN AGUNG',                  '00', NULL)
ON CONFLICT (peg_nip_baru) DO NOTHING;

-- ----------------------------------------------------------------------------
-- NOTE on `vw_pegawai_dashboard`
-- ----------------------------------------------------------------------------
-- It is intentionally left as an empty local table (section 4). v1's personnel
-- dashboard is not on the portal↔v1 handoff path, and integrasi's employee
-- table has a different shape; mapping it is a separate piece of work.
