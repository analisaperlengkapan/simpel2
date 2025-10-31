# Sinergi Validasi Frontend-Backend

## Overview

Dokumen ini menganalisis sinergi antara validasi input di backend (Authenc) dan frontend (Portal & Shared Microfrontend) untuk memastikan konsistensi dan menghindari duplikasi yang tidak perlu.

## Tanggal Analisis

30 Oktober 2025

## Prinsip Validasi

### Defense in Depth (Pertahanan Berlapis)

Validasi dilakukan di **dua layer**:

1. **Frontend (Client-side)**: Validasi cepat untuk UX yang baik
2. **Backend (Server-side)**: Validasi keamanan yang tidak bisa di-bypass

**Catatan Penting**: Validasi frontend BUKAN untuk keamanan, tetapi untuk user experience. Validasi backend adalah satu-satunya validasi yang dapat dipercaya untuk keamanan.

## Perbandingan Implementasi

### 1. Email Validation

#### Frontend (`antarmuka/shared/src/utils/validation.rs`)
```rust
pub const REGEX_EMAIL: &str = r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$";

pub fn validate_email(email: &str) -> ValidationResult {
    // Validasi dengan pesan dalam Bahasa Indonesia
    // Maksimal 255 karakter (EMAIL_MAX_LENGTH)
}
```

#### Backend (`infra/authenc/src/utils/validation.rs`)
```rust
static EMAIL_REGEX: &str = r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$";

pub fn validate_email(email: &str) -> bool {
    email_regex().is_match(email)
}
```

**Status**: ✅ **SINKRON** - Regex pattern sama, implementasi konsisten

### 2. NIP Validation (Nomor Induk Pegawai)

#### Frontend
```rust
pub const REGEX_NIP: &str = r"^[0-9]{18}$";

pub fn validate_nip(nip: &str) -> ValidationResult {
    // Pesan: "NIP harus 18 digit angka"
}
```

#### Backend
```rust
static NIP_REGEX: &str = r"^\d{18}$";

pub fn validate_nip(nip: &str) -> bool {
    nip_regex().is_match(nip)
}
```

**Status**: ✅ **SINKRON** - Pattern equivalent (`[0-9]` = `\d`), validasi konsisten

### 3. Phone Number Validation

#### Frontend
```rust
pub const REGEX_PHONE: &str = r"^(\+62|62|0)[0-9]{9,13}$";

pub fn validate_phone(phone: &str) -> ValidationResult {
    // Format Indonesia: 08xxx, +62xxx, 62xxx
}
```

#### Backend
```rust
static PHONE_REGEX: &str = r"^\+?[1-9]\d{1,14}$";

pub fn validate_phone_number(phone: &str) -> bool {
    // Format internasional (E.164)
}
```

**Status**: ⚠️ **BERBEDA** - Frontend lebih spesifik untuk Indonesia, Backend lebih umum

**Rekomendasi**:
- Frontend: Tetap gunakan validasi Indonesia-specific untuk UX
- Backend: Tetap gunakan validasi internasional untuk fleksibilitas
- Ini adalah perbedaan yang **disengaja dan valid**

### 4. Username Validation

#### Frontend
```rust
// Tidak ada validasi username spesifik di shared
// Menggunakan validate_required() dan validate_length()
```

#### Backend
```rust
static USERNAME_REGEX: &str = r"^[a-zA-Z0-9_-]{3,50}$";

pub fn validate_username(username: &str) -> bool {
    // 3-50 karakter, alphanumeric dengan _ atau -
}
```

**Status**: ⚠️ **PERLU DITAMBAHKAN DI FRONTEND**

### 5. Password Validation

#### Frontend
```rust
// Tidak ada validasi password complexity di shared
```

#### Backend
```rust
static PASSWORD_COMPLEXITY_REGEX: &str = r"^(?=.*[a-z])(?=.*[A-Z])(?=.*\d).+$";

pub fn validate_password_complexity(password: &str) -> bool {
    // Min 8 chars, uppercase, lowercase, digit
}
```

**Status**: ⚠️ **PERLU DITAMBAHKAN DI FRONTEND**

### 6. Satker Code Validation

#### Frontend
```rust
// Tidak ada validasi satker code di shared
```

#### Backend
```rust
static SATKER_CODE_REGEX: &str = r"^[A-Z0-9]{2,20}$";

pub fn validate_satker_code(code: &str) -> bool {
    // 2-20 uppercase alphanumeric
}
```

**Status**: ⚠️ **PERLU DITAMBAHKAN DI FRONTEND**

## Sanitization

### Frontend (`antarmuka/shared/src/utils/security.rs`)

