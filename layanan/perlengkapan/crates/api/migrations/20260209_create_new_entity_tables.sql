-- ============================================================================
-- Migration: Create New Entity Tables
-- Description: Tables for roadmap, mapping, fulfillment, parallel approvals, and pemakaian
-- Author: SIMPEL Team
-- Created: 2026-02-09
-- Requirements: REQ-K008, REQ-M007, REQ-K009, REQ-P001
-- ============================================================================

-- Ensure schema exists
CREATE SCHEMA IF NOT EXISTS perlengkapan;

-- Enable required extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pg_trgm";

-- ============================================================================
-- ROADMAP SARPRAS (5-Year Infrastructure Roadmap)
-- Requirement: REQ-K008
-- ============================================================================

CREATE TABLE IF NOT EXISTS perlengkapan.roadmap_sarpras (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- Satker Reference (from MySIMKARI via Authenc)
    satker_id UUID NOT NULL,

    -- Period (5-year planning period)
    periode_mulai INTEGER NOT NULL,
    periode_akhir INTEGER NOT NULL,

    -- Asset Information
    kode_barang VARCHAR(50) NOT NULL,
    nama_barang VARCHAR(255) NOT NULL,
    tahun_rencana INTEGER NOT NULL,

    -- Quantity Planning
    jumlah_kebutuhan INTEGER NOT NULL CHECK (jumlah_kebutuhan > 0),
    jumlah_terpenuhi INTEGER DEFAULT 0 CHECK (jumlah_terpenuhi >= 0),

    -- Budget Planning
    estimasi_anggaran NUMERIC(15,2),
    realisasi_anggaran NUMERIC(15,2) DEFAULT 0,

    -- Status
    status_pemenuhan VARCHAR(50) DEFAULT 'PLANNED' CHECK (status_pemenuhan IN ('PLANNED', 'IN_PROGRESS', 'COMPLETED', 'CANCELLED')),

    -- Additional Information
    prioritas INTEGER DEFAULT 0,
    keterangan TEXT,

    -- Audit Trail
    created_by UUID,
    updated_by UUID,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),

    -- Constraints
    CONSTRAINT chk_tahun_in_periode CHECK (tahun_rencana >= periode_mulai AND tahun_rencana <= periode_akhir),
    CONSTRAINT chk_periode_valid CHECK (periode_akhir = periode_mulai + 4), -- 5-year period
    CONSTRAINT chk_jumlah_valid CHECK (jumlah_terpenuhi <= jumlah_kebutuhan)
);

-- Indexes for roadmap_sarpras
CREATE INDEX idx_roadmap_satker ON perlengkapan.roadmap_sarpras(satker_id);
CREATE INDEX idx_roadmap_periode ON perlengkapan.roadmap_sarpras(periode_mulai, periode_akhir);
CREATE INDEX idx_roadmap_tahun ON perlengkapan.roadmap_sarpras(tahun_rencana);
CREATE INDEX idx_roadmap_kode_barang ON perlengkapan.roadmap_sarpras(kode_barang);
CREATE INDEX idx_roadmap_status ON perlengkapan.roadmap_sarpras(status_pemenuhan);
CREATE INDEX idx_roadmap_satker_tahun ON perlengkapan.roadmap_sarpras(satker_id, tahun_rencana);

COMMENT ON TABLE perlengkapan.roadmap_sarpras IS '5-year infrastructure and facilities roadmap planning';
COMMENT ON COLUMN perlengkapan.roadmap_sarpras.periode_mulai IS 'Start year of 5-year period (e.g., 2025)';
COMMENT ON COLUMN perlengkapan.roadmap_sarpras.periode_akhir IS 'End year of 5-year period (e.g., 2029)';
COMMENT ON COLUMN perlengkapan.roadmap_sarpras.tahun_rencana IS 'Specific year within the period for this item';

-- ============================================================================
-- MAPPING KODEFIKASI (Non-Standard Code Mapping)
-- Requirement: REQ-M007
-- ============================================================================

