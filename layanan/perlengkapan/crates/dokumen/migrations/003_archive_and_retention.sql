-- Archive and Retention Policy Schema
-- Supports automatic document archival and retention management

-- Documents Table (if not exists from previous migrations)
CREATE TABLE IF NOT EXISTS dokumen.documents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    filename VARCHAR(255) NOT NULL,
    content_type VARCHAR(100) NOT NULL,
    size BIGINT NOT NULL,
    storage_path VARCHAR(500) NOT NULL,
    owner_id UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    is_archived BOOLEAN NOT NULL DEFAULT FALSE,
    checksum VARCHAR(64),
    encrypted BOOLEAN NOT NULL DEFAULT FALSE,
    current_version INTEGER NOT NULL DEFAULT 1,
    metadata JSONB
);

-- Document Versions Table
CREATE TABLE IF NOT EXISTS dokumen.document_versions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    document_id UUID NOT NULL REFERENCES dokumen.documents(id) ON DELETE CASCADE,
    version INTEGER NOT NULL,
    storage_path VARCHAR(500) NOT NULL,
    size BIGINT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    checksum VARCHAR(64),
    encrypted BOOLEAN NOT NULL DEFAULT FALSE,
    metadata JSONB,
    UNIQUE(document_id, version)
);

-- Archive Collections Table
CREATE TABLE IF NOT EXISTS dokumen.archive_collections (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    owner_id UUID
);

-- Archive Documents Table (many-to-many relationship)
CREATE TABLE IF NOT EXISTS dokumen.archive_documents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    collection_id UUID NOT NULL REFERENCES dokumen.archive_collections(id) ON DELETE CASCADE,
    document_id UUID NOT NULL REFERENCES dokumen.documents(id) ON DELETE CASCADE,
    added_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(collection_id, document_id)
);

-- Audit Log Table
CREATE TABLE IF NOT EXISTS dokumen.audit_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    document_id UUID REFERENCES dokumen.documents(id) ON DELETE SET NULL,
    user_id UUID,
    action VARCHAR(100) NOT NULL,
    details JSONB,
    ip_address VARCHAR(45),
    user_agent TEXT,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- OCR Results Table
CREATE TABLE IF NOT EXISTS dokumen.ocr_results (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    document_id UUID NOT NULL REFERENCES dokumen.documents(id) ON DELETE CASCADE,
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    text TEXT,
    accuracy REAL,
    processed_at TIMESTAMPTZ,
    error_message TEXT
);

-- Document Tags Table
CREATE TABLE IF NOT EXISTS dokumen.document_tags (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    document_id UUID NOT NULL REFERENCES dokumen.documents(id) ON DELETE CASCADE,
    tag VARCHAR(100) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(document_id, tag)
);

-- Document Permissions Table
CREATE TABLE IF NOT EXISTS dokumen.document_permissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    document_id UUID NOT NULL REFERENCES dokumen.documents(id) ON DELETE CASCADE,
    user_id UUID NOT NULL,
    role VARCHAR(50) NOT NULL,
    granted_by UUID,
    granted_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(document_id, user_id)
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_documents_owner ON dokumen.documents(owner_id);
CREATE INDEX IF NOT EXISTS idx_documents_archived ON dokumen.documents(is_archived);
CREATE INDEX IF NOT EXISTS idx_documents_created_at ON dokumen.documents(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_documents_metadata_gin ON dokumen.documents USING GIN(metadata);

CREATE INDEX IF NOT EXISTS idx_document_versions_document ON dokumen.document_versions(document_id, version DESC);

CREATE INDEX IF NOT EXISTS idx_archive_collections_owner ON dokumen.archive_collections(owner_id);
CREATE INDEX IF NOT EXISTS idx_archive_documents_collection ON dokumen.archive_documents(collection_id);
CREATE INDEX IF NOT EXISTS idx_archive_documents_document ON dokumen.archive_documents(document_id);

CREATE INDEX IF NOT EXISTS idx_audit_log_document ON dokumen.audit_log(document_id, timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_audit_log_user ON dokumen.audit_log(user_id, timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_audit_log_timestamp ON dokumen.audit_log(timestamp DESC);

CREATE INDEX IF NOT EXISTS idx_ocr_results_document ON dokumen.ocr_results(document_id);
CREATE INDEX IF NOT EXISTS idx_ocr_results_status ON dokumen.ocr_results(status);

CREATE INDEX IF NOT EXISTS idx_document_tags_document ON dokumen.document_tags(document_id);
CREATE INDEX IF NOT EXISTS idx_document_tags_tag ON dokumen.document_tags(tag);

CREATE INDEX IF NOT EXISTS idx_document_permissions_document ON dokumen.document_permissions(document_id);
CREATE INDEX IF NOT EXISTS idx_document_permissions_user ON dokumen.document_permissions(user_id);

-- Comments
COMMENT ON TABLE dokumen.documents IS 'Main documents table with archival support';
COMMENT ON TABLE dokumen.document_versions IS 'Document version history';
COMMENT ON TABLE dokumen.archive_collections IS 'Archive collections for organizing archived documents';
COMMENT ON TABLE dokumen.archive_documents IS 'Documents in archive collections';
COMMENT ON TABLE dokumen.audit_log IS 'Audit trail for all document operations';

COMMENT ON COLUMN dokumen.documents.is_archived IS 'Whether document is archived (moved to archive storage)';
COMMENT ON COLUMN dokumen.documents.metadata IS 'Document metadata including document_type, satker_id, workflow_state, archive_path, retention_years';
COMMENT ON COLUMN dokumen.documents.checksum IS 'SHA-256 checksum for integrity verification';

-- Add retention policy information to metadata
-- Example metadata structure:
-- {
--   "document_type": "SK" | "rekapitulasi",
--   "satker_id": "uuid",
--   "workflow_state": "COMPLETED",
--   "archive_path": "archive_filename.pdf",
--   "retention_years": 5 | 3
-- }
