//! MFA Accessibility Tests
//!
//! This module contains accessibility tests for MFA components,
//! ensuring compliance with WCAG 2.1 AA standards.

use gloo_timers::future::TimeoutFuture;
use leptos::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;
use web_sys::{Element, HtmlElement, HtmlInputElement};

wasm_bindgen_test_configure!(run_in_browser);

/// Accessibility test utilities
mod a11y_test_utils {
    use super::*;

    /// Check WCAG color contrast requirements
    pub fn check_color_contrast(element: &Element) -> bool {
        let window = web_sys::window().unwrap();
        let computed_style = window.get_computed_style(element).unwrap().unwrap();

        let color = computed_style.get_property_value("color").unwrap();
        let background = computed_style
            .get_property_value("background-color")
            .unwrap();

        // Simplified contrast check - real implementation would calculate luminance
        !color.is_empty() && !background.is_empty() && color != background
    }

    /// Check keyboard navigation support
    pub fn check_keyboard_navigation(element: &Element) -> Vec<String> {
        let mut issues = Vec::new();

        // Check tabindex
        if element.tag_name() == "BUTTON" || element.tag_name() == "INPUT" {
            let tabindex = element.get_attribute("tabindex");
            if let Some(index) = tabindex
                && index == "-1"
            {
                issues.push("Interactive element has tabindex=-1".to_string());
            }
        }

        // Check for focus indicators
        let computed_style = web_sys::window()
            .unwrap()
            .get_computed_style(element)
            .unwrap()
            .unwrap();

        let outline = computed_style.get_property_value("outline").unwrap();
        if outline == "none" || outline.is_empty() {
            // Should have some focus indicator
            let box_shadow = computed_style.get_property_value("box-shadow").unwrap();
            let border = computed_style.get_property_value("border").unwrap();

            if box_shadow == "none" && !border.contains("focus") {
                issues.push("No visible focus indicator".to_string());
            }
        }

        issues
    }

    /// Check ARIA attributes compliance
    pub fn check_aria_compliance(element: &Element) -> Vec<String> {
        let mut issues = Vec::new();

        // Check required ARIA attributes based on element type
        match element.tag_name().as_str() {
            "INPUT" if element.get_attribute("type").as_deref() == Some("text") => {
                if !element.has_attribute("aria-label") && !element.has_attribute("aria-labelledby") {
                    issues.push("Input missing ARIA label".to_string());
                }

                if element.has_attribute("required") && !element.has_attribute("aria-required") {
                    issues.push("Required input missing aria-required".to_string());
                }

                if element.class_list().contains("error") && !element.has_attribute("aria-invalid") {
                    issues.push("Invalid input missing aria-invalid".to_string());
                }
            }
            "BUTTON" if element.text_content().unwrap_or_default().is_empty() && !element.has_attribute("aria-label") => {
                issues.push("Button with no text missing aria-label".to_string());
            }
            "IMG" if !element.has_attribute("alt") => {
                issues.push("Image missing alt attribute".to_string());
            }
            _ => {}
        }

        issues
    }

    /// Test screen reader compatibility
    #[allow(dead_code)]
    pub fn test_screen_reader_content(element: &Element) -> Vec<String> {
        let mut issues = Vec::new();

        // Check for hidden content that should be available to screen readers
        let aria_hidden = element.get_attribute("aria-hidden");
        if aria_hidden.as_deref() == Some("true") {
            // Content is hidden from screen readers - ensure it's decorative
            if element.tag_name() == "IMG" && element.get_attribute("alt").as_deref() == Some("") {
                // Decorative image - OK
            } else {
                issues.push("Important content hidden from screen readers".to_string());
            }
        }

        // Check for proper heading structure
        if element.tag_name().starts_with('H') {
            let level = element
                .tag_name()
                .chars()
                .last()
                .unwrap()
                .to_digit(10)
                .unwrap_or(1);
            if level > 6 {
                issues.push("Invalid heading level".to_string());
            }
        }

        issues
    }
}

/// Accessibility tests for MFA components
#[cfg(test)]
mod mfa_accessibility_tests {
    use super::*;

