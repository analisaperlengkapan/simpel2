# SSO Cookie Implementation - Backend vs Frontend Comparison

## Tujuan Dokumen

Dokumen ini membandingkan implementasi SSO cookie di backend dan frontend untuk memastikan:
1. **Tidak ada duplikasi fungsi** yang seharusnya hanya ada di satu sisi
2. **Sinergi yang baik** antara backend dan frontend
3. **Separation of concerns** yang jelas

## Perbandingan Struktur Data

### Backend: `SsoSession`
```rust
// infra/authenc/src/utils/sso_cookie.rs
pub struct SsoSession {
    pub session_id: String,
    pub user_id: String,
    pub username: String,
    pub email: Option<String>,
    pub roles: Vec<String>,
    pub created_at: DateTime<Utc>,      // ← Chrono DateTime
    pub expires_at: DateTime<Utc>,      // ← Chrono DateTime
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}
```

### Frontend: `SsoSession`
```rust
// antarmuka/shared/src/utils/sso_cookie.rs
pub struct SsoSession {
    pub session_id: String,
    pub user_id: String,
    pub username: String,
    pub email: Option<String>,
    pub roles: Vec<String>,
    pub created_at: String,             // ← ISO 8601 String
    pub expires_at: String,             // ← ISO 8601 String
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}
```

**Perbedaan**:
- Backend menggunakan `DateTime<Utc>` untuk operasi timestamp
- Frontend menggunakan `String` (ISO 8601) karena serialisasi JSON
- **Tidak ada duplikasi**: Struktur sama, tipe berbeda sesuai kebutuhan

## Perbandingan Fungsi

### 1. Cookie Creation

| Aspek | Backend | Frontend |
|-------|---------|----------|
| **Fungsi** | `create_cookie()` | ❌ Tidak ada |
| **Tanggung Jawab**ookie dengan security attributes | - |
| **Alasan** | Only backend can set HttpOnly cookies | Frontend tidak bisa set HttpOnly |
| **Status** | ✅ Tidak ada duplikasi | ✅ Tidak ada duplikasi |

**Backend Only**:
```rust
// Backend: SsoCookieManager::create_cookie()
pub fn create_cookie(&self, session: &SsoSession) -> Result<String> {
    let session_json = session.to_json()?;
    let encoded_session = base64::encode(&session_json);

    // Build cookie with security attributes
    let cookie = format!(
        "{}={}; Path={}; Max-Age={}; Domain={}; Secure; HttpOnly; SameSite={}",
        self.config.name,
        encoded_session,
        self.config.path,
        self.config.max_age,
        self.config.domain.as_ref().unwrap(),
        self.config.same_site
    );

    Ok(cookie)
}
```

### 2. Cookie Deletion

| Aspek | Backend | Frontend |
|-------|---------|----------|
| **Fungsi** | `delete_cookie()` | ❌ Tidak ada |
| **Tanggung Jawab** | Membuat cookie dengan Max-Age=0 | - |
| **Alasan** | Backend controls cookie lifecycle | Frontend triggers via logout endpoint |
| **Status** | ✅ Tidak ada duplikasi | ✅ Tidak ada duplikasi |

**Backend Only**:
```rust
// Backend: SsoCookieManager::delete_cookie()
pub fn delete_cookie(&self) -> String {
    format!(
        "{}=; Path={}; Max-Age=0; Domain={}; Secure; HttpOnly; SameSite={}",
        self.config.name,
        self.config.path,
        self.config.domain.as_ref().unwrap(),
        self.config.same_site
    )
}
```

### 3. Cookie Reading

| Aspek | Backend | Frontend |
|-------|---------|----------|
| **Fungsi** | `extract_session()` | `read_session()` |
| **Tanggung Jawab** | Read from HTTP headers | Read from document.cookie |
| **Alasan** | Different APIs for server vs browser | Different APIs for server vs browser |
| **Status** | ✅ Tidak ada duplikasi (berbeda API) | ✅ Tidak ada duplikasi (berbeda API) |

