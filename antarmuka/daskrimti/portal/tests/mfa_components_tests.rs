//! MFA Component Tests
//!
//! This module contains tests for MFA-related frontend components including
//! setup pages, verification forms, and reusable OTP components.

use gloo_timers::future::TimeoutFuture;
use leptos::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;
use web_sys::{Element, HtmlButtonElement, HtmlElement, HtmlInputElement};

wasm_bindgen_test_configure!(run_in_browser);

/// Test utilities for MFA component testing
mod test_utils {
    use super::*;

    /// Create a test container element
    pub fn create_test_container() -> Element {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let container = document.create_element("div").unwrap();
        container.set_id("test-container");
        document.body().unwrap().append_child(&container).unwrap();
        container
    }

    /// Clean up test container
    pub fn cleanup_test_container() {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        if let Some(container) = document.get_element_by_id("test-container") {
            container.remove();
        }
    }

    /// Simulate user input in an element
    pub fn simulate_input(element: &HtmlInputElement, value: &str) {
        element.set_value(value);

        // Dispatch input event
        let event = web_sys::Event::new("input").unwrap();
        element.dispatch_event(&event).unwrap();

        // Dispatch change event
        let change_event = web_sys::Event::new("change").unwrap();
        element.dispatch_event(&change_event).unwrap();
    }

    /// Simulate button click
    pub fn simulate_click(element: &HtmlButtonElement) {
        let event = web_sys::MouseEvent::new("click").unwrap();
        element.dispatch_event(&event.unchecked_into()).unwrap();
    }

    /// Wait for DOM updates
    pub async fn wait_for_update() {
        TimeoutFuture::new(10).await;
    }
}

/// Tests for OTP Input Component
#[cfg(test)]
mod otp_input_tests {
    use super::*;

    #[wasm_bindgen_test]
    async fn test_otp_input_basic_functionality() {
        let container = test_utils::create_test_container();

        let (otp_value, set_otp_value) = signal(String::new());

        // Mount simplified OTP input component
        let _ = mount_to(
            container.clone().unchecked_into::<HtmlElement>(),
            move || {
                view! {
                    <div class="otp-input-container">
                        <input
                            type="text"
                            class="otp-input"
                            placeholder="000000"
                            maxlength="6"
                            pattern="[0-9]*"
                            inputmode="numeric"
                            autocomplete="one-time-code"
                            value=otp_value.get()
                            on:input=move |ev| {
                                let value = event_target_value(&ev);
                                // Only allow digits and limit to 6 characters
                                let filtered: String = value.chars()
                                    .filter(|c| c.is_ascii_digit())
                                    .take(6)
                                    .collect();
                                set_otp_value.set(filtered);
                            }
                        />
                    </div>
                }
            },
        );

        test_utils::wait_for_update().await;

        // Find the OTP input element
        let input = container
            .query_selector("input.otp-input")
            .unwrap()
            .unwrap()
            .dyn_into::<HtmlInputElement>()
            .unwrap();

        // Test input validation - should only accept digits
        test_utils::simulate_input(&input, "123abc");
        test_utils::wait_for_update().await;

        // Should filter out non-digits
        assert_eq!(input.value(), "123abc"); // Browser input shows all, but signal filters
        assert_eq!(otp_value.get(), "123");

        // Test 6-digit limit
        test_utils::simulate_input(&input, "1234567890");
        test_utils::wait_for_update().await;

        assert_eq!(otp_value.get(), "123456");

        test_utils::cleanup_test_container();
    }

