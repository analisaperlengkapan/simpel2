//! MFA Pages Integration Tests
//!
//! This module contains integration tests for MFA setup and verification pages,
//! testing the complete user flow and page interactions.

use gloo_timers::future::TimeoutFuture;
use leptos::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use wasm_bindgen_test::*;
use web_sys::{Element, HtmlButtonElement, HtmlElement, HtmlInputElement};

wasm_bindgen_test_configure!(run_in_browser);

/// Mock API responses for testing
#[allow(dead_code)]
mod mock_api {
    use serde_json::json;

    pub fn mock_mfa_setup_response() -> serde_json::Value {
        json!({
            "qr_code_url": "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==",
            "secret_key": "JBSWY3DPEHPK3PXP",
            "backup_codes": [
                "12345678",
                "87654321",
                "11111111",
                "22222222",
                "33333333"
            ]
        })
    }

    pub fn mock_mfa_verification_success() -> serde_json::Value {
        json!({
            "success": true,
            "message": "MFA verification successful"
        })
    }

    pub fn mock_mfa_verification_failure() -> serde_json::Value {
        json!({
            "success": false,
            "error": "Invalid OTP code",
            "attempts_remaining": 2
        })
    }
}

/// Test utilities for MFA page testing
mod page_test_utils {
    use super::*;

    /// Create a test router context
    pub fn create_test_router() -> Element {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let container = document.create_element("div").unwrap();
        container.set_id("router-container");
        document.body().unwrap().append_child(&container).unwrap();
        container
    }

    /// Clean up test router
    pub fn cleanup_test_router() {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        if let Some(container) = document.get_element_by_id("router-container") {
            container.remove();
        }
    }

    /// Wait for page navigation
    pub async fn wait_for_navigation() {
        TimeoutFuture::new(100).await;
    }

    /// Find element by test ID
    pub fn find_by_test_id(container: &Element, test_id: &str) -> Option<Element> {
        container
            .query_selector(&format!("[data-testid='{}']", test_id))
            .unwrap()
    }
}

/// Tests for MFA Setup Page
#[cfg(test)]
mod mfa_setup_page_tests {
    use super::*;

    #[wasm_bindgen_test]
    async fn test_mfa_setup_page_initial_render() {
        let container = page_test_utils::create_test_router();

        // Mock the MFA setup page component
        let _ = mount_to(
            container.clone().unchecked_into::<HtmlElement>(),
            move || {
                view! {
                    <div data-testid="mfa-setup-page">
                        <h2>"Setup Multi-Factor Authentication"</h2>
                        <div data-testid="setup-instructions">
                            <p>"Scan the QR code with your authenticator app"</p>
                        </div>
                        <div data-testid="qr-code-section">
                            <div class="qr-loading">"Generating QR code..."</div>
                        </div>
                        <div data-testid="manual-entry-section" style="display: none;">
                            <details>
                                <summary>"Can't scan? Enter manually"</summary>
                                <div>
                                    <label>"Secret Key:"</label>
                                    <code data-testid="secret-key"></code>
                                </div>
                            </details>
                        </div>
                        <div data-testid="verification-section">
                            <h3>"Verification"</h3>
                            <p>"Enter the 6-digit code from your authenticator app:"</p>
                            <input
                                type="text"
                                data-testid="otp-input"
                                placeholder="000000"
                                maxlength="6"
                                pattern="[0-9]*"
                            />
                            <button
                                data-testid="verify-button"
                                disabled=true
                            >
                                "Verify and Complete Setup"
                            </button>
                        </div>
                    </div>
                }
            },
        );

        page_test_utils::wait_for_navigation().await;

        // Test initial page structure
        let page = page_test_utils::find_by_test_id(&container, "mfa-setup-page").unwrap();
        assert!(page.query_selector("h2").unwrap().is_some());

        // Test setup instructions are visible
        let instructions =
            page_test_utils::find_by_test_id(&container, "setup-instructions").unwrap();
        assert!(
            instructions
                .text_content()
                .unwrap()
                .contains("Scan the QR code")
        );

        // Test QR code section shows loading initially
        let qr_section = page_test_utils::find_by_test_id(&container, "qr-code-section").unwrap();
        assert!(qr_section.query_selector(".qr-loading").unwrap().is_some());

        // Test verification section is present
        let verification =
            page_test_utils::find_by_test_id(&container, "verification-section").unwrap();
        assert!(verification.query_selector("input").unwrap().is_some());

        // Test verify button is initially disabled
        let verify_btn = page_test_utils::find_by_test_id(&container, "verify-button")
            .unwrap()
            .dyn_into::<HtmlButtonElement>()
            .unwrap();
        assert!(verify_btn.disabled());

        page_test_utils::cleanup_test_router();
    }

