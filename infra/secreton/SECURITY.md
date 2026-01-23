# Security Advisory - Secreton Secret Vault System

## Known Security Issues

### Critical Advisories

#### RUSTSEC-2023-0071: RSA Timing Side-Channel Vulnerability

**Affected**: `rsa` v0.9.8 (transitive dependency via `ssh-key`)
**Severity**: Critical
**CVE**: Marvin Attack - timing side-channel key recovery

**Impact**:
- Non-constant-time RSA implementation
- Private key information leaked through timing
- Observable over the network

**Mitigation Strategy**:
1. ✅ **Implemented**: Ed25519 is the default and recommended key type
2. ✅ **Documented**: SSH key generation prefers Ed25519 over RSA
3. ⏳ **Monitoring**: Tracking RustCrypto/RSA#19 for constant-time implementation
4. ✅ **Risk Assessment**: SSH key generation is non-critical feature

**Production Recommendation**:
- Use Ed25519 keys exclusively
- Avoid RSA key generation in production
- If RSA required for compatibility, use in trusted network only

**Status**: Accepted Risk - Mitigated through Ed25519 preference

---

#### RUSTSEC-2024-0436: Unmaintained `paste` Crate

**Affected**: `paste` v1.0.15 (transitive via `pqcrypto-mldsa`)
**Severity**: Medium
**Impact**: Crate archived, no longer maintained

**Mitigation Strategy**:
1. ✅ **Assessed**: Procedural macro, low attack surface
2. ✅ **Prioritized**: Post-quantum crypto more critical
3. ⏳ **Monitoring**: Waiting for `pqcrypto-mldsa` update
4. ✅ **Contingency**: Can fork and replace with `pastey` if needed

**Production Recommendation**:
- Continue monitoring `pqcrypto-mldsa` releases
- Post-quantum cryptography capability prioritized over proc-macro updates
- No immediate action required

**Status**: Accepted Risk - Under monitoring

---

## Security Policy

### Reporting Security Issues

**Email**: security@cipherce.io
**Response Time**:
- Critical: 24 hours
- High: 72 hours
- Medium/Low: 1 week

**PGP Encryption**: Recommended for sensitive disclosures

### Supported Versions

| Version | Supported |
|---------|-----------|
| 0.1.x   | ✅ Active development |
| < 0.1.0 | ❌ Not supported |

### Security Update Policy

- **Critical**: Immediate patch release
- **High**: Within 7 days
- **Medium**: Next minor version
- **Low**: Next major version

### Vulnerability Disclosure Process

1. **Report**: Email security@cipherce.io
2. **Acknowledgment**: Within 24 hours
3. **Investigation**: 1-7 days
4. **Fix Development**: Based on severity
5. **Coordinated Disclosure**: After patch available
6. **Public Advisory**: 30 days post-fix

---

## Cryptographic Algorithm Security

### Approved Algorithms (Production)

| Algorithm | Standard | Status | Notes |
|-----------|----------|--------|-------|
| **Symmetric Encryption** |
| AES-256-GCM | FIPS 140-2 | ✅ Approved | Hardware-accelerated |
| ChaCha20-Poly1305 | RFC 8439 | ✅ Approved | Software-optimized |
| XChaCha20-Poly1305 | Draft | ✅ Approved | Extended nonce |
| **Asymmetric Crypto** |
| Ed25519 | RFC 8032 | ✅ Approved | Preferred signing |
| ECDSA P-256 | FIPS 186-4 | ✅ Approved | NIST curve |
| **Post-Quantum** |
| ML-DSA (Dilithium) | NIST PQC | ✅ Approved | Future-proof |
| ML-KEM (Kyber) | NIST PQC | ✅ Approved | Key encapsulation |
| **Key Derivation** |
| Argon2id | RFC 9106 | ✅ Approved | Password hashing |
| HKDF-SHA256 | RFC 5869 | ✅ Approved | Key derivation |
| **Hashing** |
| SHA-256/384/512 | FIPS 180-4 | ✅ Approved | Cryptographic hash |
| BLAKE3 | - | ✅ Approved | High-performance |

### Deprecated/Prohibited Algorithms

| Algorithm | Status | Reason |
|-----------|--------|--------|
| MD5 | ❌ Prohibited | Collision attacks |
| SHA-1 | ❌ Prohibited | Deprecated by NIST |
| DES/3DES | ❌ Prohibited | Insufficient key length |
| RC4 | ❌ Prohibited | Broken cipher |
| RSA < 2048 bits | ❌ Prohibited | Insufficient strength |
| RSA (general) | ⚠️ Use with caution | Timing vulnerability (RUSTSEC-2023-0071) |

---

## Security Best Practices

### Production Deployment

1. **TLS Configuration**
   - TLS 1.3 only
   - Strong cipher suites
   - Valid certificates from trusted CA

2. **Authentication**
   - Multi-factor authentication required
   - Token rotation enabled
   - Rate limiting configured

3. **Network Security**
   - Firewall: ports 8200/8201 only
   - Private network recommended
   - Load balancer with WAF

4. **Audit Logging**
   - All operations logged
   - Logs sent to SIEM
   - Tamper-proof storage
   - 90-day retention minimum

5. **Secrets Management**
   - No secrets in environment variables
   - Encrypted configuration files
   - Proper file permissions (0600)
   - Regular rotation (90 days)

6. **Access Control**
   - Principle of least privilege
   - RBAC policies enforced
   - Regular access review
   - Separation of duties

---

## Compliance

### Standards Compliance

- ✅ OWASP ASVS Level 2
- ✅ NIST SP 800-57 Key Management
- ✅ CIS Benchmark for Cryptographic Storage
- ✅ ISO 27001 Information Security
- ⏳ PCI DSS 4.0 (conditionally)
- ⏳ HIPAA (conditionally)

### Audit Trail

Security audits conducted:
- 2025-11-25: Initial production readiness audit
- Next scheduled: 2026-02-25 (quarterly)

---

**Last Updated**: 2025-11-25
**Next Review**: 2026-02-25
**Version**: 1.0