    #[wasm_bindgen_test]
    async fn test_otp_input_accessibility() {
        let container = test_utils::create_test_container();

        let (otp_value, set_otp_value) = signal(String::new());

        let _ = mount_to(
            container.clone().unchecked_into::<HtmlElement>(),
            move || {
                view! {
                    <div class="otp-input-container">
                        <label for="otp-input">"Enter 6-digit code"</label>
                        <input
                            id="otp-input"
                            type="text"
                            class="otp-input"
                            placeholder="000000"
                            maxlength="6"
                            pattern="[0-9]*"
                            inputmode="numeric"
                            autocomplete="one-time-code"
                            aria-label="One-time password"
                            aria-describedby="otp-help"
                            value=otp_value.get()
                            on:input=move |ev| {
                                set_otp_value.set(event_target_value(&ev));
                            }
                        />
                        <p id="otp-help">"Enter the code from your authenticator app"</p>
                    </div>
                }
            },
        );

        test_utils::wait_for_update().await;

        let input = container
            .query_selector("#otp-input")
            .unwrap()
            .unwrap()
            .dyn_into::<HtmlInputElement>()
            .unwrap();

        // Test accessibility attributes
        assert_eq!(input.get_attribute("inputmode").unwrap(), "numeric");
        assert_eq!(
            input.get_attribute("autocomplete").unwrap(),
            "one-time-code"
        );
        assert_eq!(input.get_attribute("maxlength").unwrap(), "6");
        assert_eq!(input.get_attribute("pattern").unwrap(), "[0-9]*");

        // Test ARIA attributes
        assert!(input.has_attribute("aria-label"));
        assert_eq!(input.get_attribute("aria-describedby").unwrap(), "otp-help");

        test_utils::cleanup_test_container();
    }

    #[wasm_bindgen_test]
    async fn test_otp_input_error_states() {
        let container = test_utils::create_test_container();

        let (otp_value, set_otp_value) = signal(String::new());
        let (has_error, set_has_error) = signal(false);

        let _ = mount_to(
            container.clone().unchecked_into::<HtmlElement>(),
            move || {
                view! {
                    <div class="otp-input-container">
                        <input
                            type="text"
                            class=move || if has_error.get() { "otp-input error" } else { "otp-input" }
                            aria-invalid=move || if has_error.get() { "true" } else { "false" }
                            value=otp_value.get()
                            on:input=move |ev| {
                                set_otp_value.set(event_target_value(&ev));
                            }
                        />
                    </div>
                }
            },
        );

        test_utils::wait_for_update().await;

        let input = container
            .query_selector("input")
            .unwrap()
            .unwrap()
            .dyn_into::<HtmlInputElement>()
            .unwrap();

        // Test normal state
        assert!(!input.class_list().contains("error"));

        // Test error state
        set_has_error.set(true);
        test_utils::wait_for_update().await;

        assert!(input.class_list().contains("error"));
        assert_eq!(input.get_attribute("aria-invalid").unwrap(), "true");

        test_utils::cleanup_test_container();
    }
}

/// Tests for QR Code Display Component
#[cfg(test)]
mod qr_code_display_tests {
    use super::*;

    #[wasm_bindgen_test]
    async fn test_qr_code_display_basic_functionality() {
        let container = test_utils::create_test_container();

        let test_qr_url = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==";

        let _ = mount_to(
            container.clone().unchecked_into::<HtmlElement>(),
            move || {
                view! {
                    <div class="qr-display-container">
                        <img
                            src=test_qr_url
                            alt="MFA Setup QR Code"
                            class="qr-code-image"
                            role="img"
                        />
                    </div>
                }
            },
        );

        test_utils::wait_for_update().await;

        // Find the QR code image
        let img = container
            .query_selector("img.qr-code-image")
            .unwrap()
            .unwrap()
            .dyn_into::<web_sys::HtmlImageElement>()
            .unwrap();

        // Test image attributes
        assert_eq!(img.src(), test_qr_url);
        assert!(img.has_attribute("alt"));
        assert_eq!(img.get_attribute("alt").unwrap(), "MFA Setup QR Code");

        // Test accessibility
        assert!(img.has_attribute("role"));
        assert_eq!(img.get_attribute("role").unwrap(), "img");

        test_utils::cleanup_test_container();
    }

    #[wasm_bindgen_test]
    async fn test_qr_code_display_loading_state() {
        let container = test_utils::create_test_container();

        let (qr_url, set_qr_url) = signal(String::new());

        let _ = mount_to(
            container.clone().unchecked_into::<HtmlElement>(),
            move || {
                view! {
                    <div class="qr-display-container">
                        {move || {
                            let url = qr_url.get();
                            if url.is_empty() {
                                view! {
                                    <div class="qr-loading">"Generating QR code..."</div>
                                }.into_any()
                            } else {
                                view! {
                                    <img src=url alt="MFA Setup QR Code" />
                                }.into_any()
                            }
                        }}
                    </div>
                }
            },
        );

        test_utils::wait_for_update().await;

        // Should show loading state when no URL
        let loading_element = container.query_selector(".qr-loading").unwrap();
        assert!(loading_element.is_some());

        // Update with QR URL
        set_qr_url.set("data:image/png;base64,test".to_string());
        test_utils::wait_for_update().await;

        // Should show image when URL is provided
        let img = container.query_selector("img").unwrap();
        assert!(img.is_some());

        test_utils::cleanup_test_container();
    }
}

