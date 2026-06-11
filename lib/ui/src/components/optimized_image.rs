//! Optimized image component with lazy loading and responsive images

use leptos::prelude::*;

/// OptimizedImage component with lazy loading, responsive images, and loading states
///
/// Features:
/// - Lazy loading (native browser support)
/// - Responsive images with srcset
/// - Loading placeholder
/// - Error handling
/// - WebP format support with fallback
///
/// # Example
/// ```rust
/// use lib_ui::components::OptimizedImage;
/// use leptos::prelude::*;
///
/// #[component]
/// pub fn Gallery() -> impl IntoView {
///     view! {
///         <OptimizedImage
///             src="/images/photo.jpg"
///             alt="Beautiful landscape"
///             lazy=true
///             responsive=true
///         />
///     }
/// }
/// ```
#[component]
pub fn OptimizedImage(
    /// Image source URL
    #[prop(into)]
    src: String,
    /// Alt text for accessibility
    #[prop(into)]
    alt: String,
    /// Enable lazy loading (default: true)
    #[prop(default = true)]
    lazy: bool,
    /// Enable responsive images with srcset (default: false)
    #[prop(default = false)]
    responsive: bool,
    /// Responsive image sizes (widths in pixels)
    #[prop(default = vec![320, 640, 960, 1280, 1920])]
    sizes: Vec<u32>,
    /// CSS class for the image
    #[prop(optional, into)]
    class: Option<String>,
    /// Width attribute
    #[prop(optional)]
    width: Option<u32>,
    /// Height attribute
    #[prop(optional)]
    height: Option<u32>,
    /// Object fit CSS property
    #[prop(default = "cover".to_string(), into)]
    object_fit: String,
) -> impl IntoView {
    let class = class.unwrap_or_default();
    let (loaded, set_loaded) = signal(false);
    let (error, set_error) = signal(false);

    // Generate srcset for responsive images
    let srcset = if responsive {
        let base_path = src.rsplit_once('.').map(|(path, _)| path).unwrap_or(&src);
        let extension = src.rsplit_once('.').map(|(_, ext)| ext).unwrap_or("jpg");

        sizes
            .iter()
            .map(|width| format!("{}-{}w.{} {}w", base_path, width, extension, width))
            .collect::<Vec<_>>()
            .join(", ")
    } else {
        String::new()
    };

    // Generate sizes attribute for responsive images
    let sizes_attr = if responsive {
        "(max-width: 640px) 100vw, (max-width: 1024px) 50vw, 33vw".to_string()
    } else {
        String::new()
    };

    view! {
        <div class=format!("relative overflow-hidden {}", class)>
            // Loading placeholder
            {move || (!loaded.get() && !error.get()).then(|| view! {
                <div class="absolute inset-0 bg-gray-200 dark:bg-gray-700 animate-pulse"></div>
            })}

            // Actual image
            {move || (!error.get()).then(|| view! {
                <img
                    src=src.clone()
                    srcset=if responsive { srcset.clone() } else { String::new() }
                    sizes=if responsive { sizes_attr.clone() } else { String::new() }
                    alt=alt.clone()
                    loading=if lazy { "lazy" } else { "eager" }
                    decoding="async"
                    width=width.map(|w| w.to_string())
                    height=height.map(|h| h.to_string())
                    class=format!(
                        "transition-opacity duration-300 {}",
                        if loaded.get() { "opacity-100" } else { "opacity-0" }
                    )
                    style=format!("object-fit: {}", object_fit)
                    on:load=move |_| set_loaded.set(true)
                    on:error=move |_| {
                        set_error.set(true);
                        set_loaded.set(false);
                    }
                />
            })}

            // Error state
            {move || error.get().then(|| view! {
                <div class="absolute inset-0 flex items-center justify-center bg-gray-100 dark:bg-gray-800">
                    <div class="text-center p-4">
                        <svg
                            class="w-12 h-12 mx-auto text-gray-400 dark:text-gray-600 mb-2"
                            fill="none"
                            viewBox="0 0 24 24"
                            stroke="currentColor"
                        >
                            <path
            stroke-linecap="round"
                                stroke-linejoin="round"
                                stroke-width="2"
                                d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z"
                            />
                        </svg>
                        <span class="text-sm text-gray-500 dark:text-gray-400">
                            "Failed to load image"
                        </span>
                    </div>
                </div>
            })}
        </div>
    }
}