```rust
pub fn sanitize_input(input: &str) -> String {
    // XSS prevention: escape HTML entities
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        // ...
}

pub fn sanitize_html(input: &str) -> String {
    // Whitelist-based HTML sanitization
    // Remove script tags, event handlers, etc.
}
```

### Backend (`infra/authenc/src/utils/validation.rs`)

```rust
pub fn sanitize_string(input: &str, max_length: usize) -> String {
    // Remove null bytes, control characters
    // Trim whitespace, limit length
}

pub fn sanitize_username(username: &str) -> String {
    // Keep only alphanumeric, _, -
}

pub fn sanitize_email(email: &str) -> String {
    // Lowercase, allowed email characters only
}
```

**Status**: ✅ **KOMPLEMENTER** - Frontend fokus XSS, Backend fokus injection & data integrity

## Rekomendasi Implementasi

### 1. Tambahkan Validasi di Frontend (Shared Microfrontend)

Buat file baru: `antarmuka/shared/src/utils/auth_validation.rs`

```rust
//! Validasi khusus untuk authentication dan user management
//! Sinkron dengan backend validation di authenc

use crate::core::types::*;
use regex::Regex;

/// Validate username format
/// Sinkron dengan: infra/authenc/src/utils/validation.rs::validate_username
pub fn validate_username(username: &str) -> ValidationResult {
    let regex = Regex::new(r"^[a-zA-Z0-9_-]{3,50}$").unwrap();

    if username.is_empty() {
        return ValidationResult::invalid(vec![ValidationError::new(
            "username",
            "Username tidak boleh kosong",
        )]);
    }

    if !regex.is_match(username) {
        return ValidationResult::invalid(vec![ValidationError::new(
            "username",
            "Username harus 3-50 karakter (huruf, angka, _, -)",
        )]);
    }

    ValidationResult::valid()
}

/// Validate password complexity
/// Sinkron dengan: infra/authenc/src/utils/validation.rs::validate_password_complexity
pub fn validate_password(password: &str) -> ValidationResult {
    let mut errors = Vec::new();

    if password.len() < 8 {
        errors.push(ValidationError::new(
            "password",
            "Password minimal 8 karakter",
        ));
    }

    if !password.chars().any(|c| c.is_uppercase()) {
        errors.push(ValidationError::new(
            "password",
            "Password harus mengandung huruf besar",
        ));
    }

    if !password.chars().any(|c| c.is_lowercase()) {
        errors.push(ValidationError::new(
            "password",
            "Password harus mengandung huruf kecil",
        ));
    }

    if !password.chars().any(|c| c.is_numeric()) {
        errors.push(ValidationError::new(
            "password",
            "Password harus mengandung angka",
        ));
    }

    if errors.is_empty() {
        ValidationResult::valid()
    } else {
        ValidationResult::invalid(errors)
    }
}

/// Validate satker code format
/// Sinkron dengan: infra/authenc/src/utils/validation.rs::validate_satker_code
pub fn validate_satker_code(code: &str) -> ValidationResult {
    let regex = Regex::new(r"^[A-Z0-9]{2,20}$").unwrap();

    if code.is_empty() {
        return ValidationResult::invalid(vec![ValidationError::new(
            "satker_code",
            "Kode satker tidak boleh kosong",
        )]);
    }

    if !regex.is_match(code) {
        return ValidationResult::invalid(vec![ValidationError::new(
            "satker_code",
            "Kode satker harus 2-20 karakter (huruf besar dan angka)",
        )]);
    }

    ValidationResult::valid()
}
```

### 2. Update Shared Module Exports

File: `antarmuka/shared/src/utils/mod.rs`

```rust
pub mod validation;
pub mod auth_validation; // TAMBAHKAN INI
pub mod security;
// ...
```

### 3. Gunakan di Portal Login Form

File: `antarmuka/portal/src/pages/login.rs` (contoh)

```rust
use shared_microfrontend::utils::auth_validation::*;

// Di dalam form handler
let username_validation = validate_username(&username);
if !username_validation.valid {
    // Show errors
    return;
}

let password_validation = validate_password(&password);
if !password_validation.valid {
    // Show errors
    return;
}

// Submit ke backend
```

### 4. Shared Constants untuk Sinkronisasi

Buat file: `antarmuka/shared/src/core/auth_constants.rs`

