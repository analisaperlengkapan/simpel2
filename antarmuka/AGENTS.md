# 🤖 AGENTS.md - Antarmuka (Frontend Microfrontends)

> **Notice to Agents**: File ini adalah pedoman (Level 2 Archetype) untuk SELURUH Frontend Microfrontends (`antarmuka/`) di monorepo SIMPEL. Baca file ini sebelum memodifikasi kode frontend.

## 📑 Daftar Isi (Table of Contents)
1. 🏛️ Frontend Architecture
2. 🔐 Authentication Flow (Frontend Side)
--- *Batas Truncation* ---
3. 🦀 Code Patterns (Leptos 0.8.x)
4. ⚡ Performance Optimization (WASM, Trunk)
5. 🧪 Testing Strategy (Frontend)

## 🏛️ Frontend Architecture

Frontend SIMPEL menggunakan arsitektur **Microfrontend** berbasis WebAssembly (WASM) Client-Side Rendering (CSR).

- **Framework**: Leptos v0.8.x
- **Kompilasi**: Trunk (`trunk build --release`)
- **UI Library**: Komponen bersama harus diletakkan di `lib/ui/` dan diimpor.
- **Komunikasi**: Microfrontends WAJIB berkomunikasi ke backend menggunakan **REST API (JSON/HTTP)**. Dilarang keras memanggil gRPC backend (seperti Authenc atau Secreton) secara langsung.

## 🔐 Authentication Flow (Frontend Side)

1. **Pengecekan Sesi**: Periksa `localStorage` (BUKAN `sessionStorage`) untuk sinkronisasi token JWT lintas tab.
2. **Validasi**: Panggil `/api/auth/status` (REST API). Jika 401 Unauthorized, *redirect* ke halaman login.
3. **Penggunaan Token**: Sertakan JWT di *header* HTTP untuk setiap permintaan REST ke backend.

> **Catatan**: Gunakan *hooks* dari `lib_ui::hooks::use_auth` untuk standardisasi.

---

## 🦀 Code Patterns

### Leptos 0.8.x Pattern

```rust
use leptos::prelude::*;
use lib_ui::prelude::*;  // Shared UI components

// ✅ Signal creation (NOT create_signal!)
#[component]
pub fn Counter(initial: i32) -> impl IntoView {
    let (count, set_count) = signal(initial);

    let increment = move |_| *set_count.write() += 1;
    let decrement = move |_| *set_count.write() -= 1;

    view! {
        <div class="counter">
            <button on:click=decrement>"-1"</button>
            <span>{count}</span>
            <button on:click=increment>"+1"</button>
        </div>
    }
}

// ✅ Global State Management (NO prop-drilling!)
#[component]
pub fn AppRoot() -> impl IntoView {
    // Provide state globally at the root
    let (user_session, set_user_session) = signal(None::<UserSession>);
    provide_context((user_session, set_user_session));
    
    view! { <MainRouter /> }
}

// ✅ Resource for async data (calls REST API, NOT gRPC!)
#[component]
pub fn UserList() -> impl IntoView {
    let users = Resource::new(
        || (),
        |_| async move {
            gloo_net::http::Request::get("/api/v1/users")
                .send().await?.json::<Vec<User>>().await
        }
    );

    view! {
        <Suspense fallback=|| view! { <Loading /> }>
            {move || users.get().map(|result| match result {
                Ok(users) => view! { <UserTable users=users /> }.into_any(),
                Err(e) => view! { <Error message=e.to_string() /> }.into_any(),
            })}
        </Suspense>
    }
}

// ✅ Main mount (CSR)
pub fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(App);
}
```

---

## ⚡ Performance Optimization Patterns

### WASM Optimization (Trunk & Cargo)

- **Code splitting**: Gunakan *lazy loading* pada *routes* untuk memperkecil ukuran *bundle* awal.
- **Compression**: Gunakan kompresi gzip/brotli untuk *production builds*.
- **Tree shaking**: Singkirkan *unused code*.

**Konfigurasi `Trunk.toml` Standar:**
```toml
[tools]
wasm-bindgen = "0.2"
wasm-opt = ['-O3', '--enable-bulk-memory']

[build]
release = true
```

### Signal Reactivity Best Practices
Gunakan Leptos 0.8.x *signal API* untuk reaktivitas yang efisien:

```rust
use leptos::prelude::*;

// ✅ Gunakan signal() untuk reaktivitas pasif
let (count, set_count) = signal(0);

// ✅ Gunakan derived signal untuk computed values
let doubled = move || *count.read() * 2;

// ❌ Hindari create_signal() iterasi lama
// let (count, set_count) = create_signal(0);
```

### Debouncing untuk User Input
Untuk *input* pencarian yang memicu panggilan API mahal, gunakan *debouncing*:

```rust
use leptos::prelude::*;

let (search_query, set_search_query) = signal(String::new());

// Debounced search - hanya trigger setelah user berhenti mengetik
let debounced_search = Memo::new(move |_| {
    // Implement debouncing logic (menggunakan timer dari gloo-timers)
    search_query.read().clone()
});
```

---

## 🧪 Testing Strategy (Frontend)

- **E2E Testing**: Gunakan Playwright untuk menguji seluruh alur pengguna (user flows) di antarmuka web, termasuk penanganan CAPTCHA jika diperlukan.
- **Komponen**: Uji *logic* statis secara independen. Hindari menempatkan *business logic* kompleks di dalam komponen antarmuka.

---

## 🔐 Secret Boundary (Frontend NEVER Touches Secreton)

- Frontend Leptos **TIDAK** memanggil Secreton (gRPC/HTTP) langsung. Semua secret hidup di server-side (backend Rust / authenc / Secreton).
- `config.json` runtime (di-generate `entrypoint.sh` saat container start) hanya berisi **URL public** (`PORTAL_URL`, `AUTHENC_URL`, `API_URL`) — bukan secret.
- JWT token user disimpan di `localStorage` (key `auth_token`, canonical), dikirim ke backend di header `Authorization: Bearer <token>`. Tidak ada API key client-side.
- Saat `secretonAuth.enabled=true` di Helm: pod portal & perlengkapan **TIDAK** punya projected SA token Secreton (tidak butuh — frontend tidak fetch secret).
- CSP `connect-src` dibatasi ke domain backend resmi (`https://*.kejaksaan.go.id`) — set via `values.yaml` `portal.csp.connectSrc`.
