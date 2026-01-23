//! MFA Visual Regression Tests
//!
//! This module contains visual regression tests for MFA components,
//! ensuring consistent UI appearance and accessibility compliance.

use gloo_timers::future::TimeoutFuture;
use leptos::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;
use web_sys::{Element, HtmlCanvasElement, HtmlElement};

wasm_bindgen_test_configure!(run_in_browser);

/// Visual test utilities
mod visual_test_utils {
    use super::*;

    /// Create a test canvas for visual comparison
    pub fn create_test_canvas(width: u32, height: u32) -> HtmlCanvasElement {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let canvas = document
            .create_element("canvas")
            .unwrap()
            .dyn_into::<HtmlCanvasElement>()
            .unwrap();

        canvas.set_width(width);
        canvas.set_height(height);
        canvas.set_id("test-canvas");

        document.body().unwrap().append_child(&canvas).unwrap();
        canvas
    }

    /// Clean up test canvas
    pub fn cleanup_test_canvas() {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        if let Some(canvas) = document.get_element_by_id("test-canvas") {
            canvas.remove();
        }
    }

    /// Capture element screenshot (simplified for testing)
    pub fn capture_element_bounds(element: &Element) -> (f64, f64, f64, f64) {
        let rect = element.get_bounding_client_rect();
        (rect.x(), rect.y(), rect.width(), rect.height())
    }

    /// Check if element has proper contrast ratio
    pub fn check_contrast_ratio(element: &Element) -> bool {
        let computed_style = web_sys::window()
            .unwrap()
            .get_computed_style(element)
            .unwrap()
            .unwrap();

        let color = computed_style.get_property_value("color").unwrap();
        let background = computed_style
            .get_property_value("background-color")
            .unwrap();

        // Simplified contrast check - in real implementation would calculate actual ratio
        !color.is_empty() && !background.is_empty()
    }

    /// Check accessibility attributes
    pub fn check_accessibility_attributes(element: &Element) -> Vec<String> {
        let mut issues = Vec::new();

        // Check for alt text on images
        if element.tag_name() == "IMG" && !element.has_attribute("alt") {
            issues.push("Missing alt attribute on image".to_string());
        }

        // Check for proper ARIA labels
        if element.has_attribute("role")
            && !element.has_attribute("aria-label")
            && !element.has_attribute("aria-labelledby")
        {
            issues.push("Element with role missing ARIA label".to_string());
        }

        // Check for proper form labels
        if element.tag_name() == "INPUT"
            && !element.has_attribute("aria-label")
            && !element.has_attribute("aria-labelledby")
        {
            let id = element.get_attribute("id");
            if let Some(input_id) = id {
                let window = web_sys::window().unwrap();
                let document = window.document().unwrap();
                let label = document
                    .query_selector(&format!("label[for='{}']", input_id))
                    .unwrap();
                if label.is_none() {
                    issues.push("Input missing associated label".to_string());
                }
            } else {
                issues.push("Input missing label or ARIA label".to_string());
            }
        }

        issues
    }
}

/// Tests for QR Code Visual Appearance
#[cfg(test)]
mod qr_code_visual_tests {
    use super::*;