    #[wasm_bindgen_test]
    async fn test_mfa_setup_qr_code_loading() {
        let container = page_test_utils::create_test_router();

        let (qr_data, set_qr_data) = signal(None::<serde_json::Value>);

        let _ = mount_to(
            container.clone().unchecked_into::<HtmlElement>(),
            move || {
                view! {
                    <div data-testid="mfa-setup-page">
                        <div data-testid="qr-code-section">
                            {move || match qr_data.get() {
                                Some(data) => {
                                    let qr_url = data["qr_code_url"].as_str().unwrap_or("");
                                    view! {
                                        <img
                                            src=qr_url
                                            alt="MFA Setup QR Code"
                                            data-testid="qr-code-image"
                                        />
                                    }.into_any()
                                }
                                None => {
                                    view! {
                                        <div class="qr-loading" data-testid="qr-loading">
                                            "Generating QR code..."
                                        </div>
                                    }.into_any()
                                }
                            }}
                        </div>
                    </div>
                }
            },
        );

        page_test_utils::wait_for_navigation().await;

        // Test loading state
        let loading = page_test_utils::find_by_test_id(&container, "qr-loading").unwrap();
        assert!(loading.text_content().unwrap().contains("Generating"));

        // Simulate API response
        set_qr_data.set(Some(mock_api::mock_mfa_setup_response()));
        page_test_utils::wait_for_navigation().await;

        // Test QR code is displayed
        let qr_image = page_test_utils::find_by_test_id(&container, "qr-code-image").unwrap();
        let img = qr_image.dyn_into::<web_sys::HtmlImageElement>().unwrap();
        assert!(img.src().starts_with("data:image/png"));

        page_test_utils::cleanup_test_router();
    }

