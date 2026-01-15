# Contoh Penggunaan Validasi di Portal

## Overview

Dokumen ini menunjukkan cara menggunakan validasi frontend yang sudah sinkron dengan backend.

## Import Modules

```rust
use shared_microfrontend::utils::auth_validation::*;
use shared_microfrontend::core::types::*;
```

## 1. Login Form dengan Validasi

```rust
use leptos::prelude::*;
use shared_microfrontend::utils::auth_validation::*;

#[component]
pub fn LoginForm() -> impl IntoView {
    // Form state
    let (username, set_username) = signal(String::new());
    let (password, set_password) = signal(String::new());
    let (realm, set_realm) = signal("master".to_string());

    // Validation errors
    let (username_errors, set_username_errors) = signal(Vec::<String>::new());
    let (password_errors, set_password_errors) = signal(Vec::<String>::new());
    let (realm_errors, set_realm_errors) = signal(Vec::<String>::new());

    // Submit handler
    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();

        // Clear previous errors
        set_username_errors.set(Vec::new());
        set_password_errors.set(Vec::new());
        set_realm_errors.set(Vec::new());

        let mut has_errors = false;

        // Validate username
        let username_validation = validate_username(&username.get());
        if !username_validation.valid {
            set_username_errors.set(
                username_validation.errors.iter()
                    .map(|e| e.message.clone())
                    .collect()
            );
            has_errors = true;
        }

        // Validate password
        let password_validation = validate_password(&password.get());
        if !password_validation.valid {
            set_password_errors.set(
                password_validation.errors.iter()
                    .map(|e| e.message.clone())
                    .collect()
            );
            has_errors = true;
        }

        // Validate realm
        let realm_validation = validate_realm(&realm.get());
        if !realm_validation.valid {
            set_realm_errors.set(
                realm_validation.errors.iter()
                    .map(|e| e.message.clone())
                    .collect()
            );
            has_errors = true;
        }

        // If validation passes, submit to backend
        if !has_errors {
            // Call API
            spawn_local(async move {
                let response = api_login(
                    &username.get(),
                    &password.get(),
                    &realm.get()
                ).await;

                // Handle response
                match response {
                    Ok(data) => {
                        // Success - redirect or show success
                    }
                    Err(e) => {
                        // Backend validation error or network error
                        // Show error message
                    }
                }
            });
        }
    };

    view! {
        <form on:submit=on_submit class="space-y-4">
            // Username field
            <div>
                <label for="username" class="block text-sm font-medium">
                    "Username"
                </label>
                <input
                    type="text"
                    id="username"
                    prop:value=move || username.get()
                    on:input=move |ev| set_username.set(event_target_value(&ev))
                    class="mt-1 block w-full rounded-md border-gray-300"
                />
                <Show when=move || !username_errors.get().is_empty()>
                    <div class="mt-1 text-sm text-red-600">
                        {move || username_errors.get().join(", ")}
                    </div>
                </Show>
            </div>

            // Password field
            <div>
                <label for="password" class="block text-sm font-medium">
                    "Password"
                </label>
                <input
                    type="password"
                    id="password"
                    prop:value=move || password.get()
                    on:input=move |ev| set_password.set(event_target_value(&ev))
                    class="mt-1 block w-full rounded-md border-gray-300"
                />
                <Show when=move || !password_errors.get().is_empty()>
                    <div class="mt-1 text-sm text-red-600">
                        {move || password_errors.get().join(", ")}
                    </div>
                </Show>
            </div>

            // Submit button
            <button
                type="submit"
                class="w-full bg-primary text-white py-2 px-4 rounded-md hover:bg-primary-dark"
            >
                "Login"
            </button>
        </form>
    }
}
```

## 2. User Registration Form dengan Real-time Validation

