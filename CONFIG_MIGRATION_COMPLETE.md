# Configuration Migration - Completed

## ✅ Folder `/config` di Root Proyek Telah Dihapus

Semua konfigurasi telah dipindahkan ke lokasi yang lebih tepat sesuai dengan prinsip **decentralized configuration** dan **separation of concerns**.

## 📁 Struktur Konfigurasi Baru

### 1. **IAM/Security Configuration** → `infra/authenc/config/`

**Lokasi:** `/srv/proyek/simpelv2/infra/authenc/config/`

**Files:**

- `default.toml` - Default config untuk Authenc IAM
- `production.toml` - Production overrides untuk Authenc IAM
- `captcha.development.toml` - Captcha config untuk development
- `captcha.monitoring.toml` - Captcha config dengan monitoring
- `captcha.production.toml` - Captcha config untuk production
- `event_retention.example.toml` - Example untuk event retention policy
- `key_rotation.example.toml` - Example untuk key rotation policy
- `openapi.yaml` - OpenAPI specification

**Reasoning:**

- Captcha adalah mekanisme **security** untuk mencegah bot/automated attacks
- Captcha merupakan bagian dari **authentication layer**, bukan business logic
- Authenc sebagai IAM service adalah tempat yang tepat untuk semua security configs

### 2. **Monitoring Configuration** → `infra/monitoring/config/`

**Lokasi:** `/srv/proyek/simpelv2/infra/monitoring/config/`

**Struktur:**

```
infra/monitoring/config/
├── prometheus/
│   └── captcha.yml          # Prometheus config untuk captcha monitoring
└── alertmanager/
    └── captcha.yml          # Alertmanager config untuk captcha alerts
```

**Reasoning:**

- Prometheus dan Alertmanager adalah **infrastructure monitoring tools**
- Config ini terkait dengan observability, bukan business logic
- Lokasi di `infra/` sesuai dengan nature-nya sebagai infrastructure component

### 3. **Secret Management Configuration** → `infra/secreton/config/`

**Lokasi:** `/srv/proyek/simpelv2/infra/secreton/config/`

**Files:**

- `default.toml` - Default config untuk Secreton
- `production.toml` - Production overrides untuk Secreton
- `default.toml.backup` - Backup dari config lama (untuk referensi)
- `production.toml.backup` - Backup dari config lama (untuk referensi)
- `raft.toml` - Raft consensus config (future use)
- `vault.toml` - Vault compatibility config (future use)

**Reasoning:**

- Secret management adalah critical infrastructure component
- Config terpisah memudahkan audit dan compliance
- Port standardized: HTTP 8200, gRPC 8201

## 🔄 Update yang Dilakukan

### Docker Compose

**File:** `docker-compose.captcha.yml`

```yaml
# BEFORE:
volumes:
  - ./config/prometheus/captcha.yml:/etc/prometheus/prometheus.yml:ro
  - ./config/alertmanager/captcha.yml:/etc/alertmanager/config.yml:ro

# AFTER:
volumes:
  - ./infra/monitoring/config/prometheus/captcha.yml:/etc/prometheus/prometheus.yml:ro
  - ./infra/monitoring/config/alertmanager/captcha.yml:/etc/alertmanager/config.yml:ro
```

## 🎯 Benefits

### ✅ Separation of Concerns

- Security configs (Authenc) terpisah dari monitoring configs
- Setiap service memiliki config directory sendiri
- Clear ownership: siapa yang bertanggung jawab atas config apa

### ✅ Better Organization

- Tidak ada lagi folder `/config` yang campur aduk di root
- Config diletakkan dekat dengan service yang menggunakannya
- Easier navigation: `infra/authenc/config/`, `infra/secreton/config/`, `infra/monitoring/config/`

### ✅ Auditability & Compliance

- Config changes tracked di Git per-service
- Easier untuk audit trail (ISO 27001, NIST)
- Clear config hierarchy: default → env-specific → env vars

