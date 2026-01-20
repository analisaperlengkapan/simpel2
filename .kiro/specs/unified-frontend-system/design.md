# Design Document

## Overview

Desain sistem antarmuka terpadu SIMPelv2 yang mengintegrasikan portal utama, shared component library, dan 11 microfrontend dengan arsitektur modern, authentication flow terpusat, dan user experience yang optimal.

### System Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         User Browser                             │
├─────────────────────────────────────────────────────────────────┤
│                                                                   │
│  ┌──────────────┐    ┌──────────────────────────────────────┐  │
│  │   Portal     │◄───┤  Microfrontends (11 apps)            │  │
│  │  (Gateway)   │    │  - Badiklat  - Pidum   - Pembinaan   │  │
│  │              │    │  - Datun     - Pidsus  - Keuangan    │  │
│  │  - Auth      │    │  - Intel     - Pidmil  - Perencanaan │  │
│  │  - Dashboard │    │  - Pengawasan         - Perlengkapan │  │
│  │  - Routing   │    │  - Pemulihan Aset                    │  │
│  └──────┬───────┘    └──────────────┬───────────────────────┘  │
│         │                            │                           │
│         └────────────┬───────────────┘                           │
│                      │                                           │
│         ┌────────────▼────────────────┐                         │
│         │  Shared Component Library   │                         │
│         │  - UI Components            │                         │
│         │  - Hooks & Utilities        │                         │
│         │  - Design System            │                         │
│         │  - Auth Client              │                         │
│         └────────────┬────────────────┘                         │
│                      │                                           │
└──────────────────────┼───────────────────────────────────────────┘
                       │
         ┌─────────────▼──────────────┐
         │   Backend Services         │
         │  - Authenc (IAM + MFA)     │
         │  - Secreton (Vault)        │
         │  - API Gateway             │
         │  - Business Services       │
         └────────────────────────────┘
```

### Key Design Principles

1. **Centralized Authentication**: Portal sebagai single point of authentication
2. **Microfrontend Independence**: Setiap app dapat dikembangkan dan di-deploy independen
3. **Shared Foundation**: Component library untuk konsistensi dan reusability
4. **Progressive Enhancement**: PWA capabilities dengan offline support
5. **Accessibility First**: WCAG 2.1 AA compliance di semua level
6. **Performance Optimized**: Code splitting, lazy loading, virtual scrolling

## Architecture

### 1. Portal Architecture (Authentication Gateway)

Portal bertindak sebagai authentication gateway dan dashboard utama.

#### Components


**Portal Structure:**
```
portal/
├── src/
│   ├── app.rs                 # Main app with routing
│   ├── features/
│   │   ├── auth/
│   │   │   ├── service.rs     # Auth service (login, logout, session)
│   │   │   ├── types.rs       # UserSession, LoginCredentials
│   │   │   └── oauth.rs       # OAuth2/OIDC client
│   │   ├── microfrontends/
│   │   │   ├── registry.rs    # MF registry & metadata
│   │   │   └── loader.rs      # Dynamic MF loading
│   │   └── dashboard/
│   │       ├── widgets.rs     # Dashboard widgets
│   │       └── analytics.rs   # Analytics integration
│   ├── pages/
│   │   ├── home.rs            # Landing page
│   │   ├── login.rs           # Login with CAPTCHA
│   │   ├── mfa_setup.rs       # MFA enrollment
│   │   ├── mfa_verify.rs      # MFA verification
│   │   ├── dashboard.rs       # Main dashboard
│   │   ├── apps.rs            # App selector
│   │   └── callback.rs        # OAuth callback handler
│   └── components/
│       ├── layout/            # Layout components
│       ├── navigation/        # Nav components
│       └── cards/             # Card components
└── styles/
    └── main.scss              # Portal-specific styles
```

#### Authentication Flow

**Complete Flow Diagram with MFA:**
```
┌─────────────────┐
│ Microfrontend   │
│ (Unauthenticated)│
└────────┬────────┘
         │ 1. Show "Login" button
         │
         ▼
┌────────────────────┐
│ Click Login        │
└────────┬───────────┘
         │ 2. Redirect to Portal
         │    with return_url param
         ▼
┌────────────────────────┐
│ Portal Login Page      │
│ - Username             │
│ - Password             │
│ - CAPTCHA (AI-resistant)│
└────────┬───────────────┘
         │ 3. Submit credentials
         │
         ▼
┌────────────────────────┐
│ Authenc API            │
│ /token endpoint        │
│ (via Envoy Gateway)    │
└────────┬───────────────┘
         │ 4. Return JWT + MFA status
         │
         ▼
    ┌────────────┐
    │ Check MFA  │
    │ Status     │
    └─────┬──────┘
          │
    ┌─────┴──────┐
    │            │
    ▼            ▼
┌─────────┐  ┌──────────┐
│MFA Not  │  │MFA Already│
│Enabled  │  │Enabled   │
└────┬────┘  └────┬─────┘
     │            │
     │ MANDATORY  │
     │ MFA SETUP  │
     │            │
     ▼            ▼
┌─────────────┐ ┌──────────────┐
│ MFA Setup   │ │ MFA Verify   │
│ Page        │ │ Page         │
│             │ │              │
│ 1. Generate │ │ 1. Enter OTP │
│    QR Code  │ │ 2. Validate  │
│ 2. Scan with│ │ 3. Max 3     │
│    Auth App │ │    attempts  │
│ 3. Verify   │ │              │
│    First OTP│ │              │
│ 4. Generate │ │              │
│    Backup   │ │              │
│    Codes    │ │              │
└─────┬───────┘ └──────┬───────┘
      │                │
      │ 5. MFA Complete│
      └────────┬───────┘
               │
               ▼
        ┌──────────────┐
        │ Store Session│
        │ - localStorage│
        │ - JWT token  │
        │ - User info  │
        │ - MFA status │
        └──────┬───────┘
               │
               ▼
        ┌──────────────┐
        │ Redirect     │
        │ Based on     │
        │ Origin       │
        └──────┬───────┘
               │
        ┌──────┴───────┐
        │              │
        ▼              ▼
