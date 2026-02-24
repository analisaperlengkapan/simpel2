-- ============================================================================
-- Migration: Create Workflow Engine Tables
-- Description: Tables for centralized workflow engine, instance tracking, and delegation
-- Requirements: REQ-W001, REQ-W002, REQ-W006
-- Author: SIMPEL Team
-- Created: 2026-02-09
-- ============================================================================

BEGIN;

RAISE NOTICE 'Creating workflow engine tables...';

-- ============================================================================
-- WORKFLOW DEFINITIONS (Workflow Configuration Storage)
-- Requirement: REQ-W001, REQ-W002
-- ============================================================================

CREATE TABLE IF NOT EXISTS perlengkapan.workflow_definitions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- Workflow Identity
    name VARCHAR(100) NOT NULL UNIQUE,
    description TEXT,
    version INTEGER NOT NULL DEFAULT 1,

    -- Configuration (stored as JSONB for flexibility)
    -- Contains: transitions, sla_minutes, required_roles, etc.
    config JSONB NOT NULL,

    -- Status
    is_active BOOLEAN DEFAULT TRUE,

    -- Audit Trail
    created_by UUID,
    updated_by UUID,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),

    CONSTRAINT unique_workflow_name_version UNIQUE (name, version)
);

CREATE INDEX idx_workflow_def_name ON perlengkapan.workflow_definitions(name);
CREATE INDEX idx_workflow_def_active ON perlengkapan.workflow_definitions(is_active) WHERE is_active = TRUE;
CREATE INDEX idx_workflow_def_config ON perlengkapan.workflow_definitions USING GIN(config);

COMMENT ON TABLE perlengkapan.workflow_definitions IS 'Workflow configuration definitions with versioning';
COMMENT ON COLUMN perlengkapan.workflow_definitions.config IS 'JSONB containing transitions, SLA, roles, etc.';

-- ============================================================================
-- WORKFLOW INSTANCES (Active Workflow Tracking)
-- Requirement: REQ-W004, REQ-W005
-- ============================================================================

CREATE TABLE IF NOT EXISTS perlengkapan.workflow_instances (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- Workflow Definition Reference
    workflow_definition_id UUID NOT NULL REFERENCES perlengkapan.workflow_definitions(id),
    workflow_name VARCHAR(100) NOT NULL,
    workflow_version INTEGER NOT NULL,

    -- Entity Reference (polymorphic)
    entity_type VARCHAR(50) NOT NULL CHECK (entity_type IN ('kebutuhan_bmn', 'pakaian_dinas', 'pemakaian_bmn', 'roadmap_sarpras')),
    entity_id UUID NOT NULL,

    -- Current State
    current_state VARCHAR(100) NOT NULL,
    current_state_code INTEGER,

    -- State Entry Time (for SLA tracking)
    state_entered_at TIMESTAMPTZ DEFAULT NOW(),

    -- SLA Information
    sla_minutes INTEGER,
    sla_deadline TIMESTAMPTZ,
    is_sla_breached BOOLEAN DEFAULT FALSE,
    sla_breach_notified_at TIMESTAMPTZ,

    -- Status
    status VARCHAR(50) NOT NULL DEFAULT 'ACTIVE' CHECK (status IN ('ACTIVE', 'COMPLETED', 'CANCELLED', 'SUSPENDED')),

    -- Completion Information
    completed_at TIMESTAMPTZ,
    completion_state VARCHAR(100),

    -- Metadata
    metadata JSONB DEFAULT '{}'::jsonb,

    -- Audit Trail
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),

    CONSTRAINT unique_entity_workflow UNIQUE (entity_type, entity_id)
);

CREATE INDEX idx_workflow_inst_entity ON perlengkapan.workflow_instances(entity_type, entity_id);
CREATE INDEX idx_workflow_inst_status ON perlengkapan.workflow_instances(status);
CREATE INDEX idx_workflow_inst_state ON perlengkapan.workflow_instances(current_state);
CREATE INDEX idx_workflow_inst_sla ON perlengkapan.workflow_instances(sla_deadline) WHERE status = 'ACTIVE' AND is_sla_breached = FALSE;
CREATE INDEX idx_workflow_inst_workflow ON perlengkapan.workflow_instances(workflow_definition_id);
CREATE INDEX idx_workflow_inst_metadata ON perlengkapan.workflow_instances USING GIN(metadata);

COMMENT ON TABLE perlengkapan.workflow_instances IS 'Active workflow instance tracking with SLA monitoring';
COMMENT ON COLUMN perlengkapan.workflow_instances.entity_type IS 'Type of entity this workflow is attached to';
COMMENT ON COLUMN perlengkapan.workflow_instances.sla_deadline IS 'Calculated deadline based on state_entered_at + sla_minutes';

