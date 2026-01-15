---
applyTo: "infra/**/*.rs"
---

# Infrastructure Component Guidelines (Authenc & Secreton)

## Part of Main Workspace
Authenc and Secreton are now part of the main workspace (not separate):
```bash
cargo build -p authenc          # Build IAM system
cargo build -p secreton         # Build secret management
cargo build -p secreton-api     # Build REST/gRPC API
cargo build -p secreton-grpc    # Build gRPC service
cargo build -p secreton-agent   # Build sidecar agent
cargo build -p secreton-cli     # Build CLI tool
```

## Service Ports
| Service | HTTP | gRPC |
|---------|------|------|
| Authenc | 8088 | 9088 |
| Secreton | 8200 | 9090 |

## Security Requirements
- All communication via mTLS
- Ed25519 for JWT signing
- ChaCha20-Poly1305 for encryption
- Hardware Security Module (HSM) support via `lib-hsm`

## Testing
```bash
./scripts/verify-health.sh       # Health check
./scripts/test-integration.sh    # Full API tests
```

## Secreton Crate Structure
```
infra/secreton/
├── Cargo.toml          # Main secreton crate
└── crates/
    ├── api/            # secreton-api: REST/gRPC handlers
    ├── core/           # secreton-core: Core types
    ├── grpc/           # secreton-grpc: gRPC service
    ├── agent/          # secreton-agent: Sidecar
    └── cli/            # secreton-cli: CLI tool
```

## Never Do
- ❌ Call from microfrontends directly
- ❌ Expose to public internet without API gateway
- ❌ Store secrets in environment variables
- ❌ Use RSA for new implementations