┌──────────────┐  ┌─────────────┐
│ From MF?     │  │ From Portal?│
│ → return_url │  │ → Dashboard │
└──────────────┘  └─────────────┘
```

**MFA Enforcement Rules:**
1. **First Login**: User MUST setup MFA (mandatory)
2. **Subsequent Logins**: User MUST verify MFA (if enabled)
3. **No Bypass**: MFA cannot be skipped or disabled by user
4. **Backup Codes**: Generated during setup for emergency access
5. **Lockout**: 3 failed attempts = 15 minute lockout

#### Session Management

**Session Storage Strategy:**
```rust
// Session stored in localStorage for persistence
{
  "user_session": {
    "id": "uuid",
    "username": "user@kejaksaan.go.id",
    "name": "Ahmad Wijaya",
    "role": "Admin",
    "division": "Datun",
    "mfa_enabled": true,
    "access_token": "jwt_token",
    "refresh_token": "refresh_token",
    "expires_at": "2025-10-15T10:00:00Z"
  }
}
```

**Session Sharing Mechanism:**
- Portal stores session in localStorage
- Microfrontends read session from localStorage
- Event-based sync using `storage` event
- Automatic token refresh before expiry
- Logout broadcasts to all tabs/windows

### 2. Shared Component Library Architecture

Shared library menyediakan foundation untuk semua aplikasi.

#### Module Structure


```
shared/
├── src/
│   ├── core/
│   │   ├── types.rs           # Common types
│   │   ├── constants.rs       # Constants & config
│   │   └── theme.rs           # Theme system
│   ├── components/
│   │   ├── layout/
│   │   │   ├── card.rs
│   │   │   ├── container.rs
│   │   │   ├── grid.rs
│   │   │   └── stack.rs
│   │   ├── forms/
│   │   │   ├── input.rs
│   │   │   ├── button.rs
│   │   │   ├── select.rs
│   │   │   ├── textarea.rs
│   │   │   └── checkbox.rs
│   │   ├── feedback/
│   │   │   ├── toast.rs
│   │   │   ├── modal.rs
│   │   │   ├── alert.rs
│   │   │   ├── loading.rs
│   │   │   └── progress.rs
│   │   ├── navigation/
│   │   │   ├── header.rs
│   │   │   ├── breadcrumb.rs
│   │   │   ├── nav_menu.rs
│   │   │   ├── sidebar.rs
│   │   │   └── logo.rs
│   │   ├── display/
│   │   │   ├── table.rs
│   │   │   ├── badge.rs
│   │   │   ├── list.rs
│   │   │   ├── empty_state.rs
│   │   │   ├── avatar.rs
│   │   │   └── pagination.rs
│   │   ├── advanced/
│   │   │   ├── virtual_scroll.rs
│   │   │   ├── infinite_scroll.rs
│   │   │   ├── drag_drop.rs
│   │   │   ├── context_menu.rs
│   │   │   └── tooltip.rs
│   │   └── captcha/
│   │       └── captcha.rs     # AI-resistant CAPTCHA
│   ├── hooks/
│   │   ├── use_storage.rs     # localStorage/sessionStorage
│   │   ├── use_media_query.rs # Responsive breakpoints
│   │   ├── use_debounce.rs    # Debouncing
│   │   ├── use_auth.rs        # Auth state management
│   │   └── use_api.rs         # API client
│   ├── utils/
│   │   ├── validation.rs      # Input validation
│   │   ├── formatters.rs      # Data formatting
│   │   ├── helpers.rs         # Helper functions
│   │   └── api_client.rs      # HTTP client
│   └── lib.rs                 # Library entry & prelude
└── styles/
    ├── variables.css          # CSS variables
    ├── base.css               # Base styles
    ├── components.css         # Component styles
    ├── utilities.css          # Utility classes
    └── main.css               # Main stylesheet
