-- Migration: Create batch operation log table
-- Date: 2026-02-10
-- Purpose: Track batch operations for audit trail
-- Requirements: REQ-K004

-- Create batch operation log table
CREATE TABLE IF NOT EXISTS perlengkapan.batch_operation_log (
    id BIGSERIAL PRIMARY KEY,
    batch_id UUID NOT NULL,
    operation_type VARCHAR(100) NOT NULL,
    total_items INTEGER NOT NULL,
    successful_items INTEGER NOT NULL,
    failed_items INTEGER NOT NULL,
    user_id UUID REFERENCES authenc.users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes for efficient querying
CREATE INDEX idx_batch_operation_log_batch_id ON perlengkapan.batch_operation_log(batch_id);
CREATE INDEX idx_batch_operation_log_user_id ON perlengkapan.batch_operation_log(user_id);
CREATE INDEX idx_batch_operation_log_created_at ON perlengkapan.batch_operation_log(created_at DESC);
CREATE INDEX idx_batch_operation_log_operation_type ON perlengkapan.batch_operation_log(operation_type);

-- Add comments
COMMENT ON TABLE perlengkapan.batch_operation_log IS 'Audit log for batch operations on kebutuhan BMN';
COMMENT ON COLUMN perlengkapan.batch_operation_log.batch_id IS 'Unique identifier for the batch operation';
COMMENT ON COLUMN perlengkapan.batch_operation_log.operation_type IS 'Type of batch operation (batch_approve, batch_reject, batch_update_status)';
COMMENT ON COLUMN perlengkapan.batch_operation_log.total_items IS 'Total number of items in the batch';
COMMENT ON COLUMN perlengkapan.batch_operation_log.successful_items IS 'Number of items successfully processed';
COMMENT ON COLUMN perlengkapan.batch_operation_log.failed_items IS 'Number of items that failed processing';
COMMENT ON COLUMN perlengkapan.batch_operation_log.user_id IS 'User who initiated the batch operation';
COMMENT ON COLUMN perlengkapan.batch_operation_log.created_at IS 'Timestamp when the batch operation was executed';
