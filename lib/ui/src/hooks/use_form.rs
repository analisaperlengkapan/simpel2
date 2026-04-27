//! Form state management hook — eliminates per-field signal boilerplate.
//!
//! Similar to `react-hook-form` + `Zod` / Laravel `FormRequest::rules()`.
//!
//! # Basic Usage
//!
//! ```rust,ignore
//! use lib_ui::hooks::use_form::use_form;
//!
//! #[derive(Clone, Default, PartialEq)]
//! struct MyForm {
//!     nama: String,
//!     kategori: String,
//! }
//!
//! #[component]
//! fn MyFormPage() -> impl IntoView {
//!     let form = use_form(MyForm::default());
//!
//!     view! {
//!         <input
//!             prop:value=move || form.get().nama.clone()
//!             on:input=move |ev| form.update(|f| f.nama = event_target_value(&ev))
//!         />
//!         <button
//!             prop:disabled=move || form.submitting.get()
//!             on:click=move |_| form.submit(|data| async move {
//!                 api::create(data).await.map_err(|e| e.user_message())
//!             })
//!         />
//!     }
//! }
//! ```
//!
//! # With Validation
//!
//! ```rust,ignore
//! use lib_ui::hooks::use_form::{use_form, FieldErrors};
//! use lib_ui::components::FieldError;
//!
//! let form = use_form(MyForm::default()).with_validator(|data| {
//!     let mut errors = FieldErrors::new();
//!     if data.nama.trim().len() < 3 {
//!         errors.add("nama", "Nama minimal 3 karakter");
//!     }
//!     if data.kategori.is_empty() {
//!         errors.add("kategori", "Kategori wajib dipilih");
//!     }
//!     errors
//! });
//!
//! view! {
//!     <input
//!         prop:value=move || form.get().nama.clone()
//!         on:input=move |ev| form.update(|f| f.nama = event_target_value(&ev))
//!         on:blur=move |_| form.validate_field("nama")
//!     />
//!     <FieldError form=form field="nama" />
//!
//!     <button on:click=move |_| {
//!         // begin_submit_validated runs validator first; returns None on failure
//!         // so we skip the API call and let <FieldError> render the messages.
//!         let Some(data) = form.begin_submit_validated() else { return };
//!         leptos::task::spawn_local(async move {
//!             match api::create(data).await {
//!                 Ok(_) => form.finish_ok(),
//!                 Err(e) => form.finish_err(e.to_string()),
//!             }
//!         });
//!     } />
//! }
//! ```

use leptos::prelude::*;
use crate::components::icon::AppIcon;
use phosphor_leptos::{WARNING_CIRCLE};
use std::collections::HashMap;
use std::sync::Arc;

/// Per-field validation errors, keyed by field name.
///
/// Similar to Laravel's `ValidationException::errors()` or Zod's `.flatten().fieldErrors`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FieldErrors {
    map: HashMap<String, Vec<String>>,
}

impl FieldErrors {
    /// Create an empty error collection.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an error message for a field. Multiple errors per field are supported.
    pub fn add(&mut self, field: impl Into<String>, message: impl Into<String>) {
        self.map.entry(field.into()).or_default().push(message.into());
    }

    /// Check if any field has errors.
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Check if a specific field has errors.
    pub fn has(&self, field: &str) -> bool {
        self.map.get(field).is_some_and(|v| !v.is_empty())
    }

    /// Get the first error message for a field (most common display case).
    pub fn first(&self, field: &str) -> Option<&str> {
        self.map.get(field).and_then(|v| v.first()).map(String::as_str)
    }

    /// Get all error messages for a field.
    pub fn get(&self, field: &str) -> Option<&[String]> {
        self.map.get(field).map(|v| v.as_slice())
    }

    /// Total number of error messages across all fields.
    pub fn count(&self) -> usize {
        self.map.values().map(Vec::len).sum()
    }
}

/// Type-erased validator function. Stored in `StoredValue` so `FormState`
/// stays `Copy` and can be moved into closures freely.
type ValidatorFn<T> = Arc<dyn Fn(&T) -> FieldErrors + Send + Sync + 'static>;

