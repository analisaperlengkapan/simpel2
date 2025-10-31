# Asset Optimization Guide

This guide covers best practices for optimizing asset loading in SIMPelv2 microfrontends.

## Overview

Asset optimization is critical for performance, especially for government applications that may be accessed over slower connections. This guide covers:

1. **Image Optimization** - Lazy loading, responsive images, WebP format
2. **Font Optimization** - Preloading, font-display, subsetting
3. **CSS/JS Minification** - Build-time optimization
4. **Critical Asset Preloading** - Prioritizing important resources

## Image Optimization

### Using OptimizedImage Component

The shared library provides an `OptimizedImage` component with built-in optimization features:

```rust
use shared_microfrontend::components::OptimizedImage;

#[component]
pub fn Gallery() -> impl IntoView {
    view! {
        <OptimizedImage
            src="/images/photo.jpg"
            alt="Beautiful landscape"
            lazy=true              // Enable lazy loading
            responsive=true        // Enable responsive images
            sizes=vec![320, 640, 960, 1280, 1920]  // Responsive breakpoints
            object_fit="cover"
        />
    }
}
```

### Features

- **Lazy Loading**: Images load only when they enter the viewport
- **Responsive Images**: Automatically generates srcset for different screen sizes
- **Loading Placeholder**: Shows animated placeholder while loading
- **Error Handling**: Displays fallback UI if image fails to load
- **WebP Support**: Automatically uses WebP format when supported

### Avatar Component

For user avatars with fallback to initials:

```rust
use shared_microfrontend::components::Avatar;

#[component]
pub fn UserProfile() -> impl IntoView {
    view! {
        <Avatar
            src=Some("/images/user.jpg".to_string())
            name="Ahmad Wijaya"
            size=AvatarSize::Large
        />
    }
}
```

### Preloading Critical Images

For images that need to load immediately (e.g., logos, hero images):

```rust
use shared_microfrontend::utils::preload_image;

#[component]
pub fn App() -> impl IntoView {
    // Preload critical images on mount
    #[cfg(target_arch = "wasm32")]
    {
        preload_image("/assets/logo.png");
        preload_image("/assets/hero-image.jpg");
    }

    view! {
        // Your app content
    }
}
```

### WebP Format Detection

Check if browser supports WebP and serve optimized format:

```rust
use shared_microfrontend::utils::{is_webp_supported, get_optimized_image_url};

let image_url = if is_webp_supported() {
    "/images/photo.webp"
} else {
    "/images/photo.jpg"
};

// Or use the helper function
let optimized_url = get_optimized_image_url("/images/photo.jpg");
// Returns "/images/photo.webp" if supported, otherwise "/images/photo.jpg"
```

## Font Optimization

### Font Loading Strategy

The shared library provides a comprehensive font loading strategy:

```rust
use shared_microfrontend::utils::FontLoadingStrategy;

#[component]
pub fn App() -> impl IntoView {
    // Initialize font optimization
    #[cfg(target_arch = "wasm32")]
    {
        let strategy = FontLoadingStrategy {
            use_swap: true,           // Use font-display: swap
            preload_critical: true,   // Preload critical fonts
            use_subset: true,         // Use font subsetting
            use_variable: false,      // Use variable fonts (not widely supported yet)
        };
        strategy.apply();
    }

    view! {
        // Your app content
    }
}
```

### Preloading Fonts

Preload critical fonts for better performance:

```rust
use shared_microfrontend::utils::{preload_font, FontType};

#[cfg(target_arch = "wasm32")]
{
    preload_font("/fonts/inter-regular.woff2", FontType::Woff2);
    preload_font("/fonts/inter-medium.woff2", FontType::Woff2);
    preload_font("/fonts/inter-semibold.woff2", FontType::Woff2);
}
```

### Font Display Swap

Use `font-display: swap` to show text immediately with fallback fonts:

```rust
use shared_microfrontend::utils::setup_font_display_swap;

#[cfg(target_arch = "wasm32")]
setup_font_display_swap();
```

