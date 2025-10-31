# Design Document: Microfrontend OAuth2 Integration

## Overview

Design document ini menjelaskan arsitektur dan implementasi OAuth2 Authorization Code Flow untuk microfrontend integration dengan Portal sebagai Authorization Server.

## Architecture

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      User Browser                           │
│                                                             │
│  ┌──────────────┐         ┌──────────────┐                │
│  │ Microfrontend│         │    Portal    │                │
│  │  (Client)    │◄───────►│ (Auth Server)│                │
│  └──────┬───────┘         └──────┬───────┘                │
│         │                        │                         │
│         └────────────┬───────────┘                         │
│                      │                                     │
└──────────────────────┼─────────────────────────────────────┘
                       │
                       │ HTTPS + Cookies
                       │
┌──────────────────────┼─────────────────────────────────────┐
│                      ▼                                      │
│              ┌──────────────┐                              │
│              │   Authenc    │                              │
│              │   Backend    │                              │
│              └──────┬───────┘                              │
│                     │                                      │
│         ┌───────────┼───────────┐                         │
│         ▼           ▼           ▼                         │
│    ┌────────┐  ┌────────┐  ┌────────┐                   │
│    │  DB    │  │ Redis  │  │ Kafka  │                   │
│    └────────┘  └────────┘  └────────┘                   │
│                                                           │
└───────────────────────────────────────────────────────────┘
```

### Component Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                    Microfrontend                            │
│                                                             │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  App Component                                       │  │
│  │  - Check authentication                              │  │
│  │  - Show login button if not authenticated           │  │
│  └──────────────────────────────────────────────────────┘  │
│                          │                                  │
│                          ▼                                  │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  OAuth2 Login Component (NEW)                       │  │
│  │  - Generate state parameter                         │  │
│  │  - Build authorization URL                          │  │
│  │  - Redirect to Portal                               │  │
│  └──────────────────────────────────────────────────────┘  │
│                                                             │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  OAuth2 Callback Component (NEW)                    │  │
│  │  - Validate state parameter                         │  │
│  │  - Exchange code for tokens                         │  │
│  │  - Save session                                     │  │
│  │  - Redirect to app                                  │  │
│  └──────────────────────────────────────────────────────┘  │
│                          │                                  │
│                          ▼                                  │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  Shared Library (use_auth)                          │  │
│  │  - Auth context                                     │  │
│  │  - Session management                               │  │
│  │  - SSO cookie reader                                │  │
│  │  - Logout logic                                     │  │
│  └──────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

## Data Models

### OAuth2 State

```rust
#[derive(Serialize, Deserialize)]
pub struct OAuth2State {
    /// Random state for CSRF protection
    pub state: String,

    /// Original URL user tried to access
    pub return_url: Option<String>,

    /// Timestamp when state was created
    pub created_at: i64,

    /// Client ID (microfrontend name)
    pub client_id: String,
}
```

### OAuth2 Config

```rust
#[derive(Clone)]
pub struct OAuth2Config {
    /// Authenc base URL
    pub authenc_url: String,

    /// Portal base URL
    pub portal_url: String,

    /// Client ID (microfrontend name)
    pub client_id: String,

    /// Redirect URI (callback URL)
    pub redirect_uri: String,

    /// OAuth2 scopes
    pub scopes: Vec<String>,
}
```

### Authorization Code (Backend)

```rust
#[derive(Serialize, Deserialize)]
pub struct AuthorizationCode {
    /// The authorization code
    pub code: String,

    /// User ID who authorized
    pub user_id: Uuid,

    /// Client ID
    pub client_id: String,

    /// Redirect URI
    pub redirect_uri: String,

    /// Scopes granted
    pub scopes: Vec<String>,

    /// Code challenge for PKCE (optional)
    pub code_challenge: Option<String>,

    /// Expiration timestamp
    pub expires_at: DateTime<Utc>,