```rust
//! Constants untuk authentication
//! HARUS SINKRON dengan infra/authenc/src/utils/validation.rs

/// Username length constraints
pub const USERNAME_MIN_LENGTH: usize = 3;
pub const USERNAME_MAX_LENGTH: usize = 50;

/// Password length constraints
pub const PASSWORD_MIN_LENGTH: usize = 8;
pub const PASSWORD_MAX_LENGTH: usize = 128;

/// Satker code length constraints
pub const SATKER_CODE_MIN_LENGTH: usize = 2;
pub const SATKER_CODE_MAX_LENGTH: usize = 20;

/// NIP length (fixed)
pub const NIP_LENGTH: usize = 18;

/// Email max length
pub const EMAIL_MAX_LENGTH: usize = 255;

/// Request body size limit (1MB)
pub const MAX_REQUEST_BODY_SIZE: usize = 1_048_576;
```

## Perbedaan yang Valid (By Design)

### 1. Bahasa Error Messages

- **Frontend**: Bahasa Indonesia (untuk user)
- **Backend**: Bahasa Inggris (untuk API, logging, debugging)

**Alasan**: Frontend mengutamakan UX untuk user Indonesia, Backend mengutamakan standar internasional untuk developer dan integrasi.

### 2. Validasi Phone Number

- **Frontend**: Format Indonesia (`+62`, `08xxx`)
- **Backend**: Format internasional (E.164)

**Alasan**: Frontend memberikan guidance spesifik untuk user Indonesia, Backend menerima format internasional untuk fleksibilitas.

### 3. Tingkat Detail Validasi

- **Frontend**: Validasi basic untuk UX cepat
- **Backend**: Validasi comprehensive untuk keamanan

**Alasan**: Frontend tidak boleh terlalu strict (mengganggu UX), Backend harus strict (keamanan).

## Checklist Sinkronisasi

### ✅ Sudah Sinkron
- [x] Email regex pattern
- [x] NIP format (18 digits)
- [x] Sanitization approach (komplementer)

### ⚠️ Perlu Ditambahkan di Frontend
- [ ] Username validation (3-50 chars, alphanumeric + _ -)
- [ ] Password complexity validation
- [ ] Satker code validation (2-20 uppercase alphanumeric)

### ✅ Perbedaan yang Valid
- [x] Phone number format (Indonesia vs International)
- [x] Error message language (ID vs EN)
- [x] Validation strictness (UX vs Security)

## Testing Strategy

### Frontend Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_username_validation_sync_with_backend() {
        // Test cases yang sama dengan backend
        assert!(validate_username("user123").valid);
        assert!(validate_username("test_user").valid);
        assert!(!validate_username("ab").valid); // too short
        assert!(!validate_username("user@name").valid); // invalid char
    }
}
```

### Integration Tests
- Test login flow dengan validasi frontend + backend
- Verify error messages konsisten
- Test edge cases di kedua layer

## Maintenance Guidelines

### Saat Update Validasi

1. **Update Backend First**
   - Implementasi di `infra/authenc/src/utils/validation.rs`
   - Update tests
   - Update dokumentasi

2. **Update Frontend**
   - Implementasi di `antarmuka/shared/src/utils/auth_validation.rs`
   - Update constants jika perlu
   - Update tests

3. **Update Documentation**
   - Update dokumen ini
   - Update API documentation
   - Update user documentation

### Code Review Checklist

- [ ] Regex patterns sama antara frontend dan backend?
- [ ] Length constraints konsisten?
- [ ] Error messages jelas dan helpful?
- [ ] Tests mencakup edge cases yang sama?
- [ ] Dokumentasi sudah diupdate?

## Kesimpulan

### Implementasi Saat Ini

**Backend (Authenc)**: ✅ **COMPLETE**
- Garde validation framework
- Comprehensive validators (email, username, password, NIP, satker, phone)
- Sanitization functions
- Request size limits (1MB)
- Custom garde validators

**Frontend (Shared)**: ⚠️ **PARTIAL**
- Basic validation (email, NIP, phone, name)
- XSS prevention & sanitization
- Perlu ditambahkan: username, password, satker code validation

### Action Items

1. **High Priority**: Tambahkan `auth_validation.rs` di shared microfrontend
2. **Medium Priority**: Buat shared constants untuk sinkronisasi
3. **Low Priority**: Integration tests untuk validasi end-to-end

### Prinsip Utama

> **"Never trust the client"** - Validasi frontend adalah untuk UX, validasi backend adalah untuk security. Keduanya diperlukan dan saling melengkapi, bukan duplikasi.

## References

- Backend Validation: `infra/authenc/src/utils/validation.rs`
- Frontend Validation: `antarmuka/shared/src/utils/validation.rs`
- Frontend Security: `antarmuka/shared/src/utils/security.rs`
- Backend Implementation Doc: `infra/authenc/TASK_7.5_INPUT_VALIDATION_IMPLEMENTATION.md`
