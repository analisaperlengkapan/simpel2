# Requirements Document: Microfrontend OAuth2 Integration

## Introduction

Spec ini mendefinisikan implementasi OAuth2 Authorization Code Flow untuk semua microfrontend (Badiklat, Datun, Intel, Pembinaan, Pemulihan Aset, Pengawasan, Pidmil, Pidsus, Pidum) agar dapat melakukan authentication melalui Portal sebagai OAuth2 Authorization Server.

Saat ini, microfrontend hanya menggunakan `use_auth()` hook dari shared library untuk check authentication status, tetapi tidak memiliki mekanisme untuk redirect ke Portal untuk login dan handle OAuth2 callback.

## Glossary

- **Microfrontend**: Aplikasi frontend modular (Badiklat, Datun, Intel, dll) yang berjalan di subdomain terpisah
- **Portal**: Aplikasi gateway yang berfungsi sebagai OAuth2 Authorization Server dan login UI
- **Authenc**: Backend service untuk authentication dan authorization
- **OAuth2 Authorization Code Flow**: Standard OAuth2 flow untuk web applications
- **SSO Cookie**: Cookie dengan domain `.simpel.kejaksaan.go.id` yang shared across subdomains
- **Callback Handler**: Component yang menerima authorization code dari Portal dan exchange untuk tokens
- **CAPTCHA**: Challenge-response test untuk memastikan user adalah manusia
- **MFA**: Multi-Factor Authentication menggunakan TOTP (Time-based One-Time Password)
- **Secreton**: Secret management service untuk menyimpan MFA secrets

## Requirements

### Requirement 1: OAuth2 Login Redirect

**User Story:** Sebagai user yang mengakses microfrontend tanpa authentication, saya ingin diarahkan ke Portal untuk login, sehingga saya dapat authenticate dengan CAPTCHA dan MFA.

#### Acceptance Criteria

1. WHEN user mengakses microfrontend tanpa session, THE Microfrontend SHALL menampilkan halaman dengan tombol "Masuk"
2. WHEN user click tombol login, THE Microfrontend SHALL redirect ke Portal OAuth2 authorization endpoint dengan parameter client_id, redirect_uri, state, dan scope
3. THE Microfrontend SHALL generate random state parameter untuk CSRF protection
4. THE Microfrontend SHALL save state parameter ke sessionStorage untuk validation nanti
5. THE redirect_uri SHALL menggunakan format `https://{microfrontend}.simpel.kejaksaan.go.id/callback`

### Requirement 2: OAuth2 Callback Handler

**User Story:** Sebagai microfrontend, saya ingin menerima authorization code dari Portal dan exchange untuk access token, sehingga user dapat authenticated.

#### Acceptance Criteria

1. THE Microfrontend SHALL memiliki route `/callback` untuk menerima OAuth2 callback
2. WHEN callback dipanggil dengan parameter code dan state, THE Microfrontend SHALL validate state parameter matches dengan yang disimpan di sessionStorage
3. IF state tidak match, THE Microfrontend SHALL reject request dan redirect ke error page
4. WHEN state valid, THE Microfrontend SHALL call Authenc `/oidc/token` endpoint dengan authorization code untuk exchange tokens
5. THE Microfrontend SHALL menerima access_token, refresh_token, id_token, dan SSO cookie dari Authenc
6. THE Microfrontend SHALL save tokens ke localStorage dan session ke auth context
7. WHEN token exchange sukses, THE Microfrontend SHALL redirect ke halaman yang originally diminta atau home page

### Requirement 3: Portal Login Flow dengan CAPTCHA dan MFA

**User Story:** Sebagai user yang di-redirect ke Portal, saya ingin menyelesaikan CAPTCHA dan MFA verification, sehingga saya dapat login dengan aman.

#### Acceptance Criteria

