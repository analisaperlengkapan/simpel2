# Custom Branding Quick Start Guide

## 🚀 Quick Start (5 Minutes)

### For End Users

#### Step 1: Access Settings
1. Log in to Portal SIMPelv2
2. Click the ⚙️ Settings icon in the top navigation bar
3. Or navigate to `/settings`

#### Step 2: Open Branding Editor
1. Find the "Tampilan" (Appearance) section
2. Click the "Custom Branding" button (🏢 icon)
3. The branding editor modal will open

#### Step 3: Customize Your Branding
1. **Unit Name**: Enter your unit's official name
2. **Tagline**: Add an optional tagline or description
3. **Logo URLs**:
   - Light mode logo: `/assets/logo-light.svg`
   - Dark mode logo: `/assets/logo-dark.svg`
4. **Colors**: Use color pickers to select:
   - Primary color (main brand color)
   - Secondary color (supporting color)
   - Accent color (highlights and CTAs)

#### Step 4: Preview & Save
1. Check the preview at the bottom of the editor
2. If you see red error messages, adjust colors for better contrast
3. Click "Save Branding" to apply changes
4. Your branding is now active!

### For Developers

#### Quick Integration (3 Steps)

**Step 1: Wrap Your App**
```rust
use shared_microfrontend::components::BrandingProvider;

#[component]
pub fn App() -> impl IntoView {
    view! {
        <BrandingProvider unit="your-unit".to_string()>
            {/* Your app content */}
        </BrandingProvider>
    }
}
```

**Step 2: Use Branded Logo**
```rust
use shared_microfrontend::components::{BrandedLogo, BrandedLogoSize};

#[component]
pub fn Header() -> impl IntoView {
    view! {
        <BrandedLogo size=BrandedLogoSize::Medium />
    }
}
```

**Step 3: Add Editor (Optional)**
```rust
use shared_microfrontend::components::BrandingEditor;

#[component]
pub fn Settings() -> impl IntoView {
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

## 🎨 Color Selection Tips

### Good Color Combinations (WCAG AA Compliant)

✅ **Government Red & Blue**
- Primary: `#DC2626` (Red)
- Secondary: `#1E40AF` (Blue)
- Accent: `#F59E0B` (Gold)

✅ **Professional Dark**
- Primary: `#1F2937` (Dark Gray)
- Secondary: `#3B82F6` (Blue)
- Accent: `#10B981` (Green)

✅ **Modern Purple**
- Primary: `#7C3AED` (Purple)
- Secondary: `#EC4899` (Pink)
- Accent: `#F59E0B` (Orange)

### Colors to Avoid

❌ **Too Light** (Poor contrast with white text)
- `#FFFF00` (Yellow)
- `#00FFFF` (Cyan)
- `#FF69B4` (Hot Pink)

❌ **Too Similar** (Hard to distinguish)
- Primary: `#DC2626`, Secondary: `#EF4444` (Both red)

## 🖼️ Logo Guidelines

### Recommended Formats
- ✅ SVG (best for scalability)
- ✅ PNG with transparency
- ⚠️ JPG (not recommended, no transparency)

### Size Recommendations
- Small (Navbar): 32x32px minimum
- Medium (Header): 48x48px minimum
- Large (Landing): 64x64px minimum
- Extra Large (Hero): 96x96px minimum

### File Size
- Keep under 50KB for fast loading
- Optimize SVGs with SVGO
- Compress PNGs with TinyPNG

## 🔍 Troubleshooting

### Logo Not Showing?
1. Check URL is correct: `/assets/logo.svg`
2. Verify file exists in public folder
3. Check browser console for 404 errors
4. Try absolute URL: `https://your-domain.com/logo.svg`

### Colors Not Applying?
1. Clear browser cache (Ctrl+Shift+R)
2. Check localStorage: `simpelv2_branding_your-unit`
3. Verify BrandingProvider wraps your app
4. Check browser console for errors

### Validation Errors?
1. Use darker colors for better contrast
2. Test with online contrast checker
3. Aim for 4.5:1 ratio minimum
4. Try suggested color combinations above

## 📱 Mobile Considerations

### Logo Sizing
- Use Small size for mobile navbars
- Hide tagline on small screens
- Ensure logo is readable at 32px

### Color Visibility
- Test in bright sunlight
- Check both light and dark modes
- Ensure sufficient contrast

## 🎯 Best Practices

### DO ✅
- Use official unit logos
- Test in both light and dark modes
- Keep taglines concise (< 50 chars)
- Use high-contrast colors
- Provide both logo variants

### DON'T ❌
- Use copyrighted images
- Choose low-contrast colors
- Make logos too complex
- Forget to test on mobile
- Skip accessibility validation

## 🔗 Quick Links

- [Full Documentation](./CUSTOM_BRANDING.md)
- [Implementation Details](./CUSTOM_BRANDING_IMPLEMENTATION.md)
- [Component Reference](../README.md)
- [Accessibility Guidelines](https://www.w3.org/WAI/WCAG21/quickref/)

## 💡 Pro Tips

1. **Save Time**: Use pre-configured unit defaults
2. **Consistency**: Match your unit's official branding
3. **Accessibility**: Always validate colors before saving
4. **Testing**: Check in multiple browsers and devices
5. **Backup**: Export your configuration (coming soon)

## 🆘 Need Help?

1. Check the [Full Documentation](./CUSTOM_BRANDING.md)
2. Review [Troubleshooting](#troubleshooting) section
3. Contact SIMPelv2 development team
4. Submit an issue on GitLab

---

**Last Updated**: October 2025
**Version**: 1.0.0
**Status**: Production Ready ✅
