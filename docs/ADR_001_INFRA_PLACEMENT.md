# Architecture Decision Record: Placement of Authenc & Secreton

**Status**: Proposed
**Date**: 2026-01-08
**Context**: Defining whether `authenc` (IAM) and `secreton` (Vault) belong in `infra/` or `layanan/daskrimti/`.

## 1. Executive Summary
**Recommendation**: **Keep in `infra/`**.
`authenc` and `secreton` represent **Horizontal Platform Capabilities**, not **Vertical Business Domains**. Moving them to `layanan/daskrimti` would conflate *Business Logic* with *Platform Infrastructure*, leading to tighter coupling and unclear ownership boundaries.

---

## 2. Option Analysis

### Option A: `infra/` (Current & Recommended)
Treats these services as "Commodity Infrastructure" similar to a Database or Kubernetes.

| Aspect | Description |
| :--- | :--- |
| **Semantics** | They are **dependencies** for business services. `layanan-portal` *depends on* Identity. |
| **Lifecycle** | Changed rarely, high stability requirement. Upgrades are critical platform events. |
| **Access** | Accessed by *other services* via internal protocols (mTLS/gRPC), not just end-users. |

### Option B: `layanan/daskrimti/` (Business Domain)
Treats these services as "Keamanan Domain" or "Support Services".

| Aspect | Description |
| :--- | :--- |
| **Semantics** | Suggests "Authentication" is a business unit similar to "Pidum" or "Datun". |
| **Lifecycle** | Subject to business-logic churn (e.g., "Add feature X for department Y"). |
| **Access** | Perceived as peer services rather than foundational implementation details. |

---

## 3. Deep Analysis: Why `infra/`?

### ✅ Kelebihan (Why it works)
1.  **Clear Dependency Graph**:
    *   `layanan/*` -> depends on -> `infra/*`
    *   If they are peers (`layanan/auth` vs `layanan/pidum`), circular dependencies are more likely to creep in (e.g., Auth needing Pidum data to enrich tokens).
2.  **Platform Engineering vs Product Engineering**:
    *   **Infra**: Developed by Core Platform Team (Focus: Security, Stability, Performance, RFC Compliance).
    *   **Layanan**: Developed by Product Teams (Focus: Feature velocity, Business rules, UI flows).
    *   Separating folders reinforces this team structure.
3.  **Deployment Blast Radius**:
    *   Changes in `infra/` are high-risk/high-impact. They usually deploy *before* business services.
    *   Visual separation warns developers: *"I am touching the foundation, be careful."*

### ❌ Potensi Masalah jika pindah ke `layanan/`
1.  **"Big Ball of Mud"**:
    *   Developers might feel comfortable adding specific business logic into `authenc` (e.g., "Check if user is Jaksa Utama").
    *   Result: Authenc becomes bloated with business rules, making it hard to reuse for other projects or replace.
2.  **Versioning Confusion**:
    *   Business services often iterate fast (v1, v2 features).
    *   Identity/Vault protocols (OIDC, AES) change very slowly. Grouping them invites incorrect versioning expectations.

### ⚠️ Tantangan (Challenges of `infra/`)
1.  **Developer Experience (DX)**:
    *   Devs working on features might forget to run `infra` services locally.
    *   *Mitigation*: Use `docker compose` profiles to always bring them up.
2.  **Internal Paths**:
    *   `layanan` services need to import client libraries.
    *   *Solution*: We already solved this with `lib-middleware` which abstracts the calls. Services don't even need to know `authenc` exists, they just check `req.user`.

## 4. Analogi

Bayangkan Gedung Kejaksaan:
*   **`infra/` (Listrik & Air)**: Semua ruangan butuh. Tidak peduli ruangan itu untuk Pidmil atau Intel, listriknya sama. Jika listrik mati, satu gedung mati.
*   **`layanan/` (Ruangan/Divisi)**: Ada Ruang Pidum, Ruang Datun. Mereka punya aturan masing-masing.
*   **`authenc/secreton`** adalah **Sistem Keamanan & Kunci Pintu** (Security).
    *   Apakah "Pos Satpam" adalah sebuah "Divisi Hukum"? **Bukan**.
    *   Pos Satpam adalah **Infrastruktur** gedung.

## 5. Conclusion
Pertahankan di `infra/`. Ini memaksa disiplin arsitektur bahwa Authentication & Secrets Management adalah **Horizontal Layer** yang melayani seluruh organisasi, bukan sekadar modul aplikasi biasa.