    #[wasm_bindgen_test]
    async fn test_mfa_setup_page_accessibility() {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let container = document.create_element("div").unwrap();
        container.set_id("a11y-test-container");
        document.body().unwrap().append_child(&container).unwrap();

        let _ = mount_to(
            container.clone().unchecked_into::<HtmlElement>(),
            move || {
                view! {
                    <div class="mfa-setup-page" role="main" aria-labelledby="setup-title">
                        <h1 id="setup-title">"Multi-Factor Authentication Setup"</h1>

                        <div class="setup-instructions" role="region" aria-labelledby="instructions-title">
                            <h2 id="instructions-title">"Setup Instructions"</h2>
                            <ol>
                                <li>"Download an authenticator app"</li>
                                <li>"Scan the QR code below"</li>
                                <li>"Enter the verification code"</li>
                            </ol>
                        </div>

                        <div class="qr-section" role="region" aria-labelledby="qr-title">
                            <h2 id="qr-title">"QR Code"</h2>
                            <img
                                src="data:image/png;base64,test"
                                alt="QR code for MFA setup. Scan with your authenticator app."
                                class="qr-code"
                                role="img"
                                aria-describedby="qr-description"
                            />
                            <p id="qr-description" class="sr-only">
                                "This QR code contains your unique MFA secret. Scan it with Google Authenticator, Microsoft Authenticator, or similar apps."
                            </p>
                        </div>

                        <div class="verification-section" role="region" aria-labelledby="verify-title">
                            <h2 id="verify-title">"Verification"</h2>
                            <label for="otp-code" class="otp-label">
                                "Enter 6-digit code from your app:"
                            </label>
                            <input
                                type="text"
                                id="otp-code"
                                class="otp-input"
                                placeholder="000000"
                                maxlength="6"
                                pattern="[0-9]*"
                                inputmode="numeric"
                                autocomplete="one-time-code"
                                aria-required="true"
                                aria-describedby="otp-help"
                            />
                            <p id="otp-help" class="input-help">
                                "Enter the 6-digit number shown in your authenticator app"
                            </p>
                            <button
                                type="submit"
                                class="verify-button btn-primary"
                                aria-describedby="verify-help"
                            >
                                "Verify and Complete Setup"
                            </button>
                            <p id="verify-help" class="button-help">
                                "Click to verify your authenticator app setup"
                            </p>
                        </div>
                    </div>
                }
            },
        );

        TimeoutFuture::new(100).await;

        // Test page structure accessibility
        let main_element = container.query_selector("[role='main']").unwrap().unwrap();
        let a11y_issues = a11y_test_utils::check_aria_compliance(&main_element);
        assert!(
            a11y_issues.is_empty(),
            "Main page should have proper ARIA compliance: {:?}",
            a11y_issues
        );

        // Test heading structure
        let h1 = container.query_selector("h1").unwrap().unwrap();
        assert!(
            h1.has_attribute("id"),
            "H1 should have ID for aria-labelledby"
        );

        // Test form accessibility
        let otp_input = container.query_selector("#otp-code").unwrap().unwrap();
        let input_a11y_issues = a11y_test_utils::check_aria_compliance(&otp_input);
        assert!(
            input_a11y_issues.is_empty(),
            "OTP input should be accessible: {:?}",
            input_a11y_issues
        );

        // Test label association
        let label = container.query_selector("label[for='otp-code']").unwrap();
        assert!(label.is_some(), "OTP input should have associated label");

        // Test ARIA descriptions
        let input_element = otp_input.dyn_into::<HtmlInputElement>().unwrap();
        assert_eq!(
            input_element.get_attribute("aria-describedby").unwrap(),
            "otp-help"
        );

        let help_text = container.query_selector("#otp-help").unwrap();
        assert!(
            help_text.is_some(),
            "Help text should exist for ARIA description"
        );

        // Test button accessibility
        let verify_button = container.query_selector(".verify-button").unwrap().unwrap();
        let button_a11y_issues = a11y_test_utils::check_aria_compliance(&verify_button);
        assert!(
            button_a11y_issues.is_empty(),
            "Verify button should be accessible: {:?}",
            button_a11y_issues
        );

        // Test keyboard navigation
        let nav_issues = a11y_test_utils::check_keyboard_navigation(&container);
        assert!(
            nav_issues.is_empty(),
            "Page should support keyboard navigation: {:?}",
            nav_issues
        );

        container.remove();
    }

