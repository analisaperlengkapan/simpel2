//! Media query utilities untuk responsive design.
//!
//! Backed by `leptos_use::use_media_query` — returns reactive `Signal<bool>`
//! yang auto-update saat viewport berubah (resize, orientation change).
//!
//! # Migration dari versi one-shot lama
//!
//! Sebelumnya fungsi ini mengembalikan `bool` sekali-panggil. Sekarang
//! mengembalikan `Signal<bool>` agar komponen benar-benar responsif tanpa
//! perlu memasang listener `resize` manual.
//!
//! ```rust,ignore
//! // ❌ Lama (non-reactive, one-shot):
//! if is_mobile() { /* ... */ }
//!
//! // ✅ Sekarang (reactive):
//! let mobile = is_mobile();
//! view! { <Show when=move || mobile.get()>...</Show> }
//! ```
//!
//! Untuk satu-kali check di dalam event handler (bukan view), gunakan
//! `mobile.get_untracked()`.
//!
//! # Breakpoints
//!
//! Selaras dengan Tailwind CSS default:
//! - Mobile: `< 768px` (below `md`)
//! - Tablet: `768px - 1023px` (`md` to below `lg`)
//! - Desktop: `>= 1024px` (`lg`+)

use leptos::prelude::Signal;
use leptos_use::use_media_query;

/// Reactive media query signal. Update otomatis saat viewport berubah.
///
/// ```rust,ignore
/// let is_dark = matches_media_query("(prefers-color-scheme: dark)");
/// view! { <Show when=move || is_dark.get()>...</Show> }
/// ```
pub fn matches_media_query(query: &str) -> Signal<bool> {
    use_media_query(query.to_string())
}

/// Reactive signal: `true` saat viewport < 768px (mobile).
pub fn is_mobile() -> Signal<bool> {
    use_media_query("(max-width: 767px)")
}

/// Reactive signal: `true` saat viewport 768px - 1023px (tablet).
pub fn is_tablet() -> Signal<bool> {
    use_media_query("(min-width: 768px) and (max-width: 1023px)")
}

/// Reactive signal: `true` saat viewport >= 1024px (desktop).
pub fn is_desktop() -> Signal<bool> {
    use_media_query("(min-width: 1024px)")
}

/// Reactive signal: `true` saat viewport < 1024px (mobile atau tablet).
pub fn is_small_screen() -> Signal<bool> {
    use_media_query("(max-width: 1023px)")
}
