# SIMPEL API Documentation

## Overview

This document provides comprehensive API documentation for SIMPEL (Sistem Informasi Manajemen Perlengkapan). All APIs follow REST principles and return JSON responses.

**Base URL:** `https://simpel.kejaksaan.go.id/api/v1`

**Authentication:** All endpoints require JWT Bearer token in the `Authorization` header.

```
Authorization: Bearer <jwt_token>
```

---

## Table of Contents

1. [Kebutuhan BMN API](#kebutuhan-bmn-api)
2. [Pakaian Dinas API](#pakaian-dinas-api)
3. [Pemakaian BMN API](#pemakaian-bmn-api)
4. [Penghapusan BMN API](#penghapusan-bmn-api)
5. [Dashboard API](#dashboard-api)
6. [Search API](#search-api)
7. [Common Response Formats](#common-response-formats)
8. [Error Codes](#error-codes)

---

## Kebutuhan BMN API

### List Pengajuan Kebutuhan BMN

**Endpoint:** `GET /kebutuhan-bmn/pengajuan`

**Description:** Retrieve list of kebutuhan BMN submissions with pagination and filters.

**Query Parameters:**

- `page` (integer, optional): Page number (default: 1)
- `per_page` (integer, optional): Items per page (default: 20, max: 100)
- `tahun` (integer, optional): Filter by year
- `status_kode` (integer, optional): Filter by status code
- `satker_id` (string, optional): Filter by satker ID
- `search` (string, optional): Search by name

**Response:**

```json
{
  "success": true,
  "message": "Success",
  "data": {
    "items": [
      {
        "id": "uuid",
        "nama": "Kebutuhan BMN 2024",
        "tahun": 2024,
        "tgl_mulai": "2024-01-01",
        "tgl_selesai": "2024-12-31",
        "status_kode": 2000,
        "status_nama": "DRAFT",
        "created_at": "2024-01-01T00:00:00Z"
      }
    ],
    "total": 100,
    "page": 1,
    "per_page": 20
  }
}
```

### Create Pengajuan

**Endpoint:** `POST /kebutuhan-bmn/pengajuan`

**Description:** Create new kebutuhan BMN period (Validator Pusat only).

**Request Body:**

```json
{
  "nama": "Kebutuhan BMN 2024",
  "tahun": 2024,
  "tgl_mulai": "2024-01-01",
  "tgl_selesai": "2024-12-31",
  "pilihan_satker": "semua",
  "id_jenis_asset": ["uuid1", "uuid2"]
}
```

**Response:**

```json
{
  "success": true,
  "message": "Pengajuan created successfully",
  "data": {
    "id": "uuid",
    "nama": "Kebutuhan BMN 2024",
    "tahun": 2024,
    "status_kode": 2000
  }
}
```

### Get Pengajuan Detail

**Endpoint:** `GET /kebutuhan-bmn/pengajuan/{id}`

**Description:** Get detailed information about a specific pengajuan.

**Path Parameters:**

- `id` (uuid, required): Pengajuan ID

**Response:**

```json
{
  "success": true,
  "message": "Success",
  "data": {
    "id": "uuid",
    "nama": "Kebutuhan BMN 2024",
    "tahun": 2024,
    "tgl_mulai": "2024-01-01",
    "tgl_selesai": "2024-12-31",
    "pilihan_satker": "semua",
    "id_jenis_asset": ["uuid1", "uuid2"],
    "status_kode": 2000,
    "status_nama": "DRAFT",
    "satker_count": 50,
    "barang_count": 150,
    "created_at": "2024-01-01T00:00:00Z",
    "updated_at": "2024-01-01T00:00:00Z"
  }
}
```

### Transition Pengajuan Status

**Endpoint:** `POST /kebutuhan-bmn/pengajuan/{id}/transition`

**Description:** Transition pengajuan to next status in workflow.

**Path Parameters:**

- `id` (uuid, required): Pengajuan ID

**Request Body:**

```json
{
  "target_status": 2001,
  "catatan": "Optional notes"
}
```

**Response:**

```json
{
  "success": true,
  "message": "Status transitioned successfully",
  "data": {
    "id": "uuid",
    "status_kode": 2001,
    "status_nama": "INPUT_BARANG"
  }
}
```

### Search SIMAN Assets

**Endpoint:** `GET /kebutuhan-bmn/siman/search`

**Description:** Search for existing BMN assets from SIMAN integration.

**Query Parameters:**

- `search` (string, required): Search query
- `satker_id` (string, optional): Filter by satker
- `kategori` (string, optional): Filter by category
- `limit` (integer, optional): Max results (default: 50)

**Response:**

```json
{
  "success": true,
  "message": "Success",
  "data": [
    {
      "nup": "123456",
      "kode_barang": "3.1.01.01.001",
      "nama_barang": "Kendaraan Roda 4",
      "kondisi": "BAIK",
      "tahun_perolehan": 2020,
      "nilai_perolehan": 250000000,
      "satker_nama": "Kejaksaan Negeri Jakarta Pusat"
    }
  ]
}
```

---

## Pakaian Dinas API

### List Jenis Pakaian Dinas

**Endpoint:** `GET /pakaian-dinas/jenis`

**Description:** Get list of uniform types.

**Response:**

```json
{
  "success": true,
  "message": "Success",
  "data": [
    {
      "id": "uuid",
      "nama": "PDH (Pakaian Dinas Harian)",
      "kode": "PDH",
      "deskripsi": "Pakaian dinas harian untuk pegawai",
      "is_active": true
    }
  ]
}
```

### Create Pengajuan Pakaian Dinas

**Endpoint:** `POST /pakaian-dinas/pengajuan`

**Description:** Create new pakaian dinas period.

**Request Body:**

```json
{
  "nama": "Pakaian Dinas 2024",
  "tahun": 2024,
  "pilihan_satker": "sebagian",
  "dengan_unit_kerja": false,
  "jenis_pakaian_dinas_id": "uuid"
}
```

**Response:**

```json
{
  "success": true,
  "message": "Pengajuan created successfully",
  "data": {
    "id": "uuid",
    "nama": "Pakaian Dinas 2024",
    "tahun": 2024,
    "aktivitas_id": 1000
  }
}
```

### Get Pegawai with Sizes

**Endpoint:** `GET /pakaian-dinas/pegawai-satker/{satker_id}/with-sizes`

**Description:** Get employees with their uniform sizes for a satker.

**Path Parameters:**

- `satker_id` (string, required): Satker ID

**Query Parameters:**

- `pengajuan_id` (uuid, optional): Filter by pengajuan

**Response:**

```json
{
  "success": true,
  "message": "Success",
  "data": [
    {
      "nip": "199203142014031001",
      "nama": "John Doe",
      "jenis_kelamin": "L",
      "eselon": "IV/a",
      "jenis": "Jaksa",
      "ukuran": {
        "BAJU": "L",
        "CELANA": "32",
        "SEPATU": "42"
      },
      "with_hijab": false
    }
  ]
}
```

### Generate Laporan Rekap

**Endpoint:** `GET /pakaian-dinas/laporan/rekap-ukuran`

**Description:** Generate size summary report.

**Query Parameters:**

- `pengajuan_id` (uuid, required): Pengajuan ID
- `format` (string, optional): "pdf" or "excel" (default: "pdf")

**Response:** Binary file download (PDF or Excel)

---

## Pemakaian BMN API

### List Permits

**Endpoint:** `GET /pemakaian-bmn`

**Description:** Get list of BMN usage permits.

**Query Parameters:**

- `page` (integer, optional): Page number
- `per_page` (integer, optional): Items per page
- `status` (string, optional): Filter by status
- `pegawai_nip` (string, optional): Filter by employee NIP
- `satker_id` (string, optional): Filter by satker
- `search` (string, optional): Search query

**Response:**

```json
{
  "success": true,
  "message": "Success",
  "data": {
    "items": [
      {
        "id": "uuid",
        "nomor_izin": "IZN/2024/0100/001",
        "satker_id": "uuid",
        "pegawai_nip": "199203142014031001",
        "pegawai_nama": "John Doe",
        "tanggal_mulai": "2024-01-01",
        "tanggal_selesai": "2024-12-31",
        "status": "ACTIVE",
        "bmn_count": 2,
        "created_at": "2024-01-01T00:00:00Z"
      }
    ],
    "total": 50,
    "page": 1,
    "per_page": 20
  }
}
```

### Create Permit

**Endpoint:** `POST /pemakaian-bmn`

**Description:** Create new BMN usage permit.

**Request Body:**

```json
{
  "satker_id": "uuid",
  "pegawai_nip": "199203142014031001",
  "pegawai_nama": "John Doe",
  "tanggal_mulai": "2024-01-01",
  "tanggal_selesai": "2024-12-31"
}
```

**Response:**

```json
{
  "success": true,
  "message": "Permit created successfully",
  "data": {
    "id": "uuid",
    "status": "DRAFT"
  }
}
```

### Check BMN Availability

**Endpoint:** `GET /pemakaian-bmn/bmn/{bmn_nup}/availability`

**Description:** Check if a BMN is available for use (REQ-P003, REQ-P004).

**Path Parameters:**

- `bmn_nup` (string, required): BMN NUP

**Response:**

```json
{
  "success": true,
  "message": "Success",
  "data": {
    "bmn_nup": "123456",
    "is_available": false,
    "active_permit_id": "uuid",
    "active_permit_holder": "John Doe",
    "active_permit_expires": "2024-12-31"
  }
}
```

### Generate Draft Permit Document

**Endpoint:** `POST /pemakaian-bmn/{id}/generate-konsep-surat`

**Description:** Generate draft permit document in DOCX format (REQ-P006, REQ-P007).

**Path Parameters:**

- `id` (uuid, required): Permit ID

**Response:**

```json
{
  "success": true,
  "message": "Document generated successfully",
  "data": {
    "document_id": "uuid",
    "document_url": "https://storage.simpel.kejaksaan.go.id/permits/draft-uuid.docx"
  }
}
```

### Upload Signed Permit

**Endpoint:** `POST /pemakaian-bmn/{id}/upload-signed-pdf`

**Description:** Upload signed permit document (REQ-P008, REQ-P009).

**Path Parameters:**

- `id` (uuid, required): Permit ID

**Request:** Multipart form data with `file` field (PDF)

**Response:**

```json
{
  "success": true,
  "message": "Signed document uploaded successfully",
  "data": {
    "signed_document_url": "https://storage.simpel.kejaksaan.go.id/permits/signed-uuid.pdf",
    "status": "COMPLETED"
  }
}
```

### Get Active Usage Dashboard

**Endpoint:** `GET /pemakaian-bmn/monitoring/active-usage`

**Description:** Get monitoring dashboard data (REQ-P016, REQ-P017).

**Query Parameters:**

- `satker_id` (string, optional): Filter by satker
- `jenis_bmn` (string, optional): Filter by BMN type

**Response:**

```json
{
  "success": true,
  "message": "Success",
  "data": {
    "total_active_permits": 150,
    "permits_by_jenis_bmn": [
      {
        "jenis_bmn": "Kendaraan Bermotor",
        "count": 50,
        "percentage": 33.3
      }
    ],
    "permits_by_satker": [
      {
        "satker_id": "uuid",
        "satker_nama": "Kejaksaan Negeri Jakarta Pusat",
        "active_permits": 10
      }
    ],
    "expiring_soon": [
      {
        "id": "uuid",
        "nomor_izin": "IZN/2024/0100/001",
        "bmn_nama": "Toyota Avanza",
        "pegawai_nama": "John Doe",
        "tanggal_selesai": "2024-12-31",
        "days_until_expiry": 7
      }
    ]
  }
}
```

### Get BMN Utilization Report

**Endpoint:** `GET /pemakaian-bmn/monitoring/utilization-report`

**Description:** Get BMN utilization statistics (REQ-P021).

**Query Parameters:**

- `satker_id` (string, optional): Filter by satker
- `jenis_bmn` (string, optional): Filter by BMN type

**Response:**

```json
{
  "success": true,
  "message": "Success",
  "data": {
    "total_bmn": 500,
    "bmn_with_active_permits": 150,
    "bmn_without_permits": 350,
    "utilization_rate": 30.0,
    "bmn_by_type": [
      {
        "jenis_bmn": "Kendaraan Bermotor",
        "total_bmn": 100,
        "utilized_bmn": 50,
        "utilization_rate": 50.0
      }
    ]
  }
}
```

---

## Penghapusan BMN API

### List Penghapusan Requests

**Endpoint:** `GET /penghapusan-bmn`

**Description:** Get list of BMN deletion requests.

**Query Parameters:**

- `page` (integer, optional): Page number
- `per_page` (integer, optional): Items per page
- `status` (string, optional): Filter by status
- `satker_id` (string, optional): Filter by satker

**Response:**

```json
{
  "success": true,
  "message": "Success",
  "data": {
    "items": [
      {
        "id": "uuid",
        "satker_id": "uuid",
        "satker_nama": "Kejaksaan Negeri Jakarta Pusat",
        "asset_count": 5,
        "status": "DRAFT",
        "created_at": "2024-01-01T00:00:00Z"
      }
    ],
    "total": 20,
    "page": 1,
    "per_page": 20
  }
}
```

### Create Penghapusan Request

**Endpoint:** `POST /penghapusan-bmn`

**Description:** Create new BMN deletion request (REQ-PH001).

**Request Body:**

```json
{
  "satker_id": "uuid",
  "assets": [
    {
      "asset_id": "uuid",
      "kode_barang": "3.1.01.01.001",
      "nama_barang": "Kendaraan Roda 4",
      "nup": "123456",
      "alasan": "Rusak berat tidak dapat diperbaiki",
      "metode_penghapusan": "DIMUSNAHKAN"
    }
  ]
}
```

**Response:**

```json
{
  "success": true,
  "message": "Penghapusan request created successfully",
  "data": {
    "id": "uuid",
    "status": "DRAFT"
  }
}
```

### Generate SK Penghapusan

**Endpoint:** `POST /penghapusan-bmn/{id}/generate-sk`

**Description:** Generate SK Penghapusan document (REQ-PH008, REQ-PH009).

**Path Parameters:**

- `id` (uuid, required): Penghapusan request ID

**Response:**

```json
{
  "success": true,
  "message": "SK document generated successfully",
  "data": {
    "document_id": "uuid",
    "document_url": "https://storage.simpel.kejaksaan.go.id/sk/draft-uuid.docx",
    "sk_number": "SK/2024/001"
  }
}
```

### Upload Signed SK

**Endpoint:** `POST /penghapusan-bmn/{id}/upload-signed-sk`

**Description:** Upload signed SK document (REQ-PH010, REQ-PH011).

**Path Parameters:**

- `id` (uuid, required): Penghapusan request ID

**Request:** Multipart form data with `file` field (PDF)

**Response:**

```json
{
  "success": true,
  "message": "Signed SK uploaded successfully",
  "data": {
    "signed_document_url": "https://storage.simpel.kejaksaan.go.id/sk/signed-uuid.pdf",
    "status": "COMPLETED"
  }
}
```

---

## Dashboard API

### Get Perlengkapan Dashboard

**Endpoint:** `GET /dashboard/perlengkapan`

**Description:** Get perlengkapan dashboard metrics.

**Query Parameters:**

- `satker_id` (string, optional): Filter by satker
- `wilayah_code` (string, optional): Filter by wilayah

**Response:**

```json
{
  "success": true,
  "message": "Success",
  "data": {
    "kebutuhan_metrics": {
      "total_pengajuan": 100,
      "draft": 20,
      "submitted": 30,
      "approved": 40,
      "rejected": 10
    },
    "pemakaian_metrics": {
      "total_permits": 150,
      "active": 120,
      "expired": 20,
      "revoked": 10
    },
    "penghapusan_metrics": {
      "total_requests": 50,
      "pending": 20,
      "approved": 25,
      "rejected": 5
    },
    "gap_analysis": {
      "total_gap": 500,
      "high_priority": 100,
      "medium_priority": 200,
      "low_priority": 200
    }
  }
}
```

---

## Search API

### Global Search

**Endpoint:** `GET /search`

**Description:** Search across all modules (Task 11.2).

**Query Parameters:**

- `q` (string, required): Search query
- `module` (string, optional): Filter by module ("kebutuhan", "pemakaian", "penghapusan")
- `status` (string, optional): Filter by status
- `tahun` (integer, optional): Filter by year
- `page` (integer, optional): Page number
- `per_page` (integer, optional): Items per page

**Response:**

```json
{
  "success": true,
  "message": "Success",
  "data": {
    "results": [
      {
        "id": "uuid",
        "module": "kebutuhan",
        "title": "Kebutuhan BMN 2024",
        "description": "Pengajuan kebutuhan BMN untuk tahun 2024",
        "status": "APPROVED",
        "tahun": 2024,
        "satker_nama": "Kejaksaan Negeri Jakarta Pusat",
        "created_at": "2024-01-01T00:00:00Z",
        "url": "/kebutuhan-bmn/pengajuan/uuid"
      }
    ],
    "total": 50,
    "page": 1,
    "per_page": 20
  }
}
```

---

## Common Response Formats

### Success Response

```json
{
  "success": true,
  "message": "Operation successful",
  "data": { /* response data */ }
}
```

### Error Response

```json
{
  "success": false,
  "message": "Error message",
  "error": {
    "code": "ERROR_CODE",
    "details": "Detailed error information"
  }
}
```

### Paginated Response

```json
{
  "success": true,
  "message": "Success",
  "data": {
    "items": [ /* array of items */ ],
    "total": 100,
    "page": 1,
    "per_page": 20,
    "total_pages": 5
  }
}
```

---

## Error Codes

| Code | HTTP Status | Description |
|------|-------------|-------------|
| `UNAUTHORIZED` | 401 | Missing or invalid authentication token |
| `FORBIDDEN` | 403 | Insufficient permissions |
| `NOT_FOUND` | 404 | Resource not found |
| `VALIDATION_ERROR` | 400 | Request validation failed |
| `CONFLICT` | 409 | Resource conflict (e.g., duplicate) |
| `INTERNAL_ERROR` | 500 | Internal server error |
| `SERVICE_UNAVAILABLE` | 503 | Service temporarily unavailable |

---

## Rate Limiting

All API endpoints are rate-limited to:

- **100 requests per minute** per user
- **1000 requests per hour** per user

Rate limit headers are included in responses:

```
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 95
X-RateLimit-Reset: 1640000000
```

---

## Versioning

The API uses URL versioning. Current version is `v1`.

Future versions will be available at `/api/v2`, `/api/v3`, etc.

---

## Support

For API support, contact:

- **Email:** support@simpel.kejaksaan.go.id
- **Documentation:** https://docs.simpel.kejaksaan.go.id

---

**Last Updated:** February 2026
**Version:** 1.0.0
