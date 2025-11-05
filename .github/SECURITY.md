# Security Policy

## 🔒 Reporting Security Vulnerabilities

The SIMPelv2 team takes security seriously. We appreciate your efforts to responsibly disclose your findings.

### ⚠️ **DO NOT** disclose security vulnerabilities publicly

If you discover a security vulnerability, please follow these steps:

### 1. **Private Reporting (Preferred)**

Use GitHub's private vulnerability reporting feature:
1. Go to the [Security tab](https://github.com/analisaperlengkapan/simpel2/security)
2. Click "Report a vulnerability"
3. Fill out the form with details

### 2. **Email Reporting**

Send details to: **security@kejaksaan.go.id**

Include:
- Description of the vulnerability
- Steps to reproduce
- Potential impact
- Suggested fix (if any)
- Your contact information

### 3. **Encrypted Communication**

For highly sensitive issues, request our PGP key via email first.

## 🛡️ Security Response Process

1. **Acknowledgment**: We will acknowledge your report within **48 hours**
2. **Investigation**: Our security team will investigate and validate the issue
3. **Updates**: We will provide regular updates every **7 days**
4. **Fix & Release**: We will develop and test a fix
5. **Disclosure**: We request **90 days** before public disclosure
6. **Credit**: Security researchers will be credited (if desired)

## 🎯 Scope

### In Scope

- **Authentication & Authorization** (`infra/authenc/`)
- **Secreton Secreton** (`infra/secreton/`)
- **API Gateway** (`infra/gerbang/`)
- **Backend Services** (`layanan/`)
- **Frontend Security** (`antarmuka/`)
- **Infrastructure** (Docker, K8s configurations)
- **Cryptographic implementations**
- **Session management**
- **Input validation**
- **SQL injection vulnerabilities**
- **XSS vulnerabilities**
- **CSRF vulnerabilities**
- **Authentication bypass**
- **Privilege escalation**
- **Data exposure**

### Out of Scope

- Denial of Service (DoS) attacks
- Social engineering attacks
- Physical attacks
- Issues in third-party dependencies (report to maintainers)
- Issues requiring unlikely user interaction
- Vulnerabilities in outdated versions (report if affecting latest stable)

## 🔐 Security Best Practices in SIMPelv2

### Current Security Measures

1. **Zero-Trust Architecture**: No implicit trust between components
2. **Multi-Factor Authentication (MFA)**: TOTP-based 2FA
3. **Role-Based Access Control (RBAC)**: Granular permissions
4. **Cryptography**:
   - Ed25519 for digital signatures
   - Blake3 & SHA2 for hashing (NO SHA1)
   - ChaCha20-Poly1305 & AES-GCM for encryption
5. **Secure Secreton**: Secreton + Secreton for secrets management
6. **Content Security Policy (CSP)**: Strict CSP headers
7. **HSTS**: HTTP Strict Transport Security enabled
8. **Input Validation**: All user inputs validated
9. **SQL Injection Protection**: Parameterized queries (tokio-postgres)
10. **XSS Protection**: Leptos framework provides automatic escaping
11. **Audit Logging**: Immutable audit trail
12. **Dependency Scanning**: Automated cargo-audit in CI/CD
13. **Container Security**: Trivy scanning for Docker images

### Secure Development Guidelines

- **NO hardcoded credentials** - Use Secreton/environment variables
- **NO SHA1** - Use Blake3 or SHA2 (migrated)
- **NO sqlx with RSA vulnerability** - Use tokio-postgres (migrated)
- **Input validation** - Validate all external inputs
- **Least privilege** - Grant minimum necessary permissions
- **Secure defaults** - Security-first configuration
- **Regular updates** - Keep dependencies current (Dependabot)

## 📋 Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.4.x   | ✅ Yes             |
| 0.3.x   | ✅ Yes (LTS)       |
| < 0.3   | ❌ No              |

## 🏆 Security Hall of Fame

We recognize and thank security researchers who responsibly disclose vulnerabilities:

<!-- Hall of Fame entries will be added here -->

*No entries yet - be the first!*

## 📚 Security Resources

- [OWASP Top 10](https://owasp.org/www-project-top-ten/)
- [Rust Security Guidelines](https://anssi-fr.github.io/rust-guide/)
- [CWE Top 25](https://cwe.mitre.org/top25/)
- [SIMPelv2 Security Documentation](../docs/security/)

## 🔗 Related Policies

- [Code of Conduct](CODE_OF_CONDUCT.md)
- [Contributing Guidelines](CONTRIBUTING.md)
- [Privacy Policy](docs/PRIVACY.md)

---

**Last Updated**: October 2025  
**Contact**: security@kejaksaan.go.id
