---
inclusion: always
---

# Product Overview

**SIMPelv2** is a mission-critical system for the Indonesian Attorney General's Office (Kejaksaan RI).

## Architecture

- **Microservices Backend**: Domain-specific services in `layanan/` expose REST APIs
- **Microfrontend UI**: Independent WASM applications in `antarmuka/` for different departments
- **Infrastructure Services**: Identity (Authenc) and secrets management (Secreton) integrated in main workspace
- **Shared Services**: Daskrimti provides shared services (portal, AI, bantuan, dokumen, integrasi, notifikasi)

## Key Domains

- **Daskrimti**: Pusat Data Statistik Kriminal dan Teknologi Informasi (manages portal and shared services)
- **Pembinaan**: Development and capacity building (keuangan, perencanaan, perlengkapan)
- **Intel**: Intelligence operations
- **Pidsus**: Special crimes prosecution
- **Pidum**: General crimes prosecution
- **Pidmil**: Military crimes prosecution
- **Pengawasan**: Supervision and monitoring
- **Pemulihan Aset**: Asset recovery
- **Datun**: Civil and state administration
- **Badiklat**: Training and education

## Communication Rules

- Frontend → Backend: REST API (JSON/HTTP)
- Backend → Backend: gRPC (Protobuf)
- Backend → Authenc/Secreton: gRPC with mTLS
- Frontend NEVER calls Authenc/Secreton directly (always through backend proxy)