```rust
use leptos::prelude::*;
use shared_microfrontend::utils::auth_validation::*;

#[component]
pub fn UserRegistrationForm() -> impl IntoView {
    let (username, set_username) = signal(String::new());
    let (email, set_email) = signal(String::new());
    let (password, set_password) = signal(String::new());
    let (satker_code, set_satker_code) = signal(String::new());
    let (nip, set_nip) = signal(String::new());

    // Real-time validation
    let username_validation = Memo::new(move |_| validate_username(&username.get()));
    let email_validation = Memo::new(move |_| validate_email(&email.get()));
    let password_validation = Memo::new(move |_| validate_password(&password.get()));
    let satker_validation = Memo::new(move |_| validate_satker_code(&satker_code.get()));
    let nip_validation = Memo::new(move |_| validate_nip(&nip.get()));

    // Password strength indicator
    let password_strength = Memo::new(move |_| {
        get_password_strength(&password.get())
    });

    // Form is valid when all fields are valid
    let form_valid = Memo::new(move |_| {
        username_validation.get().valid
            && email_validation.get().valid
            && password_validation.get().valid
            && satker_validation.get().valid
            && nip_validation.get().valid
    });

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();

        if form_valid.get() {
            // Submit to backend
            spawn_local(async move {
                // API call
            });
        }
    };

    view! {
        <form on:submit=on_submit class="space-y-4">
            // Username field with real-time validation
            <div>
                <label for="username">"Username"</label>
                <input
                    type="text"
                    id="username"
                    prop:value=move || username.get()
                    on:input=move |ev| set_username.set(event_target_value(&ev))
                    class=move || {
                        if username.get().is_empty() {
                            "border-gray-300"
                        } else if username_validation.get().valid {
                            "border-green-500"
                        } else {
                            "border-red-500"
                        }
                    }
                />
                <Show when=move || !username_validation.get().valid && !username.get().is_empty()>
                    <div class="text-sm text-red-600">
                        {move || username_validation.get().errors.first()
                            .map(|e| e.message.clone())
                            .unwrap_or_default()
                        }
                    </div>
                </Show>
            </div>

            // Password field with strength indicator
            <div>
                <label for="password">"Password"</label>
                <input
                    type="password"
                    id="password"
                    prop:value=move || password.get()
                    on:input=move |ev| set_password.set(event_target_value(&ev))
                />

                // Password strength indicator
                <Show when=move || !password.get().is_empty()>
                    <div class="mt-2">
                        <div class="flex items-center justify-between text-sm">
                            <span>"Kekuatan Password:"</span>
                            <span class=move || password_strength.get().2>
                                {move || password_strength.get().1}
                            </span>
                        </div>
                  <div class="mt-1 h-2 bg-gray-200 rounded-full overflow-hidden">
           <div
                                class=move || {
                                    let (level, _, _) = password_strength.get();
                                    match level {
                                        0..=1 => "bg-red-500",
                                        2 => "bg-yellow-500",
                                        3 => "bg-green-500",
                                        _ => "bg-green-600",
                                    }
                                }
                                style=move || {
                                    let (level, _, _) = password_strength.get();
                                    format!("width: {}%; transition: width 0.3s", level * 25)
                                }
                            />
                        </div>
                    </div>
                </Show>

                // Password validation errors
                <Show when=move || !password_validation.get().valid && !password.get().is_empty()>
                    <ul class="mt-2 text-sm text-red-600 space-y-1">
                        {move || password_validation.get().errors.iter().map(|e| {
                            view! { <li>"• " {&e.message}</li> }
                        }).collect::<Vec<_>>()}
                    </ul>
                </Show>
            </div>

            // Satker code field
            <div>
                <label for="satker_code">"Kode Satker"</label>
                <input
                    type="text"
                    id="satker_code"
                    prop:value=move || satker_code.get()
                    on:input=move |ev| {
                        // Auto-uppercase
                        let value = event_target_value(&ev).to_uppercase();
                        set_satker_code.set(value);
                    }
                    placeholder="KEJARI"
                    class=move || {
                        if satker_code.get().is_empty() {
                            "border-gray-300"
                        } else if satker_validation.get().valid {
                            "border-green-500"
                        } else {
                            "border-red-500"
                        }
                    }
                />
                <Show when=move || !satker_validation.get().valid && !satker_code.get().is_empty()>
                    <div class="text-sm text-red-600">
                        {move || satker_validation.get().errors.first()
                            .map(|e| e.message.clone())
                            .unwrap_or_default()
                        }
                    </div>
                </Show>
            </div>

            // Submit button
            <button
                type="submit"
                disabled=move || !form_valid.get()
                class="w-full bg-primary text-white py-2 px-4 rounded-md hover:bg-primary-dark disabled:opacity-50 disabled:cursor-not-allowed"
            >
                "Daftar"
            </button>
        </form>
    }
}
```

## 3. MFA Verification Form

```rust
use leptos::prelude::*;
use shared_microfrontend::utils::auth_validation::*;

#[component]
pub fn MfaVerificationForm(
    temp_token: String,
    on_success: Callback<String>,
) -> impl IntoView {
    let (code, set_code) = signal(String::new());
    let (error, set_error) = signal(None::<String>);
    let (submitting, set_submitting) = signal(false);

    // Real-time validation
    let code_validation = Memo::new(move |_| validate_mfa_code(&code.get()));

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();

        if !code_validation.get().valid {
            set_error.set(Some("Kode MFA tidak valid".to_string()));
            return;
        }

        set_submitting.set(true);
        set_error.set(None);

        let code_value = code.get();
        let token = temp_token.clone();

        spawn_local(async move {
            match api_verify_mfa(&token, &code_value).await {
                Ok(access_token) => {
                    on_success.run(access_token);
                }
                Err(e) => {
                    set_error.set(Some(e.to_string()));
                    set_submitting.set(false);
                }
            }
        });
    };

    view! {
        <form on:submit=on_submit class="space-y-4">
            <div>
                <label for="mfa_code" class="block text-sm font-medium">
                    "Kode MFA (6 digit)"
                </label>
                <input
                    type="text"
                    id="mfa_code"
                    prop:value=move || code.get()
                    on:input=move |ev| {
                        // Only allow digits, max 6 chars
                        let value = event_target_value(&ev)
                            .chars()
                            .filter(|c| c.is_numeric())
                            .take(6)
                            .collect::<String>();
                        set_code.set(value);
                    }
                    maxlength="6"
                    placeholder="123456"
                    class="mt-1 block w-full text-center text-2xl tracking-widest"
                    autofocus
                />

                <Show when=move || error.get().is_some()>
                    <div class="mt-2 text-sm text-red-600">
                        {move || error.get().unwrap_or_default()}
                    </div>
                </Show>
            </div>

            <button
                type="submit"
                disabled=move || !code_validation.get().valid || submitting.get()
                class="w-full bg-primary text-white py-2 px-4 rounded-md hover:bg-primary-dark disabled:opacity-50"
            >
                {move || if submitting.get() { "Memverifikasi..." } else { "Verifikasi" }}
            </button>
        </form>
    }
}
```

