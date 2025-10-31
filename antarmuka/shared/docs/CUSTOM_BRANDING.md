# Custom Branding Guide

## Overview

The Custom Branding system allows each unit kerja (work unit) in Kejaksaan RI to customize their application's visual identity while maintaining accessibility standards and brand consistency.

## Features

### 1. **Custom Logos**
- Support for light and dark mode logos
- Automatic fallback to unit initial if logo not available
- Responsive sizing (Small, Medium, Large, ExtraLarge)

### 2. **Custom Color Schemes**
- Primary, Secondary, and Accent colors
- Real-time preview of changes
- Automatic WCAG 2.1 AA contrast validation

### 3. **Unit-Specific Branding**
- Pre-configured branding for all major units:
  - Badiklat (Training)
  - Datun (Civil & State Administration)
  - Intel (Intelligence)
  - Pidum (General Criminal)
  - Pidsus (Special Criminal)
  - Pidmil (Military Criminal)
  - Pengawasan (Supervision)
  - Pemulihan Aset (Asset Recovery)
  - Pembinaan (Development)

### 4. **Accessibility Validation**
- Automatic contrast ratio checking (4.5:1 minimum)
- Prevents saving of non-compliant color schemes
- Clear error messages for accessibility issues

## Usage

### Basic Setup

Wrap your application with `BrandingProvider`:

```rust
use shared_microfrontend::components::BrandingProvider;

#[component]
pub fn App() -> impl IntoView {
    view! {
        <BrandingProvider unit="badiklat".to_string()>
            // Your app content
        </BrandingProvider>
    }
}
```

### Using the Branded Logo

```rust
use shared_microfrontend::components::{BrandedLogo, BrandedLogoSize};

#[component]
pub fn Header() -> impl IntoView {
    view! {
        <BrandedLogo
            size=BrandedLogoSize::Medium
            class="my-custom-class".to_string()
        />
    }
}
```

### Accessing Branding Configuration

```rust
use shared_microfrontend::components::use_branding;

#[component]
pub fn MyComponent() -> impl IntoView {
    let (branding, set_branding) = use_branding();

    view! {
        <div>
            <h1>{move || branding.get().unit_name}</h1>
            <p>{move || branding.get().tagline.clone().unwrap_or_default()}</p>
        </div>
    }
}
```

### Branding Editor

Add the branding editor to your settings page:

```rust
use shared_microfrontend::components::BrandingEditor;

#[component]
pub fn SettingsPage() -> impl IntoView {
    let (show_editor, set_show_editor) = signal(false);

    view! {
        <button on:click=move |_| set_show_editor.set(true)>
            "Edit Branding"
        </button>

        <Show when=move || show_editor.get()>
            <BrandingEditor
                on_close=Callback::new(move |_| set_show_editor.set(false))
            />
        </Show>
    }
}
```

## Configuration Structure

```rust
pub struct BrandingConfig {
    pub unit_name: String,              // e.g., "Badiklat Kejaksaan RI"
    pub logo_url: Option<String>,       // Light mode logo
    pub logo_dark_url: Option<String>,  // Dark mode logo
    pub primary_color: String,          // Hex color (e.g., "#DC2626")
    pub secondary_color: String,        // Hex color
    pub accent_color: String,           // Hex color
    pub tagline: Option<String>,        // Optional tagline
}
```

## Storage

Branding configurations are stored in localStorage with the key pattern:
```
simpelv2_branding_{unit_name}
```

This allows each unit to maintain its own branding independently.

## Accessibility Compliance

The system automatically validates color contrast ratios according to WCAG 2.1 AA standards:

- **Minimum contrast ratio**: 4.5:1 for normal text
- **Validation**: Performed before saving
- **Error reporting**: Clear messages indicating which colors fail validation

### Example Validation Error
imary color #FFFF00 does not have sufficient contrast with white text
(WCAG AA requires 4.5:1)
```

## Best Practices

1. **Logo Files**
   - Use SVG format for scalability
   - Provide both
nd dark mode versions
   - Keep file sizes under 50KB

2. **Color Selection**
   - Test colors in both light and dark modes
   - Ensure sufficient contrast for readability
   - Maintain brand consistency across units

3. **Taglines**
   - Keep taglines concise (under 50 characters)
   - Use official unit descriptions
   - Avoid special characters

4. **Testing**
   - Test branding on multiple screen sizes
   - Verify logo visibility in both themes
   - Check color contrast with automated tools

## Integration with Portal

The portal automatically applies branding:

1. **Navbar**: Displays branded logo
2. **Settings Page**: Provides branding editor
3. **Document Title**: Updates with unit name
4. **CSS Variables**: Applies custom colors globally

## API Reference

### Components

- `BrandingProvider` - Context provider for branding
- `BrandedLogo` - Logo component with theme support
- `BrandingEditor` - Interactive branding editor

### Hooks

- `use_branding()` - Access branding configuration

### Types

- `BrandingConfig` - Branding configuration structure
- `BrandedLogoSize` - Logo size variants (Small, Medium, Large, ExtraLarge)

### Functions

- `BrandingConfig::for_unit(unit: &str)` - Get default config for unit
- `BrandingConfig::apply()` - Apply branding to document
- `BrandingConfig::validate_contrast()` - Validate color accessibility

## Troubleshooting

### Logo Not Displaying

1. Check logo URL is correct and accessible
2. Verify logo file exists in public assets
3. Check browser console for loading errors

### Colors Not Applying

1. Ensure BrandingProvider wraps your app
2. Check localStorage for saved configuration
3. Verify CSS custom properties are supported

### Validation Errors

1. Use a color contrast checker tool
2. Adjust colors to meet 4.5:1 ratio
3. Consider using darker shades for better contrast

## Examples

### Complete Portal Integration

```rust
// app.rs
use shared_microfrontend::components::BrandingProvider;

#[component]
pub fn App() -> impl IntoView {
    view! {
        <BrandingProvider unit="portal".to_string()>
            <Router>
                // Routes
            </Router>
        </BrandingProvider>
    }
}

// settings.rs
use shared_microfrontend::components::BrandingEditor;

#[component]
pub fn SettingsPage() -> impl IntoView {
    let (show_branding_editor, set_show_branding_editor) = signal(false);

    view! {
        <button on:click=move |_| set_show_branding_editor.set(true)>
            "Custom Branding"
        </button>

        <Show when=move || show_branding_editor.get()>
            <BrandingEditor
                on_close=Callback::new(move |_| set_show_branding_editor.set(false))
            />
        </Show>
    }
}

// navbar.rs
use shared_microfrontend::components::{BrandedLogo, BrandedLogoSize};

#[component]
pub fn Navbar() -> impl IntoView {
    view! {
        <nav>
            <BrandedLogo
                size=BrandedLogoSize::Medium
                class="navbar-logo".to_string()
            />
        </nav>
    }
}
```

## Support

For issues or questions about custom branding:
1. Check this documentation
2. Review the component source code
3. Contact the SIMPelv2 development team