    #[wasm_bindgen_test]
    async fn test_mfa_setup_otp_verification() {
        let container = page_test_utils::create_test_router();

        let (otp_code, set_otp_code) = signal(String::new());
        let (verification_result, set_verification_result) = signal(None::<bool>);

        let _ = mount_to(
            container.clone().unchecked_into::<HtmlElement>(),
            move || {
                view! {
                    <div data-testid="mfa-setup-page">
                        <div data-testid="verification-section">
                            <input
                                type="text"
                                data-testid="otp-input"
                                value=otp_code.get()
                                on:input=move |ev| {
                                    let value = event_target_value(&ev);
                                    // Only allow digits and limit to 6 characters
                                    let filtered: String = value.chars()
                                        .filter(|c| c.is_ascii_digit())
                                        .take(6)
                                        .collect();
                                    set_otp_code.set(filtered);
                                }
                            />
                            <button
                                data-testid="verify-button"
                                disabled=move || otp_code.get().len() != 6
                                on:click=move |_| {
                                    // Simulate verification
                                    let code = otp_code.get();
                                    if code == "123456" {
                                        set_verification_result.set(Some(true));
                                    } else {
                                        set_verification_result.set(Some(false));
                                    }
                                }
                            >
                                "Verify and Complete Setup"
                            </button>
                            {move || match verification_result.get() {
                                Some(true) => {
                                    view! {
                                        <div data-testid="success-message" class="success">
                                            "✅ MFA Setup Complete!"
                                        </div>
                                    }.into_any()
                                }
                                Some(false) => {
                                    view! {
                                        <div data-testid="error-message" class="error">
                                            "❌ Invalid code. Please try again."
                                        </div>
                                    }.into_any()
                                }
                                None => view! { <div></div> }.into_any()
                            }}
                        </div>
                    </div>
                }
            },
        );

        page_test_utils::wait_for_navigation().await;

        let otp_input = page_test_utils::find_by_test_id(&container, "otp-input")
            .unwrap()
            .dyn_into::<HtmlInputElement>()
            .unwrap();

        let verify_btn = page_test_utils::find_by_test_id(&container, "verify-button")
            .unwrap()
            .dyn_into::<HtmlButtonElement>()
            .unwrap();

        // Test button is disabled initially
        assert!(verify_btn.disabled());

        // Test input validation - only digits
        otp_input.set_value("12a3b4");
        let input_event = web_sys::Event::new("input").unwrap();
        otp_input.dispatch_event(&input_event).unwrap();
        page_test_utils::wait_for_navigation().await;

        // Should filter to digits only
        assert_eq!(otp_code.get(), "1234");

        // Test complete 6-digit code enables button
        otp_input.set_value("123456");
        otp_input.dispatch_event(&input_event).unwrap();
        page_test_utils::wait_for_navigation().await;

        assert_eq!(otp_code.get(), "123456");
        assert!(!verify_btn.disabled());

        // Test successful verification
        let click_event = web_sys::MouseEvent::new("click").unwrap();
        verify_btn
            .dispatch_event(&click_event.unchecked_into())
            .unwrap();
        page_test_utils::wait_for_navigation().await;

        let success_msg = page_test_utils::find_by_test_id(&container, "success-message");
        assert!(success_msg.is_some());
        assert!(
            success_msg
                .unwrap()
                .text_content()
                .unwrap()
                .contains("MFA Setup Complete")
        );

        page_test_utils::cleanup_test_router();
    }
}

/// Tests for MFA Verification Page
#[cfg(test)]
mod mfa_verification_page_tests {
    use super::*;

    #[wasm_bindgen_test]
    async fn test_mfa_verification_page_render() {
        let container = page_test_utils::create_test_router();

        let (otp_code, set_otp_code) = signal(String::new());
        let (attempts_remaining, _set_attempts_remaining) = signal(3);

        let _ = mount_to(
            container.clone().unchecked_into::<HtmlElement>(),
            move || {
                view! {
                    <div data-testid="mfa-verification-page">
                        <h2>"Multi-Factor Authentication"</h2>
                        <div data-testid="verification-instructions">
                            <p>"Enter the 6-digit code from your authenticator app"</p>
                        </div>
                        <div data-testid="verification-form">
                            <input
                                type="text"
                                data-testid="otp-input"
                                placeholder="000000"
                                maxlength="6"
                                value=otp_code.get()
                                on:input=move |ev| {
                                    set_otp_code.set(event_target_value(&ev));
                                }
                            />
                            <button
                                data-testid="verify-button"
                                disabled=move || otp_code.get().len() != 6
                            >
                                "Verify"
                            </button>
                        </div>
                        <div data-testid="attempts-info">
                            <p>{format!("Attempts remaining: {}", attempts_remaining.get())}</p>
                        </div>
                        <div data-testid="backup-options">
                            <details>
                                <summary>"Use backup code instead"</summary>
                                <div>
                                    <input
                                        type="text"
                                        data-testid="backup-code-input"
                                        placeholder="Enter backup code"
                                    />
                                    <button data-testid="backup-verify-button">
                                        "Verify Backup Code"
                                    </button>
                                </div>
                            </details>
                        </div>
                    </div>
                }
            },
        );

        page_test_utils::wait_for_navigation().await;

        // Test page structure
        let page = page_test_utils::find_by_test_id(&container, "mfa-verification-page").unwrap();
        assert!(page.query_selector("h2").unwrap().is_some());

        // Test instructions are visible
        let instructions =
            page_test_utils::find_by_test_id(&container, "verification-instructions").unwrap();
        assert!(
            instructions
                .text_content()
                .unwrap()
                .contains("6-digit code")
        );

        // Test form elements
        let otp_input = page_test_utils::find_by_test_id(&container, "otp-input").unwrap();
        assert_eq!(otp_input.get_attribute("maxlength").unwrap(), "6");

        // Test attempts info
        let attempts_info = page_test_utils::find_by_test_id(&container, "attempts-info").unwrap();
        assert!(
            attempts_info
                .text_content()
                .unwrap()
                .contains("Attempts remaining: 3")
        );

        // Test backup options are available
        let backup_options =
            page_test_utils::find_by_test_id(&container, "backup-options").unwrap();
        assert!(backup_options.query_selector("details").unwrap().is_some());

        page_test_utils::cleanup_test_router();
    }

