//! WebAuthn Browser API Wrapper
//!
//! Provides safe Rust wrappers around the browser's Web Authentication API
//! for passkey registration and authentication from WASM.
//! Uses proper web_sys bindings (CSP-compliant, no eval).

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
    if let Some(window) = web_sys::window() {
        let nav = window.navigator();
        let creds = js_sys::Reflect::get(
            &JsValue::from(nav),
            &JsValue::from_str("credentials"),
        );
        if let Ok(c) = creds {
            return !c.is_undefined() && !c.is_null();
        }
    }
    false
}

// ── Base64url helpers (CSP-safe, no eval) ─────────────────────────────

#[cfg(target_arch = "wasm32")]
fn b64url_decode(input: &str) -> Result<Vec<u8>, String> {
    use base64::Engine;
    base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(input)
        .map_err(|e| format!("base64url decode error: {}", e))
}

#[cfg(target_arch = "wasm32")]
fn b64url_encode(input: &[u8]) -> String {
    use base64::Engine;
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(input)
}

#[cfg(target_arch = "wasm32")]
fn b64url_to_array_buffer(b64url: &str) -> Result<JsValue, String> {
    let bytes = b64url_decode(b64url)?;
    let uint8 = js_sys::Uint8Array::new_with_length(bytes.len() as u32);
    uint8.copy_from(&bytes);
    Ok(uint8.buffer().into())
}

#[cfg(target_arch = "wasm32")]
fn array_buffer_to_b64url(buffer: &js_sys::ArrayBuffer) -> String {
    let uint8 = js_sys::Uint8Array::new(buffer);
    let mut bytes = vec![0u8; uint8.length() as usize];
    uint8.copy_to(&mut bytes);
    b64url_encode(&bytes)
}

// ── Helpers: set JS object properties ─────────────────────────────────

