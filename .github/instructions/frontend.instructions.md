---
applyTo: "antarmuka/**/*.rs"
---

# Leptos Microfrontend Guidelines

## Component Structure
```rust
use leptos::prelude::*;
use shared_microfrontend::components::*;
use shared_microfrontend::hooks::use_auth;

#[component]
pub fn MyPage() -> impl IntoView {
    let auth = use_auth();

    view! {
        <ProtectedRoute fallback=|| view! { <LoginRedirect /> }>
            <PageContent />
        </ProtectedRoute>
    }
}
```

## Auth Integration
- **ALWAYS** use `use_auth()` hook from `shared_microfrontend::hooks`
- **NEVER** implement auth logic locally
- Session is in `localStorage`, not cookies
- Redirect to Portal (`/`) for login

## Shared Components
Import from `shared-microfrontend`:
```rust
use shared_microfrontend::{
    components::{Button, Input, Modal, Alert, DataTable, ProtectedRoute},
    hooks::{use_auth, use_local_storage, use_debounce},
};
```

## Build Commands
```bash
cd antarmuka/daskrimti/portal && trunk build --release
cd antarmuka/daskrimti/portal && trunk serve --open
```

## WASM Considerations
- No dynamic imports/lazy loading
- All components must be `Send + Sync`
- Use `signal()` not `create_signal()`
- Events: `on:click=`, `on:submit=`, `on:input=`