CREATE TABLE IF NOT EXISTS perlengkapan.mapping_kodefikasi (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- Satker Reference
    satker_id UUID NOT NULL,

    -- Old/Non-Standard Code
    kode_barang_lama VARCHAR(50) NOT NULL,
    nama_barang_lama VARCHAR(255) NOT NULL,

    -- Proposed Standard Code
    kode_barang_baru VARCHAR(50),
    nama_barang_baru VARCHAR(255),
    kode_barang_baru_id UUID, -- Reference to ms_barang if exists

    -- Mapping Status
    status_mapping VARCHAR(50) NOT NULL DEFAULT 'PROPOSED' CHECK (status_mapping IN ('PROPOSED', 'VERIFIED', 'APPROVED', 'REJECTED')),

    -- Justification
    catatan_mapping TEXT,
    alasan_penolakan TEXT,

    -- Verification
    verified_by UUID,
    verified_at TIMESTAMPTZ,
    approved_by UUID,
    approved_at TIMESTAMPTZ,

    -- Audit Trail
    created_by UUID,
    updated_by UUID,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Indexes for mapping_kodefikasi
CREATE INDEX idx_mapping_satker ON perlengkapan.mapping_kodefikasi(satker_id);
CREATE INDEX idx_mapping_kode_lama ON perlengkapan.mapping_kodefikasi(kode_barang_lama);
CREATE INDEX idx_mapping_kode_baru ON perlengkapan.mapping_kodefikasi(kode_barang_baru);
CREATE INDEX idx_mapping_status ON perlengkapan.mapping_kodefikasi(status_mapping);
CREATE INDEX idx_mapping_satker_status ON perlengkapan.mapping_kodefikasi(satker_id, status_mapping);

COMMENT ON TABLE perlengkapan.mapping_kodefikasi IS 'Mapping of non-standard asset codes to standard SIMAK BMN codes';
COMMENT ON COLUMN perlengkapan.mapping_kodefikasi.kode_barang_lama IS 'Non-standard code from satker';
COMMENT ON COLUMN perlengkapan.mapping_kodefikasi.kode_barang_baru IS 'Standard SIMAK BMN code';

-- ============================================================================
-- RIWAYAT PEMENUHAN (Fulfillment History)
-- Requirement: REQ-K009
-- ============================================================================

CREATE TABLE IF NOT EXISTS perlengkapan.riwayat_pemenuhan (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- Reference to Source
    kebutuhan_bmn_id UUID REFERENCES perlengkapan.pengajuan_kebutuhan_bmn(id) ON DELETE SET NULL,
    roadmap_id UUID REFERENCES perlengkapan.roadmap_sarpras(id) ON DELETE SET NULL,

    -- Satker Reference
    satker_id UUID NOT NULL,

    -- Asset Information
    tahun_anggaran INTEGER NOT NULL,
    kode_barang VARCHAR(50) NOT NULL,
    nama_barang VARCHAR(255) NOT NULL,

    -- Fulfillment Details
    jumlah_terpenuhi INTEGER NOT NULL CHECK (jumlah_terpenuhi > 0),
    nilai_perolehan NUMERIC(15,2),

    -- Source of Fulfillment
    sumber_data VARCHAR(50) NOT NULL CHECK (sumber_data IN ('SIMAN', 'HIBAH', 'PNBP', 'APBN', 'APBD', 'LAINNYA')),
    sumber_keterangan TEXT,

    -- Date Information
    tanggal_pemenuhan DATE NOT NULL,

    -- Document Reference
    nomor_dokumen VARCHAR(100),
    file_dokumen JSONB DEFAULT '[]'::jsonb,

    -- Audit Trail
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Indexes for riwayat_pemenuhan
CREATE INDEX idx_riwayat_kebutuhan ON perlengkapan.riwayat_pemenuhan(kebutuhan_bmn_id);
CREATE INDEX idx_riwayat_roadmap ON perlengkapan.riwayat_pemenuhan(roadmap_id);
CREATE INDEX idx_riwayat_satker ON perlengkapan.riwayat_pemenuhan(satker_id);
CREATE INDEX idx_riwayat_tahun ON perlengkapan.riwayat_pemenuhan(tahun_anggaran);
CREATE INDEX idx_riwayat_kode_barang ON perlengkapan.riwayat_pemenuhan(kode_barang);
CREATE INDEX idx_riwayat_sumber ON perlengkapan.riwayat_pemenuhan(sumber_data);
CREATE INDEX idx_riwayat_tanggal ON perlengkapan.riwayat_pemenuhan(tanggal_pemenuhan DESC);
CREATE INDEX idx_riwayat_satker_tahun ON perlengkapan.riwayat_pemenuhan(satker_id, tahun_anggaran);

COMMENT ON TABLE perlengkapan.riwayat_pemenuhan IS 'History of BMN requirement fulfillment from various sources';
COMMENT ON COLUMN perlengkapan.riwayat_pemenuhan.sumber_data IS 'Source: SIMAN (procurement), HIBAH (donation), PNBP, APBN, APBD, LAINNYA';

-- ============================================================================
-- PARALLEL APPROVALS (Parallel Approval Workflow)
-- Requirement: REQ-W007
-- ============================================================================

CREATE TABLE IF NOT EXISTS perlengkapan.parallel_approvals (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- Reference to Entity
    entity_type VARCHAR(50) NOT NULL CHECK (entity_type IN ('kebutuhan_bmn', 'pakaian_dinas', 'pemakaian_bmn')),
    entity_id UUID NOT NULL,

    -- Approval Configuration
    approval_step VARCHAR(100) NOT NULL,
    required_approvers INTEGER NOT NULL CHECK (required_approvers > 0),
    approval_threshold INTEGER NOT NULL CHECK (approval_threshold > 0 AND approval_threshold <= required_approvers),

    -- Status
    status VARCHAR(50) NOT NULL DEFAULT 'PENDING' CHECK (status IN ('PENDING', 'APPROVED', 'REJECTED', 'CANCELLED')),

    -- Voting Results
    total_votes INTEGER DEFAULT 0,
    approve_votes INTEGER DEFAULT 0,
    reject_votes INTEGER DEFAULT 0,

    -- Timestamps
    started_at TIMESTAMPTZ DEFAULT NOW(),
    completed_at TIMESTAMPTZ,
    expires_at TIMESTAMPTZ,

    -- Audit Trail
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),

    CONSTRAINT chk_threshold_valid CHECK (approval_threshold <= required_approvers)
);

-- Indexes for parallel_approvals
CREATE INDEX idx_parallel_entity ON perlengkapan.parallel_approvals(entity_type, entity_id);
CREATE INDEX idx_parallel_status ON perlengkapan.parallel_approvals(status);
CREATE INDEX idx_parallel_expires ON perlengkapan.parallel_approvals(expires_at) WHERE status = 'PENDING';

COMMENT ON TABLE perlengkapan.parallel_approvals IS 'Parallel approval workflow tracking';
COMMENT ON COLUMN perlengkapan.parallel_approvals.approval_threshold IS 'Number of approvals needed to pass (e.g., 2 out of 3)';

-- ============================================================================
-- PARALLEL APPROVAL VOTES (Individual Votes)
-- Requirement: REQ-W007
-- ============================================================================

CREATE TABLE IF NOT EXISTS perlengkapan.parallel_approval_votes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- Reference to Parallel Approval
    parallel_approval_id UUID NOT NULL REFERENCES perlengkapan.parallel_approvals(id) ON DELETE CASCADE,

    -- Approver Information
    approver_user_id UUID NOT NULL,
    approver_nip VARCHAR(30),
    approver_nama VARCHAR(255),
    approver_jabatan VARCHAR(255),
    approver_role VARCHAR(100),

    -- Vote
    vote VARCHAR(20) NOT NULL CHECK (vote IN ('APPROVE', 'REJECT', 'ABSTAIN')),
    komentar TEXT,

    -- Timestamp
    voted_at TIMESTAMPTZ DEFAULT NOW(),

    -- Unique constraint: one vote per approver per approval
    CONSTRAINT unique_approver_vote UNIQUE (parallel_approval_id, approver_user_id)
);

