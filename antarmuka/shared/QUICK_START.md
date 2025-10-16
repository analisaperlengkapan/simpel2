# Quick Start: Using Refactored Shared Library

Panduan cepat untuk developer yang ingin menggunakan shared library di microfrontend.

## 🚀 Import (New Way)

### Simple - Just Import Everything

```rust
// In your microfrontend's lib.rs or component
use shared_microfrontend::prelude::*;
```

That's it! Sekarang semua komponen, utilities, dan types sudah available.

### Selective - Import What You Need

```rust
// Components
use shared_microfrontend::components::{
    Button, Card, Input, Modal, Toast,
    AppHeader, Breadcrumb, Table, Badge
};

// Utilities
use shared_microfrontend::utils::{
    validate_nik, validate_nip,
    format_currency, format_date
};

// Types
use shared_microfrontend::core::types::{
    ButtonVariant, ButtonSize, ToastType,
    NavItem, TableColumn, Pagination
};

// Theme
use shared_microfrontend::core::theme::{
    get_theme, set_theme, toggle_theme, ThemeMode
};
```

## 📦 Common Components

### Button

```rust
<Button
    variant=ButtonVariant::Primary
    size=ButtonSize::Medium
    loading=false
    on:click=move |_| { /* your handler */ }
>
    "Click Me"
</Button>
```

**Variants**: `Primary`, `Secondary`, `Danger`, `Success`, `Ghost`
**Sizes**: `Small`, `Medium`, `Large`

### Input

```rust
<Input
    id="username"
    name="username"
    label="Username"
    placeholder="Enter username"
    value=username_signal
    on_input=move |val| set_username.set(val)
    error=error_message
    required=true
/>
```

### Card

```rust
<Card title="Card Title">
    <p>"Your content here"</p>
</Card>
```

### Modal

```rust
<Modal
    title="Confirmation"
    show=show_modal
    on_close=Some(Box::new(move || set_show_modal.set(false)))
>
    <p>"Are you sure?"</p>
    <Button on:click=move |_| handle_confirm()>"Yes"</Button>
</Modal>
```

### Toast

```rust
<Toast
    toast_type=ToastType::Success
    message="Operation successful!"
    duration=3000
/>
```

**Types**: `Success`, `Error`, `Warning`, `Info`

### AppHeader

```rust
<AppHeader title="My App" subtitle="Kejaksaan RI">
    <Button variant=ButtonVariant::Ghost>"Logout"</Button>
</AppHeader>
```

### Table

```rust
<Table
    columns=vec![
        TableColumn::new("id", "ID"),
        TableColumn::new("name", "Name").sortable(),
    ]
    data=table_data
    striped=true
    hoverable=true
/>
```

### Pagination

```rust
<Pagination
    pagination=Pagination {
        current_page: 1,
        page_size: 20,
        total_items: 100,
        total_pages: 5,
    }
    on_page_change=Some(Box::new(move |page| {
        set_current_page.set(page);
    }))
/>
```

## 🛠️ Indonesian Utilities

### Validation

```rust
use shared_microfrontend::utils::*;

// NIK validation (16 digits)
let result = validate_nik("3201234567890123");
if result.is_valid() {
    // OK
} else {
    eprintln!("Errors: {:?}", result.errors);
}

// NIP validation (18 digits)
let result = validate_nip("199001012020121001");

// Phone validation
let result = validate_phone("08123456789"); // or "+6281234567890"

// Email validation
let result = validate_email("user@example.com");
```

### Formatting

```rust
use shared_microfrontend::utils::*;

// Currency: "Rp 1.000.000"
let formatted = format_currency(1000000);

// Number: "1.000.000"
let formatted = format_number(1000000);

// Date: "25/10/2025"
let formatted = format_date(Local::now());

// Relative time: "5 menit yang lalu"
let formatted = format_relative_time(datetime);

// File size: "1.50 MB"
let formatted = format_file_size(1572864);
```

## 🎨 Theming

```rust
use shared_microfrontend::core::theme::*;

// Get current theme
let current = get_theme();

// Set theme
set_theme(ThemeMode::Dark);

// Toggle theme (light ↔ dark)
toggle_theme();

// Check system preference
if prefers_dark_mode() {
    set_theme(ThemeMode::Dark);
}
```

## 📱 Responsive Utilities