    /// Whether code has been used
    pub used: bool,
}
```

## Components

### 1. OAuth2LoginButton Component

**Location**: `antarmuka/shared/src/components/auth/oauth2_login_button.rs`

**Purpose**: Reusable button component untuk trigger OAuth2 login flow

**Props**:
```rust
#[component]
pub fn OAuth2LoginButton(
    /// OAuth2 configuration
    #[prop(into)]
    config: OAuth2Config,

    /// Optional custom button text
    #[prop(optional)]
    button_text: Option<String>,

    /// Optional custom CSS class
    #[prop(optional)]
    class: Option<String>,
) -> impl IntoView
```

**Behavior**:
1. Generate random state (32 characters, cryptographically secure)
2. Save state + return_url to sessionStorage
3. Build authorization URL with parameters
4. Redirect to Portal `/oidc/authorize`

**Implementation**:
```rust
pub fn OAuth2LoginButton(config: OAuth2Config, ...) -> impl IntoView {
    let handle_login = move || {
        // 1. Generate state
        let state = generate_random_state();

        // 2. Get current URL for return
        let return_url = window().location().href().ok();

        // 3. Save to sessionStorage
        let oauth2_state = OAuth2State {
            state: state.clone(),
            return_url,
            created_at: Utc::now().timestamp(),
            client_id: config.client_id.clone(),
        };
        save_to_session_storage("oauth2_state", &oauth2_state);

        // 4. Build authorization URL
        let auth_url = format!(
            "{}/oidc/authorize?response_type=code&client_id={}&redirect_uri={}&state={}&scope={}",
            config.portal_url,
            urlencoding::encode(&config.client_id),
            urlencoding::encode(&config.redirect_uri),
            urlencoding::encode(&state),
            urlencoding::encode(&config.scopes.join(" "))
        );

        // 5. Redirect
        window().location().set_href(&auth_url);
    };

    view! {
        <button on:click=move |_| handle_login() class=class>
            {button_text.unwrap_or("Masuk".to_string())}
        </button>
    }
}
```

### 2. OAuth2CallbackPage Component

**Location**: `antarmuka/shared/src/components/auth/oauth2_callback.rs`

**Purpose**: Handle OAuth2 callback dan exchange code untuk tokens

**Implementation**:
```rust
#[component]
pub fn OAuth2CallbackPage(
    /// OAuth2 configuration
    #[prop(into)]
    config: OAuth2Config,

    /// Callback after successful authentication
    #[prop(into)]
    on_success: Callback<UserSession>,

    /// Callback on error
    #[prop(optional)]
    on_error: Option<Callback<String>>,
) -> impl IntoView {
    let (status, set_status) = signal("processing".to_string());
    let (error, set_error) = signal(None::<String>);

    // Extract code and state from URL
    Effect::new(move |_| {
        spawn_local(async move {
            match process_callback(&config).await {
                Ok(session) => {
                    set_status.set("success".to_string());
                    on_success.call(session);
                }
                Err(e) => {
                    set_status.set("error".to_string());
                    set_error.set(Some(e.to_string()));
                    if let Some(on_err) = on_error {
                        on_err.call(e.to_string());
                    }
                }
            }
        });
    });

    view! {
        <div class="oauth2-callback">
            <Show when=move || status.get() == "processing">
                <LoadingSpinner message="Completing authentication..." />
            </Show>

            <Show when=move || status.get() == "error">
                <ErrorMessage message=error.get() />
            </Show>
        </div>
    }
}

async fn process_callback(config: &OAuth2Config) -> Result<UserSession, String> {
    // 1. Extract parameters from URL
    let params = extract_url_params()?;
    let code = params.get("code").ok_or("Missing code parameter")?;
    let state = params.get("state").ok_or("Missing state parameter")?;

    // 2. Load saved state from sessionStorage
    let saved_state: OAuth2State = load_from_session_storage("oauth2_state")
        .ok_or("No saved state found")?;

    // 3. Validate state
    if state != &saved_state.state {
        return Err("Invalid state parameter".to_string());
    }

    // 4. Check state expiration (10 minutes)
    let now = Utc::now().timestamp();
    if now - saved_state.created_at > 600 {
        return Err("State expired".to_string());
    }

    // 5. Exchange code for tokens
    let token_response = exchange_code_for_tokens(config, code).await?;

    // 6. Decode JWT to UserSession
    let session = decode_jwt_to_session(&token_response.access_token)?;

    // 7. Save to localStorage
    save_to_local_storage("user_session", &session);
    save_to_local_storage("auth_token", &token_response.access_token);
    save_to_local_storage("refresh_token", &token_response.refresh_token);

    // 8. Clear sessionStorage
    remove_from_session_storage("oauth2_state");

    Ok(session)
}