```

#### Design System

**Color Palette:**
```css
:root {
  /* Primary - Kejaksaan Red */
  --color-primary: #DC2626;
  --color-primary-light: #EF4444;
  --color-primary-dark: #B91C1C;

  /* Secondary - Government Blue */
  --color-secondary: #1E40AF;
  --color-secondary-light: #3B82F6;
  --color-secondary-dark: #1E3A8A;

  /* Accent - Gold */
  --color-accent: #F59E0B;
  --color-accent-light: #FBBF24;
  --color-accent-dark: #D97706;

  /* Neutral */
  --color-gray-50: #F9FAFB;
  --color-gray-100: #F3F4F6;
  --color-gray-200: #E5E7EB;
  --color-gray-300: #D1D5DB;
  --color-gray-400: #9CA3AF;
  --color-gray-500: #6B7280;
  --color-gray-600: #4B5563;
  --color-gray-700: #374151;
  --color-gray-800: #1F2937;
  --color-gray-900: #111827;

  /* Semantic Colors */
  --color-success: #10B981;
  --color-warning: #F59E0B;
  --color-error: #EF4444;
  --color-info: #3B82F6;
}
```

**Typography:**
```css
:root {
  /* Font Families */
  --font-sans: 'Inter', system-ui, -apple-system, sans-serif;
  --font-serif: 'Merriweather', Georgia, serif;
  --font-mono: 'JetBrains Mono', 'Fira Code', monospace;

  /* Font Sizes */
  --text-xs: 0.75rem;    /* 12px */
  --text-sm: 0.875rem;   /* 14px */
  --text-base: 1rem;     /* 16px */
  --text-lg: 1.125rem;   /* 18px */
  --text-xl: 1.25rem;    /* 20px */
  --text-2xl: 1.5rem;    /* 24px */
  --text-3xl: 1.875rem;  /* 30px */
  --text-4xl: 2.25rem;   /* 36px */

  /* Font Weights */
  --font-normal: 400;
  --font-medium: 500;
  --font-semibold: 600;
  --font-bold: 700;

  /* Line Heights */
  --leading-tight: 1.25;
  --leading-normal: 1.5;
  --leading-relaxed: 1.75;
}
```

**Spacing System:**
```css
:root {
  --space-1: 0.25rem;   /* 4px */
  --space-2: 0.5rem;    /* 8px */
  --space-3: 0.75rem;   /* 12px */
  --space-4: 1rem;      /* 16px */
  --space-5: 1.25rem;   /* 20px */
  --space-6: 1.5rem;    /* 24px */
  --space-8: 2rem;      /* 32px */
  --space-10: 2.5rem;   /* 40px */
  --space-12: 3rem;     /* 48px */
  --space-16: 4rem;     /* 64px */
}
```

**Breakpoints:**
```css
/* Mobile First Approach */
--breakpoint-sm: 640px;   /* Small devices */
--breakpoint-md: 768px;   /* Medium devices */
--breakpoint-lg: 1024px;  /* Large devices */
--breakpoint-xl: 1280px;  /* Extra large */
--breakpoint-2xl: 1536px; /* 2X Extra large */
```

### 3. Microfrontend Architecture

Setiap microfrontend adalah aplikasi Leptos standalone yang terintegrasi dengan portal.

#### Standard Microfrontend Structure


```
microfrontend/
├── src/
│   ├── lib.rs                 # Entry point
│   ├── app.rs                 # Main app component
│   ├── pages/
│   │   ├── login_redirect.rs  # Login redirect page
│   │   ├── dashboard.rs       # MF dashboard
│   │   └── ...                # Domain-specific pages
│   ├── components/
│   │   ├── header.rs          # MF-specific header
│   │   ├── footer.rs          # MF-specific footer
│   │   └── ...                # Domain components
│   └── types.rs               # Domain types
├── styles/
│   └── main.css               # MF-specific styles
├── Cargo.toml
├── Trunk.toml
└── index.html
```

#### Authentication Integration

**Login Redirect Component:**
```rust
#[component]
pub fn LoginRedirectPage() -> impl IntoView {
    let navigate = use_navigate();

    // Check if user is authenticated
    let auth_state = use_auth();

    create_effect(move |_| {
        if auth_state.is_authenticated() {
            // Already authenticated, go to dashboard
            navigate("/dashboard", Default::default());
        }
    });

    let handle_login = move |_| {
        // Get current URL for return_url
        let current_url = window().location().href().unwrap();
        let portal_url = get_portal_url();
        let login_url = format!(
            "{}/login?return_url={}",
            portal_url,
            urlencoding::encode(&current_url)
        );

        // Redirect to portal login
        window().location().set_href(&login_url).unwrap();
    };

    view! {
        <AuthLayout>
            <div class="text-center">
                <Logo size="large" />
                <h1 class="text-3xl font-bold mt-6 mb-4">
                    "Selamat Datang di " {APP_NAME}
                </h1>
                <p class="text-gray-600 mb-8">
                    "Silakan login untuk mengakses aplikasi"
                </p>
                <Button
                    variant=ButtonVariant::Primary
                    size=ButtonSize::Large
                    on:click=handle_login
                >
                    <i class="fas fa-sign-in-alt mr-2"></i>
                    "Login ke Portal"
   </Button>
            </div>
        </AuthLayout>
    }
}
```

**Auth Hook:**
```rust
pub fn use_auth() -> AuthContext {
    // Read session from localStorage
    let session = use_storage("user_session", None);

    // Listen for storage events (cross-tab sync)
    use_effect(move |_| {
        let closure = Closure::wrap(Box::new(move |event: StorageEvent| {
            if event.key() == Some("user_session".to_string()) {
                // Session changed in another tab
                session.set(parse_session(event.new_value()));
            }
        }) as Box<dyn FnMut(_)>);

        window()
            .add_event_listener_with_callback(
                "storage",
                closure.as_ref().unchecked_ref()
            )
            .unwrap();

        closure.forget();
    });

    AuthContext {
        session,
        is_authenticated: move || session.get().is_some(),
        logout: move || {
            // Clear session
            session.set(None);
            // Redirect to portal logout
            let portal_url = get_portal_url();
            window()
                .location()
                .set_href(&format!("{}/logout", portal_url))
                .unwrap();
        },
    }
}
```

### 4. Component Design

#### Core Components

**Button Component:**
```rust
#[component]
pub fn Button(
    #[prop(optional)] variant: ButtonVariant,
    #[prop(optional)] size: ButtonSize,
    #[prop(optional)] disabled: bool,
    #[prop(optional)] loading: bool,
    #[prop(optional)] class: String,
    children: Children,
) -> impl IntoView {
    let variant_class = match variant {
        ButtonVariant::Primary => "bg-primary hover:bg-primary-dark text-white",
        ButtonVariant::Secondary => "bg-secondary hover:bg-secondary-dark text-white",
        ButtonVariant::Danger => "bg-error hover:bg-red-700 text-white",
        ButtonVariant::Success => "bg-success hover:bg-green-700 text-white",
        ButtonVariant::Ghost => "bg-transparent hover:bg-gray-100 text-gray-700",
    };

    let size_class = match size {
        ButtonSize::Small => "px-3 py-1.5 text-sm",
        ButtonSize::Medium => "px-4 py-2 text-base",
        ButtonSize::Large => "px-6 py-3 text-lg",
    };

    view! {
        <button
            class=format!(
                "inline-flex items-center justify-center font-medium rounded-lg \
                 transition-colors focus:outline-none focus:ring-2 focus:ring-offset-2 \
                 disabled:opacity-50 disabled:cursor-not-allowed {} {} {}",
                variant_class, size_class, class
            )
            disabled=disabled || loading
        >
            <Show when=move || loading>
                <Spinner class="mr-2" />
            </Show>
            {children()}
        </button>
    }
}
```

**Card Component:**
```rust
#[component]
pub fn Card(
    #[prop(optional)] title: Option<String>,
    #[prop(optional)] class: String,
    children: Children,
) -> impl IntoView {
    view! {
        <div class=format!(
            "bg-white dark:bg-gray-800 rounded-lg shadow-sm border \
             border-gray-200 dark:border-gray-700 {}",
            class
        )>
            {title.map(|t| view! {
                <div class="px-6 py-4 border-b border-gray-200 dark:border-gray-700">
                    <h3 class="text-lg font-semibold text-gray-900 dark:text-white">
                        {t}
                    </h3>
                </div>
            })}
            <div class="p-6">
                {children()}
            </div>
        </div>
    }
}
```

**Table Component:**
```rust
#[component]
pub fn Table<T>(
    columns: Vec<TableColumn<T>>,
    data: ReadSignal<Vec<T>>,
    #[prop(optional)] sortable: bool,
    #[prop(optional)] paginated: bool,
    #[prop(optional)] page_size: usize,
) -> impl IntoView
where
    T: Clone + 'static,
{
    let (sort_column, set_sort_column) = signal(None::<usize>);
    let (sort_direction, set_sort_direction) = signal(SortDirection::Asc);
    let (current_page, set_current_page) = signal(0);

    // Sorting logic
    let sorted_data = create_memo(move |_| {
        let mut items = data.get();
        if let Some(col_idx) = sort_column.get() {
            let column = &columns[col_idx];
            items.sort_by(|a, b| {
                let a_val = (column.accessor)(a);
                let b_val = (column.accessor)(b);
                match sort_direction.get() {
                    SortDirection::Asc => a_val.cmp(&b_val),
                    SortDirection::Desc => b_val.cmp(&a_val),
                }
            });
        }
        items
    });

    // Pagination logic
    let paginated_data = create_memo(move |_| {
        if !paginated {
            return sorted_data.get();
        }
        let start = current_page.get() * page_size;
        let end = (start + page_size).min(sorted_data.get().len());
        sorted_data.get()[start..end].to_vec()
    });

    view! {
        <div class="overflow-x-auto">
            <table class="min-w-full divide-y divide-gray-200">
                <thead class="bg-gray-50">
                    <tr>
                        {columns.iter().enumerate().map(|(idx, col)| {
                            view! {
                                <th
                                    class="px-6 py-3 text-left text-xs font-medium \
                                           text-gray-500 uppercase tracking-wider cursor-pointer"
                                    on:click=move |_| {
                                        if sortable && col.sortable {
                                            if sort_column.get() == Some(idx) {
                                                set_sort_direction.update(|d| d.toggle());
                                            } else {
                                                set_sort_column.set(Some(idx));
                                                set_sort_direction.set(SortDirection::Asc);
                                            }
                                        }
                                    }
                                >
                                    {&col.header}
                                    {(sortable && col.sortable).then(|| view! {
                                        <i class="fas fa-sort ml-2"></i>
                                    })}
                                </th>
                            }
                        }).collect_view()}
                    </tr>
                </thead>
                <tbody class="bg-white divide-y divide-gray-200">
                    {move || paginated_data.get().iter().map(|row| {
                        view! {
                            <tr class="hover:bg-gray-50">
                                {columns.iter().map(|col| {
                                    view! {
                                        <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                                            {(col.accessor)(row)}
                                        </td>
                                    }
                                }).collect_view()}
                            </tr>
                        }
                    }).collect_view()}
                </tbody>
            </table>

            {paginated.then(|| view! {
                <Pagination
                    current_page=current_page
                    total_items=move || sorted_data.get().len()
                    page_size=page_size
                    on_page_change=set_current_page
                />
            })}
        </div>
    }
}
```

## Data Models

### User Session Model


```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserSession {
    pub id: String,
    pub username: String,
    pub name: String,
    pub email: String,
    pub role: UserRole,
    pub division: String,
    pub avatar: Option<String>,
    pub mfa_enabled: bool,
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: DateTime<Utc>,
    pub permissions: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum UserRole {
    Admin,
    Supervisor,
    User,
    Guest,
}
```

### Microfrontend Registry Model

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MicrofrontendApp {
    pub id: String,
    pub name: String,
    pub description: String,
    pub icon: String,
    pub url: String,
    pub category: AppCategory,
    pub required_permissions: Vec<String>,
    pub status: AppStatus,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AppCategory {
    Prosecution,      // Pidum, Pidsus, Pidmil, Datun
    Intelligence,     // Intel
    Supervision,      // Pengawasan
    AssetRecovery,    // Pemulihan Aset
    Training,         // Badiklat
    Administration,   // Pembinaan (Keuangan, Perencanaan, Perlengkapan)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AppStatus {
    Active,
    Maintenance,
    Disabled,
}
```

## Correctness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a system-essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.*

### Authentication & Session Properties

**Property 1: Session Sharing Consistency**
*For any* authenticated user session stored in localStorage, all microfrontends accessing that session SHALL receive identical session data without requiring re-authentication.
**Validates: Requirements 3.7, 10.3**

**Property 2: Session Expiry Enforcement**
*For any* session with an `expires_at` timestamp in the past, the system SHALL redirect the user to the login page and clear the invalid session from localStorage.
**Validates: Requirements 3.8, 11.5**

**Property 3: MFA Enforcement**
*For any* user without MFA enabled, successful password authentication SHALL trigger the MFA setup flow before granting access to protected resources.
**Validates: Requirements 3.4, 4.2**

**Property 4: OAuth Callback Processing**
*For any* valid OAuth callback with authorization code and state parameter, the Portal SHALL exchange the code for tokens and redirect to the origin microfrontend with a valid session.
**Validates: Requirements 4.8**

**Property 5: Login Redirect from Microfrontend**
*For any* unauthenticated user accessing a microfrontend, clicking the login button SHALL redirect to Portal with the current URL as `return_url` parameter.
**Validates: Requirements 3.1, 3.2**

### UI Consistency Properties

**Property 6: Design Token Consistency**
*For any* component rendered across different microfrontends, the same CSS variables (colors, spacing, typography) SHALL be applied consistently.
**Validates: Requirements 1.1, 1.2**

**Property 7: Theme Switching Consistency**
*For any* theme change (light/dark), all rendered components SHALL update their styling within 100ms without page reload.
**Validates: Requirements 1.5, 18.1, 18.5**

**Property 8: Theme Preference Persistence**
*For any* user theme preference saved to localStorage, subsequent page loads SHALL restore the same theme preference.
**Validates: Requirements 18.4**

### Responsive Design Properties

**Property 9: Mobile Layout Adaptation**
*For any* viewport width less than 640px, the layout SHALL switch to mobile-optimized single-column layout with touch targets of at least 44x44px.
**Validates: Requirements 5.1, 5.5**

**Property 10: Tablet Layout Adaptation**
*For any* viewport width between 640px and 1024px, the layout SHALL adapt to medium-screen layout with appropriate spacing.
**Validates: Requirements 5.2**

**Property 11: Desktop Layout Optimization**
*For any* viewport width greater than 1024px, the layout SHALL utilize multi-column layouts and expanded navigation.
**Validates: Requirements 5.3**

**Property 12: Orientation Change Handling**
*For any* device orientation change, the layout SHALL adapt within 300ms without content loss or layout breakage.
**Validates: Requirements 5.4**

### Performance Properties

**Property 13: Virtual Scrolling for Large Lists**
*For any* list with more than 100 items, only the visible items plus a small buffer SHALL be rendered in the DOM.
**Validates: Requirements 7.2, 9.4**

**Property 14: SPA Navigation**
*For any* navigation between routes within the Portal or microfrontend, the transition SHALL occur without full page reload.
**Validates: Requirements 4.5, 10.1**

**Property 15: Lazy Loading Images**
*For any* image below the viewport fold, the image SHALL have `loading="lazy"` attribute and load only when approaching the viewport.
**Validates: Requirements 5.6, 9.5**

**Property 16: Bundle Size Constraint**
*For any* production build, the gzipped WASM bundle SHALL be less than 400KB.
**Validates: Requirements 9.3**

### Accessibility Properties

**Property 17: ARIA Labels on Interactive Elements**
*For any* interactive element (button, link, input), there SHALL be an accessible name via `aria-label`, `aria-labelledby`, or visible text content.
**Validates: Requirements 8.1**

**Property 18: Focus Indicator Visibility**
*For any* focusable element receiving keyboard focus, there SHALL be a visible focus indicator with at least 2px outline.
**Validates: Requirements 8.2**

**Property 19: Image Alt Text**
*For any* `<img>` element, there SHALL be an `alt` attribute with descriptive text or empty string for decorative images.
**Validates: Requirements 8.3**

**Property 20: Color Contrast Compliance**
*For any* text content, the color contrast ratio SHALL be at least 4.5:1 for normal text and 3:1 for large text (18px+ or 14px+ bold).
**Validates: Requirements 8.4**

**Property 21: Accessible Error Messages**
*For any* form field with validation error, the error message SHALL be associated via `aria-describedby` and announced to screen readers.
**Validates: Requirements 8.5**

**Property 22: Reduced Motion Support**
*For any* animation, when `prefers-reduced-motion: reduce` is set, the animation SHALL be disabled or significantly reduced.
**Validates: Requirements 7.7**

### Security Properties

**Property 23: HTTPS Enforcement**
*For any* API request from the frontend, the request URL SHALL use HTTPS protocol.
**Validates: Requirements 11.1**

**Property 24: Input Sanitization**
*For any* user input rendered in the DOM, the content SHALL be sanitized to prevent XSS attacks.
**Validates: Requirements 11.3**

**Property 25: CSRF Token Inclusion**
*For any* form submission or state-changing API request, a valid CSRF token SHALL be included in the request headers.
**Validates: Requirements 11.4**

**Property 26: Session Timeout**
*For any* session inactive for more than 30 minutes, the system SHALL automatically log out the user and clear session data.
**Validates: Requirements 11.5**

**Property 27: CSP Header Presence**
*For any* page response, Content-Security-Policy headers SHALL be present with strict directives.
**Validates: Requirements 11.7**

### Localization Properties

**Property 28: Indonesian Date Format**
*For any* date displayed to users, the format SHALL be DD/MM/YYYY (Indonesian standard).
**Validates: Requirements 17.2**

**Property 29: Indonesian Number Format**
*For any* number with thousands, the separator SHALL be a period (.) following Indonesian convention.
**Validates: Requirements 17.3**

**Property 30: Indonesian Currency Format**
*For any* currency amount in Rupiah, the format SHALL be "Rp X.XXX.XXX" with period as thousands separator.
**Validates: Requirements 17.4**

**Property 31: Translation Fallback**
*For any* missing translation key, the system SHALL display the default language text and log a warning to console.
**Validates: Requirements 17.6**

### Search & Filter Properties

**Property 32: Fuzzy Search Tolerance**
*For any* search query with minor typos (1-2 character differences), the search SHALL return relevant results.
**Validates: Requirements 19.1**

**Property 33: Search Result Highlighting**
*For any* search result, the matching terms SHALL be visually highlighted in the result text.
**Validates: Requirements 19.4**

**Property 34: Global Search Performance**
*For any* global search query, results SHALL be returned within 500ms.
**Validates: Requirements 4.3, 19.3**

### Error Isolation Properties

**Property 35: Microfrontend Error Isolation**
*For any* JavaScript error occurring in one microfrontend, other microfrontends and the Portal SHALL continue functioning normally.
**Validates: Requirements 10.5, 14.6**

**Property 36: Module Load Failure Fallback**
*For any* microfrontend that fails to load, the Portal SHALL display a fallback UI with retry option instead of crashing.
**Validates: Requirements 10.6**

### Real-time & Collaboration Properties

**Property 37: Real-time UI Updates**
*For any* data change received via WebSocket, the UI SHALL update within 1 second without manual refresh.
**Validates: Requirements 20.2**

**Property 38: Offline Queue Sync**
*For any* action queued while offline, the action SHALL be automatically synced when connection is restored within 5 seconds.
**Validates: Requirements 6.2, 20.6**

**Property 39: Presence Indicator Accuracy**
*For any* online user, the presence indicator SHALL show online status; for offline users, it SHALL show offline status.
**Validates: Requirements 20.1**

### Monitoring Properties

**Property 40: Error Logging with Context**
*For any* unhandled error, the error SHALL be logged with stack trace, user context, and timestamp to the monitoring service.
**Validates: Requirements 12.1**

**Property 41: Core Web Vitals Tracking**
*For any* page load, LCP, FID, and CLS metrics SHALL be measured and sent to the analytics service.
**Validates: Requirements 12.2**

**Property 42: API Response Time Monitoring**
*For any* API call, the response time SHALL be recorded and available for monitoring.
**Validates: Requirements 12.4**

## Error Handling

### Error Types

```rust
#[derive(Debug, Clone)]
pub enum AppError {
    // Authentication errors
    Unauthorized,
    SessionExpired,
    InvalidCredentials,
    MfaRequired,

    // Network errors
    NetworkError(String),
    ApiError { status: u16, message: String },

    // Validation errors
    ValidationError(Vec<ValidationError>),

    // Application errors
    NotFound,
    PermissionDenied,
    InternalError(String),
}

impl AppError {
    pub fn user_message(&self) -> String {
        match self {
            Self::Unauthorized => "Anda tidak memiliki akses. Silakan login.".to_string(),
            Self::SessionExpired => "Sesi Anda telah berakhir. Silakan login kembali.".to_string(),
            Self::InvalidCredentials => "Username atau password salah.".to_string(),
            Self::MfaRequired => "Verifikasi MFA diperlukan.".to_string(),
            Self::NetworkError(_) => "Terjadi kesalahan jaringan. Periksa koneksi Anda.".to_string(),
            Self::ApiError { message, .. } => message.clone(),
            Self::ValidationError(errors) => {
                errors.iter()
                    .map(|e| e.message.clone())
                    .collect::<Vec<_>>()
                    .join(", ")
            },
            Self::NotFound => "Data tidak ditemukan.".to_string(),
            Self::PermissionDenied => "Anda tidak memiliki izin untuk aksi ini.".to_string(),
            Self::InternalError(_) => "Terjadi kesalahan sistem. Silakan coba lagi.".to_string(),
        }
    }
}
```

### Error Display Component

```rust
#[component]
pub fn ErrorBoundary(
    fallback: impl Fn(AppError) -> View + 'static,
    children: Children,
) -> impl IntoView {
    let (error, set_error) = signal(None::<AppError>);

    provide_context(set_error);

    view! {
        <Show
            when=move || error.get().is_none()
            fallback=move || fallback(error.get().unwrap())
        >
            {children()}
        </Show>
    }
}
```

## Testing Strategy

### Unit Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_button_variants() {
        let button = Button {
            variant: ButtonVariant::Primary,
            size: ButtonSize::Medium,
            disabled: false,
            loading: false,
            class: String::new(),
            children: || view! { "Click me" },
        };

        // Assert button renders correctly
        assert!(button.to_string().contains("bg-primary"));
    }

    #[test]
    fn test_auth_session_validation() {
        let session = UserSession {
            id: "test-id".to_string(),
            username: "test@kejaksaan.go.id".to_string(),
            // ... other fields
            expires_at: Utc::now() + Duration::hours(1),
        };

        assert!(session.is_valid());
    }
}
```

### Component Testing

```rust
#[cfg(test)]
mod component_tests {
    use leptos::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn test_login_redirect() {
        mount_to_body(|| view! {
            <LoginRedirectPage />
        });

        // Assert login button is rendered
        let button = document()
            .query_selector("button")
            .unwrap()
            .unwrap();

        assert_eq!(button.text_content().unwrap(), "Login ke Portal");
    }
}
```

### Integration Testing

```rust
#[cfg(test)]
mod integration_tests {
    #[tokio::test]
    async fn test_auth_flow() {
        // 1. User clicks login button
        // 2. Redirects to portal
        // 3. Submits credentials
        // 4. Receives JWT token
        // 5. Stores session
        // 6. Redirects back to app

        let result = simulate_auth_flow().await;
        assert!(result.is_ok());
    }
}
```

### Property-Based Testing

Property-based testing validates that correctness properties hold across all valid inputs. We use `proptest` crate for Rust property-based testing.

**Testing Framework:** `proptest` (https://crates.io/crates/proptest)

**Configuration:** Each property test runs a minimum of 100 iterations.

**Property Test Format:**
```rust
use proptest::prelude::*;

