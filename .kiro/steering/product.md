# SIMPelv2 Product Overview

SIMPelv2 is a modern asset management system (Sistem Informasi Manajemen Pengelolaan BMN) for the Indonesian Attorney General's Office (Kejaksaan Agung RI). The system manages state-owned assets (Barang Milik Negara) across multiple divisions.

## Architecture

**Microfrontend + Microservices Architecture**
- 11 independent microfrontend modules (Leptos + WebAssembly)
- Backend microservices in Rust (Axum framework)
- Shared component library for UI consistency
- Zero-trust security architecture

## Key Modules

**Frontend Microfrontends** (antarmuka/):
- Portal - Main gateway and dashboard
- Badiklat - Training and education management
- Datun, Pidum, Pidmil, Pidsus - Criminal prosecution divisions
- Intel - Intelligence and analytics
- Pengawasan - Monitoring and compliance
- Pemulihan Aset - Asset recovery
- Pembinaan - Development management (Keuangan, Perencanaan, Perlengkapan)

**Backend Services** (layanan/):
- Security (keamanan) - Authentication, authorization, MFA
- AI - Document processing, OCR, ML models
- Document (dokumen) - Document management
- Dashboard (dasbor) - Real-time metrics
- Reporting (laporan) - Dynamic reports
- Configuration (konfigurasi) - System settings
- Help (bantuan) - Support system
- Integration (integrasi) - External APIs
- Notification (notifikasi) - Alerts and messaging

**Infrastructure** (infra/):
- Authenc - Identity and access management (IAM)
- Secreton - Secret management (HashiCorp Vault alternative)
- Nginx - Reverse proxy
- Gerbang - API gateway (Envoy)
- K8s - Kubernetes deployment manifests

## Target Users

Government employees at Kejaksaan RI managing state assets, legal cases, training programs, and administrative functions.

## Production Status

Live at https://simpel.kejaksaan.go.id/ with SSL/TLS, deployed on MicroK8s.