-- Indexes for parallel_approval_votes
CREATE INDEX idx_parallel_votes_approval ON perlengkapan.parallel_approval_votes(parallel_approval_id);
CREATE INDEX idx_parallel_votes_approver ON perlengkapan.parallel_approval_votes(approver_user_id);
CREATE INDEX idx_parallel_votes_vote ON perlengkapan.parallel_approval_votes(vote);

COMMENT ON TABLE perlengkapan.parallel_approval_votes IS 'Individual votes in parallel approval process';

-- ============================================================================
-- IZIN PEMAKAIAN BMN (BMN Usage Permits)
-- Requirement: REQ-P001
-- ============================================================================

CREATE TABLE IF NOT EXISTS perlengkapan.izin_pemakaian_bmn (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- Permit Information
    nomor_izin VARCHAR(100) UNIQUE NOT NULL,
    jenis_bmn VARCHAR(50) NOT NULL CHECK (jenis_bmn IN ('KENDARAAN', 'RUMAH_DINAS', 'LAPTOP', 'LAINNYA')),

    -- BMN Reference (from SIMAN)
    bmn_nup VARCHAR(50) NOT NULL,
    bmn_kode_barang VARCHAR(50) NOT NULL,
    bmn_nama_barang VARCHAR(255) NOT NULL,
    bmn_merk VARCHAR(100),
    bmn_tipe VARCHAR(100),
    bmn_tahun_perolehan INTEGER,

    -- Vehicle-specific fields (if jenis_bmn = 'KENDARAAN')
    no_polisi VARCHAR(20),
    no_bpkb VARCHAR(50),
    no_rangka VARCHAR(50),
    no_mesin VARCHAR(50),

    -- Housing-specific fields (if jenis_bmn = 'RUMAH_DINAS')
    alamat_rumah TEXT,
    luas_tanah NUMERIC(10,2),
    luas_bangunan NUMERIC(10,2),

    -- Laptop-specific fields (if jenis_bmn = 'LAPTOP')
    serial_number VARCHAR(100),
    spesifikasi TEXT,

    -- User/Pegawai Information (from MySIMKARI)
    pegawai_nip VARCHAR(30) NOT NULL,
    pegawai_nama VARCHAR(255) NOT NULL,
    pegawai_jabatan VARCHAR(255),
    pegawai_satker_id UUID NOT NULL,
    pegawai_satker_nama VARCHAR(255),

    -- Permit Period
    tanggal_mulai DATE NOT NULL,
    tanggal_selesai DATE NOT NULL,

    -- Status
    status VARCHAR(50) NOT NULL DEFAULT 'DRAFT' CHECK (status IN ('DRAFT', 'SUBMITTED', 'APPROVED', 'REJECTED', 'ACTIVE', 'EXPIRED', 'REVOKED')),

    -- Workflow Status (FK to ms_aktivitas_bmn.kode)
    status_kode INTEGER REFERENCES perlengkapan.ms_aktivitas_bmn(kode),

    -- Approval Information
    approved_by UUID,
    approved_at TIMESTAMPTZ,
    rejection_reason TEXT,

    -- Revocation
    revoked_by UUID,
    revoked_at TIMESTAMPTZ,
    revocation_reason TEXT,

    -- Renewal
    is_renewal BOOLEAN DEFAULT FALSE,
    previous_permit_id UUID REFERENCES perlengkapan.izin_pemakaian_bmn(id) ON DELETE SET NULL,

    -- Supporting Documents
    file_pendukung JSONB DEFAULT '[]'::jsonb,

    -- Additional Information
    tujuan_pemakaian TEXT,
    keterangan TEXT,

    -- Audit Trail
    created_by UUID,
    updated_by UUID,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),

    -- Constraints
    CONSTRAINT chk_tanggal_valid CHECK (tanggal_selesai >= tanggal_mulai),
    CONSTRAINT chk_vehicle_fields CHECK (
        jenis_bmn != 'KENDARAAN' OR
        (no_polisi IS NOT NULL AND no_bpkb IS NOT NULL)
    ),
    CONSTRAINT chk_housing_fields CHECK (
        jenis_bmn != 'RUMAH_DINAS' OR
        (alamat_rumah IS NOT NULL)
    )
);