// **Feature: unified-frontend-system, Property 28: Indonesian Date Format**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn test_indonesian_date_format(
        year in 2000i32..2100,
        month in 1u32..=12,
        day in 1u32..=28
    ) {
        let date = NaiveDate::from_ymd_opt(year, month, day).unwrap();
        let formatted = format_date(&date);

        // Property: Date format SHALL be DD/MM/YYYY
        let parts: Vec<&str> = formatted.split('/').collect();
        prop_assert_eq!(parts.len(), 3);
        prop_assert_eq!(parts[0].len(), 2); // DD
        prop_assert_eq!(parts[1].len(), 2); // MM
        prop_assert_eq!(parts[2].len(), 4); // YYYY
    }
}

// **Feature: unified-frontend-system, Property 29: Indonesian Number Format**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn test_indonesian_number_format(num in 1000i64..1_000_000_000) {
        let formatted = format_number(num);

        // Property: Thousands separator SHALL be period (.)
        if num >= 1000 {
            prop_assert!(formatted.contains('.'));
        }

        // Property: Parsing back should give same number
        let parsed: i64 = formatted.replace('.', "").parse().unwrap();
        prop_assert_eq!(parsed, num);
    }
}

// **Feature: unified-frontend-system, Property 30: Indonesian Currency Format**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn test_indonesian_currency_format(amount in 0i64..1_000_000_000_000) {
        let fo format_currency(amount);

        // Property: Currency SHALL start with "Rp"
        prop_assert!(formatted.starts_with("Rp"));

        // Property: Thousands separator SHALL be period
        if amount >= 1000 {
            prop_assert!(formatted.contains('.'));
        }
    }
}