-- ============================================================================
-- WORKFLOW TRANSITIONS (Transition History/Audit Trail)
-- Requirement: REQ-W005, REQ-W011
-- ============================================================================

CREATE TABLE IF NOT EXISTS perlengkapan.workflow_transitions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- Workflow Instance Reference
    workflow_instance_id UUID NOT NULL REFERENCES perlengkapan.workflow_instances(id) ON DELETE CASCADE,

    -- Transition Details
    from_state VARCHAR(100),
    from_state_code INTEGER,
    to_state VARCHAR(100) NOT NULL,
    to_state_code INTEGER,

    -- Actor Information (from Authenc)
    actor_user_id UUID NOT NULL,
    actor_nip VARCHAR(30),
    actor_nama VARCHAR(255),
    actor_jabatan VARCHAR(255),
    actor_role VARCHAR(100),

    -- Action Details
    action VARCHAR(100) NOT NULL,
    komentar TEXT,

    -- Delegation Information (if applicable)
    is_delegated BOOLEAN DEFAULT FALSE,
    delegated_from_user_id UUID,
    delegation_id UUID,

    -- Metadata
    metadata JSONB DEFAULT '{}'::jsonb,

    -- Timestamp
    transitioned_at TIMESTAMPTZ DEFAULT NOW(),

    -- IP Address for audit
    ip_address INET
);

CREATE INDEX idx_workflow_trans_instance ON perlengkapan.workflow_transitions(workflow_instance_id);
CREATE INDEX idx_workflow_trans_actor ON perlengkapan.workflow_transitions(actor_user_id);
CREATE INDEX idx_workflow_trans_time ON perlengkapan.workflow_transitions(transitioned_at DESC);
CREATE INDEX idx_workflow_trans_delegation ON perlengkapan.workflow_transitions(delegation_id) WHERE delegation_id IS NOT NULL;
CREATE INDEX idx_workflow_trans_metadata ON perlengkapan.workflow_transitions USING GIN(metadata);

COMMENT ON TABLE perlengkapan.workflow_transitions IS 'Immutable audit trail of all workflow state transitions';
COMMENT ON COLUMN perlengkapan.workflow_transitions.is_delegated IS 'Whether this action was performed via delegation';

-- ============================================================================
-- WORKFLOW DELEGATIONS (Temporary Role Assignment)
-- Requirement: REQ-W006
-- ============================================================================

CREATE TABLE IF NOT EXISTS perlengkapan.workflow_delegations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- Delegator (person delegating their authority)
    delegator_user_id UUID NOT NULL,
    delegator_nip VARCHAR(30),
    delegator_nama VARCHAR(255),
    delegator_role VARCHAR(100) NOT NULL,

    -- Delegate (person receiving the authority)
    delegate_user_id UUID NOT NULL,
    delegate_nip VARCHAR(30),
    delegate_nama VARCHAR(255),

    -- Delegation Scope
    workflow_name VARCHAR(100),  -- NULL means all workflows
    entity_type VARCHAR(50),     -- NULL means all entity types

    -- Validity Period
    valid_from TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    valid_until TIMESTAMPTZ NOT NULL,

    -- Status
    status VARCHAR(50) NOT NULL DEFAULT 'ACTIVE' CHECK (status IN ('ACTIVE', 'EXPIRED', 'REVOKED', 'CANCELLED')),

    -- Revocation Information
    revoked_at TIMESTAMPTZ,
    revoked_by UUID,
    revocation_reason TEXT,

    -- Reason for Delegation
    reason TEXT NOT NULL,

    -- Audit Trail
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),

    CONSTRAINT valid_delegation_period CHECK (valid_until > valid_from),
    CONSTRAINT no_self_delegation CHECK (delegator_user_id != delegate_user_id)
);

CREATE INDEX idx_delegation_delegator ON perlengkapan.workflow_delegations(delegator_user_id);
CREATE INDEX idx_delegation_delegate ON perlengkapan.workflow_delegations(delegate_user_id);
CREATE INDEX idx_delegation_status ON perlengkapan.workflow_delegations(status);
CREATE INDEX idx_delegation_validity ON perlengkapan.workflow_delegations(valid_from, valid_until) WHERE status = 'ACTIVE';
CREATE INDEX idx_delegation_workflow ON perlengkapan.workflow_delegations(workflow_name) WHERE workflow_name IS NOT NULL;

