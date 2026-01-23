---
inclusion: always
---

# Project Structure

## Directory Layout

```
/var/www/simpelv2/
├── Cargo.toml              # ROOT WORKSPACE (single source of truth)
├── lib/                    # Shared libraries
│   ├── ui/                 # UI components (alias: shared-microfrontend)
│   └── common/             # Common utilities (validation, cache, telemetry, db, crypto)
├── antarmuka/              # Frontend microfrontends (Leptos WASM)
│   ├── daskrimti/portal/   # Main portal (Daskrimti)
│   ├── pembinaan/          # Pembinaan microfrontends (keuangan, perencanaan, perlengkapan)
│   ├── badiklat/           # Training and education
│   ├── datun/              # Civil and state administration
│   ├── intel/              # Intelligence operations
│   ├── pemulihan_aset/     # Asset recovery
│   ├── pengawasan/         # Supervision and monitoring
│   ├── pidmil/             # Military crimes prosecution
│   ├── pidsus/             # Special crimes prosecution
│   └── pidum/              # General crimes prosecution
├── layanan/                # Backend microservices (Axum + Tonic)
│   ├── daskrimti/          # Daskrimti services (portal, ai, bantuan, dokumen, integrasi, notifikasi)
│   ├── pembinaan/          # Pembinaan services (keuangan, perlengkapan, perencanaan)
│   ├── badiklat/           # Training backend
│   ├── datun/              # Civil backend
│   ├── intel/              # Intelligence backend
│   ├── pemulihan_aset/     # Asset recovery backend
│   ├── pengawasan/         # Supervision backend
│   ├── pidmil/             # Military crimes backend
│   ├── pidsus/             # Special crimes backend
│   └── pidum/              # General crimes backend
└── infra/                  # Infrastructure (PART OF main workspace)
    ├── authenc/            # Identity provider
    └── secreton/           # Secrets vault (with sub-crates in crates/)
```

## Naming Conventions

| Type | Location | Package Name | Example |
|------|----------|--------------|---------|
| Microfrontend | `antarmuka/[domain]/[name]/` | `[name]-microfrontend` | `portal-microfrontend` |
| Microservice | `layanan/[domain]/[name]/` | `layanan-[name]` | `layanan-portal` |
| Shared Library | `lib/[name]/` | `lib-[name]` | `lib-ui` |
| Infrastructure | `infra/[name]/` | `[name]` | `authenc` |

## Key Principles

1. **Workspace Hierarchy**: Root `Cargo.toml` manages all dependencies for main workspace
2. **Infrastructure Isolation**: `authenc` and `secreton` are independent workspaces
3. **Shared Code**: Common functionality in `lib/` packages
4. **Domain Organization**: Services grouped by government department domains
5. **Microfrontend Independence**: Each frontend is a separate deployable unit

## Important Files

- `AGENTS.md` - Primary AI developer guide
- `.github/copilot-instructions.md` - Detailed coding patterns
- `CONTRIBUTING.md` - Contribution guidelines
- `docs/` - Architecture and API documentation
- `.kiro/specs/` - Feature specifications
