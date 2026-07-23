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
>
> **Skill:** untuk pengembangan/refactor Leptos (model reaktif, Resource/Action,
> ErrorBoundary, pitfalls reaktivitas) pakai Skill **`leptos-expert`** (`.claude/skills/`).

### Struktur feature-first (F0-B)

Tiap app diorganisir **per bounded-context di `src/features/<domain>/`**, bukan
page-centric. Konvensi (portal sudah mengikuti ini; `perlengkapan` menyusul):

```
antarmuka/<app>/src/
  main.rs  lib.rs  app.rs        # root component + provide_context global
  routes.rs                      # rute bertipe (satu sumber)
  features/
    <domain>/                    # mis. dashboard, profile, admin, kebutuhan_bmn
      mod.rs                     # `pub mod` + re-export glob (jaga path publik)
      page.rs | pages/           # komponen route-target
      components/                # komponen lokal fitur (opsional)
      api.rs                     # klien HTTP bertipe fitur (opsional)
      state.rs                   # signal/Resource/Action khusus fitur (opsional)
    auth/                        # context/service + OAuth + login/callback pages
    session/                     # cross-tab session monitor + sessions page
  components/                    # HANYA komponen dumb/shared lintas-fitur
  utils/                         # helper non-domain (BUKAN tempat API domain)
  tests/e2e/
```

Aturan:
- **Halaman milik fiturnya** — `pages/<x>.rs` lama → `features/<domain>/`. `mod.rs`
  me-`pub use` ulang sehingga path `crate::features::*` tetap stabil; `app.rs`
  meng-impor view dari modul fitur.
- `components/` top-level **hanya** untuk komponen dumb/shared; komponen domain
  tinggal di fiturnya. Komponen shared yang layak naik → `lib/ui`.
- Cross-cutting (mis. `microfrontends` registry) boleh tetap modul datar.
- Saat memecah/memindah: pakai path absolut `crate::features::…`, jaga re-export
  `mod.rs`. Murni pemindahan kode — verifikasi `cargo check --target
  wasm32-unknown-unknown -p <app>` + `trunk build` hijau.

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

## 🎨 Formatting & Linting (FE crates)

Dua hal yang **WAJIB** dipahami sebelum menyentuh crate FE
(`perlengkapan-microfrontend`, `portal-microfrontend`):

### 1. `view!` diformat MANUAL — leptosfmt DIHAPUS (2026-07-19)

`cargo fmt` (rustfmt) **tidak** masuk ke dalam token-tree makro, jadi isi
`view! { … }` tak tersentuh. Dulu celah itu ditutup **leptosfmt** (`0.1.33`) via
autofix bot. **Itu sudah dihapus dari `autofix.yml`** — isi `view!` sekarang
diformat tangan.

> 🚨 **Alasan penghapusan: leptosfmt MERUSAK markup secara SENYAP.** Ia
> mem-parse isi `view!` sebagai markup, sehingga **memotong ekspresi atribut di
> `>` pertama**. File `antarmuka/portal/src/features/admin/users.rs` dirusak
> **dua kali** dengan pola sama:
>
> ```rust
> // sebelum
> disabled=move || page.get() >= total_pages.get()
> class="px-3 py-1 …"
> // sesudah leptosfmt
> disabled=move || page.get()   // u32, bukan bool
> >                             // tag close liar
> = total_pages.get()           // jadi TEXT CHILD tombol
> class=
> "px-3 py-1 …"                 // juga text child
> ```
>
> **Yang membuatnya berbahaya: hasilnya TETAP KOMPILASI** (terverifikasi
> `cargo check -p portal-microfrontend --target wasm32-unknown-unknown` = exit 0).
> Tak ada gate kompilasi yang bisa menangkapnya — yang ter-deploy adalah tombol
> dengan teks sampah dan state `disabled` salah. Ini **lebih buruk** daripada
> build error. Hanya e2e yang meng-assert paginasi yang bisa menangkapnya.
>
> Karena leptosfmt tak pernah jadi gate, menghapusnya **tidak menghilangkan
> jaminan apa pun**.

**Aturan yang tetap berlaku (agar aman untuk formatter/tool apa pun):**

1. **JANGAN taruh operator perbandingan (`>`, `<`, `>=`, `<=`) di posisi
   atribut `view!`.** Ekstrak ke `Memo` di atas `view!`:

   ```rust
   let on_last_page = Memo::new(move |_| page.get() >= total_pages.get());
   // …
   <button disabled=on_last_page class="…">
   ```

   Ini pola yang dipakai `users.rs` sekarang.
