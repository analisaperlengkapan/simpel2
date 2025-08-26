# Changelog


## [Unreleased]

### Changed
- Workspace Cargo.toml: Seluruh layanan backend dan shared kini terdaftar di `[workspace].members` untuk build/test lintas layanan.
- SIMPelv2.code-workspace:
  - Semua backend dan shared sudah masuk ke `rust-analyzer.linkedProjects`.
  - Task build/check/test untuk seluruh backend sudah ditambahkan.
  - Label folder backend diperbaiki.
- Optimalisasi `.vscode/settings.json` (sebelumnya):
  - Konfigurasi Rust Analyzer disederhanakan dan hanya fitur penting yang diaktifkan.
  - Penambahan best practice Git (`git.enableCommitSigning`, `git.signCommits`).
  - Komentar rekomendasi extension tetap ada.
  - File sudah valid JSONC dan siap kolaborasi tim.

### Fixed
- Konsistensi workspace dan build lintas layanan backend.
- Memastikan workspace siap untuk pengembangan paralel dan kolaborasi tim.

---

Lihat detail perubahan dengan `git diff` pada file terkait.
