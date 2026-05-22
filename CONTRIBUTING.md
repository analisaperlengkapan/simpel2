# 🛠️ Panduan Kontribusi SIMPEL

SIMPEL adalah proyek Kejaksaan yang terbuka untuk pegawai Kejaksaan dalam kontribusi baik berpengalaman maupun pemula yang ingin belajar sambil membangun sistem modern dengan arsitektur **microfrontend** dan **microservices**.

💡 **Tidak harus langsung ahli. Kami percaya bahwa kontribusi terbaik sering kali lahir dari proses belajar bersama.**

## 🎯 **Kontributor akan mendapatkan:**

* 🏗️ **Paparan Arsitektur Modern**: Microfrontend dengan Leptos + WebAssembly
* 🦀 **Rust Ecosystem**: Backend microservices dengan performa tinggi
* ☸️ **Cloud Infrastructure**: K8s, CI/CD, observability modern
* 🔐 **Enterprise Security**: Zero-trust, multi-factor auth, compliance
* 🌱 **Lingkungan Suportif**: Mentoring dan pembelajaran berkelanjutan
* 🏛️ **Dampak Nyata**: Sistem yang digunakan pada skala instansi Kejaksaan

Mulai langkah kecil Anda hari ini — dari:
* 🎨 Mengembangkan UI microfrontend dengan Leptos
* 🔧 Menulis microservice dengan Rust
* 📚 Memperbaiki dokumentasi dan guides
* 🧪 Menambahkan test coverage dan quality assurance
* 🔐 Implementasi security features dan compliance

---

## 📑 Daftar Isi