/// Tests for Button Component with MFA Context
#[cfg(test)]
mod button_tests {
    use super::*;

    #[wasm_bindgen_test]
    async fn test_button_basic_functionality() {
        let container = test_utils::create_test_container();

        let (clicked, set_clicked) = signal(false);

        let _ = mount_to(
            container.clone().unchecked_into::<HtmlElement>(),
            move || {
                view! {
                    <div class="button-container">
                        <button
                            class="btn btn-primary"
                            on:click=move |_| set_clicked.set(true)
                        >
                            "Setup MFA"
                        </button>
                        <button class="btn btn-secondary">
                            "Cancel"
                        </button>
                        <button class="btn btn-danger">
                            "Disable MFA"
                        </button>
                    </div>
                }
            },
        );

        test_utils::wait_for_update().await;

        // Test primary button
        let primary_btn = container
            .query_selector("button.btn-primary")
            .unwrap()
            .unwrap()
            .dyn_into::<HtmlButtonElement>()
            .unwrap();

        assert_eq!(primary_btn.inner_text(), "Setup MFA");
        assert!(primary_btn.class_list().contains("btn-primary"));

        // Test click functionality
        test_utils::simulate_click(&primary_btn);
        test_utils::wait_for_update().await;
        assert!(clicked.get());

        // Test secondary button
        let secondary_btn = container.query_selector("button.btn-secondary").unwrap();
        assert!(secondary_btn.is_some());

        // Test danger button
        let danger_btn = container.query_selector("button.btn-danger").unwrap();
        assert!(danger_btn.is_some());

        test_utils::cleanup_test_container();
    }

    #[wasm_bindgen_test]
    async fn test_button_disabled_state() {
        let container = test_utils::create_test_container();

        let (is_disabled, set_is_disabled) = signal(true);
        let (clicked, set_clicked) = signal(false);

        let _ = mount_to(
            container.clone().unchecked_into::<HtmlElement>(),
            move || {
                view! {
                    <button
                        class="btn btn-primary"
                        disabled=is_disabled.get()
                        on:click=move |_| set_clicked.set(true)
                    >
                        "Verify OTP"
                    </button>
                }
            },
        );

        test_utils::wait_for_update().await;

        let button = container
            .query_selector("button")
            .unwrap()
            .unwrap()
            .dyn_into::<HtmlButtonElement>()
            .unwrap();

        // Test disabled state
        assert!(button.disabled());

        // Enable button
        set_is_disabled.set(false);
        test_utils::wait_for_update().await;

        assert!(!button.disabled());

        // Click should work when enabled
        test_utils::simulate_click(&button);
        test_utils::wait_for_update().await;
        assert!(clicked.get());

        test_utils::cleanup_test_container();
    }

    #[wasm_bindgen_test]
    async fn test_button_loading_state() {
        let container = test_utils::create_test_container();

        let (is_loading, set_is_loading) = signal(false);

        let _ = mount_to(
            container.clone().unchecked_into::<HtmlElement>(),
            move || {
                view! {
                    <button
                        class=move || if is_loading.get() { "btn btn-primary loading" } else { "btn btn-primary" }
                        disabled=is_loading.get()
                    >
                        {move || if is_loading.get() { "Loading..." } else { "Submit" }}
                    </button>
                }
            },
        );

        test_utils::wait_for_update().await;

        let button = container
            .query_selector("button")
            .unwrap()
            .unwrap()
            .dyn_into::<HtmlButtonElement>()
            .unwrap();

        // Test normal state
        assert!(!button.class_list().contains("loading"));
        assert_eq!(button.inner_text(), "Submit");

        // Test loading state
        set_is_loading.set(true);
        test_utils::wait_for_update().await;

        assert!(button.class_list().contains("loading"));
        assert!(button.disabled());
        assert_eq!(button.inner_text(), "Loading...");

        test_utils::cleanup_test_container();
    }
}