/// Reactive form state container.
///
/// Manages form data, submission state, error state, dirty tracking, and
/// per-field validation errors in a single struct — replacing 5-15
/// individual `signal()` calls.
// Clone + Copy are implemented manually so the bounds don't pick up
// `T: Copy`. `RwSignal<T>` and `StoredValue<T>` are already `Copy`
// regardless of `T` (they hold `Arc`'d state internally), so a typical
// form with `String` / `Vec<...>` fields can still keep `FormState`
// trivially copyable into closures.
pub struct FormState<T: Clone + Send + Sync + 'static> {
    /// The reactive form data. Read with `.get()`, write with `.update()`.
    pub data: RwSignal<T>,
    /// Whether a submission is in progress.
    pub submitting: RwSignal<bool>,
    /// Current top-level error message (e.g. API/network error, not field-level).
    pub error: RwSignal<Option<String>>,
    /// Whether the form was successfully submitted.
    pub success: RwSignal<bool>,
    /// Whether any field has been modified from the initial value.
    pub dirty: RwSignal<bool>,
    /// Per-field validation errors (from the validator closure).
    pub field_errors: RwSignal<FieldErrors>,
    /// The initial form data (for reset).
    initial: StoredValue<T>,
    /// Optional validator closure. Called on `validate()`, `validate_field()`,
    /// and `begin_submit_validated()`.
    validator: StoredValue<Option<ValidatorFn<T>>>,
}

impl<T: Clone + Send + Sync + 'static> Clone for FormState<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: Clone + Send + Sync + 'static> Copy for FormState<T> {}

impl<T: Clone + Send + Sync + 'static> FormState<T> {
    /// Get the current form data snapshot.
    pub fn get(&self) -> T {
        self.data.get()
    }

    /// Update the form data and mark as dirty.
    pub fn update(&self, f: impl FnOnce(&mut T)) {
        self.data.update(f);
        self.dirty.set(true);
    }

    /// Set a field value (convenience for simple assignments).
    pub fn set(&self, value: T) {
        self.data.set(value);
        self.dirty.set(true);
    }

    /// Load pre-existing data into the form **without** marking it dirty.
    ///
    /// Use this when populating an edit form from server data — the user
    /// hasn't actually modified anything yet, so `dirty` should stay `false`
    /// until they start typing. Distinct from `set()` which is for user edits.
    pub fn load(&self, value: T) {
        self.data.set(value);
        self.dirty.set(false);
    }

    /// Reset the form to its initial values and clear all state.
    pub fn reset(&self) {
        self.initial.with_value(|init| self.data.set(init.clone()));
        self.submitting.set(false);
        self.error.set(None);
        self.success.set(false);
        self.dirty.set(false);
        self.field_errors.set(FieldErrors::new());
    }

    /// Attach a validator closure. Returns `self` for chaining.
    ///
    /// The validator runs synchronously against the current form data
    /// when `validate()`, `validate_field()`, or `begin_submit_validated()` is called.
    ///
    /// ```rust,ignore
    /// let form = use_form(MyForm::default()).with_validator(|data| {
    ///     let mut errors = FieldErrors::new();
    ///     if data.nama.len() < 3 {
    ///         errors.add("nama", "Min 3 karakter");
    ///     }
    ///     errors
    /// });
    /// ```
    pub fn with_validator<F>(self, validator: F) -> Self
    where
        F: Fn(&T) -> FieldErrors + Send + Sync + 'static,
    {
        self.validator
            .set_value(Some(Arc::new(validator) as ValidatorFn<T>));
        self
    }

    /// Run the validator against current data and populate `field_errors`.
    ///
    /// Returns `true` if the form is valid (no errors), `false` otherwise.
    /// If no validator is attached, always returns `true`.
    pub fn validate(&self) -> bool {
        let validator = self.validator.with_value(|v| v.clone());
        match validator {
            Some(f) => {
                let errors = self.data.with_untracked(|d| f(d));
                let valid = errors.is_empty();
                self.field_errors.set(errors);
                valid
            }
            None => {
                self.field_errors.set(FieldErrors::new());
                true
            }
        }
    }

    /// Validate a single field and update `field_errors` for that field only.
    ///
    /// Useful for `on:blur` validation — other fields' existing errors are preserved.
    pub fn validate_field(&self, field: &str) {
        let validator = self.validator.with_value(|v| v.clone());
        if let Some(f) = validator {
            let fresh = self.data.with_untracked(|d| f(d));
            self.field_errors.update(|current| {
                // Remove any prior errors for this field
                current.map.remove(field);
                // Re-add fresh errors for this field only
                if let Some(msgs) = fresh.map.get(field) {
                    current.map.insert(field.to_string(), msgs.clone());
                }
            });
        }
    }

    /// Clear the error message.
    pub fn clear_error(&self) {
        self.error.set(None);
    }

    /// Set an error message.
    pub fn set_error(&self, msg: impl Into<String>) {
        self.error.set(Some(msg.into()));
    }

    /// Begin a submission. Sets `submitting=true`, clears error/success.
    ///
    /// Returns the current form data for use in the async handler.
    /// Call `finish_ok()` or `finish_err()` when done.
    pub fn begin_submit(&self) -> T {
        self.submitting.set(true);
        self.error.set(None);
        self.success.set(false);
        self.data.get()
    }

    /// Mark submission as successful.
    pub fn finish_ok(&self) {
        self.submitting.set(false);
        self.success.set(true);
    }

    /// Mark submission as failed with an error message.
    pub fn finish_err(&self, msg: impl Into<String>) {
        self.submitting.set(false);
        self.error.set(Some(msg.into()));
    }

    /// Begin a submission **only if validation passes**.
    ///
    /// Runs the validator first. On failure, populates `field_errors` and returns
    /// `None` (caller should skip the async handler). On success, returns a data
    /// snapshot like `begin_submit()`.
    ///
    /// ```rust,ignore
    /// let Some(data) = form.begin_submit_validated() else { return };
    /// // ... call API with data, then form.finish_ok() / finish_err()
    /// ```
    pub fn begin_submit_validated(&self) -> Option<T> {
        if !self.validate() {
            return None;
        }
        Some(self.begin_submit())
    }
}