## 4. Reusable Validated Input Component

```rust
use leptos::prelude::*;
use shared_microfrontend::core::types::*;

#[component]
pub fn ValidatedInput(
    label: String,
    value: Signal<String>,
    set_value: WriteSignal<String>,
    validator: impl Fn(&str) -> ValidationResult + 'static,
    #[prop(optional)] input_type: String,
    #[prop(optional)] placeholder: String,
) -> impl IntoView {
    let input_type = if input_type.is_empty() { "text".to_string() } else { input_type };

    let validation = Memo::new(move |_| validator(&value.get()));

    view! {
        <div class="space-y-1">
            <label class="block text-sm font-medium text-gray-700">
                {label}
            </label>
            <input
                type=input_type
                prop:value=move || value.get()
                on:input=move |ev| set_value.set(event_target_value(&ev))
                placeholder=placeholder
                class=move || {
                    let base = "mt-1 block w-full rounded-md shadow-sm";
                    if value.get().is_empty() {
                        format!("{} border-gray-300", base)
                    } else if validation.get().valid {
                        format!("{} border-green-500 focus:border-green-500 focus:ring-green-500", base)
                    } else {
                        format!("{} border-red-500 focus:border-red-500 focus:ring-red-500", base)
                    }
                }
            />
            <Show when=move || !validation.get().valid && !value.get().is_empty()>
                <div class="text-sm text-red-600">
                    {move || validation.get().errors.first()
                        .map(|e| e.message.clone())
                        .unwrap_or_default()
                    }
                </div>
            </Show>
        </div>
    }
}

// Usage:
#[component]
pub fn ExampleForm() -> impl IntoView {
    let (username, set_username) = signal(String::new());
    let (email, set_email) = signal(String::new());

    view! {
        <form>
            <ValidatedInput
                label="Username".to_string()
                value=username
                set_value=set_username
                validator=validate_username
                placeholder="user123".to_string()
            />

            <ValidatedInput
                label="Email".to_string()
                value=email
                set_value=set_email
                validator=validate_email
                input_type="email".to_string()
                placeholder="user@example.com".to_string()
            />
        </form>
    }
}
```

## Best Practices

### 1. Validasi Real-time vs On-Submit

**Real-time** (saat user mengetik):
- ✅ Baik untuk: username, email, format fields
- ❌ Hindari untuk: password (mengganggu saat mengetik)

**On-submit** (saat form di-submit):
- ✅ Baik untuk: semua field
- ✅ Wajib untuk: validasi akhir sebelum kirim ke backend

### 2. Error Messages

- Gunakan bahasa Indonesia yang jelas
- Berikan contoh format yang benar
- Jangan terlalu teknis

### 3. Visual Feedback

- Border hijau untuk valid
- Border merah untuk invalid
- Gray/default untuk belum diisi
- Loading state saat submit

### 4. Accessibility

```rust
// Tambahkan aria attributes
<input
    aria-invalid=move || !validation.get().valid
    aria-describedby="username-error"
/>
<div id="username-error" role="alert">
    {error_message}
</div>
```

### 5. Backend Validation Handling

```rust
// Selalu handle backend validation errors
match api_call().await {
    Ok(data) => { /* success */ }
    Err(ApiError::ValidationError(errors)) => {
        // Backend validation failed
        // Show backend errors to user
        for error in errors {
            show_error(&error.field, &error.message);
        }
    }
    Err(e) => { /* other errors */ }
}
```

## Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_form_validation() {
        // Test validation logic
        let result = validate_username("test_user");
        assert!(result.valid);

        let result = validate_username("ab");
        assert!(!result.valid);
    }
}
```

## Kesimpulan

Dengan menggunakan validasi yang sudah sinkron antara frontend dan backend:

1. ✅ User mendapat feedback cepat (frontend validation)
2. ✅ Keamanan terjaga (backend validation)
3. ✅ Konsistensi error messages
4. ✅ Maintenance lebih mudah (satu source of truth)
5. ✅ Testing lebih mudah (test cases sama)