// **Feature: unified-frontend-system, Property 1: Session Sharing Consistency**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn test_session_sharing_consistency(
        user_id in "[a-z0-9]{8}-[a-z0-9]{4}-[a-z0-9]{4}-[a-z0-9]{4}-[a-z0-9]{12}",
        username in "[a-z]+@kejaksaan\\.go\\.id",
        token in "[A-Za-z0-9]{64}"
    ) {
et session = UserSession {
            id: user_id.clone(),
            username: username.clone(),
            access_token: token.clone(),
            // ... other fields
        };

        // Store session
        store_session(&session);

        // Property: All reads SHALL return identical data
        let read1 = get_session();
        let read2 = get_session();

        prop_assert_eq!(read1.as_ref().map(|s| &s.id), Some(&user_id));
        prop_assert_eq!(read1.as_ref().map(|s| &s.id), read2.as_ref().map(|s| &s.id));
    }
}

// **Feature: unified-frontend-system, Property 2: Session Expiry Enforcement**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn test_session_expiry_enforcement(
        hours_ago in 1i64..1000
    ) {
        let expired_session = UserSession {
            expires_at: Utc::now() - Duration::hours(hours_ago),
            // ... other fields
        };

        // Property: Expired session SHALL be invalid
        propt!(!expired_session.is_valid());
    }
}

