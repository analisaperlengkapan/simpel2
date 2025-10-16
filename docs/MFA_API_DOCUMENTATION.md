# MFA API Documentation - SIMPelv2

## Overview
This document provides comprehensive API documentation for the Multi-Factor Authentication (MFA) system in SIMPelv2. The MFA API enables secure two-factor authentication using TOTP (Time-based One-Time Password) for government employees.

## Base URL
```
Production: https://simipelv2.kejaksaan.go.id/api/auth/mfa
Staging: https://staging.simipelv2.kejaksaan.go.id/api/auth/mfa
```

## Authentication
All MFA API endpoints require authentication via JWT token in the Authorization header:
```
Authorization: Bearer <jwt_token>
```

## API Endpoints

### 1. MFA Setup

#### POST /setup
Initiates MFA setup for a user by generating TOTP secret and QR code.

**Request Headers:**
```
Authorization: Bearer <jwt_token>
Content-Type: application/json
```

**Request Body:**
```json
{
  "user_preference": {
    "app_name": "Google Authenticator" // Optional
  }
}
```

**Response (200 OK):**
```json
{
  "success": true,
  "data": {
    "qr_code_url": "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAA...",
    "secret_key": "JBSWY3DPEHPK3PXP",
    "backup_codes": [
      "12345678",
      "87654321",
      "11223344",
      "44332211",
      "55667788",
      "88776655",
      "99887766",
      "66778899",
      "33445566",
      "66554433"
    ],
    "provisioning_uri": "otpauth://totp/SIMPelv2%20Kejaksaan%20RI:12345678@kejaksaan.go.id?secret=JBSWY3DPEHPK3PXP&issuer=SIMPelv2%20Kejaksaan%20RI"
  },
  "message": "MFA setup initiated successfully"
}
```

**Error Responses:**
```json
// 400 Bad Request - MFA already enabled
{
  "success": false,
  "error": {
    "code": "MFA_ALREADY_ENABLED",
    "message": "MFA is already enabled for this user"
  }
}

// 500 Internal Server Error - Secret generation failed
{
  "success": false,
  "error": {
    "code": "SECRET_GENERATION_FAILED",
    "message": "Failed to generate MFA secret"
  }
}
```
#### POST /verify-setup
Verifies the initial TOTP code during MFA setup to confirm successful configuration.

**Request Headers:**
```
Authorization: Bearer <jwt_token>
Content-Type: application/json
```

**Request Body:**
```json
{
  "code": "123456"
}
```

**Response (200 OK):**
```json
{
  "success": true,
  "data": {
    "mfa_enabled": true,
    "setup_completed_at": "2024-10-15T10:30:00Z"
  },
  "message": "MFA setup completed successfully"
}
```

**Error Responses:**
```json
// 400 Bad Request - Invalid code
{
  "success": false,
  "error": {
    "code": "INVALID_OTP_CODE",
    "message": "The provided OTP code is invalid or expired"
  }
}

// 429 Too Many Requests - Rate limited
{
  "success": false,
  "error": {
    "code": "RATE_LIMITED",
    "message": "Too many verification attempts. Please try again in 60 seconds",
    "retry_after": 60
  }
}
```

### 2. MFA Verification

#### POST /verify
Verifies TOTP code during login process.

**Request Headers:**
```
Authorization: Bearer <temp_jwt_token>
Content-Type: application/json
```

**Request Body:**
```json
{
  "code": "123456",
  "remember_device": false // Optional, default: false
}
```

**Response (200 OK):**
```json
{
  "success": true,
  "data": {
    "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
    "token_type": "Bearer",
    "expires_in": 28800,
    "session_id": "sess_abc123def456"
  },
  "message": "MFA verification successful"
}
```

**Error Responses:**
```json
// 401 Unauthorized - Invalid code
{
  "success": false,
  "error": {
    "code": "INVALID_OTP_CODE",
    "message": "Invalid OTP code",
    "attempts_remaining": 3
  }
}

// 423 Locked - Account locked
{
  "success": false,
  "error": {
    "code": "ACCOUNT_LOCKED",
    "message": "Account locked due to multiple failed attempts",
    "unlock_at": "2024-10-15T11:00:00Z"
  }
}
```

#### POST /verify-backup-code
Verifies backup code as alternative to TOTP.

**Request Headers:**
```
Authorization: Bearer <temp_jwt_token>
Content-Type: application/json
```

**Request Body:**
```json
{
  "backup_code": "12345678"
}
```

**Response (200 OK):**
```json
{
  "success": true,
  "data": {
    "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
    "token_type": "Bearer",
    "expires_in": 28800,
    "backup_codes_remaining": 9
  },
  "message": "Backup code verification successful"
}
```
### 3. MFA Status and Management

#### GET /status
Retrieves current MFA status for the authenticated user.

**Request Headers:**
```
Authorization: Bearer <jwt_token>
```