-- Indexes for izin_pemakaian_bmn
CREATE INDEX idx_izin_nomor ON perlengkapan.izin_pemakaian_bmn(nomor_izin);
CREATE INDEX idx_izin_jenis ON perlengkapan.izin_pemakaian_bmn(jenis_bmn);
CREATE INDEX idx_izin_nup ON perlengkapan.izin_pemakaian_bmn(bmn_nup);
CREATE INDEX idx_izin_pegawai_nip ON perlengkapan.izin_pemakaian_bmn(pegawai_nip);
CREATE INDEX idx_izin_satker ON perlengkapan.izin_pemakaian_bmn(pegawai_satker_id);
CREATE INDEX idx_izin_status ON perlengkapan.izin_pemakaian_bmn(status);
CREATE INDEX idx_izin_tanggal_selesai ON perlengkapan.izin_pemakaian_bmn(tanggal_selesai) WHERE status = 'ACTIVE';
CREATE INDEX idx_izin_no_polisi ON perlengkapan.izin_pemakaian_bmn(no_polisi) WHERE no_polisi IS NOT NULL;
CREATE INDEX idx_izin_pegawai_status ON perlengkapan.izin_pemakaian_bmn(pegawai_nip, status);
CREATE INDEX idx_izin_nup_status ON perlengkapan.izin_pemakaian_bmn(bmn_nup, status);