async fn exchange_code_for_tokens(
    config: &OAuth2Config,
    code: &str,
) -> Result<TokenResponse, String> {
    let token_url = format!("{}/oidc/token", config.authenc_url);

    let form_data = format!(
        "grant_type=authorization_code&code={}&client_id={}&redirect_uri={}",
        urlencoding::encode(code),
        urlencoding::encode(&config.client_id),
        urlencoding::encode(&config.redirect_uri)
    );

    let response = Request::post(&token_url)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .credentials(RequestCredentials::Include) // Important for SSO cookie
        .body(form_data)
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if !response.ok() {
        return Err(format!("Token exchange failed: {}", response.status()));
    }

    response.json::<TokenResponse>().await
        .map_err(|e| format!("Failed to parse response: {}", e))
}
```

### 3. LoginRedirectPage Component (Enhanced)

**Location**: `antarmuka/shared/src/components/auth/login_redirect.rs`

**Purpose**: Show login button untuk unauthenticated users

**Enhancement**: Add OAuth2 support

```rust
#[component]
pub fn LoginRedirectPage(
    /// OAuth2 configuration (optional, if not provided uses default)
    #[prop(optional)]
    oauth2_config: Option<OAuth2Config>,

    /// Custom message
    #[prop(optional)]
    message: Option<String>,
) -> impl IntoView {
    let config = oauth2_config.unwrap_or_else(|| {
        OAuth2Config {
            authenc_url: env::var("AUTHENC_URL")
                .unwrap_or("http://localhost:8080".to_string()),
            portal_url: env::var("PORTAL_URL")
                .unwrap_or("http://localhost:3000".to_string()),
            client_id: env::var("APP_NAME")
                .unwrap_or("microfrontend".to_string()),
            redirect_uri: format!(
                "{}/callback",
                window().location().origin().unwrap_or_default()
            ),
            scopes: vec!["openid".to_string(), "profile".to_string(), "email".to_string()],
        }
    });

    view! {
        <div class="login-redirect-page">
            <div class="login-card">
                <i class="fas fa-lock text-6xl text-blue-500 mb-4"></i>
                <h2 class="text-2xl font-bold mb-2">
                    "Authentication Required"
                </h2>
                <p class="text-gray-600 mb-6">
                    {message.unwrap_or("Please login to access this application.".to_string())}
                </p>

                <OAuth2LoginButton
                    config=config
                    button_text="Masuk dengan Portal"
                    class="btn-primary"
                />
            </div>
        </div>
    }
}
```

### 4. Backend: Authorization Code Storage

**Location**: `infra/authenc/src/services/oauth2_code_store.rs`

**Purpose**: Store dan validate authorization codes

**Implementation**:
```rust
pub struct OAuth2CodeStore {
    redis: Option<Arc<RedisCache>>,
    db: Arc<Database>,
}

