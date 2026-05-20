# Pemakaian BMN Monitoring Implementation

## Overview

This document describes the implementation of the pemakaian BMN monitoring dashboard (Task 27.3), which provides comprehensive monitoring and reporting capabilities for BMN usage permits.

## Requirements Implemented

- **REQ-P011**: Active usage monitoring dashboard
- **REQ-P012**: Usage history per BMN and per pegawai (already implemented in 27.1)
- **REQ-P013**: BMN utilization report

## Implementation Details

### 1. Data Models (`src/pemakaian_bmn/models.rs`)

Added new monitoring-specific models:

#### ActiveUsageMonitoringDashboard

Provides real-time overview of active permits:

- Total active permits count
- Permits grouped by BMN type (with percentages)
- Permits grouped by satker (top 10)
- Permits expiring soon (next 30 days)
- Recent activations (last 7 days)

#### BmnUtilizationReport

Comprehensive utilization analysis:

- Total BMN count (from SIMAN integration)
- BMN with active permits
- BMN without permits
- Overall utilization rate
- Utilization by BMN type
- Top 10 most utilized BMN
- Underutilized BMN (no permits in last 180 days)

#### Supporting Models

- `PermitsByJenisBmn`: Permits grouped by type with count and percentage
- `PermitsBySatker`: Active permits per satker
- `ExpiringPermitInfo`: Permit expiry information with days until expiry
- `RecentActivationInfo`: Recently activated permits
- `BmnUtilizationByType`: Utilization metrics per BMN type
- `TopUtilizedBmn`: Most frequently used BMN with usage statistics
- `UnderutilizedBmn`: BMN with low or no usage
- `MonitoringDashboardQuery`: Query parameters for filtering dashboard data

### 2. Repository Methods (`src/pemakaian_bmn/repository.rs`)

#### get_active_usage_dashboard()

Fetches active usage monitoring data with optional filters:

- Supports filtering by satker_id and jenis_bmn
- Aggregates data from multiple queries for comprehensive dashboard
- Calculates percentages and statistics

**SQL Queries:**

1. Total active permits count
2. Permits grouped by jenis_bmn with counts
3. Permits grouped by satker (top 10)
4. Permits expiring in next 30 days
5. Recently activated permits (last 7 days)

#### get_bmn_utilization_report()

Generates comprehensive utilization report:

- Integrates with SIMAN data (integrasi schema)
- Calculates utilization rates
- Identifies top utilized and underutilized BMN

**SQL Queries:**

1. Total BMN count from SIMAN (kondisi = 'BAIK')
2. BMN with active permits
3. Utilization by BMN type (using kode_barang patterns)
4. Top 10 most utilized BMN (by permit count and days used)
5. Underutilized BMN (no permits in last 180 days)

**BMN Type Classification:**

- `03.01%` → KENDARAAN_BERMOTOR
- `03.02%` → RUMAH_NEGARA
- `03.03%` → LAPTOP
- Others → LAINNYA

### 3. Service Methods (`src/pemakaian_bmn/services.rs`)

Added two monitoring service methods:

#### get_active_usage_dashboard()

- Delegates to repository method
- Logs dashboard access
- Returns `ActiveUsageMonitoringDashboard`

#### get_bmn_utilization_report()

- Delegates to repository method
- Logs report generation
- Returns `BmnUtilizationReport`

### 4. HTTP Handlers (`src/pemakaian_bmn/handlers.rs`)

Added two new REST API endpoints:

#### GET /pemakaian-bmn/monitoring/active-usage

Returns active usage monitoring dashboard data.

**Query Parameters:**

- `satker_id` (optional): Filter by satker
- `jenis_bmn` (optional): Filter by BMN type
- `start_date` (optional): Filter by date range start
- `end_date` (optional): Filter by date range end

**Response:**

```json
{
  "success": true,
  "message": "Active usage dashboard retrieved successfully",
  "data": {
    "total_active_permits": 150,
    "permits_by_jenis_bmn": [
      {
        "jenis_bmn": "KENDARAAN_BERMOTOR",
        "count": 80,
        "percentage": 53.33
      }
    ],
    "permits_by_satker": [
      {
        "satker_id": "uuid",
        "satker_nama": "Kejari Jakarta Pusat",
        "active_permits": 25
      }
    ],
    "expiring_soon": [
      {
        "id": "uuid",
        "nomor_izin": "IP/2026/02/0001",
        "bmn_nama": "Toyota Avanza",
        "pegawai_nama": "John Doe",
        "tanggal_selesai": "2026-03-15",
        "days_until_expiry": 15
      }
    ],
    "recent_activations": [
      {
        "id": "uuid",
        "nomor_izin": "IP/2026/02/0002",
        "bmn_nama": "Honda Civic",
        "pegawai_nama": "Jane Smith",
        "activated_at": "2026-02-08T10:30:00Z"
      }
    ]
  }
}
```

#### GET /pemakaian-bmn/monitoring/utilization-report

Returns BMN utilization report.

**Query Parameters:**

- `satker_id` (optional): Filter by satker
- `jenis_bmn` (optional): Filter by BMN type
- `start_date` (optional): Filter by date range start
- `end_date` (optional): Filter by date range end

