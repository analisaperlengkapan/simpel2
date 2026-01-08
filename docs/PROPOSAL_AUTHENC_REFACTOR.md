# Proposal: Authenc Structure Refactoring

**Current Status**: The user correctly identified that `authenc` structure is "kurang rapih".
**Problem**:
1.  **Flat & Crowded Directories**: `src/services` and `src/handlers` have 50+ files each.
2.  **Mixed Concerns**: Core domain logic, transport logic, and legacy code are often adjacent.
3.  **"Legacy" Trap**: Important logic is buried in `database/operations/legacy`.

## Proposed Structure (Feature-Based / Domain-Driven)

Instead of grouping by *Layer* (Handlers, Services), group by *Feature* (Authentication, Identity, Protocol).

```bash
infra/authenc/src/
├── domain/                 # PURE Business Logic & Models (No DB/HTTP deps ideally)
│   ├── identity/           # Users, Groups, Roles
│   ├── realm/              # Realms, Clients, Scopes
│   ├── auth/               # Auth Flows, Sessions, MFA
│   └── federation/         # OIDC/SAML Providers
│
├── application/            # Orchestration / Use Cases
│   ├── api/                # Internal API specifics
│   └── protocol/           # OIDC / SAML specific orchestrators
│
├── infrastructure/         # External Systems Implementation
│   ├── persistence/        # Postgres/SQL implementations (was database/operations)
│   ├── cache/              # Redis implementations
│   └── crypto/             # Specific crypto adapters
│
└── interface/              # Transport Layer (was handlers)
    ├── http/
    │   ├── oidc/           # /auth, /token, /userinfo
    │   ├── admin_api/      # /admin/*
    │   └── public_api/     # /api/*
    └── grpc/               # Tonic services
```

## Immediate "Quick Wins" (Low Effort, High Value)

If a full rewrite is too much, we can simply **Group the Flat Lists**:

### 1. Refactor `src/handlers`
Current: Flat list of ~40 files.
New:
```bash
src/handlers/
├── oidc/           # authorize.rs, token.rs, userinfo.rs, jwks.rs
├── api/            # user.rs, group.rs, realm.rs (REST API)
├── auth/           # login.rs, logout.rs, callback.rs
├── saml/           # sso.rs, metadata.rs
└── internal/       # health.rs, metrics.rs
```

### 2. Refactor `src/services`
Current: Mix of generic services and specific logic.
New:
```bash
src/services/
├── security/       # brute_force.rs, rate_limit.rs, ip_check.rs
├── identity/       # user_service.rs, password.rs, profile.rs
├── session/        # session_manager.rs, token_service.rs
└── integration/    # webhooks.rs, event_listener.rs
```

### 3. Rename `legacy`
`database/operations/legacy` seems to contain active code. Rename it to `core` or `impl` to reflect reality, or refactor it into proper repositories.

## Recommendation
Start with **Quick Win #1 (Handlers)** and **Quick Win #2 (Services)**. This cleans up the navigation significantly without requiring a massive architectural rewrite of the domain objects.
