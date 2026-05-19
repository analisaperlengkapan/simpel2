# 🤖 AGENTS.md - Panduan Komponen Antarmuka (`lib-ui`)

> **Domain Konteks**: Leptos WebAssembly (WASM) UI Component Library

---

## 🖼️ Kerangka Kerja UI (The AI Directives)

Gunakan pustaka independen UI bawaan ini untuk Microfrontends SIMPEL. **DILARANG KERAS** menulis spesifikasi UI/Komponen dari awal jika komponen Atom/Molekulnya sudah terdaftar di sini!

```mermaid
stateDiagram-v2
    [*] --> Periksa_Komponen_Ada
    Periksa_Komponen_Ada --> Re-Use_Komponen : Ada
    Periksa_Komponen_Ada --> Rancang_Komponen_Baru_Agnostik : Tidak Ada
    Rancang_Komponen_Baru_Agnostik --> Tambahkan_Props_Reaktif
    Tambahkan_Props_Reaktif --> Implementasi_di_Aplikasi
```

## 🛠 Aturan Pengembangan (Design System)

1. **Agnostik Bisnis Seisinya! (Domain-Agnostic)**
   - Saat mendesain elemen tabel, namakan `DataTable` atau `Pagination`, **bukan** `TabelBarangBMN`. Letakkan abstraksi tabel *TabelBarangBMN* di proyek `antarmuka/perlengkapan/`, bukan di sini.

2. **Gunakan Sinyal (*Signals*) dengan Hati-Hati!**
   - Dalam Leptos v0.8.x, kita menggunakan `signal()` dan tipe reaktif pasif lainnya, BUKAN `create_signal()` iterasi lama yang memicu mutabilitas manual kompleks. Ikuti pola reaktivitas modern.

3. **Komponen Bersih tanpa Efek Samping (Side-Effects)**
   - Komponen UI sebisa mungkin tidak menyimpan integrasi URL langsung ke *backend* menggunakan HTTP Client/Gloo. Komponen UI itu *"bodoh - dumb components"* dan meminta properti seperti `on_submit` atau parameter *props value* dari penginduknya.

4. **Kaidah Modular CSS Styling**
   - Di `lib-ui`, gunakan *Utility-Classes* (Tailwind-style) yang terdaftar di konfigurasi proyek. Hindari menulis gaya *inline* seperti `style="color: red"`.

5. **Pengepakan WASM**
   - `lib-ui` akan ditautkan dan dikompilasi secara dinamis (*cdylib*/ *rlib*). Setiap penambahan pustaka Cargo pihak ketiga yang Anda berikan harus kompatibel dengan OS arsitektur WASM (`wasm32-unknown-unknown`).

---

## ⚡ Performance Patterns

### Signal Reactivity Best Practices

Gunakan Leptos 0.8.x signal API untuk reaktivitas yang efisien:

```rust
use leptos::prelude::*;

// ✅ Gunakan signal() untuk reaktivitas pasif
let (count, set_count) = signal(0);

// ✅ Gunakan derived signal untuk computed values
let doubled = move || *count.read() * 2;

// ❌ Hindari create_signal() (deprecated)
// let (count, set_count) = create_signal(0);
```

### Memoization untuk Expensive Computations

Gunakan `Memo::new` untuk memoizing computed values:

```rust
use leptos::prelude::*;

let (items, set_items) = signal(vec![/* ... */]);

// Memoized computation - hanya recompute ketika items berubah
let filtered_items = Memo::new(move |_| {
    items.read()
        .iter()
        .filter(|item| item.active)
        .cloned()
        .collect::<Vec<_>>()
});
```

### Lazy Loading untuk Routes

Implement lazy loading untuk mengurangi initial bundle size:

```rust
use leptos_router::{Route, RouteRedirect};

#[component]
pub fn App() -> impl IntoView {
    view! {
        <Router>
            <Routes>
                <Route path="/" view=Home />
                <Route
                    path="/perlengkapan"
                    view=|| view! { <Perlengkapan /> }
                />
                // Lazy load heavy components
                <Route
                    path="/reports"
                    view=|| view! { <Suspense fallback=|| view! { <Loading /> }>
                        <Reports />
                    </Suspense> }
                />
            </Routes>
        </Router>
    }
}
```

### Component Reusability

Desain komponen agar reusable dan tidak mengandung logika bisnis:

```rust
// ✅ Komponen agnostik - reusable
#[component]
pub fn DataTable<T>(
    rows: Signal<Vec<T>>,
    columns: Vec<Column<T>>,
    on_row_click: Option<Callback<T>>,
) -> impl IntoView
where
    T: Clone + 'static,
{
    view! {
        <table class="data-table">
            // Generic table implementation
        </table>
    }
}

// ❌ Komponen spesifik domain - tidak reusable
#[component]
pub fn TabelBarangBMN(
    barang: Signal<Vec<Barang>>,
) -> impl IntoView {
    // Business logic seharusnya di parent
}
```

### CSS Optimization

Gunakan utility classes dan hindari inline styles:

```rust
// ✅ Gunakan utility classes
view! {
    <div class="flex items-center justify-between p-4 bg-white rounded-lg shadow">
        // Content
    </div>
}

// ❌ Hindari inline styles
view! {
    <div style="display: flex; align-items: center; justify-content: space-between; padding: 1rem; background: white; border-radius: 0.5rem; box-shadow: 0 1px 3px rgba(0,0,0,0.1);">
        // Content
    </div>
}
```

### Bundle Size Optimization

Minimize bundle size dengan best practices:

- **Tree shaking**: Hanya import yang diperlukan dari libraries
- **Code splitting**: Lazy load routes dan heavy components
- **Compression**: Gunakan gzip/brotli di production
- **Minification**: Enable `wasm-opt` dengan `-O3`

```toml
# Trunk.toml
[tools]
wasm-bindgen = "0.2"
wasm-opt = ['-O3', '--enable-bulk-memory']

[build]
release = true
```

### Debouncing untuk User Input

Gunakan debouncing untuk input yang memicu expensive operations:

```rust
use leptos::prelude::*;
use std::time::Duration;

let (search_query, set_search_query) = signal(String::new());

// Debounced search - hanya trigger setelah user berhenti mengetik
let debounced_search = Memo::new(move |_| {
    // Implement debouncing logic
    search_query.read().clone()
});
```

### Virtual Scrolling untuk Large Lists

Untuk lists dengan banyak items, gunakan virtual scrolling:

```rust
// Gunakan library seperti leptos-virtual-scroll untuk large datasets
// Hanya render items yang visible di viewport
```

### Performance Monitoring

Instrument komponen untuk performance tracking:

```rust
use leptos::prelude::*;

#[component]
pub fn PerformanceMonitoredComponent() -> impl IntoView {
    if let Some(perf) = web_sys::window().and_then(|w| w.performance()) {
        let start_time = perf.now();

        on_cleanup(move || {
            let duration = perf.now() - start_time;
            tracing::info!(duration_ms = duration, "Component render time");
        });
    }

    view! {
        // Component content
    }
}
```
