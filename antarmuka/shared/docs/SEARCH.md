# Global Search System - Shared Library

## Overview

Sistem pencarian global yang dapat digunakan di semua microfrontend SIMPelv2. Mendukung pencarian across aplikasi, halaman, dokumen, dan user dengan debouncing dan highlighting.

## Components

### 1. `GlobalSearchBar`

Komponen search bar dengan dropdown hasil pencarian.

**Usage**:
```rust
use shared_microfrontend::components::GlobalSearchBar;

#[component]
pub fn Navbar() -> impl IntoView {
    view! {
        <nav>
            <GlobalSearchBar />
        </nav>
    }
}
```

**Props**:
- `placeholder: Option<String>` - Placeholder text (default: "Cari aplikasi, halaman, atau dokumen...")
- `class: Option<String>` - Custom CSS class
- `debounce_ms: Option<u32>` - Debounce delay in milliseconds (default: 300)

### 2. `GlobalSearchButton`

Tombol search yang membuka modal (untuk mobile/space-constrained layouts).

**Usage**:
```rust
use shared_microfrontend::components::GlobalSearchButton;

#[component]
pub fn MobileNav() -> impl IntoView {
    view! {
        <nav>
            <GlobalSearchButton />
        </nav>
    }
}
```

**Props**:
- `class: Option<String>` - Custom CSS class

### 3. `SearchResultsList`

Komponen untuk menampilkan hasil pencarian dalam format list.

**Usage**:
```rust
use shared_microfrontend::components::SearchResultsList;

#[compon
n SearchPage() -> impl IntoView {
    let results = vec![/* ... */];

    view! {
        <SearchResultsList
            results=results
            on_result_click=Box::new(|result| {
                // Handle click
            })
        />
    }
}
```

## Hooks

### `use_search()`

Hook untuk mengakses search context.

**Usage**:
```rust
use shared_microfrontend::hooks::use_search;

#[component]
pub fn MyComponent() -> impl IntoView {
    let search_ctx = use_search();

    // Register searchable data
    search_ctx.register_data(vec![
        SearchResult {
            id: "1".to_string(),
            title: "My Page".to_string(),
            description: "Description".to_string(),
            category: SearchCategory::Page,
            url: "/my-page".to_string(),
            icon: "📄".to_string(),
            module: Some("My App".to_string()),
        },
    ]);

    // Perform search
    search_ctx.search("query".to_string());

    // Get results
    let results = search_ctx.results.get();

    view! {
        <div>"Found: " {results.len()}</div>
    }
}
```

### `use_debounced_search(delay_ms: u32)`

Hook untuk debounced search.

**Usage**:
```rust
use shared_microfrontend::hooks::use_debounced_search;

#[component]
pub fn SearchInput() -> impl IntoView {
    let debounced_search = use_debounced_search(300);

    let handle_input = move |ev| {
        let value = event_target_value(&ev);
        debounced_search(value);
    };

    view! {
        <input type="text" on:input=handle_input />
    }
}
```

## Types

### `SearchResult`

```rust
pub struct SearchResult {
    pub id: String,
    pub title: String,
    pub description: String,
    pub category: SearchCategory,
    pub url: String,
    pub icon: String,
    pub module: Option<String>,  // App/module name
}
```

### `SearchCategory`

```rust
pub enum SearchCategory {
    Application,  // 🚀 Blue
    Page,         // 📄 Green
    Document,     // 📝 Yellow
    User,         // 👤 Purple
    Custom(String),  // Custom category
}
```

## Setup

### Portal Setup

Di portal `App` component, register semua searchable data:

```rust
use shared_microfrontend::hooks::{use_search, SearchCategory, SearchResult};

fn setup_search_providers() {
    let search_ctx = use_search();

    // Register applications
    let apps = vec![
        SearchResult {
            id: "pidum".to_string(),
            title: "PIDUM".to_string(),
            description: "Penyidikan dan Penuntutan Pidana Umum".to_string(),
            category: SearchCategory::Application,
            url: "http://localhost:8082".to_string(),
            icon: "⚖️".to_string(),
            module: None,
        },
        // ... more apps
    ];

    search_ctx.register_data(apps);

    // Register pages
    let pages = vec![
        SearchResult {
            id: "dashboard".to_string(),
            title: "Dashboard".to_string(),
            description: "Dashboard utama".to_string(),
            category: SearchCategory::Page,
            url: "/dashboard".to_string(),
            icon: "📊".to_string(),
            module: Some("Portal".to_string()),
        },
        // ... more pages
    ];

    search_ctx.register_data(pages);
}

#[component]
pub fn App() -> impl IntoView {
    setup_search_providers();

    view! {
        // ... app content
    }
}
```

