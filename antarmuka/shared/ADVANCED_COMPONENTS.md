# Advanced Interactive Components

This document provides usage examples for the advanced interactive components in the shared library.

## VirtualScroll

The `VirtualScroll` component efficiently renders large lists by only rendering visible items plus a buffer. This dramatically improves performance for lists with hundreds or thousands of items.

### Usage Example

```rust
use shared_microfrontend::components::advanced::VirtualScroll;
use leptos::prelude::*;

#[component]
pub fn MyLargeList() -> impl IntoView {
    // Create a large dataset
    let items: Vec<String> = (0..10000)
        .map(|i| format!("Item {}", i))
        .collect();

    view! {
        <VirtualScroll
            items=items
            item_height=50.0
            viewport_height=400.0
            overscan=5
            render_item=Callback::new(move |(item, idx): (String, usize)| {
                view! {
                    <div class="p-4 border-b border-gray-200">
                        <span class="font-semibold">{idx + 1}". "</span>
                        {item}
                    </div>
                }
            })
            class="border border-gray-300 rounded"
        />
    }
}
```

### Props

- `items: Vec<T>` - The complete list of items to render
- `item_height: f64` - Height of each item in pixels (default: 50.0)
- `viewport_height: f64` - Height of the scrollable viewport in pixels (default: 400.0)
- `overscan: usize` - Number of items to render outside visible area as buffer (default: 5)
- `render_item: Callback<(T, usize)>` - Function to render each item, receives (item, index)
- `class: Option<String>` - Optional CSS classes

## InfiniteScroll

The `InfiniteScroll` component automatically loads more data when the user scrolls near the bottom of the list. Perfect for implementing "load more" functionality.

### Usage Example

```rust
use shared_microfrontend::components::advanced::InfiniteScroll;
use leptos::prelude::*;

#[component]
pub fn MyInfiniteList() -> impl IntoView {
    let (items, set_items) = signal(vec!["Item 1".to_string(), "Item 2".to_string()]);
    let (loading, set_loading) = signal(false);
    let (has_more, set_has_more) = signal(true);
    let (page, set_page) = signal(1);

    let load_more = Callback::new(move |_: ()| {
        if loading.get() || !has_more.get() {
            return;
        }

        set_loading.set(true);

        // Simulate API call
        spawn_local(async move {
            gloo_timers::future::TimeoutFuture::new(1000).await;

            let new_page = page.get() + 1;
            let new_items: Vec<String> = (0..20)
                .map(|i| format!("Item {} (Page {})", i + 1, new_page))
                .collect();

            set_items.update(|items| items.extend(new_items));
            set_page.set(new_page);
            set_loading.set(false);

            // Stop loading after 5 pages
            if new_page >= 5 {
                set_has_more.set(false);
            }
        });
    });

    view! {
        <InfiniteScroll
            items=items.get()
            loading=loading
            has_more=has_more
            on_load_more=load_more
            threshold=200.0
            render_item=Callback::new(move |(item, idx): (String, usize)| {
                view! {
                    <div class="p-4 border-b border-gray-200 hover:bg-gray-50">
                        <span class="font-semibold">{idx + 1}". "</span>
                        {item}
                    </div>
                }
            })
            class="h-96 border border-gray-300 rounded"
        />
    }
}
```

### Props

- `items: Vec<T>` - Current list of items
- `loading: Signal<bool>` - Whether more data is being loaded
- `has_more: Signal<bool>` - Whether there are more items to load
- `on_load_more: Callback<()>` - Callback to load more items
- `threshold: f64` - Distance from bottom (in pixels) to trigger load (default: 200.0)
- `render_item: Callback<(T, usize)>` - Function to render each item
- `class: Option<String>` - Optional CSS classes

## ContextMenu

The `ContextMenu` component displays a context menu when the user right-clicks on an element. Perfect for providing contextual actions.

### Usage Example

```rust
use shared_microfrontend::components::advanced::{ContextMenu, ContextMenuItem};
use leptos::prelude::*;

#[component]
pub fn MyContextMenuExample() -> impl IntoView {
    let (selected_action, set_selected_action) = signal(None::<String>);

    let menu_items = vec![
        ContextMenuItem::new("copy", "Copy")
            .with_icon("📋"),
        ContextMenuItem::new("paste", "Paste")
            .with_icon("📄"),
        ContextMenuItem::divider(),
        ContextMenuItem::new("delete", "Delete")
            .with_icon("🗑️"),
        ContextMenuItem::new("disabled", "Disabled Action")
            .disabled(),
    ];

    let handle_select = Callback::new(move |action: String| {
        set_selected_action.set(Some(action.clone()));
        logging::log!("Selected action: {}", action);
    });

    view! {
        <div class="p-8">
            <ContextMenu
                items=menu_items
                on_select=handle_select
            >
                <div class="p-8 border-2 border-dashed border-gray-300 rounded-lg text-center">
                    <p class="text-gray-600">"Right-click here to open context menu"</p>
                    {move || selected_action.get().map(|action| view! {
                        <p class="mt-4 text-sm text-gray-500">
                            "Last action: " <strong>{action}</strong>
                        </p>
                    })}
                </div>
            </ContextMenu>
        </div>
    }
}
```

### ContextMenuItem API

```rust
// Create a basic menu item
let item = ContextMenuItem::new("id", "Label");

// Add an icon
let item = ContextMenuItem::new("copy", "Copy")
    .with_icon("📋");

// Make it disabled
let item = ContextMenuItem::new("action", "Action")
    .disabled();

// Create a divider
let divider = ContextMenuItem::divider();
```

### Props

- `items: Vec<ContextMenuItem>` - Menu items to display
- `on_select: Callback<String>` - Callback when an item is selected (receives item ID)
- `class: Option<String>` - Optional CSS classes
- `children: Children` - The element that triggers the context menu on right-click

## Performance Considerations

### VirtualScroll

- **Best for**: Lists with 100+ items where all items have the same height
- **Performance**: Only renders visible items + overscan buffer
- **Memory**: Stores all items in memory but only renders a subset
- **Limitation**: All items must have the same height

### InfiniteScroll

- **Best for**: Paginated data loading, social media feeds, search results
- **Performance**: Loads data on-demand, reducing initial load time
- **Memory**: Only stores loaded items in memory
- **Network**: Makes API calls as user scrolls

### ContextMenu

- **Best for**: Providing contextual actions without cluttering the UI
- **Performance**: Lightweight, only renders when shown
- **UX**: Familiar pattern for power users
- **Accessibility**: Ensure keyboard alternatives are provided

## Accessibility

All components follow WCAG 2.1 AA guidelines:

- **VirtualScroll**: Maintains proper focus management and keyboard navigation
- **InfiniteScroll**: Provides loading indicators and "no more items" messages
- **ContextMenu**: Includes proper ARIA roles and keyboard support (Escape to close)

## Browser Compatibility

These components work in all modern browsers:
- Chrome/Edge 90+
- Firefox 88+
- Safari 14+

## Examples in the Codebase

For more examples, see:
- Portal dashboard widgets (VirtualScroll for large datasets)
- Apps page (InfiniteScroll for app listings)
- Table components (ContextMenu for row actions)