**Backend**:
```rust
// Backend: SsoCookieManager::extract_session()
pub fn extract_session(&self, headers: &HeaderMap) -> Result<Option<SsoSession>> {
    let cookie_header = headers.get(header::COOKIE)?;
    let cookie_str = cookie_header.to_str()?;

    // Parse cookie from HTTP header
    for cookie in cookie_str.split(';') {
        // ... decode and validate
    }
}
```

**Frontend**:
```rust
// Frontend: SsoCookieReader::read_session()
pub fn read_session(&self) -> Option<SsoSession> {
    let cookie_value = self.get_cookie_value()?;

    // Decode and validate
    let decoded = self.decode_base64(&cookie_value).ok()?;
    let session: SsoSession = serde_json::from_str(&decoded).ok()?;

    if session.is_valid() {
        Some(session)
    } else {
        None
    }
}

fn get_cookie_value(&self) -> Option<String> {
    let document = window()?.document()?;
    let html_doc = document.dyn_ref::<HtmlDocument>()?;
    let cookies = html_doc.cookie().ok()?;

    // Parse from document.cookie
    for cookie in cookies.split(';') {
        // ...
    }
}
```

**Kesimpulan**: Tidak ada duplikasi, berbeda API (HTTP headers vs document.cookie)

### 4. Session Validation

| Aspek | Backend | Frontend |
|-------|---------|----------|
| **Fungsi** | `is_valid()`, `is_expired()` | `is_valid()`, `is_expired()` |
| **Tanggung Jawab** | Server-side validation | Client-side validation |
| **Alasan** | Both need to validate expiration | Both need to validate expiration |
| **Status** | ⚠️ Duplikasi (tapi diperlukan) | ⚠️ Duplikasi (tapi diperlukan) |

**Backend**:
```rust
impl SsoSession {
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    pub fn is_valid(&self) -> bool {
        !self.is_expired()
    }
}
```

**Frontend**:
```rust
impl SsoSession {
    pub fn is_expired(&self) -> bool {
        if let Ok(expires) = chrono::DateTime::parse_from_rfc3339(&self.expires_at) {
            let now = chrono::Utc::now();
            expires.with_timezone(&chrono::Utc) < now
        } else {
            true // Consider expired if parsing fails
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.is_expired()
    }
}
```

**Kesimpulan**: Duplikasi diperlukan karena:
- Backend perlu validasi untuk security
- Frontend perlu validasi untuk UX (avoid unnecessary API calls)
- **Defense in depth**: Validasi di kedua sisi lebih aman

## Fungsi Unik Backend

### 1. Cookie Header Management
```rust
// Backend only
pub fn add_cookie_header(&self, headers: &mut HeaderMap, session: &SsoSession) -> Result<()>
pub fn add_delete_cookie_header(&self, headers: &mut HeaderMap) -> Result<()>
```

**Alasan**: Only backend can set HTTP response headers

### 2. OIDC Endpoints
```rust
// Backend only
pub async fn oidc_token_with_sso(...)
pub async fn oidc_authorize_with_sso(...)
pub async fn oidc_logout_with_sso(...)
```

**Alasan**: Backend handles OAuth2/OIDC flows

### 3. Session Creation
```rust
// Backend only
impl SsoSession {
    pub fn new(
        user_id: String,
        username: String,
        email: Option<String>,
        roles: Vec<String>,
        max_age_seconds: i64,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Self
}
```

**Alasan**: Only backend creates new sessions

## Fungsi Unik Frontend

### 1. Auth Context Integration
```rust
// Frontend only
impl From<SsoSession> for UserSession {
    fn from(sso: SsoSession) -> Self {
        // Convert SSO session to UserSession
    }
}
```

**Alasan**: Frontend needs to integrate with existing auth system

### 2. Session Initialization
```rust
// Frontend only
pub fn init_auth_from_sso_cookie()
```

**Alasan**: Frontend needs to restore session on app load

### 3. Session Monitoring
```rust
// Frontend only
pub fn setup_sso_session_monitor(interval_ms: i32)
```

