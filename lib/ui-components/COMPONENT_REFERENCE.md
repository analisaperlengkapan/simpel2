# 📚 Component Reference - SIMPelv2 Shared Library

Dokumentasi lengkap untuk semua komponen UI yang tersedia di shared library.

## 📖 Table of Contents

- [Layout Components](#layout-components)
- [Form Components](#form-components)
- [Feedback Components](#feedback-components)
- [Navigation Components](#navigation-components)
- [Display Components](#display-components)
- [Advanced Components](#advanced-components)
- [Authentication Components](#authentication-components)
- [Accessibility Components](#accessibility-components)

---

## Layout Components

### Card

Container dengan border, shadow, dan optional title.

**Props:**
- `title: Option<String>` - Optional card title
- `class: Option<String>` - Additional CSS classes
- `children: Children` - Card content

**Example:**
```rust
<Card title="User Profile">
    <p>"Name: Ahmad Wijaya"</p>
    <p>"Role: Administrator"</p>
</Card>
```

**Visual:**
```
┌─────────────────────────┐
│ User Profile            │
├─────────────────────────┤
│ Name: Ahmad Wijaya      │
│ Role: Administrator     │
└─────────────────────────┘
```

---

### Container

Max-width centered container untuk layout consistency.

**Props:**
- `max_width: Option<String>` - Max width (default: "1280px")
- `class: Option<String>` - Additional CSS classes
- `children: Children` - Container content

**Example:**
```rust
<Container max_width="1024px">
    <h1>"Welcome to SIMPelv2"</h1>
</Container>
```


---

### Grid

Responsive grid layout system.

**Props:**
- `cols: Option<u32>` - Number of columns (default: 1)
- `gap: Option<String>` - Gap between items (default: "1rem")
- `class: Option<String>` - Additional CSS classes
- `children: Children` - Grid items

**Example:**
```rust
<Grid cols=3 gap="2rem">
    <Card title="Card 1"><p>"Content 1"</p></Card>
    <Card title="Card 2"><p>"Content 2"</p></Card>
    <Card title="Card 3"><p>"Content 3"</p></Card>
</Grid>
```

**Responsive Behavior:**
- Mobile (< 640px): 1 column
- Tablet (640px - 1024px): 2 columns
- Desktop (> 1024px): Specified columns

---

### Stack

Flexbox stack untuk vertical atau horizontal layout.

**Props:**
- `direction: StackDirection` - `Vertical` or `Horizontal`
- `spacing: Option<String>` - Space between items (default: "1rem")
- `align: Option<String>` - Alignment (start, center, end, stretch)
- `justify: Option<String>` - Justification (start, center, end, between, around)
- `class: Option<String>` - Additional CSS classes
- `children: Children` - Stack items

**Example:**
```rust
<Stack direction=StackDirection::Horizontal spacing="0.5rem" align="center">
    <Button variant=ButtonVariant::Primary>"Save"</Button>
    <Button variant=ButtonVariant::Ghost>"Cancel"</Button>
</Stack>
```

---

### Footer

Application footer dengan copyright dan links.

**Props:**
- `copyright: String` - Copyright text
- `links: Vec<FooterLink>` - Footer links
- `class: Option<String>` - Additional CSS classes

**Example:**
```rust
let links = vec![
    FooterLink { label: "Privacy".to_string(), url: "/privacy".to_string() },
    FooterLink { label: "Terms".to_string(), url: "/terms".to_string() },
];

<Footer copyright="© 2025 Kejaksaan RI" links=links />
```

---

### Divider

Visual separator line.

**Props:**
- `orientation: DividerOrientation` - `Horizontal` or `Vertical`
- `class: Option<String>` - Additional CSS classes

**Example:**
```rust
<div>
    <p>"Section 1"</p>
    <Divider orientation=DividerOrientation::Horizontal />
    <p>"Section 2"</p>
</div>
```

---

### Section

Semantic section wrapper dengan optional title.

**Props:**
- `title: Option<String>` - Section title
- `class: Option<String>` - Additional CSS classes
- `children: Children` - Section content

**Example:**
```rust
<Section title="Recent Activity">
    <List items=activities />
</Section>
```

---

### Spacer

Flexible spacer untuk layout.

**Props:**
- `size: Option<String>` - Fixed size (e.g., "2rem")
- `class: Option<String>` - Additional CSS classes

**Example:**
```rust
<div>
    <Button>"Button 1"</Button>
    <Spacer size="1rem" />
    <Button>"Button 2"</Button>
</div>
```

---

## Form Components

### Button

Tombol dengan berbagai variant dan size.

**Props:**
- `variant: ButtonVariant` - `Primary`, `Secondary`, `Danger`, `Success`, `Ghost`
- `size: ButtonSize` - `Small`, `Medium`, `Large`
- `disabled: bool` - Disabled state (default: false)
- `loading: bool` - Loading state with spinner (default: false)
- `full_width: bool` - Full width button (default: false)
- `class: Option<String>` - Additional CSS classes
- `on:click: EventHandler` - Click handler
- `children: Children` - Button content

**Example:**
```rust
<Button
    variant=ButtonVariant::Primary
    size=ButtonSize::Medium
    loading=is_loading.get()
    on:click=move |_| handle_submit()
>
    "Submit"
</Button>
```

**Variants:**
- **Primary**: Blue background, white text (main actions)
- **Secondary**: Gray background, white text (secondary actions)
- **Danger**: Red background, white text (destructive actions)
- **Success**: Green background, white text (positive actions)
- **Ghost**: Transparent background, colored text (subtle actions)

---

### Input

Text input dengan label, validation, dan error handling.

**Props:**
- `id: String` - Input ID
- `name: String` - Input name
- `label: String` - Input label
- `value: ReadSignal<String>` - Input value signal
- `input_type: Option<String>` - Input type (default: "text")
- `placeholder: Option<String>` - Placeholder text
- `required: bool` - Required field (default: false)
- `disabled: bool` - Disabled state (default: false)
- `error: Option<String>` - Error message
- `help_text: Option<String>` - Help text below input
- `class: Option<String>` - Additional CSS classes
- `on_input: Callback<String>` - Input change handler

**Example:**
```rust
let (username, set_username) = signal(String::new());
let username_error = move || {
    if username.get().is_empty() {
        Some("Username is required".to_string())
    } else {
        None
    }
};

<Input
    id="username"
    name="username"
    label="Username"
    value=username
    placeholder="Enter your username"
    required=true
    error=username_error()
    on_input=move |val| set_username.set(val)
/>
```

---

### Select

Dropdown select dengan options.

**Props:**
- `id: String` - Select ID
- `name: String` - Select name
- `label: String` - Select label
- `value: ReadSignal<String>` - Selected value signal
- `options: Vec<SelectOption>` - Select options
- `required: bool` - Required field (default: false)
- `disabled: bool` - Disabled state (default: false)
- `error: Option<String>` - Error message
- `class: Option<String>` - Additional CSS classes
- `on_change: Callback<String>` - Change handler

**Example:**
```rust
let (role, set_role) = signal(String::new());
let options = vec![
    SelectOption { value: "admin".to_string(), label: "Administrator".to_string() },
    SelectOption { value: "user".to_string(), label: "User".to_string() },
    SelectOption { value: "guest".to_string(), label: "Guest".to_string() },
];

<Select
    id="role"
    name="role"
    label="Role"
    value=role
    options=options
    required=true
    on_change=move |val| set_role.set(val)
/>
```

---

### Textarea

Multi-line text input.

**Props:**
- `id: String` - Textarea ID
- `name: String` - Textarea name
- `label: String` - Textarea label
- `value: ReadSignal<String>` - Textarea value signal
- `placeholder: Option<String>` - Placeholder text
- `rows: Option<u32>` - Number of rows (default: 4)
- `required: bool` - Required field (default: false)
- `disabled: bool` - Disabled state (default: false)
- `error: Option<String>` - Error message
- `class: Option<String>` - Additional CSS classes
- `on_input: Callback<String>` - Input change handler

**Example:**
```rust
let (description, set_description) = signal(String::new());

<Textarea
    id="description"
    name="description"
    label="Description"
    value=description
    placeholder="Enter description..."
    rows=6
    on_input=move |val| set_description.set(val)
/>
```


---

### Checkbox

Checkbox input dengan label.

**Props:**
- `id: String` - Checkbox ID
- `name: String` - Checkbox name
- `label: String` - Checkbox label
- `checked: ReadSignal<bool>` - Checked state signal
- `disabled: bool` - Disabled state (default: false)
- `class: Option<String>` - Additional CSS classes
- `on_change: Callback<bool>` - Change handler

**Example:**
```rust
let (agreed, set_agreed) = signal(false);

<Checkbox
    id="terms"
    name="terms"
    label="I agree to the terms and conditions"
    checked=agreed
    on_change=move |val| set_agreed.set(val)
/>
```

---

### Radio

Radio button input.

**Props:**
- `id: String` - Radio ID
- `name: String` - Radio name (group)
- `label: String` - Radio label
- `value: String` - Radio value
- `checked: ReadSignal<bool>` - Checked state signal
- `disabled: bool` - Disabled state (default: false)
- `class: Option<String>` - Additional CSS classes
- `on_change: Callback<String>` - Change handler

**Example:**
```rust
let (payment_method, set_payment_method) = signal("credit".to_string());

<div>
    <Radio
        id="payment-credit"
        name="payment"
        label="Credit Card"
        value="credit"
        checked=move || payment_method.get() == "credit"
        on_change=move |val| set_payment_method.set(val)
    />
    <Radio
        id="payment-debit"
        name="payment"
        label="Debd"
        value="debit"
        checked=move || payment_method.get() == "debit"
        on_change=move |val| set_payment_method.set(val)
    />
</div>
```

---

### Switch

Toggle switch component.

**Props:**
- `id: String` - Switch ID
- `label: String` - Switch label
- `checked: ReadSignal<bool>` - Checked state signal
- `disabled: bool` - Disabled state (default: false)
- `class: Option<String>` - Additional CSS classes
- `on_change: Callback<bool>` - Change handler

**Example:**
```rust
let (notifications_enabled, set_notifications) = signal(true);

<Switch
    id="notifications"
    label="Enable Notifications"
    checked=notifications_enabled
    on_change=move |val| set_notifications.set(val)
/>
```

---

### FileInput

File upload input.

**Props:**
- `id: String` - Input ID
- `name: String` - Input name
- `label: String` - Input label
- `accept: Option<String>` - Accepted file types (e.g., "image/*")
- `multiple: bool` - Allow multiple files (default: false)
- `disabled: bool` - Disabled state (default: false)
- `error: Option<String>` - Error message
- `class: Option<String>` - Additional CSS classes
- `on_change: Callback<Vec<web_sys::File>>` - Change handler

**Example:**
```rust
let handle_files = move |files: Vec<web_sys::File>| {
    for file in files {
        logging::log!("Selected: {}", file.name());
    }
};

<FileInput
    id="documents"
    name="documents"
    label="Upload Documents"
    accept="application/pdf,.doc,.docx"
    multiple=true
    on_change=handle_files
/>
```

---

### FormGroup

Group form fields with label and optional description.

**Props:**
- `label: String` - Group label
- `description: Option<String>` - Group description
- `required: bool` - Required indicator (default: false)
- `class: Option<String>` - Additional CSS classes
- `children: Children` - Form fields

**Example:**
```rust
<FormGroup label="Personal Information" description="Enter your personal details">
    <Input id="name" name="name" label="Full Name" value=name on_input=set_name />
    <Input id="email" name="email" label="Email" value=email on_input=set_email />
</FormGroup>
```

---

## Feedback Components

### Toast

Notification toast untuk success, error, warning, info messages.

**Props:**
- `toast_type: ToastType` - `Success`, `Error`, `Warning`, `Info`
- `message: String` - Toast message
- `duration: Option<u32>` - Auto-dismiss duration in ms (default: 3000)
- `position: ToastPosition` - `TopRight`, `TopLeft`, `BottomRight`, `BottomLeft`
- `on_close: Option<Callback<()>>` - Close handler

**Example:**
```rust
let (show_toast, set_show_toast) = signal(false);

// Show toast
set_show_toast.set(true);

<Show when=move || show_toast.get()>
    <Toast
        toast_type=ToastType::Success
        message="Data saved successfully!"
        duration=3000
        position=ToastPosition::TopRight
        on_close=Some(Box::new(move || set_show_toast.set(false)))
    />
</Show>
```

---

### Modal

Modal dialog dengan backdrop.

**Props:**
- `title: String` - Modal title
- `show: ReadSignal<bool>` - Show/hide signal
- `size: ModalSize` - `Small`, `Medium`, `Large`, `FullScreen`
- `close_on_backdrop: bool` - Close when clicking backdrop (default: true)
- `show_close_button: bool` - Show close button (default: true)
- `class: Option<String>` - Additional CSS classes
- `on_close: Option<Callback<()>>` - Close handler
- `children: Children` - Modal content

**Example:**
```rust
let (show_modal, set_show_modal) = signal(false);

<Button on:click=move |_| set_show_modal.set(true)>
    "Open Modal"
</Button>

<Modal
    title="Confirm Action"
    show=show_modal
    size=ModalSize::Medium
    on_close=Some(Box::new(move || set_show_modal.set(false)))
>
    <p>"Are you sure you want to proceed?"</p>
    <Stack direction=StackDirection::Horizontal spacing="0.5rem">
        <Button variant=ButtonVariant::Danger on:click=move |_| {
            handle_confirm();
            set_show_modal.set(false);
        }>"Confirm"</Button>
        <Button variant=ButtonVariant::Ghost on:click=move |_| set_show_modal.set(false)>
            "Cancel"
        </Button>
    </Stack>
</Modal>
```

---

### Alert

Alert message box.

**Props:**
- `alert_type: AlertType` - `Success`, `Error`, `Warning`, `Info`
- `title: Option<String>` - Alert title
- `message: String` - Alert message
- `dismissible: bool` - Show dismiss button (default: false)
- `class: Option<String>` - Additional CSS classes
- `on_dismiss: Option<Callback<()>>` - Dismiss handler

**Example:**
```rust
<Alert
    alert_type=AlertType::Warning
    title="Warning"
    message="Your session will expire in 5 minutes"
    dismissible=true
/>
```

---

### Loading

Loading spinner.

**Props:**
- `size: LoadingSize` - `Small`, `Medium`, `Large`
- `message: Option<String>` - Loading message
- `class: Option<String>` - Additional CSS classes

**Example:**
```rust
<Show when=move || is_loading.get()>
    <Loading size=LoadingSize::Medium message="Loading data..." />
</Show>
```

---

### ProgressBar

Progress indicator.

**Props:**
- `value: f64` - Progress value (0-100)
- `max: f64` - Maximum value (default: 100)
- `show_label: bool` - Show percentage label (default: true)
- `color: Option<String>` - Progress bar color
- `class: Option<String>` - Additional CSS classes

**Example:**
```rust
let (progress, set_progress) = signal(0.0);

<ProgressBar
    value=progress.get()
    max=100.0
    show_label=true
    color="bg-blue-600"
/>
```

---

### Skeleton

Skeleton loader untuk content placeholders.

**Props:**
- `width: Option<String>` - Width (default: "100%")
- `height: Option<String>` - Height (default: "1rem")
- `variant: SkeletonVariant` - `Text`, `Circular`, `Rectangular`
- `class: Option<String>` - Additional CSS classes

**Example:**
```rust
<Show when=move || is_loading.get() fallback=|| view! { <UserProfile /> }>
    <div class="space-y-2">
        <Skeleton variant=SkeletonVariant::Circular width="4rem" height="4rem" />
        <Skeleton variant=SkeletonVariant::Text width="60%" />
        <Skeleton variant=SkeletonVariant::Text width="40%" />
    </div>
</Show>
```

---

### Notification

Notification badge/indicator.

**Props:**
- `count: u32` - Notification count
- `max_count: Option<u32>` - Max count to display (default: 99)
- `show_zero: bool` - Show when count is 0 (default: false)
- `class: Option<String>` - Additional CSS classes
- `children: Children` - Content to badge

**Example:**
```rust
<Notification count=5 max_count=99>
    <Button variant=ButtonVariant::Ghost>
        <i class="fas fa-bell"></i>
    </Button>
</Notification>
```

---

### Spinner

Simple spinner animation.

**Props:**
- `size: Option<String>` - Size (default: "1.5rem")
- `color: Option<String>` - Color (default: "currentColor")
- `class: Option<String>` - Additional CSS classes

**Example:**
```rust
<Spinner size="2rem" color="text-blue-600" />
```



---

## Navigation Components

### AppHeader

Application header dengan logo dan navigation.

**Props:**
- `title: String` - Application title
- `subtitle: Option<String>` - Subtitle
- `show_logo: bool` - Show Kejaksaan RI logo (default: true)
- `class: Option<String>` - Additional CSS classes
- `children: Children` - Header actions (e.g., user menu, logout button)

**Example:**
```rust
<AppHeader title="SIMPelv2" subtitle="Sistem Informasi Manajemen Pengelolaan BMN">
    <UserProfile />
    <LogoutButton />
</AppHeader>
```

---

### Breadcrumb

Breadcrumb navigation.

**Props:**
- `items: Vec<BreadcrumbItem>` - Breadcrumb items
- `separator: Option<String>` - Separator character (default: "/")
- `class: Option<Stri- Additional CSS classes

**Example:**
```rust
let items = vec![
    BreadcrumbItem { label: "Home".to_string(), url: Some("/".to_string()) },
    BreadcrumbItem { label: "Users".to_string(), url: Some("/users".to_string()) },
    BreadcrumbItem { label: "Profile".to_string(), url: None }, // Current page
];

<Breadcrumb items=items separator=">" />
```

---

### NavMenu

Navigation menu (vertical or horizontal).

**Props:**
- `items: Vec<NavItem>` - Menu items
- `orientation: NavOrientation` - `Vertical` or `Horizontal`
- `active_item: Option<String>` - Active item ID
- `class: Option<String>` - Additional CSS classes

**Example:**
```rust
let items = vec![
    NavItem {
        id: "dashboard".to_string(),
        label: "Dashboard".to_string(),
        url: "/dashboard".to_string(),
        icon: Some("📊".to_string()),
        badge: None,
    },
    NavItem {
        id: "users".to_string(),
        label: "Users".to_string(),
        url: "/users".to_string(),
        icon: Some("👥".to_string()),
        badge: Some("5".to_string()),
    },
];

<NavMenu items=items orientation=NavOrientation::Vertical active_item="dashboard" />
```

---

### Logo

Kejaksaan RI logo component.

**Props:**
- `size: LogoSize` - `Small`, `Medium`, `Large`
- `show_text: bool` - Show "Kejaksaan RI" text (default: true)
- `class: Option<String>` - Additional CSS classes

**Example:**
```rust
<Logo size=LogoSize::Medium show_text=true />
```

---

### Sidebar

Collapsible sidebar navigation.

**Props:**
- `collapsed: ReadSignal<bool>` - Collapsed state signal
- `items: Vec<NavItem>` - Navigation items
- `width: Option<String>` - Sidebar width when expanded (default: "16rem")
- `class: Option<String>` - Additional CSS classes
- `on_toggle: Option<Callback<bool>>` - Toggle handler

**Example:**
```rust
let (sidebar_collapsed, set_sidebar_collapsed) = signal(false);

<Sidebar
    collapsed=sidebar_collapsed
    items=nav_items
    width="18rem"
    on_toggle=Some(Box::new(move |collapsed| set_sidebar_collapsed.set(collapsed)))
/>
```

---

### Tabs

Tab navigation component.

**Props:**
- `tabs: Vec<Tab>` - Tab items
- `active_tab: ReadSignal<String>` - Active tab ID signal
- `variant: TabVariant` - `Line`, `Enclosed`, `Pills`
- `class: Option<String>` - Additional CSS classes
- `on_change: Callback<String>` - Tab change handler

**Example:**
```rust
let (active_tab, set_active_tab) = signal("profile".to_string());

let tabs = vec![
    Tab { id: "profile".to_string(), label: "Profile".to_string(), icon: None },
    Tab { id: "settings".to_string(), label: "Settings".to_string(), icon: Some("⚙️".to_string()) },
    Tab { id: "security".to_string(), label: "Security".to_string(), icon: Some("🔒".to_string()) },
];

<Tabs
    tabs=tabs
    active_tab=active_tab
    variant=TabVariant::Line
    on_change=move |tab_id| set_active_tab.set(tab_id)
/>

<Show when=move || active_tab.get() == "profile">
    <ProfileContent />
</Show>
<Show when=move || active_tab.get() == "settings">
    <SettingsContent />
</Show>
```

---

### MobileMenuButton

Hamburger menu button untuk mobile navigation.

**Props:**
- `open: ReadSignal<bool>` - Open state signal
- `class: Option<String>` - Additional CSS classes
- `on_click: Callback<()>` - Click handler

**Example:**
```rust
let (menu_open, set_menu_open) = signal(false);

<MobileMenuButton
    open=menu_open
    on_click=move || set_menu_open.update(|open| *open = !*open)
/>
```

---

### BackButton

Back navigation button.

**Props:**
- `label: Option<String>` - Button label (default: "Back")
- `url: Option<String>` - URL to navigate to (if None, uses browser back)
- `class: Option<String>` - Additional CSS classes

**Example:**
```rust
<BackButton label="Back to Dashboard" url="/dashboard" />
```

---

## Display Components

### Table

Data table dengan sorting, filtering, dan pagination.

**Props:**
- `columns: Vec<TableColumn<T>>` - Table columns
- `data: ReadSignal<Vec<T>>` - Table data signal
- `sortable: bool` - Enable sorting (default: false)
- `filterable: bool` - Enable filtering (default: false)
- `paginated: bool` - Enable pagination (default: false)
- `page_size: usize` - Items per page (default: 10)
- `striped: bool` - Striped rows (default: false)
- `hoverable: bool` - Hover effect (default: true)
- `class: Option<String>` - Additional CSS classes

**Example:**
```rust
#[derive(Clone)]
struct User {
    id: u32,
    name: String,
    email: String,
    role: String,
}

let users = signal(vec![
    User { id: 1, name: "Ahmad".to_string(), email: "ahmad@example.com".to_string(), role: "Admin".to_string() },
    // ... more users
]).0;

let columns = vec![
    TableColumn::new("id", "ID").sortable(),
    TableColumn::new("name", "Name").sortable(),
    TableColumn::new("email", "Email"),
    TableColumn::new("role", "Role").sortable(),
];

<Table
    columns=columns
    data=users
    sortable=true
    paginated=true
    page_size=20
    striped=true
/>
```

---

### Badge

Status badge component.

**Props:**
- `variant: BadgeVariant` - `Primary`, `Secondary`, `Success`, `Danger`, `Warning`, `Info`
- `size: BadgeSize` - `Small`, `Medium`, `Large`
- `class: Option<String>` - Additional CSS classes
- `children: Children` - Badge content

**Example:**
```rust
<Badge variant=BadgeVariant::Success size=BadgeSize::Small>
    "Active"
</Badge>

<Badge variant=BadgeVariant::Danger>
    "Blocked"
</Badge>
```

---

### List

Generic list component.

**Props:**
- `items: Vec<T>` - List items
- `render: Fn(T) -> View` - Render function for each item
- `ordered: bool` - Ordered list (default: false)
- `class: Option<String>` - Additional CSS classes

**Example:**
```rust
let items = vec!["Item 1", "Item 2", "Item 3"];

<List
    items=items
    render=|item| view! {
        <li class="p-2 hover:bg-gray-50">{item}</li>
    }
/>
```

---

### EmptyState

Empty state placeholder.

**Props:**
- `title: String` - Empty state title
- `message: String` - Empty state message
- `icon: Option<String>` - Icon (emoji or icon class)
- `action: Option<View>` - Optional action button
- `class: Option<String>` - Additional CSS classes

**Example:**
```rust
<EmptyState
    title="No Data Found"
    message="There are no items to display. Try adjusting your filters."
    icon="📭"
    action=view! {
        <Button variant=ButtonVariant::Primary on:click=move |_| reset_filters()>
            "Reset Filters"
        </Button>
    }.into_view()
/>
```

---

### Avatar

User avatar component.

**Props:**
- `name: String` - User name (for initials)
- `src: Option<String>` - Avatar image URL
- `size: AvatarSize` - `Small`, `Medium`, `Large`, `ExtraLarge`
- `shape: AvatarShape` - `Circle`, `Square`
- `class: Option<String>` - Additional CSS classes

**Example:**
```rust
<Avatar
    name="Ahmad Wijaya"
    src="https://example.com/avatar.jpg"
    size=AvatarSize::Medium
    shape=AvatarShape::Circle
/>

// Without image (shows initials)
<Avatar
    name="Budi Santoso"
    src=None
    size=AvatarSize::Large
/>
```

---

### Pagination

Pagination controls.

**Props:**
- `current_page: u32` - Current page number (1-indexed)
- `total_pages: u32` - Total number of pages
- `total_items: usize` - Total number of items
- `page_size: usize` - Items per page
- `show_page_size_selector: bool` - Show page size selector (default: false)
- `class: Option<String>` - Additional CSS classes
- `on_page_change: Callback<u32>` - Page change handler
- `on_page_size_change: Option<Callback<usize>>` - Page size change handler

**Example:**
```rust
let (current_page, set_current_page) = signal(1);
let total_items = 250;
let page_size = 20;
let total_pages = (total_items as f64 / page_size as f64).ceil() as u32;

<Pagination
    current_page=current_page.get()
    total_pages=total_pages
    total_items=total_items
    page_size=page_size
    show_page_size_selector=true
    on_page_change=move |page| set_current_page.set(page)
/>
```

---

### QrCodeDisplay

QR code display component (untuk MFA setup).

**Props:**
- `data: String` - Data to encode in QR code
- `size: Option<u32>` - QR code size in pixels (default: 200)
- `class: Option<String>` - Additional CSS classes

**Example:**
```rust
let totp_uri = "otpauth://totp/SIMPelv2:user@example.com?secret=JBSWY3DPEHPK3PXP&issuer=SIMPelv2";

<QrCodeDisplay
    data=totp_uri
    size=250
/>
```



---

## Advanced Components

### Tooltip

Tooltip component untuk contextual help.

**Props:**
- `content: String` - Tooltip content
- `position: TooltipPosition` - `Top`, `Bottom`, `Left`, `Right`
- `delay: Option<u32>` - Show delay in ms (default: 200)
- `class: Option<String>` - Additional CSS classes
- `children: Children` - Element to attach tooltip to

**Example:**
```rust
<Tooltip content="Save your changes" position=TooltipPosition::Top>
    <Button variant=ButtonVariant::Primary>
        <i class="fas fa-save"></i>
    </Button>
</Tooltip>
```

---

### Popover

Popover component untuk additional content.

**Props:**
- `trigger: PopoverTrigger` - `Click`, `Hover`, `Focus`
- `position: PopoverPosition` - `Top`, `Bottom`, `Left`, `Right`
- `title: Option<String>` - Popover title
- `content: View` - Popover content
- `class: Option<String>` - Additional CSS classes
- `children: Children` - Trigger element

**Example:**
```rust
<Popover
    trigger=PopoverTrigger::Click
    position=PopoverPosition::Bottom
    title="User Actions"
    content=view! {
        <div class="space-y-2">
            <button class="w-full text-left px-3 py-2 hover:bg-gray-100">"Edit Profile"</button>
            <button class="w-full text-left px-3 py-2 hover:bg-gray-100">"Settings"</button>
            <button class="w-full text-left px-3 py-2 hover:bg-gray-100 text-red-600">"Logout"</button>
        </div>
    }.into_view()
>
    <Button variant=ButtonVariant::Ghost>
        <i class="fas fa-ellipsis-v"></i>
    </Button>
</Popover>
```

---

### Dropdown

Dropdown menu component.

**Props:**
- `trigger: View` - Trigger element
- `items: Vec<DropdownItem>` - Dropdown items
- `position: DropdownPosition` - `BottomLeft`, `BottomRight`, `TopLeft`, `TopRight`
- `class: Option<String>` - Additional CSS classes

**Example:**
```rust
let items = vec![
    DropdownItem {
        label: "Edit".to_string(),
        icon: Some("✏️".to_string()),
        disabled: false,
        divider_after: false,
        on_click: Box::new(|| logging::log!("Edit clicked")),
    },
    DropdownItem {
        label: "Delete".to_string(),
        icon: Some("🗑️".to_string()),
        disabled: false,
        divider_after: true,
        on_click: Box::new(|| logging::log!("Delete clicked")),
    },
    DropdownItem {
        label: "Export".to_string(),
        icon: Some("📥".to_string()),
        disabled: false,
        divider_after: false,
        on_click: Box::new(|| logging::log!("Export clicked")),
    },
];

<Dropdown
    trigger=view! {
        <Button variant=ButtonVariant::Ghost>
            "Actions" <i class="fas fa-chevron-down ml-2"></i>
        </Button>
    }.into_view()
    items=items
    position=DropdownPosition::BottomRight
/>
```

---

### OptimizedImage

Responsive image dengan lazy loading dan error handling.

**Props:**
- `src: String` - Image source URL
- `alt: String` - Alt text for accessibility
- `lazy: bool` - Enable lazy loading (default: true)
- `srcset: Option<String>` - Responsive image sources
- `sizes: Option<String>` - Image sizes for responsive
- `width: Option<String>` - Width attribute
- `height: Option<String>` - Height attribute
- `object_fit: String` - Object fit (cover, contain, fill, none, scale-down)
- `show_skeleton: bool` - Show skeleton loader while loading (default: true)
- `class: Option<String>` - Additional CSS classes

**Example:**
```rust
// Basic usage
<OptimizedImage
    src="/images/profile.jpg"
    alt="User profile picture"
/>

// Responsive with srcset
let srcset = generate_srcset(
    "/images/hero-{width}.jpg",
    &[640, 768, 1024, 1280, 1536]
);
let sizes = generate_sizes("100vw", "80vw", "1200px");

<OptimizedImage
    src="/images/hero-1024.jpg"
    srcset=srcset
    sizes=sizes
    alt="Hero banner"
    object_fit="cover"
    class="w-full h-96"
/>
```

---

## Authentication Components

### LoginRedirectPage

Default page untuk unauthenticated users yang redirect ke portal.

**Props:**
- `app_name: String` - Application name
- `app_description: Option<String>` - Application description
- `class: Option<String>` - Additional CSS classes

**Example:**
```rust
<Route path="/" view=|| view! {
    <LoginRedirectPage
        app_name="Badiklat"
        app_description="Sistem Manajemen Pelatihan dan Pendidikan"
    />
} />
```

---

### ProtectedRoute

Wrapper untuk routes yang memerlukan authentication.

**Props:**
- `required_permission: Option<String>` - Required permission (optional)
- `fallback: Option<View>` - Fallback view for unauthorized (optional)
- `children: Children` - Protected content

**Example:**
```rust
// Basic protection (requires authentication)
<Route path="/dashboard" view=|| view! {
    <ProtectedRoute>
        <DashboardPage />
    </ProtectedRoute>
} />

// With permission check
<ProtectedRoute required_permission="admin:*">
    <AdminPanel />
</ProtectedRoute>

// With custom fallback
<ProtectedRoute
    required_permission="reports:read"
    fallback=view! {
        <Alert
            alert_type=AlertType::Error
            title="Access Denied"
            message="You don't have permission to view reports"
        />
    }.into_view()
>
    <ReportsPage />
</ProtectedRoute>
```

---

### LogoutButton

Pre-built logout button.

**Props:**
- `variant: ButtonVariant` - Button variant (default: Ghost)
- `size: ButtonSize` - Button size (default: Medium)
- `show_icon: bool` - Show logout icon (default: true)
- `label: Option<String>` - Button label (default: "Logout")
- `class: Option<String>` - Additional CSS classes

**Example:**
```rust
<LogoutButton
    variant=ButtonVariant::Danger
    size=ButtonSize::Small
    label="Sign Out"
/>
```

---

### UserProfile

Display current user information.

**Props:**
- `show_avatar: bool` - Show user avatar (default: true)
- `show_role: bool` - Show user role (default: true)
- `show_division: bool` - Show user division (default: true)
- `class: Option<String>` - Additional CSS classes

**Example:**
```rust
<UserProfile
    show_avatar=true
    show_role=true
    show_division=true
/>
```

---

### PermissionGuard

Show content based on permissions.

**Props:**
- `permission: String` - Required permission
- `fallback: Option<View>` - Fallback view if no permission
- `children: Children` - Protected content

**Example:**
```rust
<PermissionGuard permission="admin:*">
    <AdminPanel />
</PermissionGuard>

<PermissionGuard
    permission="reports:read"
    fallback=view! {
        <p class="text-gray-500">"You don't have access to reports"</p>
    }.into_view()
>
    <ReportsSection />
</PermissionGuard>
```

---

## Accessibility Components

### SkipLink

Skip navigation link untuk keyboard users.

**Props:**
- `target: String` - Target element ID
- `text: String` - Link text (default: "Skip to main content")
- `class: Option<String>` - Additional CSS classes

**Example:**
```rust
<SkipLink target="main-content" text="Skip to main content" />

<header>
    <nav>"Navigation"</nav>
</header>

<main id="main-content" tabindex="-1">
    <h1>"Page Content"</h1>
</main>
```

---

### Heading

Semantic heading component dengan proper hierarchy.

**Props:**
- `level: u8` - Heading level (1-6)
- `class: Option<String>` - Additional CSS classes
- `children: Children` - Heading content

**Example:**
```rust
<Heading level=1 class="text-3xl font-bold mb-4">
    "Page Title"
</Heading>

<Heading level=2 class="text-2xl font-semibold mb-3">
    "Section Title"
</Heading>
```

---

### FontSizeControl

Font size adjustment controls.

**Props:**
- `min_size: f64` - Minimum font size multiplier (default: 0.8)
- `max_size: f64` - Maximum font size multiplier (default: 1.5)
- `step: f64` - Step size (default: 0.1)
- `class: Option<String>` - Additional CSS classes

**Example:**
```rust
<div class="fixed top-4 right-4">
    <FontSizeControl min_size=0.8 max_size=2.0 step=0.1 />
</div>
```

---

### HighContrastToggle

High contrast mode toggle.

**Props:**
- `label: Option<String>` - Toggle label
- `class: Option<String>` - Additional CSS classes

**Example:**
```rust
<div class="fixed bottom-4 left-4">
    <HighContrastToggle label="High Contrast Mode" />
</div>
```

---

## Helper Functions

### Validation

```rust
use shared_microfrontend::utils::*;

// NIK validation (16 digits)
let result = validate_nik("3201234567890123");
if result.is_valid() {
    // Valid NIK
} else {
    eprintln!("Errors: {:?}", result.errors);
}

// NIP validation (18 digits)
validate_nip("199001012020121001");

// Email validation
validate_email("user@example.com");

// Phone validation (Indonesian format)
validate_phone("08123456789");
validate_phone("+6281234567890");

// Postal code validation (5 digits)
validate_postal_code("12345");
```

---

### Formatting

```rust
use shared_microfrontend::utils::*;

// Currency formatting
format_currency(1000000);  // "Rp 1.000.000"

// Number formatting
format_number(1000000);  // "1.000.000"

// Date formatting (DD/MM/YYYY)
format_date(Local::now());  // "16/10/2025"

// Relative time
format_relative_time(datetime);  // "5 menit yang lalu"

// File size
format_file_size(1572864);  // "1.50 MB"
```

---

### Image Helpers

```rust
use shared_microfrontend::utils::*;

// Generate srcset for responsive images
let srcset = generate_srcset(
    "https://cdn.example.com/image-{width}.jpg",
    &[320, 640, 1024, 1920]
);
// Result: "https://cdn.example.com/image-320.jpg 320w, ..."

// Generate sizes attribute
let sizes = generate_sizes("100vw", "50vw", "800px");
// Result: "(max-width: 640px) 100vw, (max-width: 1024px) 50vw, 800px"
```

---

## Hooks

### use_auth

Authentication state management.

```rust
use shared_microfrontend::hooks::use_auth;

let auth = use_auth();

// Check authentication
if auth.is_authenticated() {
    // User is logged in
}

// Get session
if let Some(session) = auth.get_session() {
    logging::log!("User: {}", session.name);
}

// Check permission
if auth.has_permission("admin:*") {
    // User has admin permission
}

// Check role
if auth.is_admin() {
    // User is admin
}

// Logout
auth.logout();

// Redirect to login
auth.redirect_to_login();
```

---

### use_storage

LocalStorage/SessionStorage persistence.

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

// Use as signal
let (theme, set_theme) = use_storage("theme", ThemeMode::Light);
```

---

### use_media_query

Responsive breakpoint detection.

```rust
use shared_microfrontend::hooks::*;

// Simple checks
if is_mobile() {
    // Mobile layout
}

if is_desktop() {
    // Desktop layout
}

// Custom media query
if matches_media_query("(min-width: 1024px)") {
    // Large screen
}
```

---

### use_debounce

Debounce value changes.

```rust
use shared_microfrontend::hooks::*;

let (search, set_search) = signal(String::new());
let debounced_search = use_debounce(search, 300);

// Use debounced value for API calls
create_effect(move |_| {
    let query = debounced_search.get();
    if !query.is_empty() {
        fetch_results(query);
    }
});
```

---

### use_theme

Theme management.

```rust
use shared_microfrontend::hooks::*;

let theme = use_theme();

// Get current theme
let current = theme.get();

// Set theme
theme.set(ThemeMode::Dark);

// Toggle theme
theme.toggle();
```

---

## Best Practices

### 1. Always Use Semantic HTML

```rust
// ✅ Good
<Heading level=1>"Page Title"</Heading>
<nav><NavMenu items=items /></nav>
<main><Content /></main>

// ❌ Bad
<div class="text-3xl font-bold">"Page Title"</div>
<div><NavMenu items=items /></div>
<div><Content /></div>
```

### 2. Provide Alt Text for Images

```rust
// ✅ Good
<OptimizedImage src="/logo.png" alt="Kejaksaan RI Logo" />

// ❌ Bad
<OptimizedImage src="/logo.png" alt="" />
```

### 3. Use Proper Form Labels

```rust
// ✅ Good
<Input id="email" name="email" label="Email Address" value=email on_input=set_email />

// ❌ Bad
<input type="text" placeholder="Email" />
```

### 4. Handle Loading States

```rust
// ✅ Good
<Show when=move || !is_loading.get() fallback=|| view! { <Loading /> }>
    <DataTable data=data />
</Show>

// ❌ Bad
<DataTable data=data />  // No loading indicator
```

### 5. Provide Error Feedback

```rust
// ✅ Good
<Input
    id="username"
    label="Username"
    value=username
    error=username_error()
    on_input=set_username
/>

// ❌ Bad
<Input id="username" value=username on_input=set_username />  // No error handling
```

---

## Troubleshooting

### Component Not Rendering

**Problem:** Component doesn't appear on page.

**Solutions:**
1. Check if component is imported: `use shared_microfrontend::components::*;`
2. Verify props are correct type
3. Check browser console for errors
4. Ensure parent container has proper dimensions

### Styling Issues

**Problem:** Component styling looks wrong.

**Solutions:**
1. Ensure `main.css` is imported in `index.html`
2. Check for CSS conflicts with custom styles
3. Verify Tailwind classes are correct
4. Use browser DevTools to inspect styles

### Signal Not Updating

**Problem:** Component doesn't re-render when signal changes.

**Solutions:**
1. Use `move ||` closure to access signal: `move || signal.get()`
2. Ensure signal is created with `signal()` not `create_signal()`
3. Check if signal is being updated correctly: `set_signal.set(new_value)`

### Authentication Issues

**Problem:** User not redirected to login or session not persisting.

**Solutions:**
1. Verify `use_auth` hook is called in component
2. Check localStorage for `user_session` key
3. Ensure portal URL is configured correctly
4. Verify CORS settings allow cross-origin requests

---

## Support

Untuk bantuan lebih lanjut:

- **Documentation**: `/antarmuka/shared/README.md`
- **Quick Start**: `/antarmuka/shared/QUICK_START.md`
- **Integration Guide**: `/antarmuka/shared/INTEGRATION_GUIDE.md`
- **Email**: dev@kejaksaan.go.id

---

**Built with ❤️ by Tim Pengembang SIMPelv2**
**Kejaksaan Agung Republik Indonesia**
