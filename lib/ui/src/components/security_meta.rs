// Security meta tags and CSP components
use crate::utils::security::ContentSecurityPolicy;
use leptos::prelude::*;
use leptos_meta::*;
use wasm_bindgen::JsCast;

/// Security meta tags component
/// Adds CSP and other security-related meta tags to the document head
#[component]
pub fn SecurityMeta(#[prop(optional)] custom_csp: Option<ContentSecurityPolicy>) -> impl IntoView {
    let csp = custom_csp.unwrap_or_else(ContentSecurityPolicy::default_policy);
    let csp_content = csp.build();

    view! {
        <Meta
            http_equiv="Content-Security-Policy"
            content=csp_content
        />
        <Meta
            http_equiv="X-Content-Type-Options"
            content="nosniff"
        />
        <Meta
            http_equiv="X-Frame-Options"
            content="DENY"
        />
        <Meta
            http_equiv="X-XSS-Protection"
            content="1; mode=block"
        />
        <Meta
            name="referrer"
            content="no-referrer-when-downgrade"
        />
    }
}

/// Strict CSP meta tags for high-security pages (e.g., login, MFA)
#[component]
pub fn StrictSecurityMeta() -> impl IntoView {
    let mut csp = ContentSecurityPolicy::new();

    // Very restrictive CSP for authentication pages
    csp.add_directive("default-src", vec!["'self'"]);
    csp.add_directive("script-src", vec!["'self'", "'wasm-unsafe-eval'"]);
    csp.add_directive("style-src", vec!["'self'"]);
    csp.add_directive("img-src", vec!["'self'", "data:"]);
    csp.add_directive("font-src", vec!["'self'"]);
    csp.add_directive("connect-src", vec!["'self'"]);
    csp.add_directive("frame-ancestors", vec!["'none'"]);
    csp.add_directive("base-uri", vec!["'self'"]);
    csp.add_directive("form-action", vec!["'self'"]);
    csp.add_directive("object-src", vec!["'none'"]);
    csp.add_directive("media-src", vec!["'none'"]);

    let csp_content = csp.build();

    view! {
        <Meta
            http_equiv="Content-Security-Policy"
            content=csp_content
        />
        <Meta
            http_equiv="X-Content-Type-Options"
            content="nosniff"
        />
        <Meta
            http_equiv="X-Frame-Options"
            content="DENY"
        />
        <Meta
            http_equiv="X-XSS-Protection"
            content="1; mode=block"
        />
        <Meta
            name="referrer"
            content="no-referrer"
        />
    }
}

/// CSP violation reporter component
/// Monitors and reports CSP violations
#[component]
pub fn CspViolationReporter(#[prop(optional)] report_uri: Option<String>) -> impl IntoView {
    let report_uri = report_uri.unwrap_or_else(|| "/api/csp-report".to_string());

    let report_uri_clone = report_uri.clone();

    Effect::new(move |_| {
        let report_uri_for_closure = report_uri_clone.clone();

        // Listen for CSP violations
        let closure = wasm_bindgen::closure::Closure::wrap(Box::new(
            move |event: web_sys::SecurityPolicyViolationEvent| {
                // CSP Violation detected
                web_sys::console::warn_3(
                    &"CSP Violation:".into(),
                    &event.blocked_uri().into(),
                    &event.violated_directive().into(),
                );

                // Send violation report to backend
                let report = serde_json::json!({
                    "blocked_uri": event.blocked_uri(),
                    "document_uri": event.document_uri(),
                    "violated_directive": event.violated_directive(),
                    "effective_directive": event.effective_directive(),
                    "original_policy": event.original_policy(),
                    "source_file": event.source_file(),
                    "line_number": event.line_number(),
                    "column_number": event.column_number(),
                    "timestamp": js_sys::Date::now(),
                });

                // Send report asynchronously
                let uri = report_uri_for_closure.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let _ = crate::utils::csrf::csrf_protected_request("POST", &uri, Some(report))
                        .await;
                });
            },
        ) as Box<dyn FnMut(_)>);

        if let Some(window) = web_sys::window() {
            let _ = window.add_event_listener_with_callback(
                "securitypolicyviolation",
                closure.as_ref().unchecked_ref(),
            );
            closure.forget();
        }
    });

    view! {
        <></>
    }
}

/// Nonce generator for inline scripts (CSP nonce support)
/// WASM implementation using js_sys::Math for randomness
#[cfg(target_arch = "wasm32")]
pub fn generate_csp_nonce() -> String {
    use js_sys::Math;

    // Generate a random nonce (16 bytes = 32 hex chars)
    (0..16)
        .map(|_| format!("{:02x}", (Math::random() * 255.0) as u8))
        .collect::<String>()
}

/// Nonce generator for non-WASM targets
/// Uses UUID v4 as a secure random source and formats as 32 hex chars
#[cfg(not(target_arch = "wasm32"))]
pub fn generate_csp_nonce() -> String {
    use uuid::Uuid;

    Uuid::new_v4().as_simple().to_string()
}

/// Component to inject CSP nonce into script tags
#[component]
pub fn ScriptWithNonce(
    /// Script content
    content: String,
    #[prop(optional)] nonce: Option<String>,
) -> impl IntoView {
    let nonce = nonce.unwrap_or_else(generate_csp_nonce);

    view! {
        <script nonce=nonce>
            {content}
        </script>
    }
}

#[cfg(test)]
mod tests {

    #[cfg(target_arch = "wasm32")]
    #[test]
    #[cfg(target_arch = "wasm32")]
    fn test_generate_csp_nonce() {
        let nonce = generate_csp_nonce();
        assert_eq!(nonce.len(), 32); // 16 bytes = 32 hex chars
    }
}