### ✅ Security

- Captcha config (security mechanism) berada di IAM service
- Secret management config isolated di Secreton
- Monitoring config tidak bercampur dengan security config

### ✅ Maintainability

- Developer tahu di mana mencari config untuk service tertentu
- Tidak ada ambiguitas: "config ini untuk apa?"
- Consistent pattern: `<service>/config/`

## 📖 Usage Guide

### Authenc (IAM + Captcha)

```bash
cd infra/authenc
# Config di-load dari infra/authenc/config/default.toml
# Override dengan infra/authenc/config/production.toml jika AUTHENC_ENV=production
export JWT_SECRET="your-secret"
cargo run
```

### Secreton (Secret Management)

```bash
cd infra/secreton/crates/api
# Config di-load dari infra/secreton/config/default.toml
# Override dengan infra/secreton/config/production.toml jika SECRETON_ENV=production
export JWT_SECRET="your-secret"
cargo run --bin api_server
```

### Monitoring Stack

```bash
# Prometheus & Alertmanager config di infra/monitoring/config/
docker compose -f docker-compose.captcha.yml up -d
```

## 🗂️ Complete Config Structure

```
/srv/proyek/simpelv2/
├── infra/
│   ├── authenc/
│   │   └── config/
│   │       ├── default.toml                    # Authenc base config
│   │       ├── production.toml                 # Authenc production
│   │       ├── captcha.development.toml        # Captcha dev config
│   │       ├── captcha.monitoring.toml         # Captcha with monitoring
│   │       ├── captcha.production.toml         # Captcha production
│   │       ├── event_retention.example.toml    # Event retention example
│   │       ├── key_rotation.example.toml       # Key rotation example
│   │       └── openapi.yaml                    # API documentation
│   │
│   ├── secreton/
│   │   └── config/
│   │       ├── default.toml                    # Secreton base config
│   │       ├── production.toml                 # Secreton production
│   │       ├── default.toml.backup             # Old config backup
│   │       ├── production.toml.backup          # Old config backup
│   │       ├── raft.toml                       # Raft consensus (future)
│   │       └── vault.toml                      # Vault compat (future)
│   │
│   └── monitoring/
│       └── config/
│           ├── prometheus/
│           │   └── captcha.yml                 # Prometheus config
│           └── alertmanager/
│               └── captcha.yml                 # Alertmanager config
│
└── ❌ config/  # DIHAPUS - tidak ada lagi folder config di root
```

## 🔍 Migration Summary

| **Before**                         | **After**                               | **Reason**                |
| ---------------------------------- | --------------------------------------- | ------------------------- |
| `/config/authenc.production.toml`  | ❌ Dihapus (dead code)                  | Tidak digunakan           |
| `/config/secreton.production.toml` | ❌ Dihapus (dead code)                  | Tidak digunakan           |
| `/config/captcha*.toml`            | `infra/authenc/config/captcha*.toml`    | Security/IAM component    |
| `/config/prometheus/`              | `infra/monitoring/config/prometheus/`   | Infrastructure monitoring |
| `/config/alertmanager/`            | `infra/monitoring/config/alertmanager/` | Infrastructure monitoring |

## ✅ Verification

```bash
# Cek tidak ada folder config di root
ls -d /srv/proyek/simpelv2/config  # Should fail

# Cek config Authenc
ls -la /srv/proyek/simpelv2/infra/authenc/config/

# Cek config Monitoring
ls -la /srv/proyek/simpelv2/infra/monitoring/config/

# Cek config Secreton
ls -la /srv/proyek/simpelv2/infra/secreton/config/
```

---

**Status**: ✅ **COMPLETE** - Folder `/config` di root proyek telah dihapus, semua config dipindahkan ke lokasi yang tepat
**Date**: November 5, 2025
**Impact**: High - Improved organization, security, and maintainability
