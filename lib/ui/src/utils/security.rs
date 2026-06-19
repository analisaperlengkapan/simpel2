// Security utilities for XSS prevention, input sanitization, and validation
use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// HTML sanitization for user-generated content
/// Uses a whitelist approach to allow only safe HTML tags and attributes
pub fn sanitize_html(input: &str) -> String {
    // Define allowed tags and their safe attributes
    let allowed: HashMap<&str, Vec<&str>> = [
        ("p", vec![]),
        ("br", vec![]),
        ("strong", vec![]),
        ("em", vec![]),
        ("u", vec![]),
        ("a", vec!["href", "title", "target"]),
        ("ul", vec![]),
        ("ol", vec![]),
        ("li", vec![]),
        ("h1", vec![]),
        ("h2", vec![]),
        ("h3", vec![]),
        ("h4", vec![]),
        ("h5", vec![]),
        ("h6", vec![]),
        ("blockquote", vec![]),
        ("code", vec![]),
        ("pre", vec![]),
        ("span", vec!["class"]),
        ("div", vec!["class"]),
        ("img", vec!["src", "alt", "width", "height"]),
    ]
    .into_iter()
    .collect();

    // Basic HTML sanitization using regex.
    // SECURITY NOTE: Regex-based sanitization is inherently fragile and can be bypassed.
    // For high-security requirements, use a proper HTML parser-based sanitizer like `ammonia`.
    let mut sanitized = input.to_string();

    // 1. Remove script tags and their content (including multi-line content)
    sanitized = regex::Regex::new(r"(?i)<script[^>]*>[\s\S]*?</script>")
        .unwrap()
        .replace_all(&sanitized, "")
        .to_string();

    // 2. Remove dangerous protocols that can execute code
    sanitized = regex::Regex::new(r"(?i)(?:javascript|data|vbscript):")
        .unwrap()
        .replace_all(&sanitized, "")
        .to_string();

    // 3. Enforce tag and attribute whitelist.
    // This regex catches tags and captures (1: tag name, 2: all attributes).
    let tag_re = regex::Regex::new(r#"(?i)</?([a-z1-6]+)(\b[^>]*)?>"#).unwrap();
    let attr_re = regex::Regex::new(r#"(?i)\b([a-z-]+)\s*=\s*(?:["']([^"']*)["']|([^\s>]+))"#).unwrap();

    let result = tag_re.replace_all(&sanitized, |caps: &regex::Captures| {
        let tag_name = caps.get(1).unwrap().as_str().to_lowercase();

        if let Some(allowed_attrs) = allowed.get(tag_name.as_str()) {
            let is_closing = caps.get(0).unwrap().as_str().starts_with("</");
            if is_closing {
                return format!("</{}>", tag_name);
            }

            // Rebuild opening tag with only whitelisted attributes
            let mut cleaned_tag = format!("<{}", tag_name);
            if let Some(attrs_raw) = caps.get(2) {
                for attr_caps in attr_re.captures_iter(attrs_raw.as_str()) {
                    let attr_name = attr_caps.get(1).unwrap().as_str().to_lowercase();
                    if allowed_attrs.contains(&attr_name.as_str()) {
                        let value = attr_caps.get(2).or_else(|| attr_caps.get(3)).unwrap().as_str();
                        cleaned_tag.push_str(&format!(r#" {}="{}""#, attr_name, value));
                    }
                }
            }
            cleaned_tag.push('>');
            cleaned_tag
        } else {
            String::new()
        }
    });

    result.to_string()
}

/// Sanitize user input for display
/// Escapes HTML entities to prevent XSS
pub fn sanitize_input(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
        .replace('/', "&#x2F;")
}

/// Validate and sanitize URL to prevent XSS via href attributes
pub fn sanitize_url(url: &str) -> Result<String, SecurityError> {
    let url = url.trim();

    // Check for javascript: protocol
    if url.to_lowercase().starts_with("javascript:") {
        return Err(SecurityError::InvalidUrl(
            "JavaScript URLs are not allowed".to_string(),
        ));
    }

    // Check for data: protocol
    if url.to_lowercase().starts_with("data:") {
        return Err(SecurityError::InvalidUrl(
            "Data URLs are not allowed".to_string(),
        ));
    }

    // Check for vbscript: protocol
    if url.to_lowercase().starts_with("vbscript:") {
        return Err(SecurityError::InvalidUrl(
            "VBScript URLs are not allowed".to_string(),
        ));
    }

    // Allow only http, https, mailto, and relative URLs
    if url.starts_with("http://")
        || url.starts_with("https://")
        || url.starts_with("mailto:")
        || url.starts_with('/')
        || url.starts_with('#')
    {
        Ok(url.to_string())
    } else {
        Err(SecurityError::InvalidUrl(
            "Invalid URL protocol".to_string(),
        ))
    }
}

/// Validate input against common XSS patterns
pub fn validate_input(input: &str) -> Result<(), SecurityError> {
    // Check for script tags
    if input.to_lowercase().contains("<script") {
        return Err(SecurityError::XssDetected(
            "Script tag detected".to_string(),
        ));
    }

    // Check for event handlers
    let event_handlers = vec![
        "onclick",
        "onerror",
        "onload",
        "onmouseover",
        "onmouseout",
        "onfocus",
        "onblur",
        "onchange",
        "onsubmit",
    ];

    let input_lower = input.to_lowercase();
    for handler in event_handlers {
        if input_lower.contains(handler) {
            return Err(SecurityError::XssDetected(format!(
                "Event handler {} detected",
                handler
            )));
        }
    }

    // Check for javascript: protocol
    if input_lower.contains("javascript:") {
        return Err(SecurityError::XssDetected(
            "JavaScript protocol detected".to_string(),
        ));
    }

    Ok(())
}

/// Security error types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityError {
    XssDetected(String),
    InvalidUrl(String),
    CsrfTokenMissing,
    CsrfTokenInvalid,
    SessionExpired,
    InvalidInput(String),
}

impl std::fmt::Display for SecurityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::XssDetected(msg) => write!(f, "XSS detected: {}", msg),
            Self::InvalidUrl(msg) => write!(f, "Invalid URL: {}", msg),
            Self::CsrfTokenMissing => write!(f, "CSRF token missing"),
            Self::CsrfTokenInvalid => write!(f, "CSRF token invalid"),
            Self::SessionExpired => write!(f, "Session expired"),
            Self::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
        }
    }
}

