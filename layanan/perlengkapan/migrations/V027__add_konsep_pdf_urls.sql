-- Migration V027: add PDF URL + storage-path columns for konsep surat / SK
--
-- The konsep surat / konsep SK flow produces BOTH a `.docx` (editable
-- before signing) and a `.pdf` (final immutable artifact) side by side.
-- `konsep_*_url` is the DOCX download URL; `konsep_*_pdf_url` is the PDF
-- download URL. The `*_path` columns record the filesystem path the route
-- handler streams from — internal-only, never exposed to clients.

ALTER TABLE perlengkapan.izin_pemakaian_bmn
    ADD COLUMN IF NOT EXISTS konsep_surat_pdf_url TEXT,
    ADD COLUMN IF NOT EXISTS konsep_surat_pdf_generated_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS konsep_surat_docx_path TEXT,
    ADD COLUMN IF NOT EXISTS konsep_surat_pdf_path TEXT;

COMMENT ON COLUMN perlengkapan.izin_pemakaian_bmn.konsep_surat_url
    IS 'Public download URL for the DOCX (editable) konsep surat';
-- Internal layout for the streamed artifacts:
--   ${DOCUMENT_STORAGE_PATH}/pemakaian-bmn/{id}/konsep-surat.docx
--   ${DOCUMENT_STORAGE_PATH}/pemakaian-bmn/{id}/konsep-surat.pdf
-- and similarly under penghapusan-bmn/{id}/.
COMMENT ON COLUMN perlengkapan.izin_pemakaian_bmn.konsep_surat_pdf_url
    IS 'Public download URL for the PDF (final) konsep surat';
COMMENT ON COLUMN perlengkapan.izin_pemakaian_bmn.konsep_surat_docx_path
    IS 'Server-side filesystem path the DOCX route handler streams from';
COMMENT ON COLUMN perlengkapan.izin_pemakaian_bmn.konsep_surat_pdf_path
    IS 'Server-side filesystem path the PDF route handler streams from';

ALTER TABLE perlengkapan.penghapusan_bmn
    ADD COLUMN IF NOT EXISTS konsep_sk_pdf_url TEXT,
    ADD COLUMN IF NOT EXISTS konsep_sk_pdf_generated_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS konsep_sk_docx_path TEXT,
    ADD COLUMN IF NOT EXISTS konsep_sk_pdf_path TEXT;

COMMENT ON COLUMN perlengkapan.penghapusan_bmn.konsep_sk_url
    IS 'Public download URL for the DOCX (editable) konsep SK';
COMMENT ON COLUMN perlengkapan.penghapusan_bmn.konsep_sk_pdf_url
    IS 'Public download URL for the PDF (final) konsep SK';
COMMENT ON COLUMN perlengkapan.penghapusan_bmn.konsep_sk_docx_path
    IS 'Server-side filesystem path the DOCX route handler streams from';
COMMENT ON COLUMN perlengkapan.penghapusan_bmn.konsep_sk_pdf_path
    IS 'Server-side filesystem path the PDF route handler streams from';
