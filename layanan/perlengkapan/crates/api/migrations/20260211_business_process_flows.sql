-- ============================================================================
-- Migration: Business Process Flows for Kebutuhan BMN, Pemakaian BMN, SK Penghapusan BMN
-- Date: 2026-02-11
-- Description:
--   1. Kebutuhan BMN: Add Validator Wilayah step, lampiran support, analysis fields
--   2. Pemakaian BMN: Add multi-BMN per pegawai, DOCX generation, signed PDF upload
--   3. SK Penghapusan BMN: Add proper workflow with Wilayah step, lampiran, SK generation
-- ============================================================================

BEGIN;

-- ============================================================================
-- 1. KEBUTUHAN BMN ENHANCEMENTS
-- ============================================================================

-- Add lampiran (attachment) support for operator satker submissions
ALTER TABLE perlengkapan.pengajuan_kebutuhan_bmn_satker
    ADD COLUMN IF NOT EXISTS catatan_satker TEXT,
    ADD COLUMN IF NOT EXISTS lampiran_surat_permohonan TEXT,      -- URL/path to surat permohonan
    ADD COLUMN IF NOT EXISTS lampiran_pendukung JSONB DEFAULT '[]'::jsonb, -- Additional attachments [{name, url, type}]
    ADD COLUMN IF NOT EXISTS catatan_validator_wilayah TEXT,       -- Notes from validator wilayah
    ADD COLUMN IF NOT EXISTS catatan_validator_pusat TEXT,         -- Notes from validator pusat
    ADD COLUMN IF NOT EXISTS tanggal_submit_wilayah TIMESTAMPTZ,  -- When submitted to wilayah
    ADD COLUMN IF NOT EXISTS tanggal_submit_pusat TIMESTAMPTZ,    -- When submitted to pusat
    ADD COLUMN IF NOT EXISTS validator_wilayah_id UUID,            -- Who validated at wilayah level
    ADD COLUMN IF NOT EXISTS validator_pusat_id UUID;              -- Who validated at pusat level

-- Add analysis data fields for validator pusat
ALTER TABLE perlengkapan.pengajuan_kebutuhan_bmn_satker
    ADD COLUMN IF NOT EXISTS data_eksisting_siman JSONB DEFAULT '{}'::jsonb,    -- Cached SIMAN data snapshot
    ADD COLUMN IF NOT EXISTS data_pegawai_mysimkari JSONB DEFAULT '{}'::jsonb,  -- Cached MySIMKARI data snapshot
    ADD COLUMN IF NOT EXISTS rekap_eselon JSONB DEFAULT '{}'::jsonb,            -- Eselon counts
    ADD COLUMN IF NOT EXISTS rekap_non_eselon JSONB DEFAULT '{}'::jsonb,        -- Non-eselon by golongan/pangkat/jaksa
    ADD COLUMN IF NOT EXISTS hasil_analisis JSONB DEFAULT '{}'::jsonb,          -- Analysis results
    ADD COLUMN IF NOT EXISTS is_approved BOOLEAN,                               -- NULL=pending, true=approved, false=rejected
    ADD COLUMN IF NOT EXISTS alasan_keputusan TEXT;                              -- Reason for approval/rejection

-- Add report generation tracking
ALTER TABLE perlengkapan.pengajuan_kebutuhan_bmn
    ADD COLUMN IF NOT EXISTS laporan_url TEXT,              -- Generated report URL
    ADD COLUMN IF NOT EXISTS laporan_format VARCHAR(10),    -- pdf, docx, xlsx
    ADD COLUMN IF NOT EXISTS laporan_generated_at TIMESTAMPTZ;

-- ============================================================================
-- 2. PEMAKAIAN BMN ENHANCEMENTS
-- ============================================================================

-- Create table for multi-BMN items per permit (one pegawai can have multiple BMN)
CREATE TABLE IF NOT EXISTS perlengkapan.pemakaian_bmn_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    izin_pemakaian_id UUID NOT NULL REFERENCES perlengkapan.izin_pemakaian_bmn(id) ON DELETE CASCADE,

    -- BMN Information from SIMAN
    bmn_nup VARCHAR(100) NOT NULL,
    bmn_kode_barang VARCHAR(100) NOT NULL,
    bmn_nama_barang VARCHAR(500) NOT NULL,
    bmn_merk VARCHAR(255),
    bmn_tahun_perolehan INTEGER,
    bmn_kondisi VARCHAR(50),

    -- Type-specific fields (JSONB for flexibility)
    detail_bmn JSONB DEFAULT '{}'::jsonb,  -- no_polisi, alamat, serial_number, etc.

    -- Status
    keterangan TEXT,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Add photo and document fields to izin_pemakaian_bmn