1. WHEN user di-redirect ke Portal `/oidc/authorize`, THE Portal SHALL check apakah user sudah authenticated
2. IF user belum authenticated, THE Portal SHALL redirect ke `/login` dengan return_url parameter
3. THE Portal SHALL menampilkan login form dengan CAPTCHA component
4. THE user SHALL menyelesaikan CAPTCHA sebelum dapat submit credentials
5. WHEN user submit credentials dengan valid CAPTCHA token, THE Portal SHALL call Authenc `/oidc/token` endpoint
6. IF user belum setup MFA, THE Authenc SHALL return temp_token dengan flag mfa_setup_required=true
7. THE Portal SHALL redirect ke `/mfa/setup` untuk setup TOTP authenticator
8. IF user sudah setup MFA, THE Authenc SHALL return temp_token dengan flag mfa_required=true
9. THE Portal SHALL redirect ke `/mfa/verify` untuk verify OTP code
10. WHEN MFA verification sukses, THE Portal SHALL redirect kembali ke `/oidc/authorize` dengan authenticated session

### Requirement 4: OAuth2 Authorization Grant

**User Story:** Sebagai Portal yang sudah authenticate user, saya ingin generate authorization code dan redirect kembali ke microfrontend, sehingga microfrontend dapat complete OAuth2 flow.

#### Acceptance Criteria

1. WHEN Portal `/oidc/authorize` dipanggil dengan authenticated session, THE Portal SHALL validate client_id dan redirect_uri
2. THE Portal SHALL generate authorization code yang valid untuk 10 menit
3. THE Portal SHALL save authorization code dengan associated user_id, client_id, dan redirect_uri ke database atau cache
4. THE Portal SHALL redirect ke microfrontend callback URL dengan parameter code dan state
5. THE authorization code SHALL dapat digunakan hanya sekali (one-time use)

### Requirement 5: SSO Cookie Sharing

**User Story:** Sebagai system, saya ingin SSO cookie dapat diakses oleh semua subdomain, sehingga user tidak perlu login ulang ketika pindah antar microfrontend.

#### Acceptance Criteria

1. THE Authenc SHALL set SSO cookie dengan domain `.simpel.kejaksaan.go.id`
2. THE SSO cookie SHALL memiliki flags HttpOnly=true, Secure=true, SameSite=Lax
3. THE SSO cookie SHALL berisi session data dalam format Base64-encoded JSON
4. THE SSO cookie SHALL dapat dibaca oleh Portal dan semua microfrontend
5. WHEN microfrontend detect SSO cookie, THE Microfrontend SHALL extract session data dan set ke auth context tanpa perlu OAuth2 flow

### Requirement 6: Cross-Tab Logout Synchronization

**User Story:** Sebagai user yang logout dari satu app, saya ingin semua tab dan app lain juga logout, sehingga session saya terminated secara konsisten.

#### Acceptance Criteria

1. WHEN user logout dari Portal atau microfrontend, THE App SHALL call Authenc `/oidc/logout` endpoint
2. THE Authenc SHALL invalidate semua user sessions di database
3. THE Authenc SHALL delete SSO cookie dengan Max-Age=0
4. THE Authenc SHALL publish logout event ke Kafka untuk audit trail
5. THE App SHALL broadcast logout_event ke localStorage untuk trigger storage event
6. WHEN storage event detected di tab lain, THE App SHALL clear session dan redirect ke login
7. THE logout SHALL propagate ke semua open tabs dan semua microfrontend

### Requirement 7: Token Refresh

**User Story:** Sebagai microfrontend, saya ingin automatically refresh access token ketika expired, sehingga user tidak perlu login ulang.

#### Acceptance Criteria

1. WHEN access token expired (1 hour), THE Microfrontend SHALL detect expiration
2. THE Microfrontend SHALL call Authenc `/oidc/refresh` endpoint dengan refresh_token
3. IF refresh token valid, THE Authenc SHALL issue new access_token dan rotate refresh_token
4. THE Microfrontend SHALL update tokens di localStorage dan continue operation
5. IF refresh token expired atau invalid, THE Microfrontend SHALL clear session dan redirect ke login
6. THE token refresh SHALL happen transparently tanpa interrupt user

