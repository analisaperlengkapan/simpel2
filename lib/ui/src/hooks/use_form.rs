//! Form state management hook — eliminates per-field signal boilerplate.
//!
//! Similar to `react-hook-form` / Laravel `$request->validate()`.
//!
//! # Usage
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

use leptos::prelude::*;

/// Reactive form state container.
///
/// Manages form data, submission state, error state, and dirty tracking
/// in a single struct — replacing 5-15 individual `signal()` calls.
#[derive(Clone, Copy)]
pub struct FormState<T: Clone + 'static> {
    /// The reactive form data. Read with `.get()`, write with `.update()`.
    pub data: RwSignal<T>,
    /// Whether a submission is in progress.
    pub submitting: RwSignal<bool>,
    /// Current error message (if any).
    pub error: RwSignal<Option<String>>,
    /// Whether the form was successfully submitted.
    pub success: RwSignal<bool>,
    /// Whether any field has been modified from the initial value.
    pub dirty: RwSignal<bool>,
    /// The initial form data (for reset).
    initial: StoredValue<T>,
}

impl<T: Clone + 'static> FormState<T> {
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

    /// Reset the form to its initial values and clear all state.
    pub fn reset(&self) {
        self.initial.with_value(|init| self.data.set(init.clone()));
        self.submitting.set(false);
        self.error.set(None);
        self.success.set(false);
        self.dirty.set(false);
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
pub fn use_form<T: Clone + 'static>(initial: T) -> FormState<T> {
    FormState {
        data: RwSignal::new(initial.clone()),
        initial: StoredValue::new(initial),
        submitting: RwSignal::new(false),
        error: RwSignal::new(None),
        success: RwSignal::new(false),
        dirty: RwSignal::new(false),
    }
}
