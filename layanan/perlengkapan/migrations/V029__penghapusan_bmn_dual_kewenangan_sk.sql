-- V029: Dual kewenangan SK Penghapusan BMN (Fase 1.9)
--
-- Sebelumnya semua SK Penghapusan diasumsikan ditandatangani oleh
-- Jaksa Agung Muda Pembinaan (Pusat). Stakeholder eksplisit minta
-- (plan §6.1) dukungan dua jalur:
-- - PUSAT: SK signed by Jaksa Agung Muda Pembinaan
-- - WILAYAH: SK signed by Kepala Kejaksaan Tinggi (mewakili oleh
--   Validator Wilayah di sistem ini — tidak ada role terpisah)
--
-- Workflow branching (di service layer):
--   SubmitWilayah ─ kewenangan=PUSAT  ─→ SubmitPusat → VerifikasiPusat →
--                                       KonsepSKGenerated → SKSigned → Completed
--                ─ kewenangan=WILAYAH ─→ KonsepSKWilayahGenerated (4010) →
--                                       SKSignedWilayah (4011) → Completed

ALTER TABLE perlengkapan.penghapusan_bmn
    ADD COLUMN IF NOT EXISTS kewenangan_penetap_sk TEXT NOT NULL DEFAULT 'PUSAT',
    ADD COLUMN IF NOT EXISTS penetap_sk_jabatan TEXT,
    ADD COLUMN IF NOT EXISTS konsep_sk_wilayah_url TEXT,
    ADD COLUMN IF NOT EXISTS konsep_sk_wilayah_pdf_url TEXT,
    ADD COLUMN IF NOT EXISTS konsep_sk_wilayah_generated_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS signed_sk_wilayah_pdf_url TEXT,
    ADD COLUMN IF NOT EXISTS signed_sk_wilayah_pdf_uploaded_at TIMESTAMPTZ;

ALTER TABLE perlengkapan.penghapusan_bmn
    ADD CONSTRAINT chk_penghapusan_bmn_kewenangan
    CHECK (kewenangan_penetap_sk IN ('PUSAT', 'WILAYAH'));

COMMENT ON COLUMN perlengkapan.penghapusan_bmn.kewenangan_penetap_sk IS
    'Otoritas penetap SK: PUSAT (Jaksa Agung Muda Pembinaan) atau WILAYAH (Kepala Kejaksaan Tinggi). Menentukan branching workflow & lokasi SK signed disimpan (kolom kanonik vs kolom _wilayah).';
COMMENT ON COLUMN perlengkapan.penghapusan_bmn.penetap_sk_jabatan IS
    'Label jabatan penetap SK (cth: "Jaksa Agung Muda Pembinaan" atau "Kepala Kejaksaan Tinggi DKI Jakarta"). Disimpan utk audit & cetak template.';
COMMENT ON COLUMN perlengkapan.penghapusan_bmn.konsep_sk_wilayah_url IS
    'URL konsep SK DOCX (editable) jalur WILAYAH. NULL jika kewenangan=PUSAT.';
COMMENT ON COLUMN perlengkapan.penghapusan_bmn.signed_sk_wilayah_pdf_url IS
    'URL SK PDF yg sudah ditandatangani Kepala Kejati. NULL jika kewenangan=PUSAT.';