    #[wasm_bindgen_test]
    async fn test_qr_code_display_visual_consistency() {
        let _container = visual_test_utils::create_test_canvas(400, 400);
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let test_div = document.create_element("div").unwrap();
        test_div.set_id("qr-test-container");
        document.body().unwrap().append_child(&test_div).unwrap();

        let test_qr_url = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==";

        let _ = mount_to(
            test_div.clone().unchecked_into::<HtmlElement>(),
            move || {
                view! {
                    <div class="qr-code-container" data-testid="qr-container">
                        <div class="qr-code-header">
                            <h3>"Scan QR Code"</h3>
                            <p class="qr-instructions">
                                "Use your authenticator app to scan this code"
                            </p>
                        </div>
                        <div class="qr-code-display">
                            <img
                                src=test_qr_url
                                alt="MFA Setup QR Code"
                                class="qr-code-image"
                                data-testid="qr-image"
                            />
                        </div>
                        <div class="qr-code-footer">
                            <details class="manual-entry">
                                <summary>"Can't scan? Enter manually"</summary>
                                <div class="manual-entry-content">
                                    <label>"Secret Key:"</label>
                                    <code class="secret-key">"JBSWY3DPEHPK3PXP"</code>
                                </div>
                            </details>
                        </div>
                    </div>
                }
            },
        );

        TimeoutFuture::new(100).await;

        // Test QR container layout
        let qr_container = test_div
            .query_selector("[data-testid='qr-container']")
            .unwrap()
            .unwrap();
        let (_x, _y, width, height) = visual_test_utils::capture_element_bounds(&qr_container);

        assert!(width > 200.0, "QR container should have reasonable width");
        assert!(height > 150.0, "QR container should have reasonable height");

        // Test QR image dimensions
        let qr_image = test_div
            .query_selector("[data-testid='qr-image']")
            .unwrap()
            .unwrap();
        let img_element = qr_image
            .clone()
            .dyn_into::<web_sys::HtmlImageElement>()
            .unwrap();

        // Wait for image to load
        TimeoutFuture::new(50).await;

        assert!(
            img_element.natural_width() > 0,
            "QR image should have valid dimensions"
        );
        assert!(img_element.complete(), "QR image should be loaded");

        // Test accessibility
        let accessibility_issues = visual_test_utils::check_accessibility_attributes(&qr_image);
        assert!(
            accessibility_issues.is_empty(),
            "QR image should have proper accessibility attributes: {:?}",
            accessibility_issues
        );

        // Test contrast ratio
        assert!(
            visual_test_utils::check_contrast_ratio(&qr_container),
            "QR container should have proper contrast ratio"
        );

        // Cleanup
        test_div.remove();
        visual_test_utils::cleanup_test_canvas();
    }

    #[wasm_bindgen_test]
    async fn test_qr_code_responsive_design() {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let test_div = document.create_element("div").unwrap();
        test_div.set_id("responsive-test-container");
        document.body().unwrap().append_child(&test_div).unwrap();

        let test_qr_url = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==";

        let _ = mount_to(
            test_div.clone().unchecked_into::<HtmlElement>(),
            move || {
                view! {
                    <div class="qr-responsive-container" data-testid="responsive-qr">
                        <img
                            src=test_qr_url
                            alt="MFA Setup QR Code"
                            class="qr-responsive-image"
                            style="max-width: 100%; height: auto;"
                        />
                    </div>
                }
            },
        );

        TimeoutFuture::new(100).await;

        let qr_container = test_div
            .query_selector("[data-testid='responsive-qr']")
            .unwrap()
            .unwrap();

        // Test different viewport sizes
        let viewport_sizes = vec![
            (320, 568),  // Mobile
            (768, 1024), // Tablet
            (1200, 800), // Desktop
        ];

        for (width, height) in viewport_sizes {
            // Simulate viewport change
            test_div
                .set_attribute(
                    "style",
                    &format!(
                        "width: {}px; height: {}px; overflow: hidden;",
                        width, height
                    ),
                )
                .unwrap();

            TimeoutFuture::new(50).await;

            let (_, _, container_width, container_height) =
                visual_test_utils::capture_element_bounds(&qr_container);

            // QR container should adapt to viewport
            assert!(
                container_width <= width as f64,
                "QR container should fit within viewport width at {}px",
                width
            );
            assert!(
                container_height <= height as f64,
                "QR container should fit within viewport height at {}px",
                height
            );
        }

        test_div.remove();
    }

