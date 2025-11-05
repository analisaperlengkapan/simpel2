---
trigger: manual
---

{
  "version": "2.0.0",
  "rules": [
    {
      "id": "vulnerability-assessment",
      "name": "Vulnerability Assessment & Penetration Testing",
      "description": "Comprehensive security testing workflow untuk SIMPelv2: Vulnerability Assessment → Penetration Testing → Security Hardening",
      "icon": "🛡️",
      "mode": "structured",
      "temperature": 0.2,
      "capabilities": {
        "multiTurn": true,
        "fileOperations": true,
        "codeGeneration": true
      },
      "tools": [
        "workspace",
        "files",
        "codebase",
        "think",
        "sequentialthinking"
      ],
      "workflow": {
        "entrypoint": "start",
        "steps": {
          "start": {
            "prompt": [
              "🛡️ Selamat datang di *Vulnerability Assessment & Penetration Testing*!",
              "Workflow komprehensif untuk mengidentifikasi dan mengeksploitasi kerentanan keamanan di SIMPelv2.",
              "",
              "**Pendekatan sistematis kami:**",
              "- 🔍 **Vulnerability Assessment**: Identifikasi kerentanan secara menyeluruh",
              "- ⚔️ **Penetration Testing**: Eksploitasi kerentanan untuk validasi",
              "- 🛠️ **Security Hardening**: Rekomendasi perbaikan keamanan",
              "",
              "**Cakupan testing meliputi:**",
              "- 🔐 Authentication & Authorization",
              "- 🌐 Web Application Security",
              "- 🗄️ Database Security",
              "- 🔗 API Security",
              "- ⚙️ Configuration Security",
              "- 🔒 Cryptography Implementation",
              "",
              "Mari mulai dengan memahami target sistem dan ruang lingkup testing."
            ],
            "next": "vulnerability_assessment"
          },

          "vulnerability_assessment": {
            "instructions": [
              "🔍 **TAHAP 1: VULNERABILITY ASSESSMENT**",
              "",
              "Gunakan tools untuk menganalisis sistem secara komprehensif:",
              "1. `@codebase` → Analisis arsitektur dan komponen sistem",
              "2. `@think` → Identifikasi area risiko tinggi",
              "3. `@sequentialthinking` → Breakdown sistematis komponen",
              "",
              "📋 **Langkah Assessment:**",
              "",
              "**1. Infrastructure Analysis**",
              "- [ ] 1.1 Analisis konfigurasi Docker & Kubernetes",
              "- [ ] 1.2 Review konfigurasi Nginx & Envoy gateway",
              "- [ ] 1.3 Pemeriksaan konfigurasi Secreton",
              "- [ ] 1.4 Audit konfigurasi PostgreSQL",
              "",
              "**2. Backend Security (Rust/Axum)**",
              "- [ ] 2.1 Analisis dependency Cargo.toml untuk vulnerabilitas known",
              "- [ ] 2.2 Review implementasi JWT authentication",
              "- [ ] 2.3 Audit middleware authorization",
              "- [ ] 2.4 Pemeriksaan CORS configuration",
              "- [ ] 2.5 Review rate limiting implementation",
              "",
              "**3. Frontend Security (Leptos/WASM)**",
              "- [ ] 3.1 Audit Content Security Policy (CSP)",
              "- [ ] 3.2 Review XSS protection mechanisms",
              "- [ ] 3.3 Analisis bundle security headers",
              "- [ ] 3.4 Pemeriksaan secure cookie configuration",
              "",
              "**4. API Security**",
              "- [ ] 4.1 Review OpenAPI/Swagger documentation untuk endpoint security",
              "- [ ] 4.2 Audit input validation & sanitization",
              "- [ ] 4.3 Pemeriksaan API rate limiting",
              "- [ ] 4.4 Review error handling untuk information disclosure",
              "",
              "**5. Database Security**",
              "- [ ] 5.1 Audit PostgreSQL configuration & permissions",
              "- [ ] 5.2 Review connection pooling security",
              "- [ ] 5.3 Pemeriksaan SQL injection protection",
              "- [ ] 5.4 Audit backup encryption",
              "",
              "**6. Cryptography & Data Protection**",
              "- [ ] 6.1 Review encryption key management di Vault",
              "- [ ] 6.2 Audit SSL/TLS configuration",
              "- [ ] 6.3 Pemeriksaan secure random generation",
              "- [ ] 6.4 Review data classification & protection",
              "",
              "Buat file `vulnerability_assessment.md` dengan struktur berpenomoran dan temuan awal."
            ],
            "confirm": {
              "template": [
                "---",
                "⏸️ CHECKPOINT - VULNERABILITY ASSESSMENT",
                "",
                "Assessment selesai dengan {{findings}} temuan potensial dari {{categories}} kategori.",
                "",
                "Opsi:",
                "1. ✅ Approve - lanjut ke Penetration Testing",
                "2. 🔧 Perlu perbaikan - sebutkan nomor area yang perlu direview",
                "3. 💬 Butuh klarifikasi - diskusi temuan spesifik",
                "---"
              ]
            },
            "next": "penetration_testing"
          },

          "penetration_testing": {
            "instructions": [
              "⚔️ **TAHAP 2: PENETRATION TESTING**",
              "",
              "Sekarang kita akan mencoba mengeksploitasi kerentanan yang teridentifikasi.",
              "Fokus pada testing praktis dan controlled exploitation.",
              "",
              "**Testing Methodology:**",
              "- Black-box testing: Tidak ada akses ke source code",
              "- Grey-box testing: Akses terbatas ke dokumentasi",
              "- White-box testing: Full akses untuk deep analysis",
              "",
              "**Test Cases:**",
              "",
              "**1. Authentication Bypass**",
              "- [ ] 1.1 Test JWT token manipulation",
              "- [ ] 1.2 Attempt session hijacking",
              "- [ ] 1.3 Test MFA bypass scenarios",
              "- [ ] 1.4 Review password policy enforcement",
              "",
              "**2. Authorization Flaws**",
              "- [ ] 2.1 Test privilege escalation",
              "- [ ] 2.2 Attempt access ke resources unauthorized",
              "- [ ] 2.3 Review role-based access control",
              "- [ ] 2.4 Test API endpoint permissions",
              "",
              "**3. Input Validation & Injection**",
              "- [ ] 3.1 Test SQL injection vectors",
              "- [ ] 3.2 Attempt XSS attacks",
              "- [ ] 3.3 Test command injection",
              "- [ ] 3.4 Review file upload restrictions",
              "",
              "**4. Web Application Security**",
              "- [ ] 4.1 Test CSRF protection",
              "- [ ] 4.2 Attempt clickjacking attacks",
              "- [ ] 4.3 Review security headers",
              "- [ ] 4.4 Test HSTS implementation",
              "",
              "**5. API Security Testing**",
              "- [ ] 5.1 Test API rate limiting",
              "- [ ] 5.2 Attempt API key brute force",
              "- [ ] 5.3 Review API versioning security",
              "- [ ] 5.4 Test parameter tampering",
              "",
              "**6. Configuration & Infrastructure**",
              "- [ ] 6.1 Test SSL/TLS weaknesses",
              "- [ ] 6.2 Review Docker security",
              "- [ ] 6.3 Test Kubernetes RBAC",
              "- [ ] 6.4 Attempt service discovery exploits",
              "",
              "**Risk Assessment:**",
              "- **Critical**: System compromise, data breach",
              "- **High**: Unauthorized access, data exposure",
              "- **Medium**: Performance impact, DoS",
              "- **Low**: Information disclosure, best practice",
              "",
              "Dokumentasikan semua test cases dan hasilnya di `penetration_test_report.md`."
            ],
            "confirm": {
              "template": [
                "---",
                "⏸️ CHECKPOINT - PENETRATION TESTING",
                "",
                "Testing selesai dengan {{tests}} test case dan {{exploits}} successful exploit.",
                "Risk level: {{riskLevel}}",
                "",
                "Opsi:",
                "1. ✅ Approve - lanjut ke Security Hardening",
                "2. 🔧 Butuh remediation - fokus pada critical findings",
                "3. 💬 Review hasil testing - diskusi exploit details",
                "---"
              ]
            },
            "next": "security_hardening"
          },

          "security_hardening": {
            "instructions": [
              "🛠️ **TAHAP 3: SECURITY HARDENING**",
              "",
              "Berdasarkan findings dari vulnerability assessment dan penetration testing,",
              "kita akan membuat rekomendasi konkret untuk memperkuat keamanan sistem.",
              "",
              "**Hardening Categories:**",
              "",
              "**1. Authentication & Access Control**",
              "- [ ] 1.1 Implementasi MFA yang lebih kuat",
              "- [ ] 1.2 Review dan update password policies",
              "- [ ] 1.3 Enhanced session management",
              "- [ ] 1.4 Improved JWT security implementation",
              "",
              "**2. Input Validation & Data Sanitization**",
              "- [ ] 2.1 Comprehensive input validation framework",
              "- [ ] 2.2 SQL injection prevention mechanisms",
              "- [ ] 2.3 XSS protection filters",
              "- [ ] 2.4 File upload security enhancements",
              "",
              "**3. Web Security Headers**",
              "- [ ] 3.1 Implementasi security headers lengkap",
              "- [ ] 3.2 Content Security Policy (CSP) hardening",
              "- [ ] 3.3 HSTS dan HPKP configuration",
              "- [ ] 3.4 Security.txt implementation",
              "",
              "**4. API Security**",
              "- [ ] 4.1 API gateway security enhancements",
              "- [ ] 4.2 Rate limiting dan throttling",
              "- [ ] 4.3 API versioning security",
              "- [ ] 4.4 Input/output encoding",
              "",
              "**5. Infrastructure Security**",
              "- [ ] 5.1 Docker & Kubernetes hardening",
              "- [ ] 5.2 Network security policies",
              "- [ ] 5.3 Database security enhancements",
              "- [ ] 5.4 Monitoring dan alerting setup",
              "",
              "**6. Cryptography & Data Protection**",
              "- [ ] 6.1 Key management improvements",
              "- [ ] 6.2 Encryption at rest dan in transit",
              "- [ ] 6.3 Secure random generation",
              "- [ ] 6.4 Certificate management",
              "",
              "**Implementation Priority:**",
              "- 🔴 Critical: Fix immediately",
              "- 🟠 High: Fix dalam 1 minggu",
              "- 🟡 Medium: Fix dalam 1 bulan",
              "- 🔵 Low: Fix dalam 3 bulan",
              "",
              "Buat `security_hardening_plan.md` dengan timeline dan owner untuk setiap task."
            ],
            "confirm": {
              "template": [
                "---",
                "⏸️ CHECKPOINT - SECURITY HARDENING",
                "",
                "Plan dibuat dengan {{recommendations}} rekomendasi dan {{criticalItems}} critical items.",
                "",
                "Opsi:",
                "1. ✅ Approve - implementasi hardening plan",
                "2. 🔧 Adjust priorities - ubah timeline atau scope",
                "3. 💬 Technical discussion - klarifikasi implementasi",
                "---"
              ]
            },
            "next": "implementation"
          },

          "implementation": {
            "instructions": [
              "🚀 **TAHAP 4: IMPLEMENTATION & VALIDATION**",
              "",
              "Implementasikan security hardening recommendations satu per satu.",
              "Setiap implementasi harus divalidasi dengan testing.",
              "",
              "**Implementation Process:**",
              "",
              "Untuk setiap security fix:",
              "- [ ] Identifikasi file dan komponen yang perlu diubah",
              "- [ ] Buat backup dari konfigurasi existing",
              "- [ ] Implementasikan perubahan dengan code review",
              "- [ ] Test perubahan di environment staging",
              "- [ ] Jalankan regression testing",
              "- [ ] Deploy ke production dengan rollback plan",
              "",
              "**Validation Checklist:**",
              "- [ ] Security scan pasca-implementasi",
              "- [ ] Penetration testing ulang untuk area yang di-fix",
              "- [ ] Performance testing untuk memastikan tidak ada impact",
              "- [ ] Documentation update untuk security procedures",
              "",
              "**Monitoring & Alerting:**",
              "- [ ] Setup monitoring untuk security events",
              "- [ ] Configure alerting untuk suspicious activities",
              "- [ ] Implementasi log analysis untuk threat detection",
              "",
              "Update `implementation_report.md` dengan status setiap task dan hasil validation."
            ],
            "confirm": {
              "template": [
                "---",
                "✅ SECURITY IMPLEMENTATION COMPLETE",
                "",
                "Semua security hardening telah diimplementasikan dan divalidasi.",
                "{{testsPassed}}/{{totalTests}} security tests passed.",
                "",
                "Opsi:",
                "1. 🎉 Selesai - close security assessment",
                "2. 🔄 Re-assessment - jalankan ulang setelah beberapa bulan",
                "3. 📋 Continuous monitoring - setup ongoing security monitoring",
                "---"
              ]
            }
          }
        }
      },

      "validation": {
        "rules": [
          "- Setiap temuan wajib memiliki severity level dan remediation steps",
          "- Penetration testing harus mengikuti ethical hacking guidelines",
          "- Security hardening plan harus memiliki timeline dan owner",
          "- Tidak boleh ada false positive tanpa verification",
          "- Dokumentasi harus comprehensive dan actionable"
        ]
      },

      "communication": {
        "style": "technical-professional",
        "emojis": {
          "vulnerability_assessment": "🔍",
          "penetration_testing": "⚔️",
          "security_hardening": "🛠️",
          "implementation": "🚀"
        },
        "confirmationFormat": [
          "---",
          "⏸️ CHECKPOINT - {{phaseName}}",
          "{{summary}}",
          "",
          "Opsi:",
          "1. ✅ Approve - lanjut ke tahap berikutnya",
          "2. 🔧 Perlu revisi - sebutkan area spesifik",
          "3. 💬 Diskusi teknis - klarifikasi implementasi",
          "---"
        ]
      }
    }
  ]
}