// **Feature: unified-frontend-system, Property 13: Virtual Scrolling for Large Lists**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn test_virtual_scroll_renders_only_visible(
        total_items in 101usize..10000,
        viewport_height in 300.0f64..1000.0,
        item_height in 30.0f64..100.0
    ) {
        let visible_count = (viewport_height / item_height).ceil() as usize;
        let buffer = 2; // Buffer items above/below viewport
        let max_rendered = visible_count + buffer * 2;

        // Property: Rendered items SHALL be less than total for large lists
        prop_assert!(max_rendered < total_items);
    }
}

// **Feature: unified-frontend-system, Property 32: Fuzzy Search Tolerance**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn test_fuzzy_search_tolerance(
        query in "[a-z]{4,10}",
        typo_pos in 0usize..4
    ) {
        // Create query with 1 typo
        let mut chars: Vec<char> = query.chars().collect();
        if typo_pos < chars.len() {
            chars[typo_pos] = 'x'; // Introduce typo
        }
        let typo_query: String = chars.into_iter().collect();

        let items = vec![query.clone()];
        let results = fuzzy_search(&typo_query, &items);

        // Property: Search with 1 typo SHALL still find the item
        prop_assert!(!results.is_empty());
    }
}
```

**Property Test Tagging Convention:**
Each property-based test MUST include a comment with the format:
`// **Feature: {feature_name}, Property {number}: {property_text}**`

This links the test to the corresponding correctness property in the design document.

## Performance Optimization
### Code Splitting Strategy

```rust
// Lazy load routes
#[component]
pub fn App() -> impl IntoView {
    view! {
        <Router>
            <Routes>
                // Eager load critical routes
                <Route path="/" view=HomePage />
                <Route path="/login" view=LoginPage />

                // Lazy load feature routes
                <Route path="/dashboard" view=|| {
                    lazy(|| import("./pages/dashboard.rs"))
                } />
                <Route path="/apps" view=|| {
                    lazy(|| import("./pages/apps.rs"))
                } />
            </Routes>
        </Router>
    }
}
```

### Virtual Scrolling

```rust
#[component]
pub fn VirtualScroll<T>(
    items: ReadSignal<Vec<T>>,
    item_height: f64,
    viewport_height: f64,
    render_item: impl Fn(T, usize) -> View + 'static,
) -> impl IntoView
where
    T: Clone + 'static,
{
    let (scroll_top, set_scroll_top) = signal(0.0);

    // Calculate visible range
    let visible_range = create_memo(move |_| {
        let start = (scroll_top.get() / item_height).floor() as usize;
        let visible_count = (viewport_height / item_height).ceil() as usize;
        let end = (start + visible_count + 1).min(items.get().len());
        start..end
    });

    view! {
        <div
            class="overflow-y-auto"
            style=format!("height: {}px", viewport_height)
            on:scroll=move |e| {
                let target = event_target::<web_sys::Element>(&e);
                set_scroll_top.set(target.scroll_top() as f64);
            }
        >
            <div style=format!("height: {}px; position: relative",
                items.get().len() as f64 * item_height)>
                {move || {
                    let range = visible_range.get();
                    let items_vec = items.get();

                    items_vec[range.clone()].iter().enumerate().map(|(idx, item)| {
                        let actual_idx = range.start + idx;
                        let top = actual_idx as f64 * item_height;

                        view! {
                            <div style=format!("position: absolute; top: {}px; width: 100%", top)>
                                {render_item(item.clone(), actual_idx)}
                            </div>
                        }
                    }).collect_view()
                }}
            </div>
        </div>
    }
}
```

### Image Optimization

```rust
#[component]
pub fn OptimizedImage(
    src: String,
    alt: String,
    #[prop(optional)] lazy: bool,
) -> impl IntoView {
    let (loaded, set_loaded) = signal(false);
    let (error, set_error) = signal(false);

    view! {
        <div class="relative">
            {(!loaded.get() && !error.get()).then(|| view! {
                <div class="absolute inset-0 bg-gray-200 animate-pulse"></div>
            })}

            <img
                src=src
                alt=alt
                loading=if lazy { "lazy" } else { "eager" }
                class=move || if loaded.get() { "opacity-100" } else { "opacity-0" }
                on:load=move |_| set_loaded.set(true)
                on:error=move |_| set_error.set(true)
            />

            {error.get().then(|| view! {
                <div class="absolute inset-0 flex items-center justify-center bg-gray-100">
                    <span class="text-gray-400">"Failed to load image"</span>
                </div>
            })}
        </div>
    }
}
```

## Accessibility Implementation

### ARIA Labels

```rust
#[component]
pub fn AccessibleButton(
    label: String,
    #[prop(optional)] aria_label: Option<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <button
            aria-label=aria_label.unwrap_or(label.clone())
            role="button"
            tabindex="0"
        >
            {children()}
        </button>
    }
}
```

### Keyboard Navigation

```rust
pub fn use_keyboard_navigation() {
    use_effect(|_| {
        let closure = Closure::wrap(Box::new(move |event: KeyboardEvent| {
            match event.key().as_str() {
                "Tab" => {
                    // Handle tab navigation
                    if event.shift_key() {
                        // Shift+Tab - previous element
                    } else {
                        // Tab - next element
                    }
                },
                "Enter" | " " => {
                    // Activate focused element
                    if let Some(active) = document().active_element() {
                        active.dispatch_event(&Event::new("click").unwrap()).unwrap();
                    }
                },
                "Escape" => {
                    // Close modals/dropdowns
                    close_all_overlays();
                },
                _ => {}
            }
        }) as Box<dyn FnMut(_)>);

        document()
            .add_event_listener_with_callback(
                "keydown",
                closure.as_ref().unchecked_ref()
            )
            .unwrap();

        closure.forget();
    });
}
```

