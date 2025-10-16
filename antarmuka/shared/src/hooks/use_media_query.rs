//! Media query utilities untuk responsive design

use web_sys::window;

/// Check if media query matches
pub fn matches_media_query(query: &str) -> bool {
    window()
        .and_then(|w| w.match_media(query).ok().flatten())
        .map(|mql| mql.matches())
        .unwrap_or(false)
}

/// Check if device is mobile (< 768px)
pub fn is_mobile() -> bool {
    matches_media_query("(max-width: 767px)")
}

/// Check if device is tablet (768px - 1023px)
pub fn is_tablet() -> bool {
    matches_media_query("(min-width: 768px) and (max-width: 1023px)")
}

/// Check if device is desktop (>= 1024px)
pub fn is_desktop() -> bool {
    matches_media_query("(min-width: 1024px)")
}

/// Check if small screen (< 1024px)
pub fn is_small_screen() -> bool {
    matches_media_query("(max-width: 1023px)")
}