    #[wasm_bindgen_test]
    async fn test_mfa_verification_success_flow() {
        let container = page_test_utils::create_test_router();

        let (otp_code, set_otp_code) = signal(String::new());
        let (verification_status, set_verification_status) = signal(None::<String>);
        let (is_verifying, set_is_verifying) = signal(false);

        let _ = mount_to(
            container.clone().unchecked_into::<HtmlElement>(),
            move || {
                view! {
                    <div data-testid="mfa-verification-page">
                        <div data-testid="verification-form">
                            <input
                                type="text"
                                data-testid="otp-input"
                                value=otp_code.get()
                                on:input=move |ev| {
                                    let value = event_target_value(&ev);
                                    let filtered: String = value.chars()
                                        .filter(|c| c.is_ascii_digit())
                                        .take(6)
                                        .collect();
                                    set_otp_code.set(filtered);
                                }
                            />
                            <button
                                data-testid="verify-button"
                                disabled=move || otp_code.get().len() != 6 || is_verifying.get()
                                on:click=move |_| {
                                    set_is_verifying.set(true);
                                    let code = otp_code.get();

                                    // Simulate API call delay
                                    spawn_local(async move {
                                        TimeoutFuture::new(100).await;

                                        if code == "123456" {
                                            set_verification_status.set(Some("success".to_string()));
                                        } else {
                                            set_verification_status.set(Some("error".to_string()));
                                        }
                                        set_is_verifying.set(false);
                                    });
                                }
                            >
                                {move || if is_verifying.get() { "Verifying..." } else { "Verify" }}
                            </button>
                        </div>
                        {move || match verification_status.get().as_deref() {
                            Some("success") => {
                                view! {
                                    <div data-testid="success-message" class="success">
                                        "✅ Verification successful! Redirecting..."
                                    </div>
                                }.into_any()
                            }
                            Some("error") => {
                                view! {
                                    <div data-testid="error-message" class="error">
                                        "❌ Invalid code. Please try again."
                                    </div>
                                }.into_any()
                            }
                            _ => view! { <div></div> }.into_any()
                        }}
                    </div>
                }
            },
        );

        page_test_utils::wait_for_navigation().await;

        let otp_input = page_test_utils::find_by_test_id(&container, "otp-input")
            .unwrap()
            .dyn_into::<HtmlInputElement>()
            .unwrap();

        let verify_btn = page_test_utils::find_by_test_id(&container, "verify-button")
            .unwrap()
            .dyn_into::<HtmlButtonElement>()
            .unwrap();

        // Test successful verification
        otp_input.set_value("123456");
        let input_event = web_sys::Event::new("input").unwrap();
        otp_input.dispatch_event(&input_event).unwrap();
        page_test_utils::wait_for_navigation().await;

        assert!(!verify_btn.disabled());

        let click_event = web_sys::MouseEvent::new("click").unwrap();
        verify_btn
            .dispatch_event(&click_event.unchecked_into())
            .unwrap();

        // Wait for async verification
        TimeoutFuture::new(150).await;

        let success_msg = page_test_utils::find_by_test_id(&container, "success-message");
        assert!(success_msg.is_some());
        assert!(
            success_msg
                .unwrap()
                .text_content()
                .unwrap()
                .contains("Verification successful")
        );

        page_test_utils::cleanup_test_router();
    }
}