ALTER TABLE perlengkapan.izin_pemakaian_bmn
    ADD COLUMN IF NOT EXISTS foto_pegawai TEXT,               -- URL to pegawai photo
    ADD COLUMN IF NOT EXISTS konsep_surat_url TEXT,           -- Generated DOCX concept URL
    ADD COLUMN IF NOT EXISTS konsep_surat_generated_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS signed_pdf_url TEXT,             -- Uploaded signed PDF URL
    ADD COLUMN IF NOT EXISTS signed_pdf_uploaded_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS is_completed BOOLEAN DEFAULT FALSE,
    ADD COLUMN IF NOT EXISTS pegawai_golongan VARCHAR(50),
    ADD COLUMN IF NOT EXISTS pegawai_pangkat VARCHAR(100),
    ADD COLUMN IF NOT EXISTS pegawai_unit_kerja VARCHAR(255);

-- Create index for multi-BMN items
CREATE INDEX IF NOT EXISTS idx_pemakaian_bmn_items_izin_id
    ON perlengkapan.pemakaian_bmn_items(izin_pemakaian_id);

CREATE INDEX IF NOT EXISTS idx_pemakaian_bmn_items_nup
    ON perlengkapan.pemakaian_bmn_items(bmn_nup);

-- ============================================================================
-- 3. SK PENGHAPUSAN BMN ENHANCEMENTS
-- ============================================================================

-- Add proper workflow fields to penghapusan_bmn
ALTER TABLE perlengkapan.penghapusan_bmn
    ADD COLUMN IF NOT EXISTS lampiran_persyaratan TEXT,              -- URL to requirement document
    ADD COLUMN IF NOT EXISTS lampiran_pendukung JSONB DEFAULT '[]'::jsonb,
    ADD COLUMN IF NOT EXISTS catatan_operator TEXT,
    ADD COLUMN IF NOT EXISTS catatan_validator_wilayah TEXT,
    ADD COLUMN IF NOT EXISTS catatan_validator_pusat TEXT,
    ADD COLUMN IF NOT EXISTS validator_wilayah_id UUID,
    ADD COLUMN IF NOT EXISTS validator_pusat_id UUID,
    ADD COLUMN IF NOT EXISTS tanggal_submit_wilayah TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS tanggal_verifikasi_wilayah TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS tanggal_submit_pusat TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS tanggal_verifikasi_pusat TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS konsep_sk_url TEXT,                     -- Generated DOCX SK concept URL
    ADD COLUMN IF NOT EXISTS konsep_sk_generated_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS signed_sk_pdf_url TEXT,                 -- Uploaded signed SK PDF
    ADD COLUMN IF NOT EXISTS signed_sk_pdf_uploaded_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS is_completed BOOLEAN DEFAULT FALSE;

-- Update status values for penghapusan_bmn to support wilayah workflow
-- Current status: DRAFT, SUBMITTED, REVIEWED, APPROVED, REJECTED
-- New status flow: DRAFT -> SUBMITTED_WILAYAH -> VERIFIED_WILAYAH -> SUBMITTED_PUSAT ->
--                  VERIFIED_PUSAT -> SK_GENERATED -> SK_SIGNED -> COMPLETED
-- Also: RETURNED_TO_SATKER (from wilayah), REJECTED (from pusat)

-- ============================================================================
-- 4. WORKFLOW STATUS REFERENCE TABLE
-- ============================================================================