* [🧭 Alur Kontribusi](#-alur-kontribusi)
* [🌿 Struktur Branch](#-struktur-branch)
* [📝 Konvensi Penamaan](#-konvensi-penamaan-naming-conventions)
* [📛 Format Commit](#-format-commit)
* [📋 Template Merge Request](#-template-merge-request)
* [🤖 Bentuk Kontribusi yang Didukung](#-bentuk-kontribusi-yang-didukung)
* [💡 Tips: Gunakan AI untuk Mendukung Proses Kontribusi](#-tips-gunakan-ai-untuk-mendukung-proses-kontribusi)
* [🧩 Memaksimalkan Fitur GitLab](#-memaksimalkan-fitur-gitlab)
* [🔒 Kerahasiaan](#-kerahasiaan--security-guidelines)
* [🛡️ Pelaporan Keamanan](#-security-reporting--support)

---

## 🧭 Alur Kontribusi

### 👩‍💻 Persiapan Awal untuk Developer

#### **🖥️ Setup Development Environment**

1. **Install Rust & Toolchain**

   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   rustup component add clippy rustfmt
   rustup target add wasm32-unknown-unknown
   ```

2. **Install Frontend Tools**

   ```bash
   cargo install trunk
   # atau: cargo binstall trunk
   ```

3. **Install Development Tools**

   ```bash
   cargo install cargo-audit cargo-watch
   ```

4. **Clone & Setup Project**

   ```bash
   git clone <URL_REPOSITORY> simpel
   cd simpel
   cp .env.example .env
   ```

5. **Start Development Environment**

   ```bash
   # Start database dan cache
   docker compose up -d postgres redis

   # Jalankan backend
   cargo run --bin layanan-perlengkapan

   # Jalankan frontend (di terminal terpisah)
   cd antarmuka/portal && trunk serve --port 8080 --open
   ```

### ⚠️ Batasan Arsitektural Penting (Wajib Dibaca)

1. **Dilarang menggunakan SQLx**: Ekosistem SIMPEL mewajibkan penggunaan `tokio-postgres`, `deadpool-postgres`, dan `refinery` murni untuk operasi *database*.
2. **Batas Shared Libraries**: `lib/common/` telah dipecah menjadi `lib/core/` (tipe WASM-safe), `lib/backend/` (infrastruktur backend: DB, middleware, gRPC), dan `lib/crypto/` (primitif kripto). DILARANG menambahkan dependensi async/backend ke `lib-core` untuk menjaga kompatibilitas WASM. Jika butuh shared logic baru yang tidak cocok di tiga crate tersebut, buat *shared crate* terpisah di folder `lib/` (misalnya `lib-telemetry` atau `lib-auth-client`) dan daftarkan pada root `Cargo.toml`.

#### **🪟 Windows Setup (WSL2)**

1. **Install WSL2**: Windows Subsystem for Linux
2. **Install VS Code**: Dengan WSL extension
3. **Setup Git**: Configure `user.name` dan `user.email`
4. **Install Docker Desktop**: Dengan WSL2 backend
5. **Follow Linux steps**: Dalam WSL2 environment

### 🔄 **Workflow Kontribusi**

1. **🍴 Fork Repository** → Clone → Checkout ke `main`
2. **🌿 Buat Feature Branch** (`feature/nama-fitur` atau `fix/deskripsi`)
3. **💻 Development**:
   * Frontend: Edit microfrontend dengan Leptos
   * Backend: Develop microservice dengan Rust
   * Documentation: Update README, API docs
4. **🧪 Testing & Validation**:
   * **Unit Tests**: Wajib ditambahkan secara *inline* (`#[cfg(test)]`) pada file yang sama dengan logika yang dibuat.
   * **Integration Tests**: Tambahkan pengujian yang lebih kompleks (seperti API endpoints & DB) di folder `tests/`.
   * **E2E Tests**: Khusus kontribusi UI (Microfrontend), buat skenario tes menggunakan Playwright bila memungkinkan.

   ```bash
   cargo test --workspace                                    # Semua tes
   cargo clippy --workspace --all-targets -- -D warnings     # Linter
   cargo audit                                               # Vulnerability check
   ```

5. **📝 Commit** dengan format baku → Push → Buat **Merge Request**

> ⚠️ **Important**: Semua merge request ke `main` branch. Feature branches dari `main` untuk consistency.

---

## 🌿 Struktur Branch

```
main ← production (stable releases)
├── feature/microfrontend-optimization
├── feature/security-improvements
├── fix/performance-issues
├── docs/api-documentation
├── infra/kubernetes-deployment
└── hotfix/critical-security-patch
```

---

## 📝 Konvensi Penamaan (Naming Conventions)

Untuk menghindari *warning* dari *linter* dan mematuhi idiom Rust, gunakan standar berikut:
* `snake_case` untuk nama variabel, fungsi, atribut objek, nama modul, dan nama *file*.
* `PascalCase` untuk nama `struct`, `enum`, `trait`, dan *type alias*.
* `SCREAMING_SNAKE_CASE` untuk penamaan variabel konstanta dan *static global*.

---

## 📛 Format Commit

Gunakan **Conventional Commits** untuk consistency:

```bash
feat: add real-time dashboard untuk pemantauan aset
fix: resolve authentication token expiry issue
docs: update microfrontend development guide
perf: optimize WASM bundle size untuk portal
security: implement CSP headers untuk semua frontends
test: add integration tests untuk authenc
refactor: standardize error handling across services
ci: update deployment pipeline untuk K8s
```

**Commit Types:**
* `feat`: New features atau enhancements
* `fix`: Bug fixes dan error resolution
* `docs`: Documentation changes
* `perf`: Performance improvements
* `security`: Security-related changes
* `test`: Test additions atau improvements
* `refactor`: Code refactoring tanpa functional changes
* `ci`: CI/CD pipeline changes
* `infra`: Infrastructure dan deployment changes

---

## 📋 Template Merge Request

```markdown
## 🎯 Ringkasan Perubahan
<!-- Jelaskan apa yang diubah dan mengapa -->
- ✨ Menambahkan [fitur baru/enhancement]
- 🐛 Memperbaiki [bug/issue tertentu]

## 🔧 Jenis Kontribusi
- [ ] 🎨 Frontend Microfrontend (Leptos + WASM)
- [ ] ⚙️ Backend Microservice (Rust + Axum)
- [ ] 🔐 Security Enhancement
- [ ] 📚 Documentation Update
- [ ] 🧪 Testing & Quality Assurance
- [ ] 🏗️ Infrastructure & DevOps

## ✅ Quality Checklist
- [ ] 🧪 Tests passing (`cargo test --workspace`)
- [ ] 🦀 Formatted (`cargo fmt --all`)
- [ ] 📝 No clippy warnings (`cargo clippy --workspace --all-targets -- -D warnings`)
- [ ] 🔒 Security scan clean (`cargo audit`)
- [ ] 📖 Documentation updated
- [ ] 🏗️ Build successful

## 🧪 Testing Performed
<!-- Jelaskan testing yang sudah dilakukan -->
- [ ] Manual testing di development environment
- [ ] Cross-browser testing (jika frontend changes)
- [ ] Performance testing (jika ada perubahan performa)

## 📸 Screenshots/Logs
<!-- Lampirkan screenshot untuk UI changes atau logs untuk backend changes -->

## 🔗 Related Issues
Closes #[issue-number]
```

---

## 🤖 Bentuk Kontribusi yang Didukung

### 👥 **Peran dan Kemampuan Kontributor**

**Semua level welcome — dari pemula hingga expert:**

#### **🎨 Frontend Developer**

* **Tech Stack**: Leptos 0.8.x, WebAssembly, Trunk, CSS
* **Responsibilities**: Microfrontend development, UI/UX, responsive design
* **Projects**: Portal dashboard, modular interfaces, component library

#### **⚙️ Backend Developer**

* **Tech Stack**: Rust, Axum 0.8.x, Tonic (gRPC), PostgreSQL, Docker
* **Responsibilities**: Microservices, APIs, database design, performance
* **Projects**: Perlengkapan API, Authenc identity services, Secreton vault

#### **🔐 Security Specialist**

* **Tech Stack**: Authenc, Secreton, Ed25519, RBAC, mTLS
* **Responsibilities**: Authentication, authorization, compliance, audit
* **Projects**: Zero-trust implementation, MFA, WebAuthn, SAML federation

#### **🧪 QA Engineer**

* **Tech Stack**: Rust testing framework, integration tests
* **Responsibilities**: Test automation, performance testing, security testing
* **Projects**: CI/CD testing, quality gates, monitoring

#### **🏗️ DevOps/Infrastructure**

* **Tech Stack**: Kubernetes, Docker, Nginx, Prometheus, Grafana
* **Responsibilities**: Infrastructure, deployment, monitoring, scaling
* **Projects**: K8s deployment, CI/CD pipelines, observability stack

#### **📖 Technical Writer**

* **Tech Stack**: Markdown, API documentation, Mermaid diagrams
* **Responsibilities**: Documentation, guides, API specs, tutorials
* **Projects**: Developer guides, API documentation, architecture docs

---

## 💡 Tips: Gunakan AI untuk Mendukung Proses Kontribusi

Manfaatkan alat bantu AI untuk meningkatkan efisiensi:

* **ChatGPT / Gemini / Claude** — Refactor kode, generate schema, debugging
* **GitHub Copilot / Codeium** — Saran kode otomatis
* **Rust Analyzer** — Intelligent code analysis dan suggestions

> Tetap lakukan validasi manual. AI adalah alat bantu, bukan pengganti tanggung jawab kontribusi.

### 🧰 **Development Tools**

```bash
# VS Code extensions yang recommended
code --install-extension rust-lang.rust-analyzer
code --install-extension ms-vscode-remote.remote-wsl
code --install-extension GitLab.gitlab-workflow
```

---

## 🧩 Memaksimalkan Fitur GitLab

### 🐞 **GitLab Issues**

* Gunakan untuk pelaporan bug & diskusi
* Gunakan label: `bug`, `enhancement`, `good first issue`, `security`

### 🎯 **Milestone**

* Kaitkan issue & MR ke milestone (mis. `v1.0.0`, `Sprint-Q1`)

### ✅ **Merge Request (MR)**

* Gunakan template MR di atas
* Tambahkan reviewer sesuai `CODEOWNERS`
* Tambahkan label status (`ready`, `needs review`)

### 🔄 **GitLab CI/CD**

```yaml
# .gitlab-ci.yml stages
stages:
  - validate       # Code formatting, linting
  - test          # Unit, integration tests
  - security      # Security vulnerability scan
  - build         # Build binaries dan WASM
  - deploy        # Deploy to staging/production
```

### 👥 **Code Review (CODEOWNERS)**

```
# Automatic reviewers
/antarmuka/              @frontend-team
/layanan/perlengkapan/   @perlengkapan-team
/layanan/authenc/        @security-team
/layanan/secreton/       @security-team
/lib/                    @core-team
/docs/                   @documentation-team
/infra/                  @devops-team
```

---

## 🔒 Kerahasiaan & Security Guidelines

### 🔐 **Information Security**

* **Classified Data**: Semua kode dan data internal bersifat rahasia
* **Access Control**: Gunakan principle of least privilege
* **Secure Development**: Follow OWASP security guidelines
* **Data Protection**: Implement proper data encryption dan masking

### ⚖️ **Compliance Requirements**

* **Internal Use Only**: Kode tidak boleh disebarkan tanpa izin resmi
* **Audit Trail**: Semua aktivitas development ter-log dan traceable
* **Legal Compliance**: Mengikuti regulasi pemerintah dan hukum negara

---

## 🛡️ Security Reporting & Support

### 🚨 **Security Vulnerability Reporting**

**Jika menemukan security vulnerability:**

1. **🚫 JANGAN** post di public issue tracker
2. **📧 Email Security Team**: `security@kejaksaan.go.id`
3. **⏰ Response Time**: Acknowledgment dalam 24 jam

### 🆘 **Getting Help & Support**

**📚 Documentation:**
* [🏗️ Architecture Guide](docs/README.md)
* [🤖 AI Agent Guide](AGENTS.md)
* [📦 Perlengkapan Service](layanan/perlengkapan/AGENTS.md)
* [🔗 Integrasi Service](layanan/integrasi/AGENTS.md)
* [🔐 Authenc Details](layanan/authenc/AGENTS.md)
* [🔒 Secreton Details](layanan/secreton/AGENTS.md)

**💬 Communication Channels:**
* **GitLab Issues**: Technical questions, bug reports
* **GitLab Discussions**: Architecture discussions, RFC proposals
* **Internal Slack**: `#simpel-dev` untuk daily discussions

**🏢 Contact Information:**
* **Project Lead**: Biro Perlengkapan Kejaksaan RI
* **Technical Lead**: SIMPEL Architecture Team
* **Email**: `biro.perlengkapan@kejaksaan.go.id`

### 🎯 **Escalation Path**

```
Developer Question → GitLab Issue → Team Lead → Division Head
Security Issue → Security Team → CISO → Executive Level
```

---

## 🙏 Acknowledgments

Terima kasih kepada semua kontributor yang telah membantu membangun **SIMPEL**:

* 🏛️ **Kejaksaan RI**: Institutional support dan vision
* 👥 **Development Team**: Dedication dalam building modern architecture
* 🔐 **Security Team**: Ensuring enterprise-grade security
* 📚 **Documentation Team**: Creating comprehensive guides
* 🧪 **QA Team**: Maintaining quality standards
* 🏗️ **DevOps Team**: Reliable infrastructure dan deployment

**Mari bersama-sama membangun sistem manajemen BMN yang lebih efisien, aman, dan cerdas untuk kemajuan bangsa Indonesia** 🇮🇩

---

*Last updated: February 2026 — SIMPEL Development Team*