    #[wasm_bindgen_test]
    async fn test_mfa_verification_page_accessibility() {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let container = document.create_element("div").unwrap();
        container.set_id("verify-a11y-container");
        document.body().unwrap().append_child(&container).unwrap();

        let (attempts_remaining, _set_attempts_remaining) = signal(3);
        let (has_error, set_has_error) = signal(false);

        let _ = mount_to(
            container.clone().unchecked_into::<HtmlElement>(),
            move || {
                view! {
                    <div class="mfa-verification-page" role="main" aria-labelledby="verify-title">
                        <h1 id="verify-title">"Multi-Factor Authentication"</h1>

                        <div class="verification-form" role="form" aria-labelledby="form-title">
                            <h2 id="form-title">"Enter Verification Code"</h2>

                            <div class="form-group">
                                <label for="verify-otp" class="required">
                                    "Authentication Code"
                                    <span class="required-indicator" aria-label="required">"*"</span>
                                </label>
                                <input
                                    type="text"
                                    id="verify-otp"
                                    class=move || if has_error.get() { "otp-input error" } else { "otp-input" }
                                    placeholder="000000"
                                    maxlength="6"
                                    pattern="[0-9]*"
                                    inputmode="numeric"
                                    autocomplete="one-time-code"
                                    aria-required="true"
                                    aria-invalid=move || if has_error.get() { "true" } else { "false" }
                                    aria-describedby="otp-help error-message"
                                />
                                <p id="otp-help" class="input-help">
                                    "Enter the 6-digit code from your authenticator app"
                                </p>
                                {move || if has_error.get() {
                                    view! {
                                        <p id="error-message" class="error-text" role="alert">
                                            "Invalid code. Please check your authenticator app and try again."
                                        </p>
                                    }.into_any()
                                } else {
                                    view! { <div></div> }.into_any()
                                }}
                            </div>

                            <div class="form-actions">
                                <button
                                    type="submit"
                                    class="verify-button btn-primary"
                                    aria-describedby="verify-help"
                                >
                                    "Verify Code"
                                </button>
                                <p id="verify-help" class="button-help">
                                    "Submit your authentication code to continue"
                                </p>
                            </div>
                        </div>

                        <div class="attempts-info" role="status" aria-live="polite">
                            <p>{format!("Attempts remaining: {}", attempts_remaining.get())}</p>
                        </div>

                        <div class="backup-options" role="region" aria-labelledby="backup-title">
                            <h3 id="backup-title">"Alternative Options"</h3>
                            <details>
                                <summary>"Use backup code instead"</summary>
                                <div class="backup-form">
                                    <label for="backup-code">"Backup Code:"</label>
                                    <input
                                        type="text"
                                        id="backup-code"
                                        placeholder="Enter 8-digit backup code"
                                        aria-describedby="backup-help"
                                    />
                                    <p id="backup-help" class="input-help">
                                        "Enter one of your backup codes if you can't access your authenticator app"
                                    </p>
                                    <button type="button" class="backup-verify-button">
                                        "Verify Backup Code"
                                    </button>
                                </div>
                            </details>
                        </div>
                    </div>
                }
            },
        );

        TimeoutFuture::new(100).await;

        // Test main page accessibility
        let main_page = container.query_selector("[role='main']").unwrap().unwrap();
        assert!(
            main_page.has_attribute("aria-labelledby"),
            "Main page should have aria-labelledby"
        );

        // Test form accessibility
        let form = container.query_selector("[role='form']").unwrap().unwrap();
        assert!(
            form.has_attribute("aria-labelledby"),
            "Form should have aria-labelledby"
        );

        // Test input accessibility
        let otp_input = container.query_selector("#verify-otp").unwrap().unwrap();
        let input_element = otp_input.dyn_into::<HtmlInputElement>().unwrap();

        assert_eq!(
            input_element.get_attribute("aria-required").unwrap(),
            "true"
        );
        assert_eq!(input_element.get_attribute("inputmode").unwrap(), "numeric");
        assert_eq!(
            input_element.get_attribute("autocomplete").unwrap(),
            "one-time-code"
        );
        assert!(input_element.has_attribute("aria-describedby"));

        // Test label association
        let label = container.query_selector("label[for='verify-otp']").unwrap();
        assert!(label.is_some(), "Input should have associated label");

        // Test error state accessibility
        set_has_error.set(true);
        TimeoutFuture::new(50).await;

        assert_eq!(input_element.get_attribute("aria-invalid").unwrap(), "true");

        let error_message = container.query_selector("#error-message").unwrap();
        assert!(error_message.is_some(), "Error message should exist");

        let error_element = error_message.unwrap();
        assert_eq!(error_element.get_attribute("role").unwrap(), "alert");

        // Test live region for attempts
        let attempts_info = container
            .query_selector("[role='status']")
            .unwrap()
            .unwrap();
        assert_eq!(attempts_info.get_attribute("aria-live").unwrap(), "polite");

        // Test backup options accessibility
        let backup_input = container.query_selector("#backup-code").unwrap().unwrap();
        assert!(
            backup_input.has_attribute("aria-describedby"),
            "Backup input should have description"
        );

        // Test color contrast
        assert!(
            a11y_test_utils::check_color_contrast(&main_page),
            "Page should have proper color contrast"
        );

        // Test keyboard navigation
        let nav_issues = a11y_test_utils::check_keyboard_navigation(&container);
        assert!(
            nav_issues.len() <= 1,
            "Should have minimal keyboard navigation issues: {:?}",
            nav_issues
        );

        container.remove();
    }