### Microfrontend Setup

Di microfrontend, register data yang spesifik untuk app tersebut:

```rust
use shared_microfrontend::hooks::{use_search, SearchCategory, SearchResult};

#[component]
pub fn MyApp() -> impl IntoView {
    let search_ctx = use_search();

    // Register app-specific pages
    search_ctx.register_data(vec![
        SearchResult {
            id: "my-page-1".to_string(),
            title: "My Page 1".to_string(),
            description: "Description of page 1".to_string(),
            category: SearchCategory::Page,
            url: "/my-page-1".to_string(),
            icon: "📄".to_string(),
            module: Some("My App".to_string()),
        },
    ]);

    view! {
        <div>
            <GlobalSearchBar />
            // ... app content
        </div>
    }
}
```

## Features

✅ **Debounced Search**: 300ms default debounce untuk performance
✅ **Cross-Module**: Search across semua aplikasi dan halaman
✅ **Categorized**: Results dikelompokkan berdasarkan kategori
✅ **Relevance Sorting**: Exact matches first, then partial matches
✅ **Responsive**: Mobile-optimized dengan modal option
✅ **Keyboard Navigation**: Support keyboard shortcuts (coming soon)
✅ **Highlighting**: Search term highlighting (coming soon)
✅ **Recent Searches**: History pencarian (coming soon)

## Styling

### Custom Styling

Komponen menggunakan Tailwind CSS classes. Untuk custom styling:

```rust
view! {
    <GlobalSearchBar
        class="my-custom-class".to_string()
        placeholder="Custom placeholder".to_string()
    />
}
```

### Dark Mode

Semua komponen support dark mode secara otomatis dengan `dark:` classes.

## Advanced Usage

### API Search

Untuk search dari API backend:

```rust
use shared_microfrontend::hooks::use_search;
use gloo_net::http::Request;

async fn fetch_search_results(query: String) -> Result<Vec<SearchResult>, String> {
    let url = format!("/api/search?q={}", urlencoding::encode(&query));

    let response = Request::get(&url)
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if response.ok() {
        response
            .json::<Vec<SearchResult>>()
            .await
            .map_err(|e| format!("Parse error: {}", e))
    } else {
        Err(format!("HTTP error: {}", response.status()))
    }
}

#[component]
pub fn MyComponent() -> impl IntoView {
    let search_ctx = use_search();

    // Fetch and register data from API
    spawn_local(async move {
        if let Ok(results) = fetch_search_results("".to_string()).await {
            search_ctx.register_data(results);
        }
    });

    view! {
        <GlobalSearchBar />
    }
}
```

### Custom Categories

```rust
use shared_microfrontend::hooks::SearchCategory;

let custom_result = SearchResult {
    id: "custom-1".to_string(),
    title: "Custom Item".to_string(),
    description: "Custom description".to_string(),
    category: SearchCategory::Custom("My Category".to_string()),
    url: "/custom".to_string(),
    icon: "🎨".to_string(),
    module: Some("My App".to_string()),
};
```

## Best Practices

1. **Register Data Early**: Register searchable data di App component initialization
2. **Use Debouncing**: Default 300ms sudah optimal, jangan terlalu rendah
3. **Limit Results**: Jangan register terlalu banyak data (max ~1000 items)
4. **Clear Descriptions**: Gunakan deskripsi yang jelas dan informatif
5. **Consistent Icons**: Gunakan emoji atau icon yang konsisten
6. **Module Names**: Selalu set module name untuk clarity

## Troubleshooting

### Search Not Working

1. Check if data is registered: `search_ctx.register_data(data)`
2. Verify SearchResult structure is correct
3. Check browser console for errors

### No Results Found

1. Verify query matches title or description
2. Check if data is actually registered
3. Try exact match first

### Performance Issues

1. Reduce debounce delay if too slow
2. Limit number of registered items
3. Consider pagination for large datasets

## Migration from Portal-Only

**Before** (Portal-specific):
```rust
use crate::components::navigation::GlobalSearch;
```

**After** (Shared):
```rust
use shared_microfrontend::components::GlobalSearchBar;
```

API sama, hanya lokasi import yang berubah.

## Future Enhancements

- [ ] Keyboard shortcuts (Ctrl+K / Cmd+K)
- [ ] Search term highlighting in results
- [ ] Recent searches history
- [ ] Search suggestions/autocomplete
- [ ] Advanced filters (by category, module, date)
- [ ] Search analytics
- [ ] Voice search (optional)

## Support

- **Documentation**: `antarmuka/shared/docs/SEARCH.md`
- **Source**: `antarmuka/shared/src/hooks/use_search.rs`
- **Components**: `antarmuka/shared/src/components/search.rs`