-- Full-text search index for permit search
CREATE INDEX idx_izin_nama_barang_trgm ON perlengkapan.izin_pemakaian_bmn USING gin(bmn_nama_barang gin_trgm_ops);
CREATE INDEX idx_izin_pegawai_nama_trgm ON perlengkapan.izin_pemakaian_bmn USING gin(pegawai_nama gin_trgm_ops);

COMMENT ON TABLE perlengkapan.izin_pemakaian_bmn IS 'BMN usage permits for vehicles, housing, laptops, etc.';
COMMENT ON COLUMN perlengkapan.izin_pemakaian_bmn.nomor_izin IS 'Auto-generated permit number';
COMMENT ON COLUMN perlengkapan.izin_pemakaian_bmn.bmn_nup IS 'NUP (Nomor Urut Pendaftaran) from SIMAN';
COMMENT ON COLUMN perlengkapan.izin_pemakaian_bmn.is_renewal IS 'TRUE if this is a renewal of previous permit';

-- ============================================================================
-- IZIN PEMAKAIAN BMN AKTIVITAS (Workflow History for Permits)
-- ============================================================================

CREATE TABLE IF NOT EXISTS perlengkapan.izin_pemakaian_bmn_aktivitas (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- Reference to Permit
    izin_pemakaian_id UUID NOT NULL REFERENCES perlengkapan.izin_pemakaian_bmn(id) ON DELETE CASCADE,

    -- Workflow Transition
    from_status VARCHAR(50),
    to_status VARCHAR(50) NOT NULL,
    from_status_kode INTEGER,
    to_status_kode INTEGER REFERENCES perlengkapan.ms_aktivitas_bmn(kode),

    -- Actor Information (from Authenc)
    user_id UUID,
    nip VARCHAR(30),
    nama VARCHAR(255),
    pangkat VARCHAR(100),
    jabatan VARCHAR(255),
    role VARCHAR(100),

    -- Action Details
    aksi VARCHAR(50) NOT NULL,
    komentar TEXT,

    -- Timestamp
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Indexes for izin_pemakaian_bmn_aktivitas
CREATE INDEX idx_izin_aktivitas_izin ON perlengkapan.izin_pemakaian_bmn_aktivitas(izin_pemakaian_id);
CREATE INDEX idx_izin_aktivitas_user ON perlengkapan.izin_pemakaian_bmn_aktivitas(user_id);
CREATE INDEX idx_izin_aktivitas_created ON perlengkapan.izin_pemakaian_bmn_aktivitas(created_at DESC);

COMMENT ON TABLE perlengkapan.izin_pemakaian_bmn_aktivitas IS 'Workflow history audit trail for BMN usage permits';

-- ============================================================================
-- TRIGGERS FOR UPDATED_AT
-- ============================================================================

CREATE OR REPLACE FUNCTION perlengkapan.update_entity_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Apply triggers to all new tables
CREATE TRIGGER trg_roadmap_updated_at
    BEFORE UPDATE ON perlengkapan.roadmap_sarpras
    FOR EACH ROW EXECUTE FUNCTION perlengkapan.update_entity_updated_at();

CREATE TRIGGER trg_mapping_updated_at
    BEFORE UPDATE ON perlengkapan.mapping_kodefikasi
    FOR EACH ROW EXECUTE FUNCTION perlengkapan.update_entity_updated_at();

CREATE TRIGGER trg_riwayat_updated_at
    BEFORE UPDATE ON perlengkapan.riwayat_pemenuhan
    FOR EACH ROW EXECUTE FUNCTION perlengkapan.update_entity_updated_at();

CREATE TRIGGER trg_parallel_approvals_updated_at
    BEFORE UPDATE ON perlengkapan.parallel_approvals
    FOR EACH ROW EXECUTE FUNCTION perlengkapan.update_entity_updated_at();

CREATE TRIGGER trg_izin_pemakaian_updated_at
    BEFORE UPDATE ON perlengkapan.izin_pemakaian_bmn
    FOR EACH ROW EXECUTE FUNCTION perlengkapan.update_entity_updated_at();

-- ============================================================================
-- VIEWS FOR REPORTING
-- ============================================================================

-- View: Active BMN Usage Permits
CREATE OR REPLACE VIEW perlengkapan.v_izin_pemakaian_aktif AS
SELECT
    i.id,
    i.nomor_izin,
    i.jenis_bmn,
    i.bmn_nup,
    i.bmn_nama_barang,
    i.no_polisi,
    i.pegawai_nip,
    i.pegawai_nama,
    i.pegawai_jabatan,
    i.pegawai_satker_nama,
    i.tanggal_mulai,
    i.tanggal_selesai,
    i.status,
    (i.tanggal_selesai - CURRENT_DATE) as days_until_expiry,
    CASE
        WHEN i.tanggal_selesai < CURRENT_DATE THEN 'EXPIRED'
        WHEN i.tanggal_selesai < CURRENT_DATE + INTERVAL '30 days' THEN 'EXPIRING_SOON'
        ELSE 'ACTIVE'
    END as expiry_status
FROM perlengkapan.izin_pemakaian_bmn i
WHERE i.status = 'ACTIVE'
ORDER BY i.tanggal_selesai;

COMMENT ON VIEW perlengkapan.v_izin_pemakaian_aktif IS 'Active BMN usage permits with expiry status';

-- View: Roadmap vs Realization
CREATE OR REPLACE VIEW perlengkapan.v_roadmap_realization AS
SELECT
    r.id,
    r.satker_id,
    r.periode_mulai,
    r.periode_akhir,
    r.tahun_rencana,
    r.kode_barang,
    r.nama_barang,
    r.jumlah_kebutuhan,
    r.jumlah_terpenuhi,
    r.estimasi_anggaran,
    r.realisasi_anggaran,
    r.status_pemenuhan,
    CASE
        WHEN r.jumlah_kebutuhan > 0 THEN
            ROUND((r.jumlah_terpenuhi::NUMERIC / r.jumlah_kebutuhan::NUMERIC) * 100, 2)
        ELSE 0
    END as persentase_pemenuhan,
    CASE
        WHEN r.estimasi_anggaran > 0 THEN
            ROUND((r.realisasi_anggaran / r.estimasi_anggaran) * 100, 2)
        ELSE 0
    END as persentase_realisasi_anggaran,
    (SELECT COUNT(*) FROM perlengkapan.riwayat_pemenuhan rp
     WHERE rp.roadmap_id = r.id) as jumlah_pemenuhan
FROM perlengkapan.roadmap_sarpras r
ORDER BY r.satker_id, r.tahun_rencana;

COMMENT ON VIEW perlengkapan.v_roadmap_realization IS 'Roadmap planning vs actual realization comparison';

-- View: Mapping Kodefikasi Progress
CREATE OR REPLACE VIEW perlengkapan.v_mapping_progress AS
SELECT
    satker_id,
    COUNT(*) as total_mapping,
    COUNT(*) FILTER (WHERE status_mapping = 'PROPOSED') as proposed_count,
    COUNT(*) FILTER (WHERE status_mapping = 'VERIFIED') as verified_count,
    COUNT(*) FILTER (WHERE status_mapping = 'APPROVED') as approved_count,
    COUNT(*) FILTER (WHERE status_mapping = 'REJECTED') as rejected_count,
    ROUND((COUNT(*) FILTER (WHERE status_mapping = 'APPROVED')::NUMERIC / NULLIF(COUNT(*), 0)::NUMERIC) * 100, 2) as approval_rate
FROM perlengkapan.mapping_kodefikasi
GROUP BY satker_id
ORDER BY satker_id;

COMMENT ON VIEW perlengkapan.v_mapping_progress IS 'Progress of code mapping by satker';

-- ============================================================================
-- COMPLETION MESSAGE
-- ============================================================================

DO $$
BEGIN
    RAISE NOTICE '✅ New entity tables created successfully';
    RAISE NOTICE '📊 Created: 7 new tables';
    RAISE NOTICE '  - roadmap_sarpras (5-year infrastructure roadmap)';
    RAISE NOTICE '  - mapping_kodefikasi (non-standard code mapping)';
    RAISE NOTICE '  - riwayat_pemenuhan (fulfillment history)';
    RAISE NOTICE '  - parallel_approvals (parallel approval workflow)';
    RAISE NOTICE '  - parallel_approval_votes (approval votes)';
    RAISE NOTICE '  - izin_pemakaian_bmn (BMN usage permits)';
    RAISE NOTICE '  - izin_pemakaian_bmn_aktivitas (permit workflow history)';
    RAISE NOTICE '📈 Created: 40+ indexes (FK, composite, full-text)';
    RAISE NOTICE '👁️ Created: 3 views for reporting';
    RAISE NOTICE '🔧 Created: 5 triggers for updated_at';
    RAISE NOTICE '🔍 Schema: perlengkapan';
END $$;