**Response:**

```json
{
  "success": true,
  "message": "BMN utilization report generated successfully",
  "data": {
    "total_bmn": 500,
    "bmn_with_active_permits": 150,
    "bmn_without_permits": 350,
    "utilization_rate": 30.0,
    "bmn_by_type": [
      {
        "jenis_bmn": "KENDARAAN_BERMOTOR",
        "total_bmn": 200,
        "utilized_bmn": 80,
        "utilization_rate": 40.0
      }
    ],
    "top_utilized_bmn": [
      {
        "bmn_nup": "NUP001",
        "bmn_nama": "Toyota Avanza",
        "jenis_bmn": "KENDARAAN_BERMOTOR",
        "total_permits": 15,
        "total_days_used": 450,
        "current_holder": "John Doe"
      }
    ],
    "underutilized_bmn": [
      {
        "bmn_nup": "NUP500",
        "bmn_nama": "Laptop Dell",
        "jenis_bmn": "LAPTOP",
        "last_used_date": "2025-08-15",
        "days_since_last_use": 180
      }
    ]
  }
}
```

### 5. Routes (`src/routes.rs`)

Registered two new monitoring routes:

- `GET /pemakaian-bmn/monitoring/active-usage`
- `GET /pemakaian-bmn/monitoring/utilization-report`

Both routes require JWT authentication via middleware.

## Integration with Existing Features

### SIMAN Integration

The utilization report integrates with SIMAN data from the `integrasi` schema:

- Queries `integrasi.siman_aset_tanah` for total BMN count
- Filters by `kondisi = 'BAIK'` to only count usable BMN
- Uses `kode_barang` patterns to classify BMN types

### Existing History Endpoints

The monitoring dashboard complements existing history endpoints:

- `GET /pemakaian-bmn/bmn/:bmn_nup/history` - Detailed history per BMN
- `GET /pemakaian-bmn/pegawai/:pegawai_nip/history` - Detailed history per pegawai

## Usage Examples

### Active Usage Dashboard

```bash
# Get overall active usage dashboard
curl -H "Authorization: Bearer $TOKEN" \
  http://localhost:3020/api/v1/pemakaian-bmn/monitoring/active-usage

# Filter by satker
curl -H "Authorization: Bearer $TOKEN" \
  "http://localhost:3020/api/v1/pemakaian-bmn/monitoring/active-usage?satker_id=uuid"

# Filter by BMN type
curl -H "Authorization: Bearer $TOKEN" \
  "http://localhost:3020/api/v1/pemakaian-bmn/monitoring/active-usage?jenis_bmn=KENDARAAN_BERMOTOR"
```

### Utilization Report

```bash
# Get overall utilization report
curl -H "Authorization: Bearer $TOKEN" \
  http://localhost:3020/api/v1/pemakaian-bmn/monitoring/utilization-report

# Filter by satker
curl -H "Authorization: Bearer $TOKEN" \
  "http://localhost:3020/api/v1/pemakaian-bmn/monitoring/utilization-report?satker_id=uuid"
```

## Performance Considerations

### Database Queries

- All queries use appropriate indexes (created in migration 20260209_create_new_entity_tables.sql)
- Aggregation queries are optimized with GROUP BY and LIMIT clauses
- SIMAN integration queries filter by `kondisi = 'BAIK'` to reduce dataset

### Caching Recommendations

For production deployment, consider caching:

- Active usage dashboard: 5-minute TTL
- Utilization report: 1-hour TTL (less frequently changing data)

### Query Optimization

- Use EXPLAIN ANALYZE to verify query performance
- Add composite indexes if specific filter combinations are frequently used
- Consider materialized views for complex aggregations if performance becomes an issue

## Testing

### Manual Testing

1. Create test permits with various statuses
2. Verify dashboard shows correct counts and percentages
3. Test filtering by satker and jenis_bmn
4. Verify expiring permits appear correctly
5. Check utilization report calculations

### Integration Testing

Test the monitoring endpoints with:

- Empty database (no permits)
- Single permit
- Multiple permits across different types and satkers
- Permits expiring at various dates
- BMN with and without active permits

## Future Enhancements

Potential improvements for future iterations:

1. **Real-time Updates**: WebSocket support for live dashboard updates
2. **Export Functionality**: Export reports to PDF/Excel
3. **Trend Analysis**: Historical utilization trends over time
4. **Alerts**: Configurable alerts for low utilization or expiring permits
5. **Predictive Analytics**: ML-based predictions for BMN demand
6. **Custom Dashboards**: User-configurable dashboard widgets
7. **Drill-down Reports**: Detailed analysis per satker/BMN type

## Compliance

This implementation satisfies:

- ✅ REQ-P011: Active usage monitoring dashboard
- ✅ REQ-P012: Usage history per BMN and per pegawai (implemented in 27.1)
- ✅ REQ-P013: BMN utilization report

All requirements from task 27.3 have been successfully implemented.

---

**Implementation Date**: February 10, 2026
**Author**: Kiro AI Agent
**Status**: Complete
