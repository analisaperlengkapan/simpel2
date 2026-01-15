# SIMPelv2 - Arsitektur Integrasi Microservice

## Arsitektur yang Benar

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              ANTARMUKA (Frontend)                            │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ┌─────────────────────┐    ┌─────────────────────┐    ┌─────────────────┐  │
│  │  Portal (Yew WASM)  │    │ Perlengkapan (WASM) │    │ Keuangan, dll   │  │
│  │  :8090              │    │ :8091               │    │ :809X           │  │
│  ├─────────────────────┤    ├─────────────────────┤    ├─────────────────┤  │
│  │ - Login/Logout      │    │ - Manajemen BMN     │    │ - Module UI     │  │
│  │ - Dashboard         │    │ - Inventaris        │    │ - Reports       │  │
│  │ - User Profile      │    │ - Laporan           │    │                 │  │
│  └──────────┬──────────┘    └──────────┬──────────┘    └────────┬────────┘  │
│             │                          │                        │           │
│             │ SEMUA REQUEST MELALUI LAYANAN (tidak langsung ke Authenc)     │
│             ▼                          ▼                        ▼           │
└─────────────────────────────────────────────────────────────────────────────┘
              │                          │                        │
              ▼                          ▼                        ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                               LAYANAN (Backend)                              │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ┌─────────────────────┐    ┌─────────────────────┐    ┌─────────────────┐  │
│  │ Layanan Portal      │    │ Layanan Perlengkapan│    │ Layanan Lainnya │  │
│  │  :8081              │    │ :8082               │    │ :808X           │  │
│  ├─────────────────────┤    ├─────────────────────┤    ├─────────────────┤  │
│  │ - Auth Proxy        │    │ - Business Logic    │    │ - Domain Logic  │  │
│  │ - Session Mgmt      │    │ - Data Access       │    │ - Data Access   │  │
│  │ - JWT Validation    │    │ - JWT Validation    │    │ - JWT Valid.    │  │
│  └─────────┬───────────┘    └─────────┬───────────┘    └────────┬────────┘  │
│            │                          │                         │           │
│            │  LAYANAN BERKOMUNIKASI DENGAN INFRA SERVICES       │           │
│            ▼                          ▼                         ▼           │
└─────────────────────────────────────────────────────────────────────────────┘
              │                          │                        │
              │          ┌───────────────┴───────────────┐        │
              │          │                               │        │
              ▼          ▼                               ▼        ▼
┌─────────────────────────────┐       ┌─────────────────────────────────────┐
│         AUTHENC (IAM)       │       │           SECRETON (Vault)          │
│           :8088/:9088       │       │           :8200/:9090               │
├─────────────────────────────┤       ├─────────────────────────────────────┤
│ - User Authentication       │       │ - Secret Storage                    │
│ - JWT Issuance (EdDSA)      │◄─────►│ - Transit Encryption                │
│ - JWKS Endpoint             │ JWKS  │ - JWT Validation (via JWKS)         │
│ - Role Management           │       │ - Policy Management                 │
│ - Session Management        │       │ - Key Management                    │
└─────────────────────────────┘       └─────────────────────────────────────┘
              │                                     │
              ▼                                     ▼
     ┌──────────────┐                      ┌──────────────┐
     │  PostgreSQL  │                      │  PostgreSQL  │
     │  (authenc)   │                      │  (secreton)  │
     └──────────────┘                      └──────────────┘
```

## Aliran Autentikasi (Authentication Flow)

```
1. User membuka Portal Frontend (:8090)
           │
           ▼
2. Frontend mengirim login request ke Layanan Portal (:8081)
           │
           ▼
3. Layanan Portal meneruskan ke Authenc (:8088)
           │
           ▼
4. Authenc memvalidasi credentials, mengeluarkan JWT (EdDSA)
           │
           ▼
5. Layanan Portal menerima JWT, mengembalikan ke Frontend
           │
           ▼
6. Frontend menyimpan JWT di localStorage/cookie
           │
           ▼
7. Request selanjutnya: Frontend → Layanan (dengan JWT)
           │
           ▼
8. Layanan memvalidasi JWT via JWKS dari Authenc
           │
           ▼
