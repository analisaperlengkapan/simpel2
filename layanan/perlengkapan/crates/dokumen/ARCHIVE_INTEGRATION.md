# Document Archival Integration

## Overview

This document describes the automatic document archival and retention policy implementation for the SIMPEL document service.

## Features

### 1. Automatic Document Archival

Documents are automatically archived after workflow completion:

- **Trigger**: When workflow state reaches `COMPLETED`
- **Process**:
  - Document is copied to archive storage (`/var/data/dokumen/archive`)
  - Document status is updated to `is_archived = true`
  - Archive path is stored in document metadata
  - Original document is kept for safety

### 2. Retention Policy

Different document types have different retention periods:

| Document Type | Retention Period |
|--------------|------------------|
| SK (Surat Keputusan) | 5 years |
| SK Penghapusan | 5 years |
| SK Kebutuhan | 5 years |
| Rekapitulasi | 3 years |
| Rekap Pakaian Dinas | 3 years |
| Default | 3 years |

### 3. Automatic Deletion

Documents past their retention period are automatically deleted:

- **Schedule**: Daily at 02:00 AM
- **Process**:
  - Check all archived documents
  - Calculate age based on creation date
  - Delete documents older than retention period
  - Delete both physical file and database record

### 4. Archive Search

Search archived documents with filters:

- Date range (from/to)
- Document type
- Satker ID
- Text search (filename or metadata)
- Pagination support

## API Endpoints

### Get Archived Document

```http
GET /archive/documents/{id}
```

Returns the archived document file.

**Response**: Binary file with appropriate content-type and filename headers.

### Search Archived Documents

```http
GET /archive/documents/search?date_from=2024-01-01&date_to=2024-12-31&document_type=SK&satker_id=uuid&query=search&limit=20&offset=0
```

**Query Parameters**:
- `date_from` (optional): Start date (ISO 8601)
- `date_to` (optional): End date (ISO 8601)
- `document_type` (optional): Document type filter
- `satker_id` (optional): Satker UUID filter
- `query` (optional): Text search query
- `limit` (optional): Results per page (default: 20)
- `offset` (optional): Pagination offset (default: 0)

**Response**:
```json
[
  {
    "id": "uuid",
    "filename": "SK_Penghapusan_2024.pdf",
    "content_type": "application/pdf",
    "size": 1024000,
    "storage_path": "archive_SK_Penghapusan_2024.pdf",
    "owner_id": "uuid",
    "created_at": "2024-01-15T10:30:00Z",
    "updated_at": "2024-01-15T10:30:00Z",
    "is_archived": true,
    "checksum": "sha256hash",
    "encrypted": true,
    "current_version": 1,
    "metadata": {
      "document_type": "SK",
      "satker_id": "uuid",
      "workflow_state": "COMPLETED",
      "archive_path": "archive_SK_Penghapusan_2024.pdf",
      "retention_years": 5
    }
  }
]
```

### Delete Expired Documents (Admin Only)

```http
DELETE /archive/documents/expired
```

Manually trigger deletion of expired documents.

**Response**:
```json
{
  "deleted_count": 42,
  "message": "Deleted 42 expired documents"
}
```

## Configuration

### Environment Variables

```bash
# Archive storage path (separate from main storage)
ARCHIVE_STORAGE_PATH=/var/data/dokumen/archive

# Main storage path
STORAGE_PATH=/var/data/dokumen
```

### Retention Policy Configuration

Retention policies are defined in `src/archive.rs`:

```rust
impl RetentionPolicy {
    pub fn for_document_type(document_type: &str) -> Self {
        match document_type {
            "SK" | "sk_penghapusan" | "sk_kebutuhan" => Self {
                document_type: document_type.to_string(),
                retention_years: 5,
            },
            "rekapitulasi" | "rekap_pakaian_dinas" => Self {
                document_type: document_type.to_string(),
                retention_years: 3,
            },
            _ => Self {
                document_type: document_type.to_string(),
                retention_years: 3, // Default 3 years
            },
        }
    }
}
```

## Scheduler

The document scheduler runs in the background and performs:

1. **Archival Check** (Daily at 02:00 AM):
   - Find documents with `workflow_state = 'COMPLETED'` and `is_archived = false`
   - Archive each document to archive storage
   - Update document status

2. **Expiry Check** (Daily at 02:00 AM):
   - Find all archived documents
   - Check age against retention policy
   - Delete expired documents

## Database Schema

### Documents Table

```sql
CREATE TABLE dokumen.documents (
    id UUID PRIMARY KEY,
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
```

### Metadata Structure

```json
{
  "document_type": "SK",
  "satker_id": "uuid",
  "workflow_state": "COMPLETED",
  "archive_path": "archive_filename.pdf",
  "retention_years": 5
}
```

## Integration with Workflow

When a workflow transitions to `COMPLETED` state:

1. Workflow engine calls document service to generate document
2. Document is stored in main storage
3. Document metadata includes `workflow_state: "COMPLETED"`
4. Scheduler detects completed workflow documents
5. Document is automatically archived to archive storage
6. After retention period, document is automatically deleted

## Security

- **Access Control**: Only authorized users can retrieve archived documents
- **Audit Logging**: All archive operations are logged
- **Encryption**: Archived documents are encrypted at rest
- **Checksum Verification**: SHA-256 checksums ensure integrity

## Monitoring

The scheduler logs all operations:

```
INFO: Running scheduled document archival and cleanup
INFO: Archived document {uuid}
INFO: Archived 5 documents
INFO: Deleted expired document {uuid} (type: SK, age: 6 years)
INFO: Deleted 3 expired documents
```

## Testing

To test the archival integration:

1. Create a document with `workflow_state: "COMPLETED"` in metadata
2. Wait for scheduler to run (or trigger manually)
3. Verify document is archived (`is_archived = true`)
4. Verify archive file exists in archive storage
5. Test retrieval via API endpoint
6. Test search with various filters

## Compliance

This implementation satisfies:

- **REQ-D012**: Document retention policy
- **NFR-A005**: Recovery Point Objective (RPO) ≤ 1 hour

## Future Enhancements

- [ ] Support for MinIO/S3 archive storage
- [ ] Lifecycle policies for automatic tiering (hot → cold → glacier)
- [ ] Compression for archived documents
- [ ] Batch archival API for manual operations
- [ ] Archive restoration workflow
- [ ] Legal hold support (prevent deletion)
- [ ] Compliance reporting (retention compliance dashboard)
