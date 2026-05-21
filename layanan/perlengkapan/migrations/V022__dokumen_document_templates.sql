-- Document Templates Schema
-- Supports template-based document generation with versioning

CREATE SCHEMA IF NOT EXISTS dokumen;

-- Document Templates Table
CREATE TABLE IF NOT EXISTS dokumen.document_templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    template_type VARCHAR(100) NOT NULL, -- 'sk_penghapusan', 'izin_pemakaian', 'rekapitulasi', etc.
    content TEXT NOT NULL, -- Handlebars template content
    format VARCHAR(50) NOT NULL DEFAULT 'html', -- 'html', 'markdown', 'latex'
    output_format VARCHAR(50) NOT NULL DEFAULT 'pdf', -- 'pdf', 'excel', 'word'

    -- Versioning
    version INTEGER NOT NULL DEFAULT 1,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    parent_template_id UUID REFERENCES dokumen.document_templates(id),

    -- Metadata
    variables JSONB NOT NULL DEFAULT '{}', -- Expected template variables
    sample_data JSONB, -- Sample data for preview
    letterhead_config JSONB, -- Letterhead configuration

    -- Audit
    created_by UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_by UUID,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    UNIQUE(name, version)
);

-- Generated Documents Table
CREATE TABLE IF NOT EXISTS dokumen.generated_documents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    template_id UUID NOT NULL REFERENCES dokumen.document_templates(id),
    document_number VARCHAR(255) NOT NULL UNIQUE,
    filename VARCHAR(255) NOT NULL,
    storage_path VARCHAR(500) NOT NULL,
    format VARCHAR(50) NOT NULL,
    size BIGINT NOT NULL,
    checksum VARCHAR(64) NOT NULL, -- SHA-256

    -- Generation context
    generated_data JSONB NOT NULL, -- Data used for generation
    generated_by UUID NOT NULL,
    generated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Status
    status VARCHAR(50) NOT NULL DEFAULT 'generated', -- 'generated', 'signed', 'archived', 'deleted'

    -- Metadata
    metadata JSONB,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Document Template Versions History
CREATE TABLE IF NOT EXISTS dokumen.template_versions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    template_id UUID NOT NULL REFERENCES dokumen.document_templates(id),
    version INTEGER NOT NULL,
    content TEXT NOT NULL,
    variables JSONB NOT NULL DEFAULT '{}',
    created_by UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    change_notes TEXT,

    UNIQUE(template_id, version)
);

-- Indexes
CREATE INDEX idx_templates_type ON dokumen.document_templates(template_type);
CREATE INDEX idx_templates_active ON dokumen.document_templates(is_active);
CREATE INDEX idx_templates_created_at ON dokumen.document_templates(created_at DESC);

CREATE INDEX idx_generated_docs_template ON dokumen.generated_documents(template_id);
CREATE INDEX idx_generated_docs_number ON dokumen.generated_documents(document_number);
CREATE INDEX idx_generated_docs_generated_by ON dokumen.generated_documents(generated_by);
CREATE INDEX idx_generated_docs_generated_at ON dokumen.generated_documents(generated_at DESC);
CREATE INDEX idx_generated_docs_status ON dokumen.generated_documents(status);

CREATE INDEX idx_template_versions_template ON dokumen.template_versions(template_id, version DESC);

-- Comments
COMMENT ON TABLE dokumen.document_templates IS 'Document templates for automated generation';
COMMENT ON TABLE dokumen.generated_documents IS 'Generated documents from templates';
COMMENT ON TABLE dokumen.template_versions IS 'Version history of document templates';

COMMENT ON COLUMN dokumen.document_templates.content IS 'Handlebars template content';
COMMENT ON COLUMN dokumen.document_templates.variables IS 'Expected template variables with types and descriptions';
COMMENT ON COLUMN dokumen.document_templates.letterhead_config IS 'Configuration for official letterhead (logo, header, footer)';
COMMENT ON COLUMN dokumen.generated_documents.checksum IS 'SHA-256 checksum for integrity verification';