impl std::error::Error for SecurityError {}

/// Safe HTML component that sanitizes content before rendering
#[component]
pub fn SafeHtml(
    /// HTML content to sanitize and render
    html: String,
    #[prop(optional)] class: String,
) -> impl IntoView {
    let sanitized = sanitize_html(&html);

    view! { <div class=class inner_html=sanitized></div> }
}

/// Secure input component with automatic XSS validation
#[component]
pub fn SecureInput(
    /// Input value signal
    value: RwSignal<String>,
    #[prop(optional)] placeholder: String,
    #[prop(optional)] input_type: String,
    #[prop(optional)] class: String,
    #[prop(optional)] on_validation_error: Option<Callback<SecurityError>>,
) -> impl IntoView {
    let input_type = if input_type.is_empty() {
        "text".to_string()
    } else {
        input_type
    };
    let (error, set_error) = signal(None::<SecurityError>);

    let on_input = move |ev| {
        let val = event_target_value(&ev);

        // Validate input for XSS
        match validate_input(&val) {
            Ok(_) => {
                value.set(val);
                set_error.set(None);
            }
            Err(e) => {
                set_error.set(Some(e.clone()));
                if let Some(callback) = on_validation_error {
                    callback.run(e);
                }
            }
        }
    };

    view! {
        <div class="secure-input-wrapper">
            <input
                type=input_type
                value=move || value.get()
                placeholder=placeholder
                class=class
                on:input=on_input
            />
            {move || {
                error
                    .get()
                    .map(|e| view! { <div class="text-red-600 text-sm mt-1">{e.to_string()}</div> })
            }}
        </div>
    }
}

/// Content Security Policy helper
pub struct ContentSecurityPolicy {
    directives: HashMap<String, Vec<String>>,
}

impl ContentSecurityPolicy {
    pub fn new() -> Self {
        Self {
            directives: HashMap::new(),
        }
    }