/// Create a new form state with the given initial values.
///
/// # Example
///
/// ```rust,ignore
/// let form = use_form(CreateItemRequest {
///     nama: String::new(),
///     kategori: String::new(),
/// });
/// ```
pub fn use_form<T: Clone + Send + Sync + 'static>(initial: T) -> FormState<T> {
    FormState {
        data: RwSignal::new(initial.clone()),
        initial: StoredValue::new(initial),
        submitting: RwSignal::new(false),
        error: RwSignal::new(None),
        success: RwSignal::new(false),
        dirty: RwSignal::new(false),
        field_errors: RwSignal::new(FieldErrors::new()),
        validator: StoredValue::new(None),
    }
}

// ============================================================================
// FIELD ERROR COMPONENT — renders the first validation error for a field
// ============================================================================

/// Displays the first validation error for a specific field.
///
/// Renders nothing when the field has no errors, making it safe to place
/// below every input unconditionally.
///
/// ```rust,ignore
/// <input on:blur=move |_| form.validate_field("nama") ... />
/// <FieldError form=form field="nama" />
/// ```
#[component]
pub fn FieldError<T>(
    /// The form state to read field errors from.
    form: FormState<T>,
    /// The field name to display errors for.
    #[prop(into)]
    field: String,
) -> impl IntoView
where
    T: Clone + Send + Sync + 'static,
{
    let field = StoredValue::new(field);
    let message = move || {
        form.field_errors.with(|errors| {
            field.with_value(|f| errors.first(f).map(str::to_string))
        })
    };

    view! {
        <Show when=move || message().is_some()>
            <p class="mt-1 flex items-center gap-1.5 text-xs text-red-400">
                <span class="text-[0.7rem]"><AppIcon icon=WARNING_CIRCLE /></span>
                {move || message().unwrap_or_default()}
            </p>
        </Show>
    }
}
