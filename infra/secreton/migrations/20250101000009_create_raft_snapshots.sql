SET search_path = secreton, public;
-- Create raft_snapshots table for storing Raft cluster snapshots
-- This table stores encrypted, compressed snapshots with integrity verification

CREATE TABLE IF NOT EXISTS raft_snapshots (
    -- Unique snapshot identifier
    snapshot_id VARCHAR(255) PRIMARY KEY,

    -- Creation timestamp
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Original size in bytes (before compression)
    size_bytes BIGINT NOT NULL,

    -- Compressed size in bytes
    compressed_size_bytes BIGINT NOT NULL,

    -- Last included log index in snapshot
    last_included_index BIGINT NOT NULL,

    -- Last included term in snapshot
    last_included_term BIGINT NOT NULL,

    -- SHA-256 checksum of encrypted data
    checksum VARCHAR(64) NOT NULL,

    -- Encrypted and compressed snapshot data
    encrypted_data BYTEA NOT NULL,

    -- Ed25519 signature (hex-encoded)
    signature VARCHAR(128),

    -- Metadata for additional information
    metadata JSONB DEFAULT '{}'::jsonb,

    -- Indexes for efficient querying
    CONSTRAINT raft_snapshots_size_check CHECK (size_bytes > 0),
    CONSTRAINT raft_snapshots_compressed_size_check CHECK (compressed_size_bytes > 0)
);

-- Index for listing snapshots by creation time
CREATE INDEX IF NOT EXISTS idx_raft_snapshots_created_at ON raft_snapshots(created_at DESC);

-- Index for querying by log index
CREATE INDEX IF NOT EXISTS idx_raft_snapshots_last_included_index ON raft_snapshots(last_included_index);

-- Index for querying by term
CREATE INDEX IF NOT EXISTS idx_raft_snapshots_last_included_term ON raft_snapshots(last_included_term);

-- Comments for documentation
COMMENT ON TABLE raft_snapshots IS 'Stores encrypted Raft cluster snapshots for backup and restore operations';
COMMENT ON COLUMN raft_snapshots.snapshot_id IS 'Unique identifier for the snapshot (format: snapshot-{timestamp}-{uuid})';
COMMENT ON COLUMN raft_snapshots.created_at IS 'Timestamp when the snapshot was created';
COMMENT ON COLUMN raft_snapshots.size_bytes IS 'Original size of snapshot data before compression';
COMMENT ON COLUMN raft_snapshots.compressed_size_bytes IS 'Size of snapshot data after compression';
COMMENT ON COLUMN raft_snapshots.last_included_index IS 'Last log index included in this snapshot';
COMMENT ON COLUMN raft_snapshots.last_included_term IS 'Last term included in this snapshot';
COMMENT ON COLUMN raft_snapshots.checksum IS 'SHA-256 checksum of encrypted data for integrity verification';
COMMENT ON COLUMN raft_snapshots.encrypted_data IS 'Encrypted and compressed snapshot data';
COMMENT ON COLUMN raft_snapshots.signature IS 'Ed25519 signature for authenticity verification';
COMMENT ON COLUMN raft_snapshots.metadata IS 'Additional metadata in JSON format';