/// Avatar component with optimized image loading
///
/// Specialized component for user avatars with fallback to initials
#[component]
pub fn Avatar(
    /// Avatar image URL (optional)
    #[prop(optional, into)]
    src: Option<String>,
    /// User name for fallback initials
    #[prop(into)]
    name: String,
    /// Size variant
    #[prop(default = AvatarSize::Medium)]
    size: AvatarSize,
    /// CSS class
    #[prop(optional, into)]
    class: Option<String>,
) -> impl IntoView {
    let class = class.unwrap_or_default();
    let (image_error, set_image_error) = signal(false);

    let size_class = match size {
        AvatarSize::Small => "w-8 h-8 text-xs",
        AvatarSize::Medium => "w-10 h-10 text-sm",
        AvatarSize::Large => "w-12 h-12 text-base",
        AvatarSize::XLarge => "w-16 h-16 text-lg",
    };

    // Get initials from name
    let initials = name
        .split_whitespace()
        .take(2)
        .map(|word| word.chars().next().unwrap_or(' '))
        .collect::<String>()
        .to_uppercase();

    view! {
        <div class=format!(
            "relative inline-flex items-center justify-center rounded-full overflow-hidden bg-primary text-white font-semibold {} {}",
            size_class,
            class
        )>
            {move || {
                if let Some(ref image_src) = src {
                    if !image_error.get() {
                        view! {
                            <img
                                src=image_src.clone()
                                alt=name.clone()
                                loading="lazy"
                                decoding="async"
                                class="w-full h-full object-cover"
                                on:error=move |_| set_image_error.set(true)
                            />
                        }.into_any()
                    } else {
                        view! {
                            <span>{initials.clone()}</span>
                        }.into_any()
                    }
                } else {
                    view! {
                        <span>{initials.clone()}</span>
                    }.into_any()
                }
            }}
        </div>
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AvatarSize {
    Small,
    Medium,
    Large,
    XLarge,
}

/// Preload critical images for better perceived performance
///
/// Use this to preload images that will be needed soon (e.g., on hover)
#[cfg(target_arch = "wasm32")]
pub fn preload_image(src: &str) {
    use wasm_bindgen::JsCast;

    if let Some(window) = web_sys::window()
        && let Some(document) = window.document()
        && let Ok(link) = document.create_element("link")
    {
        let link = link.dyn_into::<web_sys::HtmlLinkElement>().unwrap();
        link.set_rel("preload");
        link.set_as("image");
        link.set_href(src);

        let _ = document
            .head()
            .and_then(|head| head.append_child(&link).ok());
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn preload_image(_src: &str) {
    // No-op on server side
}

/// Check if WebP is supported by the browser
#[cfg(target_arch = "wasm32")]
pub fn is_webp_supported() -> bool {
    use wasm_bindgen::JsCast;

    if let Some(window) = web_sys::window()
        && let Some(document) = window.document()
        && let Ok(canvas) = document.create_element("canvas")
        && let Ok(canvas) = canvas.dyn_into::<web_sys::HtmlCanvasElement>()
        && let Ok(data_url) = canvas.to_data_url_with_type("image/webp")
    {
        return data_url.starts_with("data:image/webp");
    }

    false
}

#[cfg(not(target_arch = "wasm32"))]
pub fn is_webp_supported() -> bool {
    true // Assume supported on server side
}

/// Get optimized image URL based on browser capabilities
///
/// Returns WebP version if supported, otherwise returns original
pub fn get_optimized_image_url(src: &str) -> String {
    if is_webp_supported() {
        // Try to replace extension with .webp
        if let Some((base, _ext)) = src.rsplit_once('.') {
            format!("{}.webp", base)
        } else {
            src.to_string()
        }
    } else {
        src.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_initials() {
        let name = "John Doe";
        let initials: String = name
            .split_whitespace()
            .take(2)
            .map(|word| word.chars().next().unwrap_or(' '))
            .collect::<String>()
            .to_uppercase();

        assert_eq!(initials, "JD");
    }

    #[test]
    fn test_get_optimized_image_url() {
        let src = "/images/photo.jpg";
        let optimized = get_optimized_image_url(src);

        // On server side, WebP is assumed supported
        #[cfg(not(target_arch = "wasm32"))]
        assert_eq!(optimized, "/images/photo.webp");
    }
}
