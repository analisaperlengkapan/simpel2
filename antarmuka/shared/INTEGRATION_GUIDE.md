# 🚀 Integration Guide - Panduan Integrasi Shared Components

## Overview

Panduan lengkap untuk mengintegrasikan shared components yang telah dioptimalkan dan dimodernisasi ke setiap microfrontend.

## 📦 Fitur Baru

### 1. Progressive Web App (PWA)

- ✅ Offline detection & handling
- ✅ Service Worker integration
- ✅ Install prompts
- ✅ Update notifications

### 2. Interactive Components

- ✅ Drag & Drop
- ✅ Context Menus
- ✅ Keyboard Shortcuts
- ✅ Tooltips & Popovers
- ✅ Smooth Scroll

### 3. Advanced Data Components

- ✅ Virtual Scrolling (10,000+ items)
- ✅ Infinite Scroll
- ✅ Advanced Data Tables
- ✅ Skeleton Loaders

### 4. Animation System

- ✅ CSS-based animations (GPU accelerated)
- ✅ Scroll-triggered animations
- ✅ Staggered animations
- ✅ Reduced motion support

### 5. Accessibility (A11y)

- ✅ WCAG 2.1 AA compliant
- ✅ Screen reader support
- ✅ Keyboard navigation
- ✅ Focus management
- ✅ High contrast mode
- ✅ Font size controls

## 🔧 Setup

### 1. Update Cargo.toml

Pastikan dependency ke shared library sudah benar:

```toml
[dependencies]
shared-microfrontend = { path = "../shared" }
```

### 2. Import Prelude

Di `src/lib.rs` atau komponen utama:

```rust
use shared_microfrontend::prelude::*;
```

## 📝 Contoh Penggunaan

### Progressive Web Features

```rust
use leptos::prelude::*;
use shared_microfrontend::prelude::*;

#[component]
pub fn App() -> impl IntoView {
    view! {
        // Network status indicator
        <NetworkStatusIndicator
            offline_message="Anda offline. Data akan disinkronkan saat online."
            position="bottom-right"
        />

        // Install prompt for PWA
        <InstallPrompt
            title="Install SIMPelv2"
            description="Install untuk akses lebih cepat dan fitur offline"
        />

        // Update notification
        <UpdateNotification />

        // Your app content
        <div class="container mx-auto">
            <h1>"Welcome to SIMPelv2"</h1>
        </div>
    }
}
```

### Interactive Components

#### Drag and Drop

```rust
#[component]
pub fn TaskBoard() -> impl IntoView {
    let (tasks, set_tasks) = signal(vec!["Task 1", "Task 2", "Task 3"]);

    let handle_drop = move |data: String| {
        // Handle dropped task
        logging::log!("Dropped: {}", data);
    };

    view! {
        <div class="grid grid-cols-2 gap-4">
            <div>
                <h2>"Available Tasks"</h2>
                {move || tasks.get().iter().enumerate().map(|(idx, task)| {
                    view! {
                        <Draggable
                            id=format!("task-{}", idx)
                            data=task.to_string()
                        >
                            <div class="bg-white p-4 rounded shadow">
                                {task}
                            </div>
                        </Draggable>
                    }
                }).collect::<Vec<_>>()}
            </div>

            <DropZone
                on_drop=Box::new(handle_drop)
                placeholder="Drop tasks here"
            >
                <div class="min-h-[200px]"></div>
            </DropZone>
        </div>
    }
}
```

#### Context Menu