#[cfg(target_arch = "wasm32")]
fn js_set(obj: &js_sys::Object, key: &str, val: &JsValue) -> Result<(), String> {
    js_sys::Reflect::set(obj, &JsValue::from_str(key), val)
        .map_err(|_| format!("Failed to set property '{}'", key))?;
    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn json_str(v: &serde_json::Value) -> Option<&str> {
    v.as_str()
}

#[cfg(target_arch = "wasm32")]
fn json_u64_or(v: &serde_json::Value, default: u64) -> u64 {
    v.as_u64().unwrap_or(default)
}

/// Create a new credential (passkey registration)
///
/// Uses web_sys::CredentialsContainer::create() — CSP-compliant, no eval.
#[cfg(target_arch = "wasm32")]
pub async fn create_credential(
    options_json: &serde_json::Value,
) -> Result<serde_json::Value, String> {
    use wasm_bindgen_futures::JsFuture;

    let pk = options_json
        .get("publicKey")
        .or_else(|| options_json.get("public_key"))
        .unwrap_or(options_json);

    // ── rp ────────────────────────────────────────────────────────────
    let rp = js_sys::Object::new();
    if let Some(rp_json) = pk.get("rp") {
        if let Some(name) = json_str(rp_json.get("name").unwrap_or(&serde_json::Value::Null)) {
            js_set(&rp, "name", &JsValue::from_str(name))?;
        }
        if let Some(id) = rp_json.get("id").and_then(|v| v.as_str()) {
            js_set(&rp, "id", &JsValue::from_str(id))?;
        }
    }

    // ── user ──────────────────────────────────────────────────────────
    let user = js_sys::Object::new();
    if let Some(u) = pk.get("user") {
        let user_id_str = u
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or("Missing user.id")?;
        js_set(&user, "id", &b64url_to_array_buffer(user_id_str)?)?;
        if let Some(name) = u.get("name").and_then(|v| v.as_str()) {
            js_set(&user, "name", &JsValue::from_str(name))?;
        }
        let display_name = u
            .get("displayName")
            .or_else(|| u.get("display_name"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        js_set(&user, "displayName", &JsValue::from_str(display_name))?;
    }

    // ── challenge ─────────────────────────────────────────────────────
    let challenge_str = pk
        .get("challenge")
        .and_then(|v| v.as_str())
        .ok_or("Missing challenge")?;
    let challenge = b64url_to_array_buffer(challenge_str)?;

    // ── pubKeyCredParams ──────────────────────────────────────────────
    let params_json = pk
        .get("pubKeyCredParams")
        .or_else(|| pk.get("pub_key_cred_params"))
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let params = js_sys::Array::new();
    for p in &params_json {
        let obj = js_sys::Object::new();
        if let Some(t) = p.get("type").and_then(|v| v.as_str()) {
            js_set(&obj, "type", &JsValue::from_str(t))?;
        }
        if let Some(alg) = p.get("alg").and_then(|v| v.as_i64()) {
            js_set(&obj, "alg", &JsValue::from_f64(alg as f64))?;
        }
        params.push(&obj);
    }

    // ── authenticatorSelection ────────────────────────────────────────
    let auth_sel = js_sys::Object::new();
    if let Some(sel) = pk
        .get("authenticatorSelection")
        .or_else(|| pk.get("authenticator_selection"))
    {
        if let Some(v) = sel
            .get("authenticatorAttachment")
            .or_else(|| sel.get("authenticator_attachment"))
            .and_then(|v| v.as_str())
        {
            js_set(&auth_sel, "authenticatorAttachment", &JsValue::from_str(v))?;
        }
        if let Some(v) = sel
            .get("residentKey")
            .or_else(|| sel.get("resident_key"))
            .and_then(|v| v.as_str())
        {
            js_set(&auth_sel, "residentKey", &JsValue::from_str(v))?;
        }
        if let Some(v) = sel
            .get("userVerification")
            .or_else(|| sel.get("user_verification"))
            .and_then(|v| v.as_str())
        {
            js_set(&auth_sel, "userVerification", &JsValue::from_str(v))?;
        }
    } else {
        js_set(
            &auth_sel,
            "authenticatorAttachment",
            &JsValue::from_str("platform"),
        )?;
        js_set(&auth_sel, "residentKey", &JsValue::from_str("preferred"))?;
        js_set(
            &auth_sel,
            "userVerification",
            &JsValue::from_str("preferred"),
        )?;
    }

    // ── excludeCredentials ────────────────────────────────────────────
    let exclude_json = pk
        .get("excludeCredentials")
        .or_else(|| pk.get("exclude_credentials"))
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let exclude = js_sys::Array::new();
    for c in &exclude_json {
        let obj = js_sys::Object::new();
        if let Some(t) = c.get("type").and_then(|v| v.as_str()) {
            js_set(&obj, "type", &JsValue::from_str(t))?;
        }
        if let Some(id) = c.get("id").and_then(|v| v.as_str()) {
            js_set(&obj, "id", &b64url_to_array_buffer(id)?)?;
        }
        if let Some(transports) = c.get("transports").and_then(|v| v.as_array()) {
            let arr = js_sys::Array::new();
            for t in transports {
                if let Some(s) = t.as_str() {
                    arr.push(&JsValue::from_str(s));
                }
            }
            js_set(&obj, "transports", &arr)?;
        }
        exclude.push(&obj);
    }

    // ── Build publicKey options object ────────────────────────────────
    let public_key = js_sys::Object::new();
    js_set(&public_key, "rp", &rp)?;
    js_set(&public_key, "user", &user)?;
    js_set(&public_key, "challenge", &challenge)?;
    js_set(&public_key, "pubKeyCredParams", &params)?;
    js_set(
        &public_key,
        "timeout",
        &JsValue::from_f64(json_u64_or(
            pk.get("timeout").unwrap_or(&serde_json::Value::Null),
            60000,
        ) as f64),
    )?;
    let attestation = pk
        .get("attestation")
        .and_then(|v| v.as_str())
        .unwrap_or("none");
    js_set(&public_key, "attestation", &JsValue::from_str(attestation))?;
    js_set(&public_key, "authenticatorSelection", &auth_sel)?;
    js_set(&public_key, "excludeCredentials", &exclude)?;

    // Wrap in CredentialCreationOptions-like object
    let create_options = js_sys::Object::new();
    js_set(&create_options, "publicKey", &public_key)?;

    // ── Call navigator.credentials.create() ───────────────────────────
    let window = web_sys::window().ok_or("No window object")?;
    let nav = window.navigator();
    let credentials: web_sys::CredentialsContainer = nav
        .credentials();
    let promise = credentials
        .create_with_options(
            &web_sys::CredentialCreationOptions::from(JsValue::from(create_options)),
        )
        .map_err(|e| format!("credentials.create() failed: {:?}", e))?;
    let js_result = JsFuture::from(promise)
        .await
        .map_err(|e| format!("Passkey creation cancelled or failed: {:?}", e))?;

    // ── Extract response from PublicKeyCredential ─────────────────────
    let cred: web_sys::PublicKeyCredential = js_result
        .dyn_into()
        .map_err(|_| "Result is not a PublicKeyCredential")?;

    let raw_id = js_sys::Reflect::get(&cred, &JsValue::from_str("rawId"))
        .map_err(|_| "No rawId")?;
    let raw_id_buf: js_sys::ArrayBuffer = raw_id.dyn_into().map_err(|_| "rawId not ArrayBuffer")?;

    let response = cred.response();
    let attest_resp: web_sys::AuthenticatorAttestationResponse = response
        .dyn_into()
        .map_err(|_| "Not an attestation response")?;

    let attestation_obj = attest_resp.attestation_object();
    let client_data = js_sys::Reflect::get(&attest_resp, &JsValue::from_str("clientDataJSON"))
        .map_err(|_| "No clientDataJSON")?;
    let client_data_buf: js_sys::ArrayBuffer =
        client_data.dyn_into().map_err(|_| "clientDataJSON not ArrayBuffer")?;

    Ok(serde_json::json!({
        "id": cred.id(),
        "rawId": array_buffer_to_b64url(&raw_id_buf),
        "type": cred.type_(),
        "response": {
            "attestationObject": array_buffer_to_b64url(&attestation_obj),
            "clientDataJSON": array_buffer_to_b64url(&client_data_buf)
        }
    }))
}

/// Get an existing credential (passkey authentication)
///
/// Uses web_sys::CredentialsContainer::get() — CSP-compliant, no eval.
#[cfg(target_arch = "wasm32")]
pub async fn get_credential(options_json: &serde_json::Value) -> Result<serde_json::Value, String> {
    use wasm_bindgen_futures::JsFuture;

    let pk = options_json
        .get("publicKey")
        .or_else(|| options_json.get("public_key"))
        .unwrap_or(options_json);

    // ── challenge ─────────────────────────────────────────────────────
    let challenge_str = pk
        .get("challenge")
        .and_then(|v| v.as_str())
        .ok_or("Missing challenge")?;
    let challenge = b64url_to_array_buffer(challenge_str)?;

    // ── allowCredentials ──────────────────────────────────────────────
    let allow_json = pk
        .get("allowCredentials")
        .or_else(|| pk.get("allow_credentials"))
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let allow = js_sys::Array::new();
    for c in &allow_json {
        let obj = js_sys::Object::new();
        if let Some(t) = c.get("type").and_then(|v| v.as_str()) {
            js_set(&obj, "type", &JsValue::from_str(t))?;
        }
        if let Some(id) = c.get("id").and_then(|v| v.as_str()) {
            js_set(&obj, "id", &b64url_to_array_buffer(id)?)?;
        }
        if let Some(transports) = c.get("transports").and_then(|v| v.as_array()) {
            let arr = js_sys::Array::new();
            for t in transports {
                if let Some(s) = t.as_str() {
                    arr.push(&JsValue::from_str(s));
                }
            }
            js_set(&obj, "transports", &arr)?;
        }
        allow.push(&obj);
    }

    // ── Build publicKey options object ────────────────────────────────
    let public_key = js_sys::Object::new();
    js_set(&public_key, "challenge", &challenge)?;
    js_set(
        &public_key,
        "timeout",
        &JsValue::from_f64(json_u64_or(
            pk.get("timeout").unwrap_or(&serde_json::Value::Null),
            60000,
        ) as f64),
    )?;
    if let Some(rp_id) = pk
        .get("rpId")
        .or_else(|| pk.get("rp_id"))
        .and_then(|v| v.as_str())
    {
        js_set(&public_key, "rpId", &JsValue::from_str(rp_id))?;
    }
    let uv = pk
        .get("userVerification")
        .or_else(|| pk.get("user_verification"))
        .and_then(|v| v.as_str())
        .unwrap_or("preferred");
    js_set(&public_key, "userVerification", &JsValue::from_str(uv))?;
    js_set(&public_key, "allowCredentials", &allow)?;

    // Wrap in CredentialRequestOptions-like object
    let get_options = js_sys::Object::new();
    js_set(&get_options, "publicKey", &public_key)?;

    // ── Call navigator.credentials.get() ──────────────────────────────
    let window = web_sys::window().ok_or("No window object")?;
    let nav = window.navigator();
    let credentials: web_sys::CredentialsContainer = nav.credentials();
    let promise = credentials
        .get_with_options(
            &web_sys::CredentialRequestOptions::from(JsValue::from(get_options)),
        )
        .map_err(|e| format!("credentials.get() failed: {:?}", e))?;
    let js_result = JsFuture::from(promise)
        .await
        .map_err(|e| format!("Passkey authentication cancelled or failed: {:?}", e))?;

    // ── Extract response from PublicKeyCredential ─────────────────────
    let cred: web_sys::PublicKeyCredential = js_result
        .dyn_into()
        .map_err(|_| "Result is not a PublicKeyCredential")?;

    let raw_id = js_sys::Reflect::get(&cred, &JsValue::from_str("rawId"))
        .map_err(|_| "No rawId")?;
    let raw_id_buf: js_sys::ArrayBuffer = raw_id.dyn_into().map_err(|_| "rawId not ArrayBuffer")?;

    let response = cred.response();
    let assertion_resp: web_sys::AuthenticatorAssertionResponse = response
        .dyn_into()
        .map_err(|_| "Not an assertion response")?;

    let auth_data = js_sys::Reflect::get(&assertion_resp, &JsValue::from_str("authenticatorData"))
        .map_err(|_| "No authenticatorData")?;
    let auth_data_buf: js_sys::ArrayBuffer =
        auth_data.dyn_into().map_err(|_| "authenticatorData not ArrayBuffer")?;

    let client_data = js_sys::Reflect::get(&assertion_resp, &JsValue::from_str("clientDataJSON"))
        .map_err(|_| "No clientDataJSON")?;
    let client_data_buf: js_sys::ArrayBuffer =
        client_data.dyn_into().map_err(|_| "clientDataJSON not ArrayBuffer")?;

    let signature = js_sys::Reflect::get(&assertion_resp, &JsValue::from_str("signature"))
        .map_err(|_| "No signature")?;
    let signature_buf: js_sys::ArrayBuffer =
        signature.dyn_into().map_err(|_| "signature not ArrayBuffer")?;

    let mut result = serde_json::json!({
        "id": cred.id(),
        "rawId": array_buffer_to_b64url(&raw_id_buf),
        "type": cred.type_(),
        "response": {
            "authenticatorData": array_buffer_to_b64url(&auth_data_buf),
            "clientDataJSON": array_buffer_to_b64url(&client_data_buf),
            "signature": array_buffer_to_b64url(&signature_buf)
        }
    });

    // userHandle is optional
    if let Ok(uh) =
        js_sys::Reflect::get(&assertion_resp, &JsValue::from_str("userHandle"))
    {
        if !uh.is_null() && !uh.is_undefined() {
            if let Ok(uh_buf) = uh.dyn_into::<js_sys::ArrayBuffer>() {
                result["response"]["userHandle"] =
                    serde_json::Value::String(array_buffer_to_b64url(&uh_buf));
            }
        }
    }

    Ok(result)
}

// ── Non-WASM stubs ────────────────────────────────────────────────────

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