### Screen Reader Support

```rust
#[component]
pub fn LiveRegion(
    message: ReadSignal<String>,
    #[prop(optional)] politeness: AriaLive,
) -> impl IntoView {
    view! {
        <div
            role="status"
            aria-live=politeness.to_string()
            aria-atomic="true"
            class="sr-only"
        >
            {move || message.get()}
        </div>
    }
}

pub enum AriaLive {
    Polite,
    Assertive,
    Off,
}

impl ToString for AriaLive {
    fn to_string(&self) -> String {
        match self {
            Self::Polite => "polite".to_string(),
            Self::Assertive => "assertive".to_string(),
            Self::Off => "off".to_string(),
        }
    }
}
```

## Security Considerations

### XSS Prevention

```rust
// All user input is automatically escaped by Leptos
// For raw HTML, use with caution:
#[component]
pub fn SafeHtml(html: String) -> impl IntoView {
    // Sanitize HTML before rendering
    let sanitized = ammonia::clean(&html);

    view! {
        <div inner_html=sanitized></div>
    }
}
```

### CSRF Protection

```rust
pub async fn api_request<T>(
    method: &str,
    url: &str,
    body: Option<T>,
) -> Result<Response, AppError>
where
    T: Serialize,
{
    let csrf_token = get_csrf_token();

    let request = Request::new(url)
        .method(Method::from_str(method).unwrap())
        .header("X-CSRF-Token", &csrf_token)
        .header("Content-Type", "application/json");

    let request = if let Some(body) = body {
        request.json(&body)?
    } else {
        request
    };

    request.send().await.map_err(|e| AppError::NetworkError(e.to_string()))
}
```

### Content Security Policy

```html
<meta http-equiv="Content-Security-Policy" content="
    default-src 'self';
    script-src 'self' 'wasm-unsafe-eval';
    style-src 'self' 'unsafe-inline';
    img-src 'self' data: https:;
    font-src 'self' data:;
    connect-src 'self' https://api.simpelv2.kejaksaan.go.id;
    frame-ancestors 'none';
    base-uri 'self';
    form-action 'self';
">
```

## Deployment Architecture

### Build Process

```bash
# Portal build
cd antarmuka/portal
trunk build --release

# Microfrontend builds (parallel)
for app in badiklat datun intel pidum pidsus pidmil pengawasan pemulihan_aset; do
    cd antarmuka/$app
    trunk build --release &
done
wait

# Shared library is built as dependency
```

### Docker Containerization

```dockerfile
# Multi-stage build for portal
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo install trunk
RUN rustup target add wasm32-unknown-unknown
RUN cd antarmuka/portal && trunk build --release

FROM nginx:alpine
COPY --from=builder /app/antarmuka/portal/dist /usr/share/nginx/html
COPY nginx.conf /etc/nginx/nginx.conf
EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]
```

### Nginx Configuration

```nginx
server {
    listen 80;
    server_name portal.simpelv2.kejaksaan.go.id;

    root /usr/share/nginx/html;
    index index.html;

    # Gzip compression
    gzip on;
    gzip_types text/css application/javascript application/wasm;

    # WASM mime type
    types {
        application/wasm wasm;
    }

    # SPA routing
    location / {
        try_files $uri $uri/ /index.html;
    }

    # API proxy
    location /api/ {
        proxy_pass https://api.simpelv2.kejaksaan.go.id/;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
    }

    # Security headers
    add_header X-Frame-Options "SAMEORIGIN" always;
    add_header X-Content-Type-Options "nosniff" always;
    add_header X-XSS-Protection "1; mode=block" always;
    add_header Referrer-Policy "no-referrer-when-downgrade" always;
}
```

## Monitoring & Analytics

### Performance Monitoring

```rust
pub fn track_performance_metrics() {
    use web_sys::Performance;

    let performance = window().performance().unwrap();

    // Track Core Web Vitals
    let lcp = measure_lcp();  // Largest Contentful Paint
    let fid = measure_fid();  // First Input Delay
    let cls = measure_cls();  // Cumulative Layout Shift

    // Send to analytics
    send_metrics(PerformanceMetrics {
        lcp,
        fid,
        cls,
        ttfb: performance.timing().response_start() - performance.timing().request_start(),
        fcp: measure_fcp(),  // First Contentful Paint
    });
}
```

### Error Tracking

```rust
pub fn setup_error_tracking() {
    set_panic_hook();

    window().set_onerror(Some(&Closure::wrap(Box::new(
        |message: String, source: String, lineno: u32, colno: u32, error: JsValue| {
            log_error(ErrorLog {
                message,
                source,
                line: lineno,
                column: colno,
                stack: format!("{:?}", error),
                timestamp: Utc::now(),
                user_agent: window().navigator().user_agent().unwrap(),
            });
        }
    ) as Box<dyn Fn(_, _, _, _, _)>).into_js_value().unchecked_ref()));
}
```

### User Analytics

```rust
pub fn track_user_event(event: UserEvent) {
    spawn_local(async move {
        let _ = api_request(
            "POST",
            "/api/analytics/events",
            Some(event),
        ).await;
    });
}

#[derive(Serialize)]
pub struct UserEvent {
    pub event_type: String,
    pub page: String,
    pub user_id: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub metadata: serde_json::Value,
}
```


## Infrastructure Integration

### Network Architecture

```
┌──────────────────────────────────────────────────────────────┐
│                        Internet/Users                         │
└────────────────────────────┬─────────────────────────────────┘
                             │
                             ▼
                    ┌────────────────┐
                    │  Nginx (Port 80/443)
                    │  - SSL Termination
                    │  - Rate Limiting
                    │  - Security Headers
                    │  - Static Assets
                    └────────┬───────┘
                             │
                ┌────────────┼────────────┐
                │            │            │
                ▼            ▼            ▼
        ┌───────────┐  ┌──────────┐  ┌──────────────┐
        │  Portal   │  │ Micro-   │  │ Envoy Gateway│
        │  (/)      │  │ frontends│  │ (/api/*)     │
        │           │  │ (/app/*) │  │              │
        └───────────┘  └──────────┘  └──────┬───────┘
                                             │
                                             ▼
                                    ┌────────────────┐
                                    │ Backend Services│
                                    │ - Authenc (IAM)│
                                    │ - Secreton     │
                                    │ - Business APIs│
                                    └────────────────┘
```

### Nginx Configuration Strategy

**Purpose**: Nginx acts as the first layer, handling:
- SSL/TLS termination
- Static file serving for WASM bundles
- Rate limiting per endpoint
- Security headers injection
- Routing to portal and microfrontends