-- Create a reference table for all workflow statuses across modules
CREATE TABLE IF NOT EXISTS perlengkapan.ms_workflow_status (
    kode INTEGER PRIMARY KEY,
    modul VARCHAR(50) NOT NULL,     -- kebutuhan_bmn, pemakaian_bmn, penghapusan_bmn
    nama VARCHAR(100) NOT NULL,
    deskripsi TEXT,
    urutan INTEGER DEFAULT 0,
    is_terminal BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Insert Kebutuhan BMN statuses (2000-series)
INSERT INTO perlengkapan.ms_workflow_status (kode, modul, nama, deskripsi, urutan, is_terminal) VALUES
    (2000, 'kebutuhan_bmn', 'Draft', 'Validator Pusat membuat pengajuan periode', 1, false),
    (2001, 'kebutuhan_bmn', 'Input Barang', 'Operator Satker mengisi kebutuhan BMN', 2, false),
    (2002, 'kebutuhan_bmn', 'Diajukan ke Validator Wilayah', 'Operator Satker mengirim ke Validator Wilayah', 3, false),
    (2003, 'kebutuhan_bmn', 'Revisi Satker', 'Dikembalikan ke Operator Satker untuk perbaikan', 4, false),
    (2004, 'kebutuhan_bmn', 'Diajukan ke Validator Pusat', 'Validator Wilayah mengirim ke Validator Pusat', 5, false),
    (2005, 'kebutuhan_bmn', 'Analisis Kelayakan', 'Validator Pusat menganalisis kebutuhan', 6, false),
    (2006, 'kebutuhan_bmn', 'Disetujui', 'Kebutuhan BMN disetujui Validator Pusat', 7, true),
    (2007, 'kebutuhan_bmn', 'Ditolak', 'Kebutuhan BMN ditolak Validator Pusat', 8, true),
    (2008, 'kebutuhan_bmn', 'Selesai', 'Proses kebutuhan BMN selesai', 9, true),
    (2009, 'kebutuhan_bmn', 'Dibatalkan', 'Pengajuan dibatalkan', 10, true)
ON CONFLICT (kode) DO UPDATE SET nama = EXCLUDED.nama, deskripsi = EXCLUDED.deskripsi, urutan = EXCLUDED.urutan;

-- Insert Pemakaian BMN statuses (3000-series)
INSERT INTO perlengkapan.ms_workflow_status (kode, modul, nama, deskripsi, urutan, is_terminal) VALUES
    (3000, 'pemakaian_bmn', 'Draft', 'Operator Satker membuat konsep izin pemakaian', 1, false),
    (3001, 'pemakaian_bmn', 'Diajukan', 'Draft diajukan untuk persetujuan', 2, false),
    (3002, 'pemakaian_bmn', 'Disetujui', 'Izin pemakaian disetujui', 3, false),
    (3003, 'pemakaian_bmn', 'Ditolak', 'Izin pemakaian ditolak', 4, true),
    (3004, 'pemakaian_bmn', 'Aktif', 'Izin pemakaian aktif (surat sudah diupload)', 5, false),
    (3005, 'pemakaian_bmn', 'Kadaluarsa', 'Izin pemakaian telah habis masa berlaku', 6, true),
    (3006, 'pemakaian_bmn', 'Dicabut', 'Izin pemakaian dicabut', 7, true),
    (3007, 'pemakaian_bmn', 'Dibatalkan', 'Pengajuan dibatalkan', 8, true)
ON CONFLICT (kode) DO UPDATE SET nama = EXCLUDED.nama, deskripsi = EXCLUDED.deskripsi;

-- Insert Penghapusan BMN statuses (4000-series)
INSERT INTO perlengkapan.ms_workflow_status (kode, modul, nama, deskripsi, urutan, is_terminal) VALUES
    (4000, 'penghapusan_bmn', 'Draft', 'Operator Satker membuat pengajuan SK Penghapusan', 1, false),
    (4001, 'penghapusan_bmn', 'Diajukan ke Validator Wilayah', 'Operator Satker mengirim ke Validator Wilayah', 2, false),
    (4002, 'penghapusan_bmn', 'Dikembalikan ke Operator', 'Validator Wilayah mengembalikan ke Operator untuk perbaikan', 3, false),
    (4003, 'penghapusan_bmn', 'Diajukan ke Validator Pusat', 'Validator Wilayah mengirim ke Validator Pusat', 4, false),
    (4004, 'penghapusan_bmn', 'Verifikasi Pusat', 'Validator Pusat memverifikasi pengajuan', 5, false),
    (4005, 'penghapusan_bmn', 'Konsep SK Digenerate', 'Validator Pusat mengenerate konsep SK', 6, false),
    (4006, 'penghapusan_bmn', 'SK Ditandatangani', 'SK sudah ditandatangani dan PDF diupload', 7, false),
    (4007, 'penghapusan_bmn', 'Selesai', 'Proses SK Penghapusan selesai', 8, true),
    (4008, 'penghapusan_bmn', 'Ditolak', 'Pengajuan SK Penghapusan ditolak', 9, true)
ON CONFLICT (kode) DO UPDATE SET nama = EXCLUDED.nama, deskripsi = EXCLUDED.deskripsi;

COMMIT;