```rust
#[component]
pub fn DataView() -> impl IntoView {
    let (show_menu, set_show_menu) = signal(false);
    let (menu_pos, set_menu_pos) = signal((0, 0));

    let menu_items = vec![
        MenuItem {
            label: "Edit".to_string(),
            icon: Some("edit".to_string()),
            shortcut: Some("Ctrl+E".to_string()),
            disabled: false,
            divider_after: false,
            callback: Rc::new(|| logging::log!("Edit clicked")),
        },
        MenuItem {
            label: "Delete".to_string(),
            icon: Some("trash".to_string()),
            shortcut: Some("Del".to_string()),
            disabled: false,
            divider_after: true,
            callback: Rc::new(|| logging::log!("Delete clicked")),
        },
        MenuItem {
            label: "Export".to_string(),
            icon: Some("download".to_string()),
            shortcut: None,
            disabled: false,
            divider_after: false,
            callback: Rc::new(|| logging::log!("Export clicked")),
        },
    ];

    let handle_context_menu = move |e: web_sys::MouseEvent| {
        e.prevent_default();
        set_menu_pos.set((e.client_x(), e.client_y()));
        set_show_menu.set(true);
    };

    view! {
        <div
            class="p-4 bg-white rounded shadow"
            on:contextmenu=handle_context_menu
        >
            <p>"Right-click for context menu"</p>
        </div>

        <ContextMenu
            items=menu_items
            show=show_menu
            position=menu_pos
        />
    }
}
```

#### Keyboard Shortcuts

```rust
#[component]
pub fn Editor() -> impl IntoView {
    let (show_help, set_show_help) = signal(false);

    let shortcuts = vec![
        Shortcut {
            key: "s".to_string(),
            ctrl: true,
            shift: false,
            alt: false,
            callback: Rc::new(|| logging::log!("Save")),
        },
        Shortcut {
            key: "h".to_string(),
            ctrl: true,
            shift: false,
            alt: false,
            callback: Rc::new(move || set_show_help.set(true)),
        },
    ];

    use_keyboard_shortcuts(shortcuts);

    let shortcuts_help = vec![
        ("Ctrl+S".to_string(), "Save".to_string()),
        ("Ctrl+H".to_string(), "Show Help".to_string()),
    ];

    view! {
        <div>
            <p>"Editor content (Press Ctrl+H for help)"</p>

            <ShortcutHelp
                shortcuts=shortcuts_help
                show=show_help
                on_close=Box::new(move || set_show_help.set(false))
            />
        </div>
    }
}
```

#### Tooltips

```rust
#[component]
pub fn ActionButtons() -> impl IntoView {
    view! {
        <div class="flex space-x-2">
            <Tooltip content="Save your work" position="top">
                <button class="px-4 py-2 bg-blue-600 text-white rounded">
                    <i class="fas fa-save"></i>
                </button>
            </Tooltip>

            <Tooltip content="Delete item" position="top">
                <button class="px-4 py-2 bg-red-600 text-white rounded">
                    <i class="fas fa-trash"></i>
                </button>
            </Tooltip>

            <Tooltip content="Download" position="bottom" delay=500>
                <button class="px-4 py-2 bg-green-600 text-white rounded">
                    <i class="fas fa-download"></i>
                </button>
            </Tooltip>
        </div>
    }
}
```

### Advanced Data Components

#### Virtual Scroll (Large Lists)

```rust
#[component]
pub fn LargeDataList() -> impl IntoView {
    let items = (0..10000).map(|i| format!("Item {}", i)).collect::<Vec<_>>();
    let items_signal = signal(items).0;

    view! {
        <VirtualScroll
            item_count=10000
            item_height=50.0
            height=600.0
            items=items_signal
            render_item=|item: String, index: usize| {
                view! {
                    <div class="p-2 border-b hover:bg-gray-50">
                        {format!("{} - {}", index, item)}
                    </div>
                }
            }
        />
    }
}
```

#### Infinite Scroll

```rust
#[component]
pub fn InfiniteList() -> impl IntoView {
    let (items, set_items) = signal(vec![]);
    let (loading, set_loading) = signal(false);
    let (has_more, set_has_more) = signal(true);

    let load_more = move || {
        set_loading.set(true);

        // Simulate API call
        spawn_local(async move {
            // Delay to simulate network
            gloo_timers::future::TimeoutFuture::new(1000).await;

            set_items.update(|items| {
                let start = items.len();
                for i in start..start + 20 {
                    items.push(format!("Item {}", i));
                }
            });

            set_loading.set(false);

            if items.get().len() >= 100 {
                set_has_more.set(false);
            }
        });
    };

    view! {
        <InfiniteScroll
            on_load_more=load_more
            loading=loading
            has_more=has_more
        >
            <div class="space-y-2">
                {move || items.get().iter().map(|item| {
                    view! {
                        <div class="p-4 bg-white rounded shadow">
                            {item}
                        </div>
                    }
                }).collect::<Vec<_>>()}
            </div>
        </InfiniteScroll>
    }
}
```