**Alasan**: Frontend needs to monitor session validity for UX

### 4. Browser Cookie Access
```rust
// Frontend only
fn get_cookie_value(&self) -> Option<String> {
    // Access document.cookie
}
```

**Alasan**: Only frontend can access document.cookie

## Separation of Concerns

### Backend Responsibilities

| Tanggung Jawab | Implementasi | Alasan |
|----------------|--------------|--------|
| **Cookie Creation** | `create_cookie()` | Security: HttpOnly, Secure flags |
| **Cookie Deletion** | `delete_cookie()` | Lifecycle management |
| **Session Creation** | `SsoSession::new()` | Authentication logic |
| **Server Validation** | `extract_session()` | API security |
| **OIDC Endpoints** | `oidc_*_with_sso()` | OAuth2 flows |

### Frontend Responsibilities

| Tanggung Jawab | Implementasi | Alasan |
|----------------|--------------|--------|
| **Cookie Reading** | `read_session()` | Browser API access |
| **Client Validation** | `is_valid()` | UX optimization |
| **Auth Integration** | `From<SsoSession>` | UI state management |
| **Session Monitoring** | `setup_sso_session_monitor()` | Auto-logout UX |
| **Initialization** | `init_auth_from_sso_cookie()` | App bootstrap |

### Shared Responsibilities

| Tanggung Jawab | Backend | Frontend | Alasan |
|----------------|---------|----------|--------|
| **Expiration Check** | ✅ | ✅ | Defense in depth |
| **Base64 Decode** | ✅ | ✅ | Data format |
| **JSON Parse** | ✅ | ✅ | Serialization |

## Kesimpulan

### ✅ Tidak Ada Duplikasi Berbahaya

1. **Cookie Creation**: Backend only (HttpOnly requirement)
2. **Cookie Deletion**: Backend only (lifecycle control)
3. **Session Creation**: Backend only (authentication logic)
4. **OIDC Endpoints**: Backend only (OAuth2 flows)
5. **Auth Integration**: Frontend only (UI state)
6. **Session Monitoring**: Frontend only (UX)

### ⚠️ Duplikasi yang Diperlukan

1. **Expiration Validation**: Both (defense in depth)
2. **Base64 Decode**: Both (data format)
3. **JSON Parse**: Both (serialization)

**Alasan**: Security best practice - never trust client-side validation

### 🔄 Sinergi yang Baik

1. **Same Data Structure**: Backend dan frontend menggunakan struktur yang sama
2. **Same Cookie Name**: AUTHENC_SSO di kedua sisi
3. **Same Encoding**: Base64 + JSON di kedua sisi
4. **Clear Boundaries**: Backend creates, frontend reads
5. **Complementary**: Backend security, frontend UX

## Rekomendasi

### ✅ Keep As Is

- Struktur data yang sama
- Validasi di kedua sisi
- Separation of concerns yang jelas

### 🔧 Future Improvements

1. **Session Encryption**: Encrypt session data before base64 encoding
2. **HMAC Signature**: Add signature for tamper detection
3. **Redis Storage**: Store sessions server-side, cookie as key
4. **Token Refresh**: Automatic token refresh before expiration

### 📝 Documentation

- ✅ Backend implementation documented
- ✅ Frontend implementation documented
- ✅ Integration guide created
- ✅ Comparison documented

## Checklist Verifikasi

- [x] Tidak ada fungsi duplikat yang seharusnya hanya di satu sisi
- [x] Separation of concerns jelas
- [x] Backend handles security-critical operations
- [x] Frontend handles UX-critical operations
- [x] Validasi di kedua sisi untuk defense in depth
- [x] Struktur data konsisten
- [x] Cookie name konsisten
- [x] Encoding konsisten
- [x] Dokumentasi lengkap

## Status

✅ **VERIFIED** - Implementasi backend dan frontend sudah sinergi dengan baik, tidak ada duplikasi berbahaya, dan separation of concerns sudah jelas.