### Waiting for Fonts to Load

Wait for fonts to load before showing content:

```rust
use shared_microfrontend::utils::wait_for_fonts_loaded;
use leptos::task::spawn_local;

spawn_local(async move {
    wait_for_fonts_loaded().await;
    // Fonts are now loaded, update UI if needed
});
```

### Font Subsetting

For production builds, subset fonts to include only used characters:

```rust
use shared_microfrontend::utils::get_font_subset_characters;

// Get the character set for Indonesian + common punctuation
let subset_chars = get_font_subset_characters();
// "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789.,;:!?-–—()[]{}\"'/@#$%&*+=<>|~`"
```

Use this character set with font subsetting tools like `pyftsubset`:

```bash
pyftsubset inter-regular.ttf \
    --output-file=inter-regular-subset.woff2 \
    --flavor=woff2 \
    --text="ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789.,;:!?-–—()[]{}\"'/@#$%&*+=<>|~\`" \
    --layout-features="*"
```

## HTML Preload Links

Add preload links in your `index.html` for critical assets:

```html
<head>
    <!-- Preload critical fonts -->
    <link rel="preload" href="/fonts/inter-regular.woff2" as="font" type="font/woff2" crossorigin="anonymous">
    <link rel="preload" href="/fonts/inter-medium.woff2" as="font" type="font/woff2" crossorigin="anonymous">

    <!-- Preload critical images -->
    <link rel="preload" href="/assets/logo.png" as="image">

    <!-- DNS prefetch for external resources -->
    <link rel="dns-prefetch" href="https://cdn.tailwindcss.com">
    <link rel="dns-prefetch" href="https://cdnjs.cloudflare.com">

    <!-- Preconnect for faster external resource loading -->
    <link rel="preconnect" href="https://fonts.googleapis.com">
    <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
</head>
```

## CSS/JS Minification

### Trunk Build Optimization

Configure Trunk for production builds in `Trunk.toml`:

```toml
[build]
target = "index.html"
release = true
dist = "dist"
public_url = "/"

[watch]
ignore = ["dist"]

[serve]
address = "127.0.0.1"
port = 8080

# Optimize WASM
[[hooks]]
stage = "post_build"
command = "wasm-opt"
command_arguments = [
    "-Oz",                          # Optimize for size
    "--enable-bulk-memory",
    "--enable-nontrapping-float-to-int",
    "--enable-sign-ext",
    "--enable-mutable-globals",
    "dist/*.wasm",
    "-o",
    "dist/*.wasm"
]
```

### CSS Optimization

For production, use a local Tailwind CSS build instead of CDN:

1. Install Tailwind CLI:
```bash
npm install -D tailwindcss
```

2. Create `tailwind.config.js`:
```javascript
module.exports = {
  content: [
    "./src/**/*.rs",
    "./index.html",
  ],
  darkMode: 'class',
  theme: {
    extend: {
      colors: {
        primary: {
          // Your color palette
        },
      },
    },
  },
  plugins: [],
}
```

3. Build minified CSS:
```bash
npx tailwindcss -i ./styles/input.css -o ./dist/styles/output.css --minify
```

4. Update `index.html`:
```html
<!-- Replace CDN with local build -->
<link rel="stylesheet" href="/styles/output.css">
```

### WASM Optimization

Optimize WASM bundle size:

```bash
# Build with release profile
trunk build --release

