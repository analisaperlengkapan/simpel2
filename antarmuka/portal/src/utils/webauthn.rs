//! WebAuthn Browser API Wrapper
//!
//! Provides safe Rust wrappers around the browser's Web Authentication API
//! for passkey registration and authentication from WASM.

use serde::{Deserialize, Serialize};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

/// WebAuthn credential creation options (simplified for JSON transport)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreationOptions {
    pub public_key: serde_json::Value,
}

/// WebAuthn credential request options (simplified for JSON transport)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RequestOptions {
    pub public_key: serde_json::Value,
}

/// Result of a WebAuthn registration ceremony
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RegistrationResult {
    pub id: String,
    pub raw_id: String,
    pub response: RegistrationResponse,
    #[serde(rename = "type")]
    pub cred_type: String,
}

/// Registration response data
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RegistrationResponse {
    pub attestation_object: String,
    pub client_data_json: String,
}

/// Result of a WebAuthn authentication ceremony
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuthenticationResult {
    pub id: String,
    pub raw_id: String,
    pub response: AuthenticationResponse,
    #[serde(rename = "type")]
    pub cred_type: String,
}

/// Authentication response data
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuthenticationResponse {
    pub authenticator_data: String,
    pub client_data_json: String,
    pub signature: String,
    pub user_handle: Option<String>,
}

/// Check if WebAuthn is supported in this browser
pub fn is_webauthn_supported() -> bool {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            let nav = window.navigator();
            if let Ok(creds) = js_sys::Reflect::get(
                &wasm_bindgen::JsValue::from(nav),
                &wasm_bindgen::JsValue::from_str("credentials"),
            ) {
                return !creds.is_undefined() && !creds.is_null();
            }
        }
        false
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        false
    }
}

/// Check if platform authenticator (biometrics) is available
#[cfg(target_arch = "wasm32")]
pub async fn is_platform_authenticator_available() -> bool {
    let result: Result<bool, _> = async {
        let window = web_sys::window().ok_or("No window")?;
        let nav = window.navigator();
        // Use JS eval to check PublicKeyCredential.isUserVerifyingPlatformAuthenticatorAvailable
        let promise = js_sys::Reflect::get(
            &js_sys::Reflect::get(&JsValue::from(nav), &JsValue::from_str("credentials"))
                .map_err(|_| "No credentials")?,
            &JsValue::from_str("create"),
        );
        // Fallback: just check if the API exists
        if promise.is_err() {
            return Ok::<bool, String>(false);
        }
        Ok::<bool, String>(true)
    }
    .await;
    result.unwrap_or(false)
}