#### Advanced Data Table

```rust
#[component]
pub fn UsersTable() -> impl IntoView {
    #[derive(Clone)]
    struct User {
        id: u32,
        name: String,
        email: String,
        role: String,
    }

    let users = vec![
        User { id: 1, name: "Ahmad".to_string(), email: "ahmad@example.com".to_string(), role: "Admin".to_string() },
        User { id: 2, name: "Budi".to_string(), email: "budi@example.com".to_string(), role: "User".to_string() },
        // ... more users
    ];

    let users_signal = signal(users).0;

    let columns = vec![
        TableColumn {
            header: "ID".to_string(),
            accessor: Rc::new(|u: &User| u.id.to_string()),
            sortable: true,
            width: Some("80px".to_string()),
            align: "center".to_string(),
        },
        TableColumn {
            header: "Name".to_string(),
            accessor: Rc::new(|u: &User| u.name.clone()),
            sortable: true,
            width: None,
            align: "left".to_string(),
        },
        TableColumn {
            header: "Email".to_string(),
            accessor: Rc::new(|u: &User| u.email.clone()),
            sortable: true,
            width: None,
            align: "left".to_string(),
        },
        TableColumn {
            header: "Role".to_string(),
            accessor: Rc::new(|u: &User| u.role.clone()),
            sortable: true,
            width: Some("120px".to_string()),
            align: "center".to_string(),
        },
    ];

    view! {
        <DataTable
            columns=columns
            data=users_signal
            sortable=true
            paginated=true
            page_size=10
            selectable=true
            on_select=Some(Rc::new(|selected| {
                logging::log!("Selected: {:?}", selected);
            }))
            on_row_click=Some(Rc::new(|user| {
                logging::log!("Clicked: {}", user.name);
            }))
        />
    }
}
```

### Animation Components

#### Fade & Slide Animations

```rust
#[component]
pub fn AnimatedCard() -> impl IntoView {
    view! {
        <FadeIn duration=400 delay=100>
            <div class="bg-white p-6 rounded shadow">
                <h2>"Animated Card"</h2>
                <p>"This card fades in smoothly"</p>
            </div>
        </FadeIn>

        <SlideIn direction=SlideDirection::Up duration=500 delay=200>
            <div class="bg-blue-50 p-6 rounded mt-4">
                <h2>"Slide Animation"</h2>
                <p>"This slides up from bottom"</p>
            </div>
        </SlideIn>

        <ScaleIn duration=300 delay=300 from_scale=0.8>
            <button class="mt-4 px-6 py-3 bg-green-600 text-white rounded">
                "Animated Button"
            </button>
        </ScaleIn>
    }
}
```

#### Scroll Reveal

```rust
#[component]
pub fn LongPage() -> impl IntoView {
    view! {
        <div class="space-y-8">
            {(0..10).map(|i| {
                view! {
                    <ScrollReveal
                        animation_type={if i % 2 == 0 { "fade" } else { "slide-up" }}
                        duration=600
                        threshold=0.2
                    >
                        <div class="bg-white p-8 rounded shadow">
                            <h3>{format!("Section {}", i + 1)}</h3>
                            <p>"This appears when you scroll to it"</p>
                        </div>
                    </ScrollReveal>
                }
            }).collect::<Vec<_>>()}
        </div>

        <ScrollToTop threshold=300 />
    }
}
```

#### Staggered Animations