```rust
use shared_microfrontend::hooks::*;

// Simple boolean checks
if is_mobile() {
    // Show mobile layout
}

if is_desktop() {
    // Show desktop layout
}

// Custom media query
if matches_media_query("(min-width: 1024px)") {
    // Large screen
}
```

## 💾 Storage Utilities

```rust
use shared_microfrontend::hooks::*;

// Save to localStorage
save_to_storage("user_settings", &settings);

// Load from localStorage
if let Some(settings) = load_from_storage::<UserSettings>("user_settings") {
    // Use settings
}

// Remove from localStorage
remove_from_storage("user_settings");
```

## ⏱️ Debounce Utility

```rust
use shared_microfrontend::hooks::*;

// Create debounced function
let on_search_debounced = create_debounced(
    move |query: String| {
        // Your search logic here
        fetch_results(query);
    },
    300 // milliseconds
);

// Use in component
<Input
    on_input=move |val| on_search_debounced(val)
    placeholder="Search..."
/>
```

## 🎯 Navigation

### Breadcrumb

```rust
use shared_microfrontend::core::types::BreadcrumbItem;

let items = vec![
    BreadcrumbItem::new("Home", Some("/".to_string())),
    BreadcrumbItem::new("Users", Some("/users".to_string())),
    BreadcrumbItem::new("Profile", None), // Current page
];

<Breadcrumb items=items />
```

### NavMenu

```rust
use shared_microfrontend::core::types::NavItem;

let menu_items = vec![
    NavItem::new("dashboard", "Dashboard", "/")
        .with_icon("📊"),
    NavItem::new("users", "Users", "/users")
        .with_icon("👥")
        .with_badge("5"),
];

<NavMenu items=menu_items />
```

## 📄 Display Components

### Badge

```rust
<Badge variant=BadgeVariant::Success>"Active"</Badge>
<Badge variant=BadgeVariant::Danger>"Blocked"</Badge>
```

**Variants**: `Primary`, `Secondary`, `Success`, `Danger`, `Warning`, `Info`

### List

```rust
<List
    items=vec!["Item 1", "Item 2", "Item 3"]
    render=|item| view! { <li>{item}</li> }
/>
```

### EmptyState

```rust
<EmptyState
    title="No Data"
    message="There are no items to display"
    icon="📭"
/>
```

### Avatar

```rust
<Avatar
    name="John Doe"
    src=Some("https://example.com/avatar.jpg")
    size="md"
/>
```

## 🔥 Tips & Tricks

### 1. Use Prelude for Quick Prototyping

```rust
use shared_microfrontend::prelude::*;
// Everything available, start coding!
```

### 2. Selective Imports for Production

```rust
// Only import what you use (better compile times)
use shared_microfrontend::components::{Button, Card};
```

### 3. Type Inference Works

```rust
// No need to specify ButtonVariant type
<Button variant=Primary>"Click"</Button>
```

### 4. Combine with Leptos Signals

```rust
let (show_modal, set_show_modal) = signal(false);

<Modal show=show_modal.get() on_close=...>
    {/* content */}
</Modal>
```

### 5. Indonesian Validation in Forms

```rust
let (nik, set_nik) = signal(String::new());
let nik_error = move || {
    let result = validate_nik(&nik.get());
    if !result.is_valid() {
        Some(result.errors.join(", "))
    } else {
        None
    }
};

<Input
    label="NIK"
    value=nik
    error=nik_error()
    on_input=move |val| set_nik.set(val)
/>
```

## 🚨 Common Mistakes

### ❌ Don't use removed components

```rust
// These are REMOVED, don't use:
<NetworkStatusIndicator />  // ❌
<InstallPrompt />            // ❌
<VirtualScroll />            // ❌
```

### ❌ Don't use old enum values

```rust
// Old (REMOVED)
variant=ButtonVariant::Outline  // ❌

// New (USE THIS)
variant=ButtonVariant::Ghost    // ✅
```

### ❌ Don't self-close AppHeader

```rust
// Old (ERROR)
<AppHeader title="..." />  // ❌

// New (CORRECT)
<AppHeader title="...">    // ✅
    <div></div>
</AppHeader>
```

## 📚 More Info

- Full documentation: `/antarmuka/shared/README.md`
- Migration guide: `/antarmuka/shared/MIGRATION_GUIDE.md`
- Refactor report: `/antarmuka/shared/REFACTOR_SUCCESS.md`

---

**Happy Coding! 🚀**