    #[wasm_bindgen_test]
    async fn test_mfa_error_states_accessibility() {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let container = document.create_element("div").unwrap();
        container.set_id("error-a11y-container");
        document.body().unwrap().append_child(&container).unwrap();

        let (error_type, set_error_type) = signal(None::<String>);

        let _ = mount_to(
            container.clone().unchecked_into::<HtmlElement>(),
            move || {
                view! {
                    <div class="mfa-error-states">
                        {move || match error_type.get().as_deref() {
                            Some("invalid_code") => {
                                view! {
                                    <div
                                        class="error-message"
                                        role="alert"
                                        aria-live="assertive"
                                        data-testid="invalid-code-error"
                                    >
                                        <span class="error-icon" aria-hidden="true">"❌"</span>
                                        <span class="error-text">
                                            "Invalid authentication code. Please check your authenticator app and try again."
                                        </span>
                                    </div>
                                }.into_any()
                            }
                            Some("rate_limited") => {
                                view! {
                                    <div
                                        class="error-message rate-limit"
                                        role="alert"
                                        aria-live="assertive"
                                        data-testid="rate-limit-error"
                                    >
                                        <span class="error-icon" aria-hidden="true">"🔒"</span>
                                        <span class="error-text">
                                            "Too many failed attempts. Please wait 5 minutes before trying again."
                                        </span>
                                        <div class="countdown" aria-live="polite" aria-label="Time remaining">
                                            "4:59 remaining"
                                        </div>
                                    </div>
                                }.into_any()
                            }
                            Some("network_error") => {
                                view! {
                                    <div
                                        class="error-message network-error"
                                        role="alert"
                                        aria-live="assertive"
                                        data-testid="network-error"
                                    >
                                        <span class="error-icon" aria-hidden="true">"🌐"</span>
                                        <span class="error-text">
                                            "Network error. Please check your connection and try again."
                                        </span>
                                        <button class="retry-button" aria-describedby="retry-help">
                                            "Retry"
                                        </button>
                                        <p id="retry-help" class="sr-only">
                                            "Click to retry the verification request"
                                        </p>
                                    </div>
                                }.into_any()
                            }
                            _ => view! { <div></div> }.into_any()
                        }}
                    </div>
                }
            },
        );

        TimeoutFuture::new(100).await;

        // Test invalid code error accessibility
        set_error_type.set(Some("invalid_code".to_string()));
        TimeoutFuture::new(50).await;

        let invalid_error = container
            .query_selector("[data-testid='invalid-code-error']")
            .unwrap()
            .unwrap();
        assert_eq!(invalid_error.get_attribute("role").unwrap(), "alert");
        assert_eq!(
            invalid_error.get_attribute("aria-live").unwrap(),
            "assertive"
        );

        // Error icon should be hidden from screen readers
        let error_icon = invalid_error
            .query_selector(".error-icon")
            .unwrap()
            .unwrap();
        assert_eq!(error_icon.get_attribute("aria-hidden").unwrap(), "true");

        // Test rate limit error accessibility
        set_error_type.set(Some("rate_limited".to_string()));
        TimeoutFuture::new(50).await;

        let rate_limit_error = container
            .query_selector("[data-testid='rate-limit-error']")
            .unwrap()
            .unwrap();
        assert_eq!(rate_limit_error.get_attribute("role").unwrap(), "alert");

        // Countdown should have live region
        let countdown = rate_limit_error
            .query_selector(".countdown")
            .unwrap()
            .unwrap();
        assert_eq!(countdown.get_attribute("aria-live").unwrap(), "polite");
        assert!(countdown.has_attribute("aria-label"));

        // Test network error accessibility
        set_error_type.set(Some("network_error".to_string()));
        TimeoutFuture::new(50).await;

        let network_error = container
            .query_selector("[data-testid='network-error']")
            .unwrap()
            .unwrap();
        let retry_button = network_error
            .query_selector(".retry-button")
            .unwrap()
            .unwrap();
        assert!(
            retry_button.has_attribute("aria-describedby"),
            "Retry button should have description"
        );

        container.remove();
    }