impl OAuth2CodeStore {
    pub async fn sto&self, code: AuthorizationCode) -> Result<()> {
        // Store in Redis for fast access (10 minutes TTL)
        if let Some(redis) = &self.redis {
            let key = format!("oauth2:code:{}", code.code);
            redis.set(&key, &code, Duration::from_secs(600)).await?;
        }

        // Also store in database for persistence
        self.db.execute(
            "INSERT INTO authorization_codes (code, user_id, client_id, redirect_uri, scopes, expires_at, used)
             VALUES ($1, $2, $3, $4, $5, $6, $7)",
            &[&code.code, &code.user_id, &code.client_id, &code.redirect_uri,
              &code.scopes, &code.expires_at, &code.used]
        ).await?;

        Ok(())
    }

    pub async fn get_and_consume_code(&self, code: &str) -> Result<Option<AuthorizationCode>> {
        // Try Redis first
        if let Some(redis) = &self.redis {
            let key = format!("oauth2:code:{}", code);
            if let Some(auth_code) = redis.get::<AuthorizationCode>(&key).await? {
                // Mark as used
                redis.delete(&key).await?;

                // Also mark in database
                self.mark_code_as_used(code).await?;

                return Ok(Some(auth_code));
            }
        }

        // Fallback to database
        let row = self.db.query_opt(
            "SELECT * FROM authorization_codes WHERE code = $1 AND used = false AND expires_at > NOW()",
            &[&code]
        ).await?;

        if let Some(row) = row {
            let auth_code = AuthorizationCode::from_row(&row)?;

            // Mark as used
            self.mark_code_as_used(code).await?;

            Ok(Some(auth_code))
        } else {
            Ok(None)
        }
    }

    async fn mark_code_as_used(&self, code: &str) -> Result<()> {
        self.db.execute(
            "UPDATE authorization_codes SET used = true WHERE code = $1",
            &[&code]
        ).await?;
        Ok(())
    }
}
```

### 5. Backend: OAuth2 Authorize Endpoint

**Location**: `infra/authenc/src/handlers/oauth2_comprehensive.rs`

**Enhancement**: Add authorization code generation

```rust
pub async fn oauth2_authorize(
    State(state): State<Arc<OAuth2AppState>>,
    Query(params): Query<OAuth2AuthorizeParams>,
    headers: HeaderMap,
) -> Result<Response, AuthencError> {
    // 1. Validate client_id
    let client = state.oauth2_stores.client_store
        .get_client(&params.client_id).await?
        .ok_or(AuthencError::validation("Invalid client_id"))?;

    // 2. Validate redirect_uri
    if !client.redirect_uris.contains(&params.redirect_uri) {
        return Err(AuthencError::validation("Invalid redirect_uri"));
    }

    // 3. Check if user is authenticated
    let session = extract_session_from_cookie(&headers)?;

    if session.is_none() {
        // Redirect to login with return_url
        let return_url = format!(
            "/oidc/authorize?{}",
            serde_urlencoded::to_string(&params)?
        );
        let login_url = format!("/login?return_url={}", urlencoding::encode(&return_url));
        return Ok(Redirect::to(&login_url).into_response());
    }

    let session = session.unwrap();

    // 4. Generate authorization code
    let code = generate_authorization_code();

    let auth_code = AuthorizationCode {
        code: code.clone(),
        user_id: session.user_id,
        client_id: params.client_id.clone(),
        redirect_uri: params.redirect_uri.clone(),
        scopes: params.scope.split_whitespace().map(String::from).collect(),
        code_challenge: params.code_challenge,
        expires_at: Utc::now() + Duration::minutes(10),
        used: false,
    };

    // 5. Store authorization code
    state.oauth2_stores.code_store.store_code(auth_code).await?;

    // 6. Redirect back to client with code
    let redirect_url = format!(
        "{}?code={}&state={}",
        params.redirect_uri,
        urlencoding::encode(&code),
        urlencoding::encode(&params.state.unwrap_or_default())
    );

    Ok(Redirect::to(&redirect_url).into_response())
}

fn generate_authorization_code() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let bytes: Vec<u8> = (0..32).map(|_| rng.gen()).collect();
    base64::encode_config(&bytes, base64::URL_SAFE_NO_PAD)
}
```

## Error Handling

### Error Types

```rust
#[derive(Debug)]
pub enum OAuth2Error {
    /// Invalid state parameter (CSRF attack)
    InvalidState,

    /// State expired (> 10 minutes)
    StateExpired,

    /// Missing required parameter
    MissingParameter(String),

    /// Authorization code invalid or expired
    InvalidCode,

    /// Token exchange failed
    TokenExchangeFailed(String),

    /// Network error
    NetworkError(String),
}