```rust
#[component]
pub fn FeaturesList() -> impl IntoView {
    let features = vec![
        "High Performance",
        "Accessibility",
        "Progressive",
        "Secure",
        "Scalable",
    ];

    view! {
        <Stagger delay_per_item=100 animation_type="fade" duration=400>
            {features.iter().map(|feature| {
                view! {
                    <div class="p-4 bg-white rounded shadow mb-2">
                        <i class="fas fa-check-circle text-green-600 mr-2"></i>
                        {feature}
                    </div>
                }
            }).collect::<Vec<_>>()}
        </Stagger>
    }
}
```

### Accessibility Components

#### Skip Links & Focus Management

```rust
#[component]
pub fn AccessiblePage() -> impl IntoView {
    view! {
        // Skip link for keyboard users
        <SkipLink target="main-content" text="Skip to main content" />

        // Announcer for screen readers
        <Announcer />

        <header>
            <nav>"Navigation"</nav>
        </header>

        <main id="main-content" tabindex="-1">
            <Heading level=1 class="text-3xl mb-4">
                "Page Title"
            </Heading>

            <p>"Content here"</p>
        </main>

        // Font size controls
        <div class="fixed top-4 right-4">
            <FontSizeControl />
        </div>

        // High contrast toggle
        <div class="fixed bottom-4 left-4">
            <HighContrastToggle />
        </div>
    }
}
```

#### Modal with Focus Trap

```rust
#[component]
pub fn ModalExample() -> impl IntoView {
    let (show_modal, set_show_modal) = signal(false);

    view! {
        <button
            class="px-4 py-2 bg-blue-600 text-white rounded"
            on:click=move |_| set_show_modal.set(true)
        >
            "Open Modal"
        </button>

        <Show when=move || show_modal.get()>
            <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center">
                <FocusTrap active=show_modal initial_focus=Some("modal-close".to_string())>
                    <div class="bg-white p-6 rounded shadow-xl max-w-md">
                        <Heading level=2 class="text-xl mb-4">
                            "Modal Title"
                        </Heading>

                        <p class="mb-4">"Modal content here"</p>

                        <button
                            id="modal-close"
                            class="px-4 py-2 bg-gray-600 text-white rounded"
                            on:click=move |_| set_show_modal.set(false)
                        >
                            "Close"
                        </button>
                    </div>
                </FocusTrap>
            </div>
        </Show>
    }
}
```

## 🎨 Styling

Semua styling sudah tersedia di `antarmuka/shared/styles/`. Pastikan mengimport di `index.html`:

```html
<link rel="stylesheet" href="/styles/main.css" />
```

## 📱 Responsive Design

Semua komponen responsive by default dengan breakpoints:

- `sm`: 640px
- `md`: 768px
- `lg`: 1024px
- `xl`: 1280px
- `2xl`: 1536px

## ⚡ Performance Tips

1. **Virtual Scrolling**: Gunakan untuk list > 100 items
2. **Lazy Loading**: Implementasikan untuk gambar dan komponen berat
3. **Code Splitting**: Pisahkan bundle per route
4. **Memoization**: Gunakan `Memo` untuk computed values
5. **Debouncing**: Untuk input dan search

## 🔒 Security

1. Semua input sudah di-sanitize
2. XSS protection built-in
3. CSRF tokens untuk form submissions
4. Content Security Policy headers

## 📚 Resources

- [Leptos Documentation](https://leptos.dev)
- [WCAG 2.1 Guidelines](https://www.w3.org/WAI/WCAG21/quickref/)
- [Web Animations API](https://developer.mozilla.org/en-US/docs/Web/API/Web_Animations_API)
- [Service Worker API](https://developer.mozilla.org/en-US/docs/Web/API/Service_Worker_API)

## 🤝 Contributing

Untuk menambahkan komponen baru ke shared library:

1. Buat file di `antarmuka/shared/src/`
2. Export di `lib.rs`
3. Tambahkan di prelude jika perlu
4. Update dokumentasi ini
5. Test di minimal 2 microfrontend

## 📞 Support

Untuk bantuan teknis, hubungi Tim Pengembang SIMPelv2.
