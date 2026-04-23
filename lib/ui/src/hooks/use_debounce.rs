//! Debounce utilities untuk optimasi input.
//!
//! # Rekomendasi: `use_debounce_fn`
//!
//! Gunakan `leptos_use::use_debounce_fn` untuk debouncing baru — lebih
//! lengkap (leading/trailing, max_wait), well-tested, dan sudah jadi
//! standar ekosistem Leptos.
//!
//! ```rust,ignore
//! use leptos_use::use_debounce_fn;
//!
//! let debounced = use_debounce_fn(
//!     move || {
//!         // dipanggil setelah user berhenti mengetik 300ms
//!         tracing::info!(q = %query.get(), "search");
//!     },
//!     300.0, // milliseconds
//! );
//!
//! view! {
//!     <input on:input=move |ev| {
//!         set_query.set(event_target_value(&ev));
//!         debounced();
//!     } />
//! }
//! ```
//!
//! Untuk varian dengan argumen, gunakan `use_debounce_fn_with_arg`.
//!
//! # Legacy: `create_debounced`
//!
//! `create_debounced` dipertahankan sebagai alias `use_debounce_fn` untuk
//! kompabilitas, tapi call site baru sebaiknya langsung pakai
//! `leptos_use::use_debounce_fn`.
//!
//! Re-export `use_debounce_fn` dari modul ini tersedia di
//! `lib_ui::hooks::use_debounce_fn`.

pub use leptos_use::use_debounce_fn;

/// Create a debounced function.
///
/// **Deprecated-style alias** untuk `leptos_use::use_debounce_fn`. Signature
/// backward-compatible (`delay_ms: u32`) dengan versi lama. Untuk kode baru,
/// gunakan `use_debounce_fn` langsung.
pub fn create_debounced<F>(func: F, delay_ms: u32) -> impl Fn() + Clone
where
    F: Fn() + Clone + 'static,
{
    let debounced = use_debounce_fn(func, f64::from(delay_ms));
    move || {
        debounced();
    }
}