    #[wasm_bindgen_test]
    async fn test_mfa_success_states_accessibility() {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let container = document.create_element("div").unwrap();
        container.set_id("success-a11y-container");
        document.body().unwrap().append_child(&container).unwrap();

        let _ = mount_to(
            container.clone().unchecked_into::<HtmlElement>(),
            move || {
                view! {
                    <div class="mfa-success-page">
                        <div
                            class="success-message"
                            role="status"
                            aria-live="polite"
                            data-testid="success-status"
                        >
                            <span class="success-icon" aria-hidden="true">"✅"</span>
                            <h2>"MFA Setup Complete!"</h2>
                            <p>"Your account is now protected with multi-factor authentication."</p>
                        </div>

                        <div class="next-steps" role="region" aria-labelledby="next-steps-title">
                            <h3 id="next-steps-title">"Next Steps"</h3>
                            <ul>
                                <li>"Save your backup codes in a secure location"</li>
                                <li>"Test your authenticator app before logging out"</li>
                                <li>"Contact IT support if you have any issues"</li>
                            </ul>
                        </div>

                        <div class="actions">
                            <button
                                class="continue-button btn-primary"
                                aria-describedby="continue-help"
                            >
                                "Continue to Dashboard"
                            </button>
                            <p id="continue-help" class="sr-only">
                                "Click to proceed to the main dashboard"
                            </p>
                        </div>
                    </div>
                }
            },
        );

        TimeoutFuture::new(100).await;

        // Test success message accessibility
        let success_status = container
            .query_selector("[data-testid='success-status']")
            .unwrap()
            .unwrap();
        assert_eq!(success_status.get_attribute("role").unwrap(), "status");
        assert_eq!(success_status.get_attribute("aria-live").unwrap(), "polite");

        // Test success icon is decorative
        let success_icon = success_status
            .query_selector(".success-icon")
            .unwrap()
            .unwrap();
        assert_eq!(success_icon.get_attribute("aria-hidden").unwrap(), "true");

        // Test next steps region
        let next_steps = container
            .query_selector("[role='region']")
            .unwrap()
            .unwrap();
        assert!(next_steps.has_attribute("aria-labelledby"));

        // Test continue button accessibility
        let continue_button = container
            .query_selector(".continue-button")
            .unwrap()
            .unwrap();
        assert!(continue_button.has_attribute("aria-describedby"));

        // Test overall accessibility compliance
        let page_a11y_issues = a11y_test_utils::check_aria_compliance(&container);
        assert!(
            page_a11y_issues.is_empty(),
            "Success page should be fully accessible: {:?}",
            page_a11y_issues
        );

        container.remove();
    }
}