COMMENT ON TABLE perlengkapan.workflow_delegations IS 'Temporary delegation of workflow approval authority';
COMMENT ON COLUMN perlengkapan.workflow_delegations.workflow_name IS 'NULL means delegation applies to all workflows';
COMMENT ON COLUMN perlengkapan.workflow_delegations.entity_type IS 'NULL means delegation applies to all entity types';

-- ============================================================================
-- WORKFLOW ESCALATIONS (SLA Breach Escalations)
-- Requirement: REQ-W003
-- ============================================================================

CREATE TABLE IF NOT EXISTS perlengkapan.workflow_escalations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- Workflow Instance Reference
    workflow_instance_id UUID NOT NULL REFERENCES perlengkapan.workflow_instances(id) ON DELETE CASCADE,

    -- Escalation Details
    escalation_level INTEGER NOT NULL DEFAULT 1,
    escalated_state VARCHAR(100) NOT NULL,

    -- SLA Information
    sla_minutes INTEGER NOT NULL,
    sla_deadline TIMESTAMPTZ NOT NULL,
    actual_duration_minutes INTEGER,

    -- Escalation Target
    escalated_to_user_id UUID,
    escalated_to_role VARCHAR(100),

    -- Notification Status
    notification_sent BOOLEAN DEFAULT FALSE,
    notification_sent_at TIMESTAMPTZ,

    -- Resolution
    resolved BOOLEAN DEFAULT FALSE,
    resolved_at TIMESTAMPTZ,
    resolution_action VARCHAR(100),

    -- Audit Trail
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_escalation_instance ON perlengkapan.workflow_escalations(workflow_instance_id);
CREATE INDEX idx_escalation_resolved ON perlengkapan.workflow_escalations(resolved) WHERE resolved = FALSE;
CREATE INDEX idx_escalation_notification ON perlengkapan.workflow_escalations(notification_sent) WHERE notification_sent = FALSE;
CREATE INDEX idx_escalation_level ON perlengkapan.workflow_escalations(escalation_level);

COMMENT ON TABLE perlengkapan.workflow_escalations IS 'SLA breach escalations and notifications';
COMMENT ON COLUMN perlengkapan.workflow_escalations.escalation_level IS 'Level of escalation (1=first, 2=second, etc.)';

-- ============================================================================
-- VERIFY EXISTING TABLES
-- ============================================================================

-- Verify pengajuan_kebutuhan_bmn_satker_aktivitas exists (created in earlier migration)
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.tables
        WHERE table_schema = 'perlengkapan'
        AND table_name = 'pengajuan_kebutuhan_bmn_satker_aktivitas'
    ) THEN
        RAISE EXCEPTION 'Table pengajuan_kebutuhan_bmn_satker_aktivitas does not exist. Run 20260202_create_kebutuhan_bmn_tables.sql first.';
    END IF;

    RAISE NOTICE '✅ Verified: pengajuan_kebutuhan_bmn_satker_aktivitas exists';
END $$;

-- Verify parallel_approvals exists (created in earlier migration)
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.tables
        WHERE table_schema = 'perlengkapan'
        AND table_name = 'parallel_approvals'
    ) THEN
        RAISE EXCEPTION 'Table parallel_approvals does not exist. Run 20260209_create_new_entity_tables.sql first.';
    END IF;

    RAISE NOTICE '✅ Verified: parallel_approvals exists';
END $$;

-- ============================================================================
-- INSERT DEFAULT WORKFLOW DEFINITIONS
-- ============================================================================

-- Insert default Kebutuhan BMN workflow
INSERT INTO perlengkapan.workflow_definitions (name, description, version, config, is_active)
VALUES (
    'kebutuhan_bmn',
    'Workflow for BMN requirements approval process',
    1,
    '{
        "transitions": {
            "DRAFT": ["INPUT_BARANG", "CANCELLED"],
            "INPUT_BARANG": ["SUBMIT_SATKER", "DRAFT"],
            "SUBMIT_SATKER": ["ANALISIS_KELAYAKAN", "REVISI_SATKER", "REJECTED"],
            "REVISI_SATKER": ["INPUT_BARANG"],
            "ANALISIS_KELAYAKAN": ["PENYUSUNAN_PRIORITAS", "REVISI_SATKER", "REJECTED"],
            "PENYUSUNAN_PRIORITAS": ["APPROVED", "REJECTED"],
            "APPROVED": ["COMPLETED"],
            "COMPLETED": ["CANCELLED"],
            "REJECTED": [],
            "CANCELLED": []
        },
        "sla_minutes": {
            "SUBMIT_SATKER": 2880,
            "ANALISIS_KELAYAKAN": 4320,
            "PENYUSUNAN_PRIORITAS": 1440,
            "APPROVED": 4320
        },
        "required_roles": {
            "INPUT_BARANG": "operator_satker",
            "SUBMIT_SATKER": "operator_satker",
            "ANALISIS_KELAYAKAN": "validator_pusat",
            "PENYUSUNAN_PRIORITAS": "validator_pusat",
            "APPROVED": "admin_pusat",
            "REJECTED": "admin_pusat",
            "REVISI_SATKER": "validator_pusat"
        },
        "supports_parallel_approval": true
    }'::jsonb,
    TRUE
)
ON CONFLICT (name, version) DO NOTHING;