2. **JANGAN taruh komentar non-doc `//` di dalam / tepat setelah blok kode
   `{ … }` di dalam `view!`.** leptosfmt memakai fork prettyplease yang tak
   mendukung komentar di code-block → output **non-idempoten** (~17 file
   berosilasi). Relevan lagi bila suatu saat formatter `view!` dihidupkan ulang.

### 2. FE crates di-**exclude** dari `cargo fix` / `clippy --fix`

Crate FE bersifat **dual-cfg**: kode browser `#[cfg(target_arch = "wasm32")]` +
~189 stub host `#[cfg(not(target_arch = "wasm32"))]`. `cargo fix`/`clippy --fix`
hanya mengevaluasi **satu** konfigurasi cfg dan membuang yang ter-gate ke cfg lain
sebagai "dead" → merusak (pernah membobol PR #526). Maka autofix bot **meng-exclude**
kedua crate FE dari kedua fixer itu (`--exclude perlengkapan-microfrontend
--exclude portal-microfrontend`; lihat `.github/workflows/autofix.yml`). **Lint FE
diperbaiki MANUAL secara cfg-aware** — cfg-gate import wasm-only; pakai
`#[cfg_attr(not(target_arch = "wasm32"), allow(unused_variables))]` pada binding
wasm-only; **JANGAN** `_`-prefix nama yang di-bind closure/PropsBuilder Leptos.
Gate clippy di `ci.yml` tetap menjaring lint FE (di target wasm32, `--all-features`).

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

## 📋 Common Tasks

> **Catatan:** sebagian contoh di bawah masih menampilkan layout transisi `perlengkapan`
> (`pages/`, `src/api/<modul>.rs`) yang sedang dimigrasi. **Kode baru ikut feature-first**
> (`features/<domain>/{page,api,state}.rs`, lihat "Struktur feature-first (F0-B)" di atas).

### 1. Add a new MFE route with auth guard

Place the route's view component in its feature module
(`src/features/<domain>/`, see "Struktur feature-first" above), then wire it in
the router. Both Portal and Perlengkapan MFE use `leptos_router` with parent-route
layouts as guards (Next.js-style `layout.tsx`). The shape:

```rust
// In `src/lib.rs` of the MFE:
<ParentRoute path=path!("/") view=AuthenticatedLayout>
    <Route path=path!("/dashboard") view=DashboardHome />
    // ↑ AuthenticatedLayout reads UserSession from context, redirects
    //   to /login if absent — childless routes get the guard for free.
</ParentRoute>

<ParentRoute path=path!("/admin") view=AdminLayout>
    <Route path=path!("/workflow") view=WorkflowConfigManagement />
    // ↑ AdminLayout asserts is_admin() — non-admins get a "Forbidden"
    //   page rather than the child view.
</ParentRoute>
```

For a route that needs **session context** to render (e.g. derive
NIP/name from the JWT instead of receiving as a prop), use a wrapper
component — see
`antarmuka/perlengkapan/src/lib.rs::UkuranPegawaiCurrentUser` for the
canonical pattern.

### 2. Consume a new backend endpoint

The fetch flow is the same across both MFEs:

1. **Client function** in `src/api/<modul>.rs` — gate the
   `wasm32` body with `#[cfg(target_arch = "wasm32")]` and provide a
   stub for the non-wasm build so `cargo check` from `tests/` etc.
   still type-checks. Pattern:

   ```rust
   #[cfg(target_arch = "wasm32")]
   pub async fn fetch_thing(id: &str) -> Result<ApiResponse<Thing>, AppError> {
       use crate::api::client::get_auth_token;
       use gloo_net::http::Request;
       let url = format!("{}/things/{}", API_BASE, id);
       let token = get_auth_token().ok_or_else(|| AppError::Auth("…".into()))?;
       let resp = Request::get(&url)
           .header("Authorization", &format!("Bearer {}", token))
           .send().await?;
       if !resp.ok() { return Err(/* HTTP error */); }
       resp.json::<ApiResponse<Thing>>().await.map_err(|e| /* … */)
   }

   #[cfg(not(target_arch = "wasm32"))]
   pub async fn fetch_thing(_id: &str) -> Result<ApiResponse<Thing>, AppError> {
       Err(AppError::Unknown("Server-side stub".into()))
   }
   ```

2. **Caching** — for read-heavy endpoints prefer
   `leptos_fetch::QueryClient::local_resource` so re-renders hit cache
   instead of refetching. See
   `pages/workflow/config_management.rs` for the convention.

3. **POST/PUT/DELETE** — surface backend's structured error: read the
   response body on non-2xx and bubble up via `AppError::Unknown(format!("HTTP {}: {}", status, body))`
   so the user sees the actual reason, not just "500".

4. **Response DTO nullability WAJIB cermin backend.** Kalau field di model
   backend `Option<T>`, di DTO FE **harus** `Option<T>` juga. Kalau ragu,
   pilih `Option`.

   Alasannya bukan gaya, tapi radius kerusakannya: serde gagal pada
   **SELURUH** respons begitu satu field required menerima `null`. Satu kolom
   audit yang null mengosongkan **seluruh halaman**, padahal API-nya
   sehat — halaman tampak "tidak ada data", bukan error.

   Kejadian nyata (2026-07-19): `IzinPemakaianBmn.created_by`/`created_by_nama`
   ditulis `String` di FE padahal backend `Option<Uuid>`/`Option<String>` dan
   skema-nya nullable. Seed e2e tidak mengisi kolom itu → daftar Pemakaian BMN
   **kosong total** di browser. Semua field audit tetangganya
   (`approved_by`/`revoked_by`/`updated_by`) sudah `Option` — dua ini
   satu-satunya yang luput. Ironisnya keduanya **tidak pernah dibaca** UI;
   mereka ada murni untuk deserialisasi, lalu justru mematahkannya.

   **Kenapa tidak ada gerbang yang menangkap ini:** `cargo check` hijau (tipe
   FE konsisten dengan dirinya sendiri), `cargo check` backend hijau, dan spec
   e2e yang memanggil endpoint via `request.get()` juga hijau — karena ia
   membaca `body.data.map(r => r.id)` sebagai JSON mentah, tak pernah lewat
   struct Rust FE. Jadi drift ini **tak terlihat oleh kedua compiler maupun
   tes API**. Satu-satunya yang menangkapnya = spec yang membuka halamannya
   di browser.

   Sapuan pembanding FE↔BE ada di riwayat sesi (bandingkan optionality per
   field untuk struct bernama sama di `antarmuka/*/src/api/*.rs` vs
   `layanan/*/src/**`). Arah `Request`/`Query` (FE required, BE `Option`)
   aman — FE selalu mengirim nilai; yang berbahaya hanya arah **respons**.

   Perbaikan struktural yang lebih tuntas (belum dikerjakan): pindahkan DTO
   respons ke `lib/perlengkapan` yang sudah WASM-safe dan pakai **tipe yang
   sama** di kedua sisi, seperti `lib_perlengkapan::response::PaginatedResponse`
   — kalau tipenya satu, drift-nya mustahil.

### 3. Run a single MFE locally

```bash
# Portal (default port 8080)
cd antarmuka/portal && trunk serve --open

# Perlengkapan (different port to coexist with Portal)
cd antarmuka/perlengkapan && trunk serve --port 8082

# Or both at once via docker-compose (recommended for cross-MFE flows):
docker-compose up portal-mfe perlengkapan-mfe
```

`trunk serve` watches `index.html` + Rust sources and re-builds on
change. For backend-dependent pages you need the matching `layanan/*`
service running — easiest path is docker-compose so DB + Authenc +
Perlengkapan are all wired up.

To test cross-MFE token sync manually:
1. Login on Portal tab → confirm `auth_token` set in DevTools →
   Application → Local Storage.
2. Open Perlengkapan in a separate tab same origin → confirm session
   loads without re-auth.
3. Hit logout on Portal → Perlengkapan tab should redirect within
   ~2 seconds via the `logout_event` listener.

---

## 🔐 Secret Boundary (Frontend NEVER Touches Secreton)

- Frontend Leptos **TIDAK** memanggil Secreton (gRPC/HTTP) langsung. Semua secret hidup di server-side (backend Rust / authenc / Secreton).
- `config.json` runtime (di-generate `entrypoint.sh` saat container start) hanya berisi **URL public** (`PORTAL_URL`, `AUTHENC_URL`, `API_URL`) — bukan secret.
- JWT token user disimpan di `localStorage` (key `auth_token`, canonical), dikirim ke backend di header `Authorization: Bearer <token>`. Tidak ada API key client-side.
- Saat `secretonAuth.enabled=true` di Helm: pod portal & perlengkapan **TIDAK** punya projected SA token Secreton (tidak butuh — frontend tidak fetch secret).
- CSP `connect-src` dibatasi ke domain backend resmi (`https://*.kejaksaan.go.id`) — set via `values.yaml` `portal.csp.connectSrc`.
