# Changelog

## [Unreleased]

### Changed
- Optimalisasi `.vscode/settings.json`:
  - Menghapus konfigurasi Rust Analyzer yang terlalu agresif (emergency crash fix).
  - Menyederhanakan dan mengadopsi konfigurasi optimal: hanya fitur penting Rust Analyzer yang diaktifkan (`checkOnSave.enable`, `cargo.allFeatures`, `procMacro.enable`, `cargo.loadOutDirsFromCheck`, `linkedProjects`).
  - Menambahkan best practice Git (`git.enableCommitSigning`, `git.signCommits`).
  - Komentar rekomendasi extension tetap ada.

### Fixed
- Memastikan `.vscode/settings.json` valid JSONC dan siap kolaborasi tim.

---

Lihat detail perubahan dengan `git diff .vscode/settings.json`.