/// Create a new credential (passkey registration)
///
/// Takes the server-provided creation options (as JSON) and invokes
/// the browser's `navigator.credentials.create()` API.
#[cfg(target_arch = "wasm32")]
pub async fn create_credential(
    options_json: &serde_json::Value,
) -> Result<serde_json::Value, String> {
    // We need to call navigator.credentials.create() with proper ArrayBuffer types.
    // The server sends base64url-encoded values that need converting.

    let js_code = format!(
        r#"
        (async function() {{
            const options = {};

            // Decode base64url to ArrayBuffer
            function b64urlToBuffer(b64url) {{
                const b64 = b64url.replace(/-/g, '+').replace(/_/g, '/');
                const padding = '='.repeat((4 - b64.length % 4) % 4);
                const bin = atob(b64 + padding);
                const buf = new Uint8Array(bin.length);
                for (let i = 0; i < bin.length; i++) buf[i] = bin.charCodeAt(i);
                return buf.buffer;
            }}

            // Encode ArrayBuffer to base64url
            function bufferToB64url(buffer) {{
                const bytes = new Uint8Array(buffer);
                let binary = '';
                for (let i = 0; i < bytes.length; i++) binary += String.fromCharCode(bytes[i]);
                return btoa(binary).replace(/\+/g, '-').replace(/\//g, '_').replace(/=/g, '');
            }}

            // Build createCredentialOptions
            const publicKey = options.publicKey || options;

            const createOptions = {{
                publicKey: {{
                    rp: publicKey.rp,
                    user: {{
                        id: b64urlToBuffer(publicKey.user.id),
                        name: publicKey.user.name,
                        displayName: publicKey.user.displayName || publicKey.user.display_name
                    }},
                    challenge: b64urlToBuffer(publicKey.challenge),
                    pubKeyCredParams: (publicKey.pubKeyCredParams || publicKey.pub_key_cred_params || []).map(p => ({{
                        type: p.type,
                        alg: p.alg
                    }})),
                    timeout: publicKey.timeout || 60000,
                    attestation: publicKey.attestation || 'none',
                    authenticatorSelection: publicKey.authenticatorSelection || publicKey.authenticator_selection || {{
                        authenticatorAttachment: 'platform',
                        residentKey: 'preferred',
                        userVerification: 'preferred'
                    }},
                    excludeCredentials: (publicKey.excludeCredentials || publicKey.exclude_credentials || []).map(c => ({{
                        type: c.type,
                        id: b64urlToBuffer(c.id),
                        transports: c.transports
                    }}))
                }}
            }};

            const credential = await navigator.credentials.create(createOptions);

            return JSON.stringify({{
                id: credential.id,
                rawId: bufferToB64url(credential.rawId),
                type: credential.type,
                response: {{
                    attestationObject: bufferToB64url(credential.response.attestationObject),
                    clientDataJSON: bufferToB64url(credential.response.clientDataJSON)
                }}
            }});
        }})()
        "#,
        serde_json::to_string(options_json).map_err(|e| format!("Serialize error: {}", e))?
    );

    let result = eval_async(&js_code).await?;
    let result_str = result
        .as_string()
        .ok_or("Expected string result from WebAuthn create")?;
    serde_json::from_str(&result_str).map_err(|e| format!("Parse credential error: {}", e))
}

/// Get an existing credential (passkey authentication)
///
/// Takes the server-provided request options (as JSON) and invokes
/// the browser's `navigator.credentials.get()` API.
#[cfg(target_arch = "wasm32")]
pub async fn get_credential(options_json: &serde_json::Value) -> Result<serde_json::Value, String> {
    let js_code = format!(
        r#"
        (async function() {{
            const options = {};

            function b64urlToBuffer(b64url) {{
                const b64 = b64url.replace(/-/g, '+').replace(/_/g, '/');
                const padding = '='.repeat((4 - b64.length % 4) % 4);
                const bin = atob(b64 + padding);
                const buf = new Uint8Array(bin.length);
                for (let i = 0; i < bin.length; i++) buf[i] = bin.charCodeAt(i);
                return buf.buffer;
            }}

            function bufferToB64url(buffer) {{
                const bytes = new Uint8Array(buffer);
                let binary = '';
                for (let i = 0; i < bytes.length; i++) binary += String.fromCharCode(bytes[i]);
                return btoa(binary).replace(/\+/g, '-').replace(/\//g, '_').replace(/=/g, '');
            }}

            const publicKey = options.publicKey || options;

            const getOptions = {{
                publicKey: {{
                    challenge: b64urlToBuffer(publicKey.challenge),
                    timeout: publicKey.timeout || 60000,
                    rpId: publicKey.rpId || publicKey.rp_id,
                    userVerification: publicKey.userVerification || publicKey.user_verification || 'preferred',
                    allowCredentials: (publicKey.allowCredentials || publicKey.allow_credentials || []).map(c => ({{
                        type: c.type,
                        id: b64urlToBuffer(c.id),
                        transports: c.transports
                    }}))
                }}
            }};

            const assertion = await navigator.credentials.get(getOptions);

            const result = {{
                id: assertion.id,
                rawId: bufferToB64url(assertion.rawId),
                type: assertion.type,
                response: {{
                    authenticatorData: bufferToB64url(assertion.response.authenticatorData),
                    clientDataJSON: bufferToB64url(assertion.response.clientDataJSON),
                    signature: bufferToB64url(assertion.response.signature)
                }}
            }};

            if (assertion.response.userHandle) {{
                result.response.userHandle = bufferToB64url(assertion.response.userHandle);
            }}

            return JSON.stringify(result);
        }})()
        "#,
        serde_json::to_string(options_json).map_err(|e| format!("Serialize error: {}", e))?
    );

    let result = eval_async(&js_code).await?;
    let result_str = result
        .as_string()
        .ok_or("Expected string result from WebAuthn get")?;
    serde_json::from_str(&result_str).map_err(|e| format!("Parse assertion error: {}", e))
}

/// Helper: evaluate async JavaScript and return the result
#[cfg(target_arch = "wasm32")]
async fn eval_async(code: &str) -> Result<JsValue, String> {
    use wasm_bindgen_futures::JsFuture;

    let result = js_sys::eval(code).map_err(|e| format!("JS eval error: {:?}", e))?;

    // If the result is a Promise, await it
    if result.is_instance_of::<js_sys::Promise>() {
        let promise = js_sys::Promise::from(result);
        JsFuture::from(promise)
            .await
            .map_err(|e| format!("JS promise error: {:?}", e))
    } else {
        Ok(result)
    }
}

// Non-WASM stubs
#[cfg(not(target_arch = "wasm32"))]
pub async fn is_platform_authenticator_available() -> bool {
    false
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn create_credential(_options: &serde_json::Value) -> Result<serde_json::Value, String> {
    Err("WebAuthn not available in non-WASM environment".to_string())
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn get_credential(_options: &serde_json::Value) -> Result<serde_json::Value, String> {
    Err("WebAuthn not available in non-WASM environment".to_string())
}
