# CSS Minification and Optimization Guide

This guide covers CSS optimization strategies for SIMPelv2 microfrontends.

## Overview

CSS optimization is crucial for reducing bundle sizes and improving load times. This guide covers:

1. **Tailwind CSS Optimization** - Using local builds instead of CDN
2. **PurgeCSS** - Removing unused CSS
3. **Minification** - Compressing CSS files
4. **Critical CSS** - Inlining critical styles

## Tailwind CSS Optimization

### Development vs Production

**Development (Current Setup)**
```html
<!-- Using CDN for fast development -->
<script src="https://cdn.tailwindcss.com"></script>
```

**Production (Recommended)**
```html
<!-- Using local optimized build -->
<link rel="stylesheet" href="/styles/main.css">
```

### Setting Up Local Tailwind Build

1. **Install Tailwind CSS**

```bash
# In your microfrontend directory
npm init -y
npm install -D tailwindcss postcss autoprefixer cssnano
```

2. **Create Tailwind Config**

Create `tailwind.config.js`:

```javascript
/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    "./src/**/*.rs",
    "./index.html",
    "../shared/src/**/*.rs",  // Include shared components
  ],
  darkMode: 'class',
  theme: {
    extend: {
      colors: {
        primary: {
          50: '#fef2f2',
          100: '#fee2e2',
          200: '#fecaca',
          300: '#fca5a5',
          400: '#f87171',
          500: '#ef4444',
          600: '#dc2626',
          700: '#b91c1c',
          800: '#991b1b',
          900: '#7f1d1d',
        },
        secondary: {
          50: '#eff6ff',
          100: '#dbeafe',
          200: '#bfdbfe',
          300: '#93c5fd',
          400: '#60a5fa',
          500: '#3b82f6',
          600: '#2563eb',
          700: '#1d4ed8',
          800: '#1e40af',
          900: '#1e3a8a',
        },
      },
      fontFamily: {
        sans: ['Inter', 'system-ui', 'sans-serif'],
      },
    },
  },
  plugins: [],
}
```

3. **Create PostCSS Config**

Create `postcss.config.js`:

```javascript
module.exports = {
  plugins: {
    tailwindcss: {},
    autoprefixer: {},
    ...(process.env.NODE_ENV === 'production' ? { cssnano: {} } : {})
  }
}
```

4. **Create Input CSS**

Create `styles/input.css`:

```css
@tailwind base;
@tailwind components;
@tailwind utilities;

/* Custom base styles */
@layer base {
  html {
    scroll-behavior: smooth;
  }

  body {
    @apply bg-gray-50 text-gray-900 dark:bg-gray-900 dark:text-gray-100;
  }
}

/* Custom components */
@layer components {
  .btn-primary {
    @apply px-4 py-2 bg-primary text-white rounded-lg hover:bg-primary-dark transition-colors;
  }

  .card {
    @apply bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700;
  }
}

/* Custom utilities */
@layer utilities {
  .text-balance {
    text-wrap: balance;
  }
}
```

5. **Add Build Scripts**

Update `package.json`:

```json
{
  "scripts": {
    "css:dev": "tailwindcss -i ./styles/input.css -o ./styles/main.css --watch",
    "css:build": "NODE_ENV=production tailwindcss -i ./styles/input.css -o ./styles/main.css --minify",
    "build": "npm run css:build && trunk build --release"
  }
}
```

6. **Update index.html**

```html
<head>
    <!-- Remove CDN -->
    <!-- <script src="https://cdn.tailwindcss.com"></script> -->

    <!-- Add local CSS -->
    <link rel="stylesheet" href="/styles/main.css">
</head>
```

## PurgeCSS Configuration

Tailwind CSS automatically purges unused styles when building for production. Ensure your `content` paths are correct:

```javascript
// tailwind.config.js
module.exports = {
  content: [
    "./src/**/*.rs",           // All Rust source files
    "./index.html",            // HTML template
    "../shared/src/**/*.rs",   // Shared components
  ],
  // ... rest of config
}
```

### Manual PurgeCSS (if needed)

If you need more control, use PurgeCSS directly:

```bash
npm install -D @fullhuman/postcss-purgecss
```

```javascript
// postcss.config.js
const purgecss = require('@fullhuman/postcss-purgecss')

module.exports = {
  plugins: [
    require('tailwindcss'),
    require('autoprefixer'),
    ...(process.env.NODE_ENV === 'production' ? [
      purgecss({
        content: [
          './src/**/*.rs',
          './index.html',
          '../shared/src/**/*.rs',
        ],
        defaultExtractor: content => content.match(/[\w-/:]+(?<!:)/g) || [],
        safelist: [
          /^animate-/,
          /^transition-/,
          /^duration-/,
          /^ease-/,
        ]
      }),
      require('cssnano')({
        preset: 'default',
      })
    ] : [])
  ]
}
```

## Critical CSS

Extract and inline critical CSS for faster First Contentful Paint:

### Using Critical Package

```bash
npm install -D critical
```

Create `scripts/extract-critical.js`:

```javascript
const critical = require('critical');

critical.generate({
  inline: true,
  base: 'dist/',
  src: 'index.html',
  target: {
    html: 'index.html',
    css: 'styles/critical.css'
  },
  width: 1300,
  height: 900,
  dimensions: [
    {
      height: 900,
      width: 1300,
    },
    {
      height: 720,
      width: 1280,
    },
    {
      height: 568,
      width: 320,
    },
  ],
});
```

Add to build script:

```json
{
  "scripts": {
    "critical": "node scripts/extract-critical.js",
    "build": "npm run css:build && trunk build --release && npm run critical"
  }
}
```

## CSS Compression

### Using cssnano

Already included in PostCSS config for production builds:

```javascript
// postcss.config.js
module.exports = {
  plugins: {
    tailwindcss: {},
    autoprefixer: {},
    ...(process.env.NODE_ENV === 'production' ? {
      cssnano: {
        preset: ['default', {
          discardComments: {
            removeAll: true,
          },
          normalizeWhitespace: true,
          colormin: true,
          minifyFontValues: true,
          minifyGradients: true,
        }]
      }
    } : {})
  }
}
```

### Gzip/Brotli Compression

Enable in Nginx configuration:

```nginx
# Enable gzip
gzip on;
gzip_vary on;
gzip_min_length 1024;
gzip_comp_level 6;
gzip_types
    text/css
    text/javascript
    application/javascript
    application/json;

# Enable brotli (if module available)
brotli on;
brotli_comp_level 6;
brotli_types
    text/css
    text/javascript
    application/javascript
    application/json;
```

## Font Optimization in CSS

### Font-Display Swap

```css
@font-face {
  font-family: 'Inter';
  font-style: normal;
  font-weight: 400;
  font-display: swap;  /* Show fallback immediately */
  src: local('Inter'),
       url('/fonts/inter-regular.woff2') format('woff2');
}
```

### Font Subsetting

Use `pyftsubset` to create subset fonts:

```bash
# Install fonttools
pip install fonttools brotli

# Create subset
pyftsubset inter-regular.ttf \
  --output-file=inter-regular-subset.woff2 \
  --flavor=woff2 \
  --text="ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789.,;:!?-–—()[]{}\"'/@#$%&*+=<>|~\`" \
  --layout-features="*" \
  --no-hinting
```

## Build Optimization Checklist

- [ ] Replace Tailwind CDN with local build
- [ ] Configure content paths for PurgeCSS
- [ ] Enable CSS minification with cssnano
- [ ] Extract and inline critical CSS
- [ ] Use font-display: swap for all fonts
- [ ] Subset fonts to reduce file size
- [ ] Enable gzip/brotli compression
- [ ] Set appropriate cache headers
- [ ] Test bundle size reduction
- [ ] Verify no visual regressions

## Measuring Results

### Before Optimization
```bash
# Check CSS size
ls -lh styles/main.css
# Example: 3.2MB (Tailwind CDN full build)
```

### After Optimization
```bash
# Build optimized CSS
npm rbuild

# Check optimized size
ls -lh styles/main.css
# Example: 12KB (purged and minified)

# Check gzipped size
gzip -c styles/main.css | wc -c
# Example: 3KB (gzipped)
```

### Performance Metrics

Monitor these metrics before and after optimization:

- **CSS Bundle Size**: Should reduce by 95%+ with PurgeCSS
- **First Contentful Paint (FCP)**: Should improve by 200-500ms
- **Largest Contentful Paint (LCP)**: Should improve by 300-800ms
- **Total Blocking Time (TBT)**: Should reduce with smaller CSS

## Automation

### GitHub Actions / GitLab CI

```yaml
# .gitlab-ci.yml
build-css:
  stage: build
  script:
    - npm install
    - npm run css:build
  artifacts:
    paths:
      - styles/main.css
    expire_in: 1 hour

build-wasm:
  stage: build
  dependencies:
    - build-css
  script:
    - trunk build --release
  artifacts:
    paths:
      - dist/
```

## Troubleshooting

### Issue: Styles not purging correctly

**Solution**: Check content paths in `tailwind.config.js`:

```javascript
content: [
  "./src/**/*.rs",           // ✅ Correct
  "./src/**/*.{rs,html}",    // ✅ Also works
  "./src/*.rs",              // ❌ Missing nested files
]
```

### Issue: Dynamic classes being purged

**Solution**: Add to safelist:

```javascript
safelist: [
  'bg-red-500',
  'text-blue-600',
  {
    pattern: /bg-(red|blue|green)-(400|500|600)/,
  },
]
```

### Issue: Large CSS file even after purging

**Solution**: Check for unused plugins or utilities:

```javascript
// Remove unused plugins
plugins: [
  // require('@tailwindcss/forms'),  // Remove if not used
  // require('@tailwindcss/typography'),  // Remove if not used
],
```

## Resources

- [Tailwind CSS Optimization](https://tailwindcss.com/docs/optimizing-for-production)
- [PurgeCSS Documentation](https://purgecss.com/)
- [cssnano Documentation](https://cssnano.co/)
- [Critical CSS Tools](https://github.com/addyosmani/critical)
- [Font Subsetting Guide](https://markoskon.com/creating-font-subsets/)