# Further optimize with wasm-opt
wasm-opt -Oz --enable-bulk-memory dist/*.wasm -o dist/*.wasm

# Check bundle size
ls -lh dist/*.wasm
```

### Compression

Enable gzip/brotli compression in Nginx:

```nginx
# Enable gzip compression
gzip on;
gzip_vary on;
gzip_min_length 1024;
gzip_types
    text/css
    text/javascript
    text/xml
    text/plain
    application/javascript
    application/x-javascript
    application/json
    application/xml
    application/rss+xml
    application/wasm
    font/woff
    font/woff2
    image/svg+xml;

# Enable brotli compression (if available)
brotli on;
brotli_types
    text/css
    text/javascript
    text/xml
    text/plain
    application/javascript
    application/x-javascript
    application/json
    application/xml
    application/rss+xml
    application/wasm
    font/woff
    font/woff2
    image/svg+xml;
```

## Caching Strategy

Configure caching headers in Nginx:

```nginx
location ~* \.(wasm|js|css)$ {
    expires 1y;
    add_header Cache-Control "public, immutable";
}

location ~* \.(jpg|jpeg|png|gif|ico|svg|webp)$ {
    expires 1y;
    add_header Cache-Control "public, immutable";
}

location ~* \.(woff|woff2|ttf|otf)$ {
    expires 1y;
    add_header Cache-Control "public, immutable";
    add_header Access-Control-Allow-Origin "*";
}

# HTML files should not be cached
location ~* \.html$ {
    expires -1;
    add_header Cache-Control "no-cache, no-store, must-revalidate";
}
```

## Performance Monitoring

Monitor asset loading performance:

```rust
use shared_microfrontend::utils::{track_performance, PerformanceMetric};

#[component]
pub fn App() -> impl IntoView {
    #[cfg(target_arch = "wasm32")]
    {
        use leptos::task::spawn_local;

        spawn_local(async move {
            // Track First Contentful Paint
            if let Some(fcp) = get_first_contentful_paint() {
                track_performance(PerformanceMetric::FirstContentfulPaint, fcp);
            }

            // Track Largest Contentful Paint
            if let Some(lcp) = get_largest_contentful_paint() {
                track_performance(PerformanceMetric::LargestContentfulPaint, lcp);
            }
        });
    }

    view! {
        // Your app content
    }
}
```

## Best Practices Summary

### Images
- ✅ Use `OptimizedImage` component for all images
- ✅ Enable lazy loading for below-the-fold images
- ✅ Use responsive images with srcset
- ✅ Preload critical images (logo, hero)
- ✅ Serve WebP format when supported
- ✅ Optimize image file sizes (compress, resize)

### Fonts
- ✅ Use `font-display: swap` for all fonts
- ✅ Preload critical font weights (regular, medium, semibold)
- ✅ Subset fonts to include only used characters
- ✅ Use WOFF2 format (best compression)
- ✅ Limit font weights (3-4 maximum)
- ✅ Use system fonts as fallback

### CSS/JS
- ✅ Minify CSS and JS in production
- ✅ Use local Tailwind build instead of CDN
- ✅ Enable tree-shaking for unused CSS
- ✅ Optimize WASM bundle with wasm-opt
- ✅ Enable gzip/brotli compression
- ✅ Set appropriate cache headers

### General
- ✅ Use DNS prefetch for external resources
- ✅ Preconnect to critical origins
- ✅ Implement proper caching strategy
- ✅ Monitor Core Web Vitals
- ✅ Test on slow 3G network
- ✅ Measure and optimize bundle sizes

## Checklist for New Microfrontends

When creating a new microfrontend, ensure:

- [ ] Add preload links for critical fonts in `index.html`
- [ ] Add DNS prefetch for external resources
- [ ] Initialize `FontLoadingStrategy` in app component
- [ ] Use `OptimizedImage` for all images
- [ ] Preload critical images (logo, hero)
- [ ] Configure Trunk for production optimization
- [ ] Set up CSS minification
- [ ] Enable WASM optimization
- [ ] Configure Nginx caching headers
- [ ] Test performance on slow network
- [ ] Monitor Core Web Vitals

## Resources

- [Web.dev - Optimize Web Fonts](https://web.dev/optimize-webfonts/)
- [Web.dev - Optimize Images](https://web.dev/fast/#optimize-your-images)
- [MDN - Preloading Content](https://developer.mozilla.org/en-US/docs/Web/HTML/Preloading_content)
- [Trunk Documentation](https://trunkrs.dev/)
- [wasm-opt Documentation](https://github.com/WebAssembly/binaryen)
