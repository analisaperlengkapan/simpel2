-- Migration: Create Pakaian Dinas Tables
-- Module: Perlengkapan/Pakaian Dinas
-- Description: Official uniform management for Kejaksaan RI employees
-- Migrated from simpel_laravel

-- Ensure schema exists and set search path
CREATE SCHEMA IF NOT EXISTS perlengkapan;
SET search_path = perlengkapan, public;

-- ============ Master Tables ============

-- Jenis Pakaian Dinas (Uniform Types)
-- Examples: PDH, PDL, Toga Jaksa
CREATE TABLE IF NOT EXISTS ms_jenis_pakaian_dinas (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    nama VARCHAR(255) NOT NULL,
    deskripsi TEXT,
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_ms_jenis_pakaian_dinas_nama ON ms_jenis_pakaian_dinas(nama);
CREATE INDEX idx_ms_jenis_pakaian_dinas_active ON ms_jenis_pakaian_dinas(is_active);

COMMENT ON TABLE ms_jenis_pakaian_dinas IS 'Master table for uniform types (PDH, PDL, Toga, etc.)';

-- Spesifikasi Pakaian Dinas (Uniform Specifications)
-- Examples: Kemeja PDH, Celana PDH, Sepatu Dinas
CREATE TABLE IF NOT EXISTS ms_spesifikasi_pakaian_dinas (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    jenis_pakaian_dinas_id UUID NOT NULL REFERENCES ms_jenis_pakaian_dinas(id) ON DELETE CASCADE,
    nama VARCHAR(255) NOT NULL,
    gender VARCHAR(10) NOT NULL CHECK (gender IN ('L', 'P', 'SEMUA')),
    ukuran_group VARCHAR(50) NOT NULL CHECK (ukuran_group IN ('BAJU', 'CELANA', 'SEPATU')),
    deskripsi TEXT,
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_ms_spesifikasi_jenis ON ms_spesifikasi_pakaian_dinas(jenis_pakaian_dinas_id);
CREATE INDEX idx_ms_spesifikasi_gender ON ms_spesifikasi_pakaian_dinas(gender);
CREATE INDEX idx_ms_spesifikasi_ukuran_group ON ms_spesifikasi_pakaian_dinas(ukuran_group);

COMMENT ON TABLE ms_spesifikasi_pakaian_dinas IS 'Specifications for each uniform type with size category';

-- Spesifikasi Photos
CREATE TABLE IF NOT EXISTS ms_spesifikasi_pakaian_dinas_foto (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    spesifikasi_id UUID NOT NULL REFERENCES ms_spesifikasi_pakaian_dinas(id) ON DELETE CASCADE,
    path VARCHAR(500) NOT NULL,
    filename VARCHAR(255) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_ms_spesifikasi_foto_spec ON ms_spesifikasi_pakaian_dinas_foto(spesifikasi_id);

-- SubSpesifikasi Pakaian Dinas
-- Examples: Kemeja Lengan Panjang, Kemeja Lengan Pendek
CREATE TABLE IF NOT EXISTS ms_subspesifikasi_pakaian_dinas (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    spesifikasi_id UUID NOT NULL REFERENCES ms_spesifikasi_pakaian_dinas(id) ON DELETE CASCADE,
    nama VARCHAR(255) NOT NULL,
    gender VARCHAR(10) NOT NULL CHECK (gender IN ('L', 'P', 'SEMUA')),
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_ms_subspesifikasi_spec ON ms_subspesifikasi_pakaian_dinas(spesifikasi_id);

-- Ukuran (Sizes Master)
CREATE TABLE IF NOT EXISTS ms_ukuran (
    ukuran VARCHAR(20) NOT NULL,
    "group" VARCHAR(50) NOT NULL CHECK ("group" IN ('BAJU', 'CELANA', 'SEPATU')),
    urutan INTEGER DEFAULT 0,
    PRIMARY KEY (ukuran, "group")
);

-- Insert default sizes
INSERT INTO ms_ukuran (ukuran, "group", urutan) VALUES
    -- Baju (Shirt sizes)
    ('XS', 'BAJU', 1), ('S', 'BAJU', 2), ('M', 'BAJU', 3), ('L', 'BAJU', 4),
    ('XL', 'BAJU', 5), ('XXL', 'BAJU', 6), ('XXXL', 'BAJU', 7), ('4XL', 'BAJU', 8),
    -- Celana (Pants sizes)
    ('27', 'CELANA', 1), ('28', 'CELANA', 2), ('29', 'CELANA', 3), ('30', 'CELANA', 4),
    ('31', 'CELANA', 5), ('32', 'CELANA', 6), ('33', 'CELANA', 7), ('34', 'CELANA', 8),
    ('35', 'CELANA', 9), ('36', 'CELANA', 10), ('37', 'CELANA', 11), ('38', 'CELANA', 12),
    ('39', 'CELANA', 13), ('40', 'CELANA', 14), ('42', 'CELANA', 15), ('44', 'CELANA', 16),
    -- Sepatu (Shoe sizes)
    ('36', 'SEPATU', 1), ('37', 'SEPATU', 2), ('38', 'SEPATU', 3), ('39', 'SEPATU', 4),
    ('40', 'SEPATU', 5), ('41', 'SEPATU', 6), ('42', 'SEPATU', 7), ('43', 'SEPATU', 8),
    ('44', 'SEPATU', 9), ('45', 'SEPATU', 10), ('46', 'SEPATU', 11)
ON CONFLICT DO NOTHING;

-- ============ Transaction Tables ============

-- Pengajuan Pakaian Dinas (Main Request Header)
CREATE TABLE IF NOT EXISTS pengajuan_pakaian_dinas (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    nama VARCHAR(255) NOT NULL,
    deskripsi TEXT,
    tgl_mulai DATE,
    tgl_selesai DATE,
    is_reguler BOOLEAN DEFAULT TRUE,
    tahun INTEGER NOT NULL,
    pilihan_satker VARCHAR(20) NOT NULL CHECK (pilihan_satker IN ('all', 'sebagian')),
    dengan_unit_kerja BOOLEAN DEFAULT FALSE,
    jenis_pakaian_dinas_id UUID REFERENCES ms_jenis_pakaian_dinas(id),
    aktivitas_id INTEGER NOT NULL DEFAULT 1000,
    created_by UUID,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_pengajuan_pakaian_dinas_tahun ON pengajuan_pakaian_dinas(tahun);
CREATE INDEX idx_pengajuan_pakaian_dinas_status ON pengajuan_pakaian_dinas(aktivitas_id);
CREATE INDEX idx_pengajuan_pakaian_dinas_created_by ON pengajuan_pakaian_dinas(created_by);

COMMENT ON TABLE pengajuan_pakaian_dinas IS 'Main request header for uniform procurement';

-- Pengajuan Satker Terpilih (Selected Satker Units)
CREATE TABLE IF NOT EXISTS pengajuan_pakaian_dinas_satker_terpilih (
    pengajuan_id UUID NOT NULL REFERENCES pengajuan_pakaian_dinas(id) ON DELETE CASCADE,
    satker_id UUID NOT NULL,
    satker_pusat_id UUID,
    is_show_in_form BOOLEAN DEFAULT TRUE,
    PRIMARY KEY (pengajuan_id, satker_id)
);

CREATE INDEX idx_ppd_satker_terpilih_pengajuan ON pengajuan_pakaian_dinas_satker_terpilih(pengajuan_id);

-- Pengajuan Pakaian (Clothing Items in Request)
CREATE TABLE IF NOT EXISTS pengajuan_pakaian_dinas_pakaian (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    pengajuan_id UUID NOT NULL REFERENCES pengajuan_pakaian_dinas(id) ON DELETE CASCADE,
    jenis_pakaian_id UUID NOT NULL,
    jenis_pakaian_nama VARCHAR(255) NOT NULL,
    spesifikasi_id UUID NOT NULL,
    spesifikasi_nama VARCHAR(255) NOT NULL,
    spesifikasi_ukuran_group VARCHAR(50) NOT NULL,
    subspesifikasi_id UUID,
    subspesifikasi_nama VARCHAR(255),
    subspesifikasi_gender VARCHAR(10)
);

CREATE INDEX idx_ppd_pakaian_pengajuan ON pengajuan_pakaian_dinas_pakaian(pengajuan_id);

-- Pengajuan Satker (Per-Satker Submission)
CREATE TABLE IF NOT EXISTS pengajuan_pakaian_dinas_satker (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    pengajuan_id UUID NOT NULL REFERENCES pengajuan_pakaian_dinas(id) ON DELETE CASCADE,
    satker_id UUID NOT NULL,
    satker_pusat_id UUID,
    id_kejati UUID,
    id_kejari UUID,
    id_cabjari UUID,
    aktivitas_id INTEGER NOT NULL DEFAULT 1000,
    created_by UUID,
    updated_by UUID,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_ppd_satker_pengajuan ON pengajuan_pakaian_dinas_satker(pengajuan_id);
CREATE INDEX idx_ppd_satker_satker ON pengajuan_pakaian_dinas_satker(satker_id);
CREATE INDEX idx_ppd_satker_status ON pengajuan_pakaian_dinas_satker(aktivitas_id);

-- Pengajuan Satker Pegawai (Employee Data in Submission)
CREATE TABLE IF NOT EXISTS pengajuan_pakaian_dinas_satker_pegawai (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    pengajuan_satker_id UUID NOT NULL REFERENCES pengajuan_pakaian_dinas_satker(id) ON DELETE CASCADE,
    nip VARCHAR(30) NOT NULL,
    nama VARCHAR(500) NOT NULL,
    pangkat VARCHAR(255),
    jabatan VARCHAR(500),
    eselon VARCHAR(20),
    jenis_kelamin VARCHAR(1) NOT NULL CHECK (jenis_kelamin IN ('L', 'P')),
    gol_kd VARCHAR(20),
    jenis VARCHAR(100),
    with_hijab BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_ppd_satker_pegawai_satker ON pengajuan_pakaian_dinas_satker_pegawai(pengajuan_satker_id);
CREATE INDEX idx_ppd_satker_pegawai_nip ON pengajuan_pakaian_dinas_satker_pegawai(nip);

-- Pengajuan Satker Pegawai Ukuran (Employee Sizes)
CREATE TABLE IF NOT EXISTS pengajuan_pakaian_dinas_satker_pegawai_ukuran (
    pengajuan_satker_id UUID NOT NULL REFERENCES pengajuan_pakaian_dinas_satker(id) ON DELETE CASCADE,
    pegawai_id UUID NOT NULL REFERENCES pengajuan_pakaian_dinas_satker_pegawai(id) ON DELETE CASCADE,
    pakaian_id UUID NOT NULL REFERENCES pengajuan_pakaian_dinas_pakaian(id) ON DELETE CASCADE,
    ukuran VARCHAR(20) NOT NULL,
    PRIMARY KEY (pegawai_id, pakaian_id)
);

CREATE INDEX idx_ppd_ukuran_satker ON pengajuan_pakaian_dinas_satker_pegawai_ukuran(pengajuan_satker_id);

-- Pengajuan Satker Aktivitas (Workflow History)
CREATE TABLE IF NOT EXISTS pengajuan_pakaian_dinas_satker_aktivitas (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    pengajuan_satker_id UUID NOT NULL REFERENCES pengajuan_pakaian_dinas_satker(id) ON DELETE CASCADE,
    aktivitas_id INTEGER NOT NULL,
    komentar TEXT,
    nip VARCHAR(30),
    nama VARCHAR(500),
    pangkat VARCHAR(255),
    jabatan VARCHAR(500),
    role VARCHAR(100),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_ppd_aktivitas_satker ON pengajuan_pakaian_dinas_satker_aktivitas(pengajuan_satker_id);

-- ============ Persistent Employee Size Record ============

-- Pegawai Pakaian Dinas (Persistent Sizes)
CREATE TABLE IF NOT EXISTS pegawai_pakaian_dinas (
    nip VARCHAR(30) PRIMARY KEY,
    nama VARCHAR(500),
    ukuran_baju VARCHAR(20),
    ukuran_celana VARCHAR(20),
    ukuran_sepatu VARCHAR(20),
    with_hijab BOOLEAN DEFAULT FALSE,
    pangkat VARCHAR(255),
    jabatan VARCHAR(500),
    status VARCHAR(50),
    last_pengajuan_satker_pegawai_id UUID,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_pegawai_pakaian_dinas_nip ON pegawai_pakaian_dinas(nip);

COMMENT ON TABLE pegawai_pakaian_dinas IS 'Persistent record of employee uniform sizes, updated on approval';

-- ============ Insert Default Jenis Pakaian Dinas ============

INSERT INTO ms_jenis_pakaian_dinas (nama, deskripsi) VALUES
    ('PDH', 'Pakaian Dinas Harian'),
    ('PDL', 'Pakaian Dinas Lapangan'),
    ('Toga Jaksa', 'Toga untuk Jaksa'),
    ('Toga TU', 'Toga untuk Tenaga TU'),
    ('Jas Almamater', 'Jas Almamater Kejaksaan'),
    ('Batik', 'Pakaian Batik Dinas')
ON CONFLICT DO NOTHING;

-- ============ Trigger for updated_at ============

CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Apply trigger to relevant tables
DO $$
DECLARE
    table_names TEXT[] := ARRAY[
        'ms_jenis_pakaian_dinas',
        'ms_spesifikasi_pakaian_dinas',
        'ms_subspesifikasi_pakaian_dinas',
        'pengajuan_pakaian_dinas',
        'pengajuan_pakaian_dinas_satker',
        'pegawai_pakaian_dinas'
    ];
    tbl TEXT;
BEGIN
    FOREACH tbl IN ARRAY table_names
    LOOP
        EXECUTE format('
            DROP TRIGGER IF EXISTS trigger_%I_updated_at ON %I;
            CREATE TRIGGER trigger_%I_updated_at
                BEFORE UPDATE ON %I
                FOR EACH ROW
                EXECUTE FUNCTION update_updated_at_column();
        ', tbl, tbl, tbl, tbl);
    END LOOP;
END $$;