impl OAuth2Error {
    pub fn user_message(&self) -> String {
        match self {
            Self::InvalidState => "Invalid authentication state. Please try again.".to_string(),
            Self::StateExpired => "Authentication session expired. Please try again.".to_string(),
            Self::MissingParameter(p) => format!("Missing required parameter: {}", p),
            Self::InvalidCode => "Invalid authorization code. Please try logging in again.".to_string(),
            Self::TokenExchangeFailed(e) => format!("Authentication failed: {}", e),
            Self::NetworkError(e) => format!("Network error: {}. Please check your connection.", e),
        }
    }
}
```

### Error Page Component

```rust
#[component]
pub fn OAuth2ErrorPage(
    /// Error to display
    #[prop(into)]
    error: OAuth2Error,
) -> impl IntoView {
    view! {
        <div class="error-page">
            <i class="fas fa-exclamation-triangle text-6xl text-red-500 mb-4"></i>
            <h2 class="text-2xl font-bold mb-2">"Authentication Error"</h2>
            <p class="text-gray-600 mb-6">{error.user_message()}</p>

            <button
                on:click=move |_| {
                    window().location().set_href("/").ok();
                }
                class="btn-primary"
            >
                "Try Again"
            </button>
        </div>
    }
}
```

## Testing Strategy

### Unit Tests

1. **OAuth2 State Generation**
   - Test random state generation (length, uniqueness)
   - Test state serialization/deserialization
   - Test state expiration logic

2. **Authorization URL Building**
   - Test URL construction with all parameters
   - Test URL encoding
   - Test with optional parameters

3. **Callback Processing**
   - Test state validation (valid, invalid, expired)
   - Test code extraction from URL
   - Test token exchange
   - Test session creation

### Integration Tests

1. **Complete OAuth2 Flow**
   - Start from microfrontend
   - Redirect to Portal
   - Complete login + CAPTCHA + MFA
   - Callback to microfrontend
   - Verify session created

2. **Error Scenarios**
   - Invalid state parameter
   - Expired authorization code
   - Network failure during token exchange
   - Invalid redirect_uri

3. **Cross-Tab Logout**
   - Login in tab 1
   - Logout in tab 2
   - Verify tab 1 detects logout

### Performance Tests

1. **OAuth2 Redirect Speed**
   - Measure time from click to redirect
   - Target: < 500ms

2. **Callback Processing Speed**
   - Measure time from callback to authenticated
   - Target: < 1 second

3. **Concurrent OAuth2 Flows**
   - Test 1000+ concurrent flows
   - Verify no race conditions

## Security Considerations

### CSRF Protection

- State parameter is cryptographically random
- State stored in sessionStorage (not accessible cross-origin)
- State validated on callback
- State expires after 10 minutes

### XSS Protection

- SSO cookie has HttpOnly flag
- Tokens stored in localStorage (not in cookies)
- All user input sanitized
- CSP headers configured

### Authorization Code Security

- Code expires in 10 minutes
- Code can only be used once
- Code tied to specific client_id and redirect_uri
- Code stored securely (Redis + Database)

### Redirect URI Validation

- Redirect URI must be pre-registered
- Exact match validation (no wildcards)
- HTTPS required in production

## Deployment Strategy

### Phase 1: Shared Library (Week 1)
- Implement OAuth2LoginButton component
- Implement OAuth2CallbackPage component
- Add to shared library
- Unit tests

### Phase 2: Backend (Week 1-2)
- Implement authorization code storage
- Enhance /oidc/authorize endpoint
- Enhance /oidc/token endpoint
- Integration tests

### Phase 3: Portal (Week 2)
- Test OAuth2 authorization flow
- Verify CAPTCHA + MFA integration
- End-to-end tests

### Phase 4: Pilot Microfrontend (Week 3)
- Implement in Badiklat (pilot)
- Add /callback route
- Test complete flow
- Fix any issues

### Phase 5: Rollout (Week 4-6)
- Deploy to remaining microfrontends
- Monitor metrics
- Gather feedback
- Optimize

## Monitoring and Metrics

### Key Metrics

1. **OAuth2 Flow Success Rate**
   - Target: > 99%
   - Alert if < 95%

2. **Average Flow Duration**
   - Target: < 5 seconds
   - Alert if > 10 seconds

3. **Authorization Code Usage**
   - Track generated vs used codes
   - Alert on high unused rate

4. **Error Rate by Type**
   - Track InvalidState, InvalidCode, etc.
   - Alert on spikes

### Logging

```rust
// Log OAuth2 events
tracing::info!(
    user_id = %user_id,
    client_id = %client_id,
    "OAuth2 authorization code generated"
);

tracing::info!(
    user_id = %user_id,
    client_id = %client_id,
    duration_ms = %duration,
    "OAuth2 flow completed successfully"
);

tracing::warn!(
    client_id = %client_id,
    error = %error,
    "OAuth2 flow failed"
);
```

## Conclusion

Design ini menyediakan OAuth2 Authorization Code Flow yang secure, performant, dan user-friendly untuk microfrontend integration. Dengan centralized authentication di Portal, consistent session management via SSO cookie, dan proper error handling, user experience akan seamless across semua microfrontend.

