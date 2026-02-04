# 🤖 AGENTS.md - Frontend (Microfrontends)

> **Context**: This directory (`antarmuka/`) contains the **Microfrontends** built with **Leptos 0.8 (WASM)**.
> All microfrontends are compiled to WASM and loaded by the browser.

## 🏗️ Architecture & Build

- **Trunk**: We use `trunk` to build.
  - Development: `trunk serve --open`
  - Production: `trunk build --release`
- **WASM Constraints**:
  - **Dependencies**: You CANNOT use crates that depend on C libraries (unless WASM-compatible) or direct OS system calls (filesystem, threads).
  - **Time**: Use `chrono` with `wasm-bindgen` feature (usually standard `chrono` is fine for formatting, but avoid `Local::now()` relying on system time zones if possible, stick to `Utc`).
  - **Config**: Check `Trunk.toml` for `wasm-opt` settings. If you see `i32.trunc_sat_f64_u` errors, enable `nontrapping-float-to-int`.
  - **Trunk.toml Syntax**: When configuring `wasm_opt`, use double brackets `[[build.wasm_opt]]` to avoid syntax errors, as it is a list of tables.

## 🦀 Leptos 0.8 Patterns (Strict Compliance)

### 1. ⚠️ Async Event Handlers & Cloning

When using `spawn_local` inside an event handler, you **MUST** clone signals/resources *outside* the async block if they are used inside.

❌ **WRONG (Will Panic or Compile Error):**
```rust
let navigate = use_navigate();
let on_submit = move |_| {
    spawn_local(async move {
        // `navigate` might be moved or dropped incorrectly if not handled
        navigate("/success", Default::default());
    });
};
```

✅ **CORRECT:**
```rust
let navigate = use_navigate();
let on_submit = move |_| {
    let navigate = navigate.clone(); // Clone for the async block
    spawn_local(async move {
        navigate("/success", Default::default());
    });
};
```

### 2. 🔄 Closures in `view!` (The `FnOnce` Trap)

When using `<For>`, `<Show>`, or other components that take a closure, if you use a non-`Copy` variable (like `String`, `Vec`) inside the closure, you **MUST** clone it *before* the closure definition or *inside* the `view!` block but outside the closure.

❌ **WRONG (Closure becomes FnOnce, but Fn required):**
```rust
let items = vec!["a".to_string(), "b".to_string()];
view! {
    <For
        each=move || items.clone() // ❌ items moved into closure here
        key=|i| i.clone()
        children=move |i| {
            // If we used `items` here again, it would be double-moved?
            // Actually, the issue is often subtle with nested closures.
            view! { ... }
        }
    />
}
```

✅ **CORRECT (Clone pattern):**
```rust
let items = vec!["a".to_string(), "b".to_string()];
// 1. Clone for the outer condition/loop
let items_for_show = items.clone();
let items_for_loop = items.clone();

view! {
    <Show when=move || !items_for_show.is_empty()>
        {
            // 2. Clone again for the inner loop if needed
            let items_inner = items_for_loop.clone();
            view! {
                <For
                    each=move || items_inner.clone()
                    // ...
                />
            }
        }
    </Show>
}
```

### 3. 📋 Lists vs Single Views

When returning views from control flow (match/if), types must match.

- **Lists**: Use `.collect_view()` on iterators.
- **Empty/Single**: Use `vec![view!{...}].into_view()` if matching against a list.

```rust
// In a Suspense result
move || match data.get() {
    Some(items) => items.into_iter()
        .map(|item| view! { <ItemRow item/> })
        .collect_view(), // Returns Vec<View>
    None => vec![view! { <EmptyState/> }].into_view(), // Wraps in Vec to match type
}
```

### 3. 🧩 Complex Views & Recursion

If you get "recursion limit reached" errors during macro expansion (common in big forms):
Add this to the top of `src/lib.rs` (or `main.rs`):
```rust
#![recursion_limit = "256"]
```

### 4. 🔄 Data Mapping (Backend -> Frontend)

The backend uses `Uuid`, `DateTime<Utc>`, etc.
For **JSON Serialization** to WASM:
- **UUIDs**: Map to `String` in your frontend structs.
- **Dates**: Map to `String` (ISO 8601) or ensure `serde` features are enabled for `chrono` in WASM.

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserFrontend {
    pub id: String, // Backend sends Uuid, we treat as String
    pub name: String,
    pub created_at: String, // ISO String
}
```

---

## 🧪 Testing (Playwright E2E)

We use Playwright for End-to-End testing.

### 1. 🔗 Relative URLs

**NEVER** hardcode `http://localhost:8080`. Use relative paths.

❌ `page.goto("http://localhost:8093/dashboard");`
✅ `page.goto("/dashboard");`

### 2. 🎯 Selectors

Prioritize **Accessibility** selectors.

1.  `page.getByRole('button', { name: 'Submit' })`
2.  `page.getByLabel('Username')` (Requires HTML `<label for="id"><input id="id">`)
3.  `page.getByPlaceholder('Enter text')`
4.  `page.getByTestId('submit-btn')` (Last resort)

### 3. 🎭 Mocking Network

Since backend might not be fully seeded, mock API responses for stability.

```javascript
await page.route('**/api/v1/users', async route => {
  await route.fulfill({
    status: 200,
    contentType: 'application/json',
    body: JSON.stringify([{ id: "1", name: "Mock User" }]),
  });
});
```

---

## 🧭 Routing

- Use `leptos_router`.
- Use `<a>` tags for internal links (Leptos intercepts them), NOT `<button on:click=navigate>`.
- **Navigation**: `use_navigate` is for programmatic navigation (after form submit).

```rust
view! {
    <nav>
        <a href="/dashboard">"Dashboard"</a> // Good
        <a href="/settings">"Settings"</a>
    </nav>
}
```