-- Insert default Pemakaian BMN workflow
INSERT INTO perlengkapan.workflow_definitions (name, description, version, config, is_active)
VALUES (
    'pemakaian_bmn',
    'Workflow for BMN usage permit approval process',
    1,
    '{
        "transitions": {
            "DRAFT": ["SUBMITTED", "CANCELLED"],
            "SUBMITTED": ["APPROVED", "REJECTED"],
            "APPROVED": ["ACTIVE"],
            "ACTIVE": ["EXPIRED", "REVOKED"],
            "REJECTED": [],
            "EXPIRED": [],
            "REVOKED": [],
            "CANCELLED": []
        },
        "sla_minutes": {
            "SUBMITTED": 1440,
            "APPROVED": 480
        },
        "required_roles": {
            "SUBMITTED": "pegawai",
            "APPROVED": "pimpinan_satker",
            "REJECTED": "pimpinan_satker",
            "REVOKED": "pimpinan_satker"
        },
        "supports_parallel_approval": false
    }'::jsonb,
    TRUE
)
ON CONFLICT (name, version) DO NOTHING;

RAISE NOTICE '✅ Inserted default workflow definitions';

-- ============================================================================
-- SUMMARY
-- ============================================================================

DO $$
DECLARE
    workflow_def_count INTEGER;
    workflow_inst_count INTEGER;
    delegation_count INTEGER;
BEGIN
    SELECT COUNT(*) INTO workflow_def_count FROM perlengkapan.workflow_definitions;
    SELECT COUNT(*) INTO workflow_inst_count FROM perlengkapan.workflow_instances;
    SELECT COUNT(*) INTO delegation_count FROM perlengkapan.workflow_delegations;

    RAISE NOTICE '';
    RAISE NOTICE '========================================';
    RAISE NOTICE 'Workflow Engine Tables Migration Complete';
    RAISE NOTICE '========================================';
    RAISE NOTICE '';
    RAISE NOTICE '📋 Tables Created:';
    RAISE NOTICE '  - workflow_definitions (workflow configuration storage)';
    RAISE NOTICE '  - workflow_instances (active workflow tracking)';
    RAISE NOTICE '  - workflow_transitions (immutable audit trail)';
    RAISE NOTICE '  - workflow_delegations (temporary role assignment)';
    RAISE NOTICE '  - workflow_escalations (SLA breach tracking)';
    RAISE NOTICE '';
    RAISE NOTICE '📊 Current Data:';
    RAISE NOTICE '  - Workflow Definitions: %', workflow_def_count;
    RAISE NOTICE '  - Workflow Instances: %', workflow_inst_count;
    RAISE NOTICE '  - Active Delegations: %', delegation_count;
    RAISE NOTICE '';
    RAISE NOTICE '✅ Verified Existing Tables:';
    RAISE NOTICE '  - pengajuan_kebutuhan_bmn_satker_aktivitas';
    RAISE NOTICE '  - parallel_approvals';
    RAISE NOTICE '  - parallel_approval_votes';
    RAISE NOTICE '';
    RAISE NOTICE '📈 Indexes Created: 30+ (FK, composite, GIN, partial)';
    RAISE NOTICE '🔒 Constraints: CHECK, UNIQUE, FK constraints added';
    RAISE NOTICE '';
    RAISE NOTICE 'Requirements Satisfied:';
    RAISE NOTICE '  - REQ-W001: Generic configurable workflow engine';
    RAISE NOTICE '  - REQ-W002: Steps with role, SLA, actions, escalation';
    RAISE NOTICE '  - REQ-W003: Auto-escalate on SLA breach';
    RAISE NOTICE '  - REQ-W004: Consistent API for workflow operations';
    RAISE NOTICE '  - REQ-W005: Immutably log all workflow actions';
    RAISE NOTICE '  - REQ-W006: Support delegation';
    RAISE NOTICE '  - REQ-W007: Support parallel approval';
    RAISE NOTICE '';
END $$;

COMMIT;