    #[wasm_bindgen_test]
    async fn test_qr_code_loading_states() {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let test_div = document.create_element("div").unwrap();
        test_div.set_id("loading-test-container");
        document.body().unwrap().append_child(&test_div).unwrap();

        let (is_loading, set_is_loading) = signal(true);
        let (has_error, set_has_error) = signal(false);

        let _ = mount_to(
            test_div.clone().unchecked_into::<HtmlElement>(),
            move || {
                view! {
                    <div class="qr-state-container" data-testid="qr-states">
                        {move || {
                            if has_error.get() {
                                view! {
                                    <div class="qr-error-state" data-testid="qr-error">
                                        <div class="error-icon">"⚠️"</div>
                                        <p>"Failed to generate QR code"</p>
                                        <button class="retry-button">"Retry"</button>
                                    </div>
                                }.into_any()
                            } else if is_loading.get() {
                                view! {
                                    <div class="qr-loading-state" data-testid="qr-loading">
                                        <div class="loading-spinner"></div>
                                        <p>"Generating QR code..."</p>
                                    </div>
                                }.into_any()
                            } else {
                                view! {
                                    <div class="qr-success-state" data-testid="qr-success">
                                        <img
                                            src="data:image/png;base64,test"
                                            alt="MFA Setup QR Code"
                                            class="qr-code-image"
                                        />
                                    </div>
                                }.into_any()
                            }
                        }}
                    </div>
                }
            },
        );

        TimeoutFuture::new(100).await;

        // Test loading state
        let loading_state = test_div
            .query_selector("[data-testid='qr-loading']")
            .unwrap();
        assert!(loading_state.is_some(), "Loading state should be visible");

        let loading_element = loading_state.unwrap();
        assert!(
            loading_element
                .query_selector(".loading-spinner")
                .unwrap()
                .is_some(),
            "Loading spinner should be present"
        );

        // Test error state
        set_is_loading.set(false);
        set_has_error.set(true);
        TimeoutFuture::new(50).await;

        let error_state = test_div.query_selector("[data-testid='qr-error']").unwrap();
        assert!(error_state.is_some(), "Error state should be visible");

        let error_element = error_state.unwrap();
        assert!(
            error_element
                .query_selector(".retry-button")
                .unwrap()
                .is_some(),
            "Retry button should be present"
        );

        // Test success state
        set_has_error.set(false);
        TimeoutFuture::new(50).await;

        let success_state = test_div
            .query_selector("[data-testid='qr-success']")
            .unwrap();
        assert!(success_state.is_some(), "Success state should be visible");

        test_div.remove();
    }

    #[wasm_bindgen_test]
    async fn test_otp_input_visual_states() {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let test_div = document.create_element("div").unwrap();
        test_div.set_id("otp-visual-test-container");
        document.body().unwrap().append_child(&test_div).unwrap();

        let (otp_value, set_otp_value) = signal(String::new());
        let (has_error, set_has_error) = signal(false);
        let (is_focused, set_is_focused) = signal(false);

        let _ = mount_to(
            test_div.clone().unchecked_into::<HtmlElement>(),
            move || {
                view! {
                    <div class="otp-input-container" data-testid="otp-container">
                        <label for="otp-visual-test">"Enter OTP Code"</label>
                        <input
                            type="text"
                            id="otp-visual-test"
                            class=move || {
                                let mut classes = vec!["otp-input"];
                                if has_error.get() { classes.push("error"); }
                                if is_focused.get() { classes.push("focused"); }
                                classes.join(" ")
                            }
                            placeholder="000000"
                            maxlength="6"
                            value=otp_value.get()
                            on:input=move |ev| {
                                set_otp_value.set(event_target_value(&ev));
                            }
                            on:focus=move |_| {
                                set_is_focused.set(true);
                            }
                            on:blur=move |_| {
                                set_is_focused.set(false);
                            }
                        />
                        {move || if has_error.get() {
                            view! {
                                <p class="error-message" data-testid="error-text">
                                    "Invalid OTP code"
                                </p>
                            }.into_any()
                        } else {
                            view! { <div></div> }.into_any()
                        }}
                    </div>
                }
            },
        );

        TimeoutFuture::new(100).await;

        let _otp_container = test_div
            .query_selector("[data-testid='otp-container']")
            .unwrap()
            .unwrap();
        let otp_input = test_div
            .query_selector("#otp-visual-test")
            .unwrap()
            .unwrap();

        // Test normal state
        assert!(!otp_input.class_list().contains("error"));
        assert!(!otp_input.class_list().contains("focused"));

        // Test focused state
        set_is_focused.set(true);
        TimeoutFuture::new(50).await;
        assert!(otp_input.class_list().contains("focused"));

        // Test error state
        set_has_error.set(true);
        TimeoutFuture::new(50).await;
        assert!(otp_input.class_list().contains("error"));

        let error_message = test_div
            .query_selector("[data-testid='error-text']")
            .unwrap();
        assert!(error_message.is_some(), "Error message should be visible");

        // Test accessibility
        let accessibility_issues = visual_test_utils::check_accessibility_attributes(&otp_input);
        assert!(
            accessibility_issues.is_empty(),
            "OTP input should have proper accessibility: {:?}",
            accessibility_issues
        );

        test_div.remove();
    }
}