**Response (200 OK):**
```json
{
  "success": true,
  "data": {
    "mfa_enabled": true,
    "mfa_required": true,
    "setup_at": "2024-10-01T09:15:00Z",
    "last_used": "2024-10-15T08:30:00Z",
    "backup_codes_remaining": 8,
    "trusted_devices": 2
  }
}
```

#### POST /disable
Disables MFA for the current user (admin only or with additional verification).

**Request Headers:**
```
Authorization: Bearer <jwt_token>
Content-Type: application/json
```

**Request Body:**
```json
{
  "confirmation_code": "123456", // Current TOTP code required
  "reason": "Device lost, temporary disable"
}
```

**Response (200 OK):**
```json
{
  "success": true,
  "data": {
    "mfa_enabled": false,
    "disabled_at": "2024-10-15T10:45:00Z"
  },
  "message": "MFA disabled successfully"
}
```

#### POST /reset
Resets MFA configuration (generates new secret).

**Request Headers:**
```
Authorization: Bearer <jwt_token>
Content-Type: application/json
```

**Request Body:**
```json
{
  "current_code": "123456" // Current TOTP code or backup code
}
```

**Response (200 OK):**
```json
{
  "success": true,
  "data": {
    "qr_code_url": "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAA...",
    "secret_key": "NEWJBSWY3DPEHPK3PXP",
    "backup_codes": ["87654321", "12345678", "..."],
    "reset_at": "2024-10-15T10:50:00Z"
  },
  "message": "MFA reset successfully"
}
```

### 4. Backup Codes Management

#### GET /backup-codes
Retrieves remaining backup codes count (not the actual codes).

**Request Headers:**
```
Authorization: Bearer <jwt_token>
```

**Response (200 OK):**
```json
{
  "success": true,
  "data": {
    "backup_codes_remaining": 7,
    "last_used": "2024-10-10T14:20:00Z"
  }
}
```

#### POST /backup-codes/regenerate
Generates new backup codes (invalidates old ones).

**Request Headers:**
```
Authorization: Bearer <jwt_token>
Content-Type: application/json
```

**Request Body:**
```json
{
  "confirmation_code": "123456" // Current TOTP code required
}
```

**Response (200 OK):**
```json
{
  "success": true,
  "data": {
    "backup_codes": [
      "98765432",
      "23456789",
      "34567890",
      "45678901",
      "56789012",
      "67890123",
      "78901234",
      "89012345",
      "90123456",
      "01234567"
    ],
    "generated_at": "2024-10-15T11:00:00Z"
  },
  "message": "New backup codes generated successfully"
}
```
### 5. Administrative Endpoints

#### GET /admin/users
Lists MFA status for all users (admin only).

**Request Headers:**
```
Authorization: Bearer <admin_jwt_token>
```

**Query Parameters:**
```
?page=1&limit=50&satker=KEJATI_DKI&mfa_status=enabled
```

**Response (200 OK):**
```json
{
  "success": true,
  "data": {
    "users": [
      {
        "user_id": "uuid-123",
        "nip": "12345678",
        "nama": "John Doe",
        "satker_code": "KEJATI_DKI",
        "mfa_enabled": true,
        "mfa_required": true,
        "setup_at": "2024-10-01T09:15:00Z",
        "last_used": "2024-10-15T08:30:00Z"
      }
    ],
    "pagination": {
      "page": 1,
      "limit": 50,
      "total": 1250,
      "total_pages": 25
    },
    "summary": {
      "total_users": 1250,
      "mfa_enabled": 1100,
      "mfa_required": 1250,
      "adoption_rate": 88.0
    }
  }
}
```

#### POST /admin/users/{user_id}/reset
Admin reset of user MFA (admin only).

**Request Headers:**
```
Authorization: Bearer <admin_jwt_token>
Content-Type: application/json
```

**Request Body:**
```json
{
  "reason": "User reported lost device",
  "notify_user": true
}
```

**Response (200 OK):**
```json
{
  "success": true,
  "data": {
    "user_id": "uuid-123",
    "mfa_reset": true,
    "reset_at": "2024-10-15T11:15:00Z",
    "admin_id": "uuid-admin"
  },
  "message": "User MFA reset successfully"
}
```

#### POST /admin/users/{user_id}/disable
Admin disable of user MFA (admin only).

**Request Headers:**
```
Authorization: Bearer <admin_jwt_token>
Content-Type: application/json
```

**Request Body:**
```json
{
  "reason": "Temporary exemption approved",
  "duration_days": 30
}
```

**Response (200 OK):**
```json
{
  "success": true,
  "data": {
    "user_id": "uuid-123",
    "mfa_disabled": true,
    "disabled_until": "2024-11-15T11:20:00Z",
    "admin_id": "uuid-admin"
  },
  "message": "User MFA disabled successfully"
}
```

### 6. Reporting Endpoints

#### GET /admin/reports/adoption
MFA adoption report (admin only).

**Request Headers:**
```
Authorization: Bearer <admin_jwt_token>
```

**Query Parameters:**
```
?start_date=2024-10-01&end_date=2024-10-15&group_by=satker
```

