# Microfrontend Asset Optimization Integration Guide

This guide shows how to integrate asset optimization into existing and new microfrontends.

## Quick Start

### For Existing Microfrontends

Follow these steps to add asset optimization to an existing microfrontend:

#### 1. Update index.html

Add preload links for critical assets:

```html
<head>
    <!-- Existing meta tags -->
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">

    <!-- ADD: Preload critical fonts -->
    <link rel="preload" href="/fonts/inter-regular.woff2" as="font" type="font/woff2" crossorigin="anonymous">
    <link rel="preload" href="/fonts/inter-medium.woff2" as="font" type="font/woff2" crossorigin="anonymous">
    <link rel="preload" href="/fonts/inter-semibold.woff2" as="font" type="font/woff2" crossorigin="anonymous">

    <!-- ADD: Preload critical images -->
    <link rel="preload" href="/favicon.png" as="image">
    <link rel="preload" href="/assets/logo.png" as="image">

    <!-- ADD: DNS prefetch for external resources -->
    <link rel="dns-prefetch" href="https://cdn.tailwindcss.com">
    <link rel="dns-prefetch" href="https://cdnjs.cloudflare.com">

    <!-- ADD: Font-face declarations with font-display: swap -->
    <style>
        @font-face {
            font-family: 'Inter';
            font-style: normal;
            font-weight: 400;
            font-display: swap;
            src: local('Inter'), url('/fonts/inter-regular.woff2') format('woff2');
        }

        @font-face {
            font-family: 'Inter';
            font-style: normal;
            font-weight: 500;
            font-display: swap;
            src: local('Inter Medium'), url('/fonts/inter-medium.woff2') format('woff2');
        }

        @font-face {
            font-family: 'Inter';
            font-style: normal;
            font-weight: 600;
            font-display: swap;
            src: local('Inter SemiBold'), url('/fonts/inter-semibold.woff2') format('woff2');
        }

        body {
            font-family: 'Inter', -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
        }
    </style>

    <!-- Existing Tailwind and other resources -->
</head>
```

#### 2. Initialize Asset Optimization in App Component

Update your main app component (usually `src/app.rs` or `src/lib.rs`):

```rust
use leptos::prelude::*;
use shared_microfrontend::utils::{FontLoadingStrategy, preload_image};

#[component]
pub fn App() -> impl IntoView {
    // Initialize asset optimization
    #[cfg(target_arch = "wasm32")]
    {
        use leptos::prelude::Effect;

        Effect::new(move |_| {
            // Apply font loading strategy
            let font_strategy = FontLoadingStrategy::default();
            font_strategy.apply();

            // Preload critical images
            preload_image("/assets/logo.png");
            preload_image("/favicon.png");
        });
    }

    view! {
        // Your app content
    }
}
```

#### 3. Replace Image Tags with OptimizedImage

Find all `<img>` tags and replace with `OptimizedImage`:

**Before:**
```rust
view! {
    <img src="/images/document.jpg" alt="Document" class="w-full" />
}
```

**After:**
```rust
use shared_microfrontend::components::OptimizedImage;

view! {
    <OptimizedImage
        src="/images/document.jpg"
        alt="Document"
        lazy=true
        responsive=true
        class="w-full"
    />
}
```

#### 4. Update Trunk Configuration

Copy the optimized Trunk template:

```bash
cp ../shared/trunk-template-optimized.toml ./Trunk.toml
```

Or add these optimizations to your existing `Trunk.toml`:

```toml
[build]
release = true
minify = "on_release"

# WASM optimization hook
[[hooks]]
stage = "post_build"
command = "sh"
command_arguments = [
    "-c",
    "if command -v wasm-opt > /dev/null; then wasm-opt -Oz --enable-bulk-memory --enable-nontrapping-float-to-int --enable-sign-ext dist/*.wasm -o dist/*.wasm; fi"
]
```

### For New Microfrontends

When creating a new microfrontend, use the portal as a template:

```bash
# Copy portal structure
cp -r antarmuka/portal antarmuka/my-new-app

# Update Cargo.toml
# Update index.html with your app name
# Update src/lib.rs with your app name
```

The portal already includes all asset optimizations.

## Detailed Integration Steps

### Step 1: Font Optimization

#### A. Add Font Files

Ensure you have the font files in your microfrontend:

```
my-app/
├── fonts/
│   ├── inter-regular.woff2
│   ├── inter-medium.woff2
│   ├── inter-semibold.woff2
│   └── inter-bold.woff2
```

If you don't have them, copy from the portal:

```bash
cp -r antarmuka/portal/fonts ./fonts
```

#### B. Configure Trunk to Copy Fonts

In `Trunk.toml`:

```toml
[[copy-dir]]
from = "fonts"
to = "fonts"
```

#### C. Initialize Font Loading

In your app component:

```rust
use shared_microfrontend::utils::{FontLoadingStrategy, preload_font, FontType};

#[component]
pub fn App() -> impl IntoView {
    #[cfg(target_arch = "wasm32")]
    {
        use leptos::prelude::Effect;

        Effect::new(move |_| {
            // Option 1: Use default strategy
            let strategy = FontLoadingStrategy::default();
            strategy.apply();

            // Option 2: Custom strategy
            let strategy = FontLoadingStrategy {
                use_swap: true,
                preload_critical: true,
                use_subset: true,
                use_variable: false,
            };
            strategy.apply();

            // Option 3: Manual preloading
            preload_font("/fonts/inter-regular.woff2", FontType::Woff2);
            preload_font("/fonts/inter-medium.woff2", FontType::Woff2);
        });
    }

    view! {
        // Your app
    }
}
```

### Step 2: Image Optimization

#### A. Replace Static Images

**Before:**
```rust
view! {
    <img src="/images/photo.jpg" alt="Photo" />
}
```

**After:**
```rust
use shared_microfrontend::components::OptimizedImage;

view! {
    <OptimizedImage
        src="/images/photo.jpg"
        alt="Photo"
        lazy=true
    />
}
```

#### B. Use Responsive Images

For images that need different sizes:

```rust
view! {
    <OptimizedImage
        src="/images/hero.jpg"
        alt="Hero banner"
        lazy=false  // Don't lazy load above-the-fold images
        responsive=true
        sizes=vec![320, 640, 960, 1280, 1920]
        class="w-full h-96"
        object_fit="cover"
    />
}
```

#### C. Use Avatar Component

For user avatars:

```rust
use shared_microfrontend::components::{Avatar, AvatarSize};

view! {
    <Avatar
        src=Some("/images/user.jpg".to_string())
        name="Ahmad Wijaya"
        size=AvatarSize::Medium
    />
}
```

#### D. Preload Critical Images

For images that must load immediately:

```rust
use shared_microfrontend::utils::preload_image;

#[component]
pub fn App() -> impl IntoView {
    #[cfg(target_arch = "wasm32")]
    {
        use leptos::prelude::Effect;

        Effect::new(move |_| {
            preload_image("/assets/logo.png");
            preload_image("/assets/hero.jpg");
        });
    }

    view! {
        // Your app
    }
}
```

### Step 3: CSS Optimization

#### A. Setup Local Tailwind Build

```bash
# In your microfrontend directory
npm init -y
npm install -D tailwindcss postcss autoprefixer cssnano
```

#### B. Create Tailwind Config

Create `tailwind.config.js`:

```javascript
module.exports = {
  content: [
    "./src/**/*.rs",
    "./index.html",
    "../shared/src/**/*.rs",
  ],
  darkMode: 'class',
  theme: {
    extend: {
      colors: {
        primary: {
          600: '#dc2626',
          700: '#b91c1c',
        },
      },
    },
  },
  plugins: [],
}
```

#### C. Create Input CSS

Create `styles/input.css`:

```css
@tailwind base;
@tailwind components;
@tailwind utilities;
```

#### D. Add Build Scripts

In `package.json`:

```json
{
  "scripts": {
    "css:dev": "tailwindcss -i ./styles/input.css -o ./styles/main.css --watch",
    "css:build": "NODE_ENV=production tailwindcss -i ./styles/input.css -o ./styles/main.css --minify"
  }
}
```

#### E. Update index.html

Replace CDN with local build:

```html
<!-- Remove -->
<!-- <script src="https://cdn.tailwindcss.com"></script> -->

<!-- Add -->
<link rel="stylesheet" href="/styles/main.css">
```

### Step 4: WASM Optimization

#### A. Install wasm-opt

```bash
# On macOS
brew install binaryen

# On Ubuntu/Debian
sudo apt-get install binaryen

# On Windows
# Download from https://github.com/WebAssembly/binaryen/releases
```

#### B. Configure Trunk Hook

In `Trunk.toml`:

```toml
[[hooks]]
stage = "post_build"
command = "sh"
command_arguments = [
    "-c",
    "if command -v wasm-opt > /dev/null; then wasm-opt -Oz --enable-bulk-memory --enable-nontrapping-float-to-int --enable-sign-ext dist/*.wasm -o dist/*.wasm; fi"
]
```

#### C. Build and Verify

```bash
# Build with optimization
trunk build --release

# Check WASM size
ls -lh dist/*.wasm

# Should see significant size reduction
```

## Testing Optimizations

### 1. Measure Bundle Sizes

```bash
# Before optimization
trunk build --release
ls -lh dist/

# After optimization
npm run css:build
trunk build --release
ls -lh dist/

# Compare sizes
```

### 2. Test Performance

Use browser DevTools:

1. Open DevTools (F12)
2. Go to Network tab
3. Disable cache
4. Reload page
5. Check:
   - Total transfer size
   - Number of requests
   - Load time
   - First Contentful Paint
   - Largest Contentful Paint