9. Jika butuh secret, Layanan → Secreton (dengan JWT)
```

## Konfigurasi Docker Network

Semua service terhubung melalui `simpelv2-network`:

| Service | Port | Network | Berkomunikasi dengan |
|---------|------|---------|---------------------|
| Portal Frontend | 8090 | simpelv2-network | Layanan Portal |
| Perlengkapan Frontend | 8091 | simpelv2-network | Layanan Perlengkapan |
| Layanan Portal | 8081 | simpelv2-network | Authenc, Secreton |
| Layanan Perlengkapan | 8082 | simpelv2-network | Authenc, Secreton |
| Authenc | 8088 | simpelv2-network, authenc-net | PostgreSQL, Redis |
| Secreton | 8200 | simpelv2-network, secreton-network | PostgreSQL, Authenc (JWKS) |

## Environment Variables

### Layanan Portal
```env
AUTHENC_URL=http://authenc:8088
SECRETON_URL=http://secreton-server:8200
DATABASE_URL=postgresql://postgres:postgres@portal-db:5432/portal
REDIS_URL=redis://portal-redis:6379/0
JWT_SECRET=<shared_secret_for_validation>
```

### Authenc
```env
DATABASE_URL=postgres://postgres:postgres@postgres:5432/authenc
REDIS_URL=redis://redis:6379/0
JWT_SECRET=default_jwt_secret_change_in_production
```

### Secreton
```env
AUTHENC_JWKS_URL=http://authenc:8088/.well-known/jwks.json
AUTHENC_ISSUER=authenc
SECRETON_JWT_AUDIENCE=secreton
DATABASE_URL=postgresql://secreton:changeme@postgres:5432/secreton
```

## Integrasi JWT

### Authenc → Layanan
- Authenc mengeluarkan JWT dengan algoritma **EdDSA (Ed25519)**
- JWKS tersedia di `http://authenc:8088/.well-known/jwks.json`
- Format JWKS: `{ "kty": "OKP", "crv": "Ed25519", "alg": "EdDSA" }`

### Layanan → Secreton
- Secreton memvalidasi JWT via JWKS dari Authenc
- Secreton juga mendukung token HS256 untuk service-to-service auth
- Dual-verifier pattern: coba HS256 dulu, fallback ke EdDSA via JWKS

## Menjalankan Semua Service

```bash
# 1. Start Authenc
cd infra/authenc && docker compose up -d

# 2. Start Secreton
cd infra/secreton && docker compose up -d

# 3. Start Layanan Portal
cd layanan/daskrimti/portal && docker compose up -d

# 4. Start Layanan Perlengkapan (TODO: buat docker-compose)
cd layanan/pembinaan/perlengkapan && docker compose up -d

# 5. Start Frontend Portal
cd antarmuka/daskrimti/portal && trunk serve --open

# 6. Start Frontend Perlengkapan
cd antarmuka/pembinaan/perlengkapan && trunk serve --port 8091 --open
```

## Verifikasi Integrasi

```bash
# Check semua service health
curl http://localhost:8088/health  # Authenc
curl http://localhost:8200/health  # Secreton
curl http://localhost:8081/health  # Layanan Portal
curl http://localhost:8082/health  # Layanan Perlengkapan

# Check JWKS connectivity dari Secreton
docker exec secreton-server curl -s http://authenc:8088/.well-known/jwks.json

# Check network connectivity
docker network inspect simpelv2-network
```

## File yang Dimodifikasi untuk Integrasi

1. **infra/secreton/crates/api/src/auth/oidc_verifier.rs**
   - Ditambahkan OKP/EdDSA support untuk validasi JWT Authenc

2. **infra/secreton/crates/api/src/middleware.rs**
   - Ditambahkan fallback ke JWKS Authenc untuk cross-service auth

3. **infra/authenc/docker-compose.yml**
   - Ditambahkan simpelv2-network untuk koneksi dengan layanan

4. **infra/secreton/docker-compose.yml**
   - Ditambahkan simpelv2-network dan authenc-network

5. **lib/utils/connection_pool.rs**
   - Fixed http2_keep_alive methods for reqwest compatibility