### Requirement 8: Error Handling

**User Story:** Sebagai user, saya ingin melihat error message yang jelas ketika authentication gagal, sehingga saya tahu apa yang harus dilakukan.

#### Acceptance Criteria

1. IF OAuth2 state validation fails, THE Microfrontend SHALL show error "Invalid authentication state. Please try again."
2. IF authorization code exchange fails, THE Microfrontend SHALL show error "Authentication failed. Please try logging in again."
3. IF CAPTCHA validation fails, THE Portal SHALL show error "Security verification failed. Please try again."
4. IF MFA verification fails, THE Portal SHALL show error "Invalid verification code. Please try again."
5. IF network error occurs, THE App SHALL show error "Network error. Please check your connection."
6. ALL error messages SHALL be in Indonesian language
7. THE App SHALL provide "Try Again" button untuk retry authentication

### Requirement 9: Loading States

**User Story:** Sebagai user, saya ingin melihat loading indicator selama authentication process, sehingga saya tahu system sedang processing.

#### Acceptance Criteria

1. WHEN OAuth2 redirect happening, THE Microfrontend SHALL show loading spinner dengan message "Redirecting to login..."
2. WHEN callback processing, THE Microfrontend SHALL show loading spinner dengan message "Completing authentication..."
3. WHEN token exchange in progress, THE Microfrontend SHALL show loading spinner dengan message "Verifying credentials..."
4. WHEN MFA verification in progress, THE Portal SHALL disable submit button dan show loading state
5. THE loading states SHALL be accessible (ARIA labels)

### Requirement 10: Session Persistence

**User Story:** Sebagai user, saya ingin session saya persist setelah page refresh, sehingga saya tidak perlu login ulang.

#### Acceptance Criteria

1. THE Microfrontend SHALL save UserSession ke localStorage setelah successful authentication
2. WHEN page refresh, THE Microfrontend SHALL load session dari localStorage
3. IF localStorage empty, THE Microfrontend SHALL check SSO cookie
4. IF SSO cookie exists dan valid, THE Microfrontend SHALL extract session dan save ke localStorage
5. IF both localStorage dan SSO cookie empty atau invalid, THE Microfrontend SHALL show login button

### Requirement 11: Security Compliance

**User Story:** Sebagai system, saya ingin authentication flow comply dengan security best practices, sehingga user data terlindungi.

#### Acceptance Criteria

1. THE OAuth2 flow SHALL use Authorization Code Flow (NOT Implicit Flow)
2. THE state parameter SHALL be cryptographically random (minimum 32 characters)
3. THE authorization code SHALL expire dalam 10 menit
4. THE authorization code SHALL dapat digunakan hanya sekali
5. THE redirect_uri SHALL divalidasi against whitelist di backend
6. THE SSO cookie SHALL memiliki HttpOnly flag untuk prevent XSS
7. THE SSO cookie SHALL memiliki Secure flag untuk HTTPS only (production)
8. THE SSO cookie SHALL memiliki SameSite=Lax untuk CSRF protection
9. THE tokens SHALL disimpan di localStorage (NOT in cookies untuk avoid CSRF)
10. THE refresh token SHALL rotated setiap kali digunakan

### Requirement 12: Monitoring dan Audit

**User Story:** Sebagai system administrator, saya ingin monitor authentication events, sehingga saya dapat detect security issues.

#### Acceptance Criteria

1. THE Authenc SHALL log semua login attempts dengan user_id, IP address, user agent, dan timestamp
2. THE Authenc SHALL log semua MFA verification attempts dengan success/failure status
3. THE Authenc SHALL log semua logout events dengan user_id dan timestamp
4. THE Authenc SHALL publish authentication events ke Kafka untuk real-time monitoring
5. THE Authenc SHALL track failed login attempts dan implement rate limiting setelah 5 failed attempts
6. THE monitoring dashboard SHALL show login success rate, MFA verification rate, dan active sessions