**Response (200 OK):**
```json
{
  "success": true,
  "data": {
    "report_period": {
      "start_date": "2024-10-01",
      "end_date": "2024-10-15"
    },
    "overall_stats": {
      "total_users": 1250,
      "mfa_enabled": 1100,
      "adoption_rate": 88.0,
      "new_setups": 150
    },
    "breakdown": [
      {
        "satker_code": "KEJATI_DKI",
        "satker_name": "Kejaksaan Tinggi DKI Jakarta",
        "total_users": 300,
        "mfa_enabled": 280,
        "adoption_rate": 93.3
      }
    ]
  }
}
```
#### GET /admin/reports/security
Security events report (admin only).

**Request Headers:**
```
Authorization: Bearer <admin_jwt_token>
```

**Query Parameters:**
```
?start_date=2024-10-01&end_date=2024-10-15&event_type=failed_attempts
```

**Response (200 OK):**
```json
{
  "success": true,
  "data": {
    "report_period": {
      "start_date": "2024-10-01",
      "end_date": "2024-10-15"
    },
    "security_events": [
      {
        "event_type": "failed_attempts",
        "count": 45,
        "unique_users": 12,
        "unique_ips": 8
      },
      {
        "event_type": "account_lockouts",
        "count": 5,
        "unique_users": 5,
        "unique_ips": 3
      }
    ],
    "top_incidents": [
      {
        "user_nip": "12345678",
        "failed_attempts": 8,
        "last_attempt": "2024-10-15T10:30:00Z",
        "ip_address": "192.168.1.100"
      }
    ]
  }
}
```

## Error Codes Reference

| Code | HTTP Status | Description |
|------|-------------|-------------|
| `MFA_ALREADY_ENABLED` | 400 | MFA is already enabled for user |
| `MFA_NOT_ENABLED` | 400 | MFA is not enabled for user |
| `INVALID_OTP_CODE` | 401 | Invalid or expired OTP code |
| `INVALID_BACKUP_CODE` | 401 | Invalid or already used backup code |
| `ACCOUNT_LOCKED` | 423 | Account locked due to failed attempts |
| `RATE_LIMITED` | 429 | Too many requests |
| `SECRET_GENERATION_FAILED` | 500 | Failed to generate TOTP secret |
| `SECRET_STORAGE_FAILED` | 500 | Failed to store secret in vault |
| `INSUFFICIENT_PRIVILEGES` | 403 | Admin privileges required |

## Rate Limiting

| Endpoint | Limit | Window |
|----------|-------|--------|
| `/setup` | 3 requests | 1 hour |
| `/verify-setup` | 10 requests | 10 minutes |
| `/verify` | 10 requests | 10 minutes |
| `/verify-backup-code` | 5 requests | 10 minutes |
| `/reset` | 2 requests | 1 hour |
| Admin endpoints | 100 requests | 1 hour |

## Security Headers

All API responses include security headers:
```
X-Content-Type-Options: nosniff
X-Frame-Options: DENY
X-XSS-Protection: 1; mode=block
Strict-Transport-Security: max-age=31536000; includeSubDomains
Content-Security-Policy: default-src 'self'
```

## Webhook Events

The MFA system can send webhook notifications for important events:

### Event Types
- `mfa.setup.completed`
- `mfa.verification.failed`
- `mfa.account.locked`
- `mfa.backup_code.used`
- `mfa.admin.reset`

### Webhook Payload Example
```json
{
  "event_type": "mfa.setup.completed",
  "timestamp": "2024-10-15T10:30:00Z",
  "user_id": "uuid-123",
  "user_nip": "12345678",
  "ip_address": "192.168.1.100",
  "user_agent": "Mozilla/5.0...",
  "metadata": {
    "setup_method": "qr_code",
    "authenticator_app": "Google Authenticator"
  }
}
```

## SDK Examples

### JavaScript/TypeScript
```typescript
import { MfaClient } from '@simipelv2/mfa-client';

const mfaClient = new MfaClient({
  baseUrl: 'https://simipelv2.kejaksaan.go.id/api/auth/mfa',
  token: 'your-jwt-token'
});

// Setup MFA
const setupResponse = await mfaClient.setup();
console.log('QR Code:', setupResponse.data.qr_code_url);

// Verify setup
const verifyResponse = await mfaClient.verifySetup('123456');
console.log('MFA Enabled:', verifyResponse.data.mfa_enabled);

// Verify during login
const loginResponse = await mfaClient.verify('654321');
console.log('Access Token:', loginResponse.data.access_token);
```

### Python
```python
from simipelv2_mfa import MfaClient

client = MfaClient(
    base_url='https://simipelv2.kejaksaan.go.id/api/auth/mfa',
    token='your-jwt-token'
)

# Setup MFA
setup_result = client.setup()
print(f"QR Code: {setup_result['data']['qr_code_url']}")

# Verify setup
verify_result = client.verify_setup('123456')
print(f"MFA Enabled: {verify_result['data']['mfa_enabled']}")
```

---

**API Version**: v1
**Last Updated**: October 15, 2024
**Contact**: api-support@kejaksaan.go.id