    /// Add a CSP directive
    pub fn add_directive(&mut self, directive: &str, sources: Vec<&str>) {
        self.directives.insert(
            directive.to_string(),
            sources.iter().map(|s| s.to_string()).collect(),
        );
    }

    /// Build CSP header value
    pub fn build(&self) -> String {
        self.directives
            .iter()
            .map(|(directive, sources)| format!("{} {}", directive, sources.join(" ")))
            .collect::<Vec<_>>()
            .join("; ")
    }

    /// Get default CSP for SIMPEL
    pub fn default_policy() -> Self {
        let mut csp = Self::new();

        csp.add_directive("default-src", vec!["'self'"]);
        csp.add_directive("script-src", vec!["'self'", "'wasm-unsafe-eval'"]);
        csp.add_directive("style-src", vec!["'self'", "'unsafe-inline'"]);
        csp.add_directive("img-src", vec!["'self'", "data:", "https:"]);
        csp.add_directive("font-src", vec!["'self'", "data:"]);
        csp.add_directive(
            "connect-src",
            vec!["'self'", "https://api.simpelv2.kejaksaan.go.id"],
        );
        csp.add_directive("frame-ancestors", vec!["'none'"]);
        csp.add_directive("base-uri", vec!["'self'"]);
        csp.add_directive("form-action", vec!["'self'"]);

        csp
    }
}

impl Default for ContentSecurityPolicy {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_html_removes_script_tags() {
        let input = "<p>Hello</p><script>alert('xss')</script>";
        let output = sanitize_html(input);
        assert!(!output.contains("<script"));
        assert!(output.contains("<p>Hello</p>"));
    }

    #[test]
    fn test_sanitize_html_removes_event_handlers() {
        let input = r#"<div onclick="alert('xss')">Click me</div>"#;
        let output = sanitize_html(input);
        assert!(!output.contains("onclick"));

        // Test unquoted event handler
        let input = r#"<img src=x onerror=alert(1)>"#;
        let output = sanitize_html(input);
        assert!(!output.contains("onerror"));
        assert!(!output.contains("alert"));
        // Check that allowed attributes are kept while event handlers are stripped
        assert!(output.contains("<img src=\"x\">"));
    }

    #[test]
    fn test_sanitize_html_enforces_whitelist() {
        let input = "<div><p>Safe</p><script>alert(1)</script><iframe src='evil.com'></iframe></div>";
        let output = sanitize_html(input);
        assert!(output.contains("<div><p>Safe</p></div>"));
        assert!(!output.contains("<script"));
        assert!(!output.contains("<iframe"));
    }

    #[test]
    fn test_sanitize_html_whitelists_attributes() {
        let input = r#"<a href="https://example.com" target="_blank" style="color:red" rel="nofollow">Link</a>"#;
        let output = sanitize_html(input);
        assert!(output.contains("href=\"https://example.com\""));
        assert!(output.contains("target=\"_blank\""));
        assert!(!output.contains("style=\"color:red\""));
        assert!(!output.contains("rel=\"nofollow\""));
    }

    #[test]
    fn test_sanitize_input_escapes_html() {
        let input = "<script>alert('xss')</script>";
        let output = sanitize_input(input);
        assert_eq!(
            output,
            "&lt;script&gt;alert(&#x27;xss&#x27;)&lt;&#x2F;script&gt;"
        );
    }

    #[test]
    fn test_sanitize_url_blocks_javascript() {
        let result = sanitize_url("javascript:alert('xss')");
        assert!(result.is_err());
    }

    #[test]
    fn test_sanitize_url_allows_https() {
        let result = sanitize_url("https://example.com");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_input_detects_script_tag() {
        let result = validate_input("<script>alert('xss')</script>");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_input_detects_event_handler() {
        let result = validate_input(r#"<img src="x" onerror="alert('xss')">"#);
        assert!(result.is_err());
    }

    #[test]
    fn test_csp_builder() {
        let csp = ContentSecurityPolicy::default_policy();
        let policy = csp.build();
        assert!(policy.contains("default-src 'self'"));
        assert!(policy.contains("script-src 'self' 'wasm-unsafe-eval'"));
    }
}