### Requirement 13: Backward Compatibility

**User Story:** Sebagai developer, saya ingin implementasi OAuth2 tidak break existing functionality, sehingga deployment aman.

#### Acceptance Criteria

1. THE existing `use_auth()` hook SHALL continue to work tanpa perubahan
2. THE existing SSO cookie reading SHALL continue to work
3. THE existing logout functionality SHALL continue to work
4. THE new OAuth2 flow SHALL be additive (tidak replace existing code)
5. THE Portal SHALL support both direct login dan OAuth2 authorization flow
6. THE deployment SHALL dapat dilakukan secara incremental (per microfrontend)

### Requirement 14: Performance

**User Story:** Sebagai user, saya ingin authentication process cepat, sehingga saya tidak menunggu lama.

#### Acceptance Criteria

1. THE OAuth2 redirect SHALL complete dalam < 500ms
2. THE callback processing SHALL complete dalam < 1 second
3. THE token exchange SHALL complete dalam < 500ms
4. THE total authentication time (dari click login sampai authenticated) SHALL < 5 seconds (excluding user input time)
5. THE SSO cookie reading SHALL complete dalam < 50ms
6. THE localStorage access SHALL complete dalam < 10ms

### Requirement 15: Accessibility

**User Story:** Sebagai user dengan disabilities, saya ingin dapat menggunakan authentication flow dengan assistive technologies, sehingga saya dapat access system.

#### Acceptance Criteria

1. THE login button SHALL memiliki proper ARIA labels
2. THE loading states SHALL announce ke screen readers
3. THE error messages SHALL be accessible via screen readers
4. THE CAPTCHA component SHALL memiliki audio alternative
5. THE MFA setup SHALL memiliki text alternative untuk QR code
6. THE keyboard navigation SHALL work untuk semua interactive elements
7. THE focus management SHALL proper selama OAuth2 redirect dan callback

## Non-Functional Requirements

### Scalability
- System harus support 1000+ concurrent OAuth2 flows
- Authorization code storage harus efficient (Redis recommended)

### Reliability
- OAuth2 flow harus memiliki success rate > 99%
- Fallback mechanism jika Authenc temporarily unavailable

### Maintainability
- Code harus well-documented
- OAuth2 logic harus centralized di shared library
- Configuration harus externalized (environment variables)

### Security
- Comply dengan OAuth2 RFC 6749
- Implement PKCE (RFC 7636) untuk enhanced security (optional)
- Regular security audit untuk authentication flow

## Dependencies

- Authenc backend dengan OAuth2 endpoints
- Portal dengan authorization UI
- Shared library dengan auth hooks
- Secreton untuk MFA secret storage
- Redis untuk authorization code storage (recommended)
- Kafka untuk event publishing

## Assumptions

- Semua microfrontend berjalan di subdomain `*.simpel.kejaksaan.go.id`
- HTTPS enabled di production
- Browser support modern JavaScript (ES6+)
- Cookies enabled di browser
- localStorage available dan tidak full

## Constraints

- OAuth2 flow harus comply dengan RFC 6749
- CAPTCHA harus always required untuk security
- MFA harus enforced untuk semua users
- SSO cookie domain harus `.simpel.kejaksaan.go.id`
- Authorization code harus expire dalam 10 menit
- Access token harus expire dalam 1 hour
- Refresh token harus expire dalam 30 days

## Success Criteria

1. User dapat login dari microfrontend dengan redirect ke Portal
2. CAPTCHA dan MFA verification berfungsi dengan baik
3. OAuth2 callback berhasil exchange code untuk tokens
4. SSO cookie shared across semua microfrontend
5. Logout propagate ke semua tabs dan apps
6. Token refresh automatic dan transparent
7. Error handling comprehensive dan user-friendly
8. Performance memenuhi target (< 5 seconds total)
9. Security audit passed
10. All acceptance criteria met