### 3. Test on Slow Network

1. Open DevTools
2. Go to Network tab
3. Select "Slow 3G" throttling
4. Reload page
5. Verify:
   - Images lazy load correctly
   - Fonts display with fallback
   - Critical content loads first

### 4. Verify Lazy Loading

1. Open page
2. Scroll slowly
3. Watch Network tab
4. Verify images load only when scrolling into view

## Common Issues and Solutions

### Issue: Fonts not loading

**Solution**: Check CORS headers and crossorigin attribute:

```html
<link rel="preload" href="/fonts/inter-regular.woff2" as="font" type="font/woff2" crossorigin="anonymous">
```

### Issue: Images not lazy loading

**Solution**: Ensure `lazy=true` is set:

```rust
<OptimizedImage
    src="/images/photo.jpg"
    alt="Photo"
    lazy=true  // ← Make sure this is true
/>
```

### Issue: Large WASM bundle

**Solution**:
1. Check wasm-opt is installed
2. Verify Trunk hook is configured
3. Build with `--release` flag

### Issue: CSS not purging

**Solution**: Check content paths in `tailwind.config.js`:

```javascript
content: [
  "./src/**/*.rs",           // All Rust files
  "../shared/src/**/*.rs",   // Shared components
]
```

## Checklist

Use this checklist when integrating asset optimization:

- [ ] Add font preload links to index.html
- [ ] Add font-face declarations with font-display: swap
- [ ] Add DNS prefetch for external resources
- [ ] Initialize FontLoadingStrategy in app component
- [ ] Preload critical images
- [ ] Replace all `<img>` with `OptimizedImage`
- [ ] Use `Avatar` component for user avatars
- [ ] Setup local Tailwind build
- [ ] Configure PurgeCSS content paths
- [ ] Add CSS minification
- [ ] Configure wasm-opt hook in Trunk.toml
- [ ] Test on slow network
- [ ] Verify lazy loading works
- [ ] Measure bundle size reduction
- [ ] Check Core Web Vitals

## Example: Complete Integration

Here's a complete example of an optimized microfrontend:

```rust
// src/lib.rs
use leptos::prelude::*;
use shared_microfrontend::components::{OptimizedImage, Avatar, AvatarSize};
use shared_microfrontend::utils::{FontLoadingStrategy, preload_image};

#[component]
pub fn App() -> impl IntoView {
    // Initialize asset optimization
    #[cfg(target_arch = "wasm32")]
    {
        use leptos::prelude::Effect;

        Effect::new(move |_| {
            // Apply font loading strategy
            let font_strategy = FontLoadingStrategy::default();
            font_strategy.apply();

            // Preload critical images
            preload_image("/assets/logo.png");
            preload_image("/assets/hero.jpg");
        });
    }

    view! {
        <div class="min-h-screen bg-gray-50">
            // Header with logo
            <header class="bg-white shadow">
                <div class="container mx-auto px-4 py-4">
                    <img src="/assets/logo.png" alt="Logo" class="h-12" />
                </div>
            </header>

            // Hero section (not lazy loaded)
            <section class="relative">
                <OptimizedImage
                    src="/assets/hero.jpg"
                    alt="Hero banner"
                    lazy=false
                    responsive=true
                    class="w-full h-96"
                    object_fit="cover"
                />
            </section>

            // Content with lazy-loaded images
            <section class="container mx-auto px-4 py-12">
                <div class="grid grid-cols-3 gap-6">
                    <OptimizedImage
                        src="/images/photo1.jpg"
                        alt="Photo 1"
                        lazy=true
                        responsive=true
                        class="rounded-lg"
                    />
                    <OptimizedImage
                        src="/images/photo2.jpg"
                        alt="Photo 2"
                        lazy=true
                        responsive=true
                        class="rounded-lg"
                    />
                    <OptimizedImage
                        src="/images/photo3.jpg"
                        alt="Photo 3"
                        lazy=true
                        responsive=true
                        class="rounded-lg"
                    />
                </div>
            </section>

            // User profile with avatar
            <section class="container mx-auto px-4 py-12">
                <div class="flex items-center space-x-4">
                    <Avatar
                        src=Some("/images/user.jpg".to_string())
                        name="Ahmad Wijaya"
                        size=AvatarSize::Large
                    />
                    <div>
                        <h3 class="font-semibold">"Ahmad Wijaya"</h3>
                        <p class="text-sm text-gray-600">"Jaksa Agung Muda"</p>
                    </div>
                </div>
            </section>
        </div>
    }
}
```

## Resources

- [Asset Optimization Guide](./ASSET_OPTIMIZATION.md)
- [CSS Minification Guide](./CSS_MINIFICATION.md)
- [OptimizedImage Examples](../examples/optimized_image_example.rs)
- [Trunk Documentation](https://trunkrs.dev/)
- [Tailwind CSS Optimization](https://tailwindcss.com/docs/optimizing-for-production)