**Key Configuration:**
```nginx
# Rate limiting zones
limit_req_zone $binary_remote_addr zone=api:10m rate=10r/s;
limit_req_zone $binary_remote_addr zone=login:10m rate=5r/m;
limit_req_zone $binary_remote_addr zone=static:10m rate=30r/s;

server {
    listen 443 ssl http2;
    server_name simpelv2.kejaksaan.go.id;

    # Portal (root)
    location / {
        limit_req zone=static burst=20 nodelay;
        root /usr/share/nginx/html/portal;
        try_files $uri $uri/ /index.html;

        # WASM mime type
        types {
            application/wasm wasm;
        }
    }

    # Microfrontends
    location /badiklat {
        limit_req zone=static burst=20 nodelay;
        alias /usr/share/nginx/html/badiklat;
        try_files $uri $uri/ /badiklat/index.html;
    }

    # API Gateway (Envoy)
    location /api/ {
        limit_req zone=api burst=20 nodelay;
        proxy_pass http://envoy:8080;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }

    # Auth endpoints (stricter rate limit)
    location /api/auth/ {
        limit_req zone=login burst=5 nodelay;
        proxy_pass http://envoy:8080;
        # ... same proxy settings
    }
}
```

### Envoy Gateway Integration

**Purpose**: Envoy handles:
- Service mesh routing
- Load balancing
- Circuit breaking
- Retry policies
- Security headers
- CORS handling

**Key Features:**
```yaml
# Circuit breaker configuration
circuit_breakers:
  thresholds:
    - priority: DEFAULT
      max_requests: 1000
      max_retries: 3

# Retry policy
retry_policy:
  retry_on: connect-failure,refused-stream,unavailable
  num_retries: 3
  per_try_timeout: 5s

# Security headers via Lua filter
http_filters:
  - name: envoy.filters.http.lua
    typed_config:
      inline_code: |
        function envoy_on_response(response_handle)
          response_handle:headers():add("X-Frame-Options", "DENY")
          response_handle:headers():add("X-Content-Type-Options", "nosniff")
          # ... more headers
        end
```

### Microservices Communication

**API Request Flow:**
```
Frontend → Nginx → Envoy → Authenc/Business Service
   │         │       │         │
   │         │       │         └─ Validates JWT
   │         │       └─ Routes based on path
   │         └─ Rate limits & SSL
   └─ Makes authenticated request
```

**Authentication Token Flow:**
```rust
// Frontend makes API request
pub async fn api_request<T>(
    method: &str,
    url: &str,
    body: Option<T>,
) -> Result<Response, AppError>
where
    T: Serialize,
{
    // Get JWT from session
    let token = AuthService::get_token()
        .ok_or(AppError::Unauthorized)?;

    // Request goes through:
    // 1. Nginx (rate limiting)
    // 2. Envoy (routing, circuit breaking)
    // 3. Backend service (business logic)
    let request = Request::new(&format!("/api{}", url))
        .method(Method::from_str(method).unwrap())
        .header("Authorization", &format!("Bearer {}", token))
        .header("Content-Type", "application/json");

    let request = if let Some(body) = body {
        request.json(&body)?
    } else {
        request
    };

    request.send().await
        .map_err(|e| AppError::NetworkError(e.to_string()))
}
```

### Service Discovery

**Static Configuration (Development):**
```toml
# config/frontend.toml
[services]
portal_url = "http://localhost:8080"
api_gateway_url = "http://localhost:8081"
authenc_url = "http://localhost:8088"
```

**Dynamic Configuration (Production):**
```rust
// Services discovered via environment variables
pub fn get_service_url(service: &str) -> String {
    match service {
        "portal" => env::var("PORTAL_URL")
            .unwrap_or_else(|_| "https://portal.simpelv2.kejaksaan.go.id".to_string()),
        "api" => env::var("API_GATEWAY_URL")
            .unwrap_or_else(|_| "https://api.simpelv2.kejaksaan.go.id".to_string()),
        "authenc" => env::var("AUTHENC_URL")
            .unwrap_or_else(|_| "https://auth.simpelv2.kejaksaan.go.id".to_string()),
        _ => panic!("Unknown service: {}", service),
    }
}
```

### Load Balancing Strategy

**Nginx Upstream Configuration:**
```nginx
upstream portal_backend {
    least_conn;  # Least connections algorithm
    server portal-1:3000 weight=1 max_fails=3 fail_timeout=30s;
    server portal-2:3000 weight=1 max_fails=3 fail_timeout=30s;
    server portal-3:3000 weight=1 max_fails=3 fail_timeout=30s;

    keepalive 32;  # Connection pooling
}

upstream authenc_backend {
    least_conn;
    server authenc-1:8088 weight=1 max_fails=3 fail_timeout=30s;
    server authenc-2:8088 weight=1 max_fails=3 fail_timeout=30s;

    keepalive 32;
}
```

### Health Checks

**Frontend Health Check:**
```rust
#[component]
pub fn HealthCheck() -> impl IntoView {
    // Exposed at /healthz
    view! {
        <div>"OK"</div>
    }
}
```

**Nginx Health Check:**
```nginx
location /healthz {
    access_log off;
    return 200 "healthy\n";
    add_header Content-Type text/plain;
}
```

**Envoy Health Check:**
```yaml
health_checks:
  - timeout: 5s
    interval: 10s
    unhealthy_threshold: 3
    healthy_threshold: 2
    http_health_check:
      path: /healthz
      expected_statuses:
        - 200
```

### Caching Strategy

**Browser Caching (Nginx):**
```nginx
# Immutable assets (with hash in filename)
location ~* \.(wasm|js|css)$ {
    expires 1y;
    add_header Cache-Control "public, immutable";
}

# HTML (no cache, always revalidate)
location ~* \.html$ {
    expires -1;
    add_header Cache-Control "no-cache, no-store, must-revalidate";
}

# Images
location ~* \.(jpg|jpeg|png|gif|svg|webp)$ {
    expires 30d;
    add_header Cache-Control "public, max-age=2592000";
}
```

**API Response Caching (Envoy):**
```yaml
# Cache GET requests for 5 minutes
http_filters:
  - name: envoy.filters.http.cache
    typed_config:
      "@type": type.googleapis.com/envoy.extensions.filters.http.cache.v3.CacheConfig
      typed_config:
        "@type": type.googleapis.com/envoy.extensions.http.cache.simple_http_cache.v3.SimpleHttpCacheConfig
        cache_size_bytes: 104857600  # 100MB
```

### Security Considerations

**Defense in Depth:**
1. **Nginx Layer**: Rate limiting, SSL, basic filtering
2. **Envoy Layer**: Advanced routing, circuit breaking, header injection
3. **Application Layer**: JWT validation, RBAC, business logic
4. **Backend Layer**: Database access control, encryption at rest

**Security Headers (Applied at Multiple Layers):**
```
Content-Security-Policy: default-src 'self'; script-src 'self' 'wasm-unsafe-eval'
Strict-Transport-Security: max-age=63072000; includeSubDomains; preload
X-Frame-Options: DENY
X-Content-Type-Options: nosniff
X-XSS-Protection: 1; mode=block
Referrer-Policy: no-referrer-when-downgrade
Permissions-Policy: geolocation=(), microphone=(), camera=()
```

**CORS Configuration (Envoy):**
```yaml
cors:
  allow_origin_string_match:
    - exact: "https://simpelv2.kejaksaan.go.id"
    - exact: "https://portal.simpelv2.kejaksaan.go.id"
  allow_methods: "GET, POST, PUT, DELETE, OPTIONS"
  allow_headers: "Authorization, Content-Type, X-CSRF-Token"
  expose_headers: "X-Request-ID"
  max_age: "86400"
  allow_credentials: true
```
