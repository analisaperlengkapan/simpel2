//! No credential, and no fragment of one, may reach the log.
//!
//! Found on the rc47 production rehearsal (2026-11-11). A manual MonSAKTI run printed
//! `Token (8 karakter pertama): MOCK-STA...`; reading the code behind that line showed the same
//! client also logged `Authorization: Bearer <first 20 characters>` and then, in `reset_token`,
//! `Response body: <the whole reply>` — and the whole reply of the reset endpoint IS the freshly
//! issued token: `[{"TOKEN":"eyJ0eXAi..."}]`. Every token expiry wrote a complete, valid credential
//! for a Ministry of Finance API into the pod log at INFO.
//!
//! These tests drive the real client through that path against a local HTTP server and read back
//! what it logged, at TRACE, so a line demoted to `debug!` instead of removed still fails.
//!
//! The assertion is on every 8-character window of every token, not on the whole token: the old
//! code never printed a token it meant to hide, it printed "only the first 8" and "only the first
//! 20". Each test also asserts that the capture is NOT empty and holds the line that proves the
//! path under test ran — an empty log contains no credential either.

use layanan_integrasi::client::MonsaktiClient;
use layanan_integrasi::config::Config;
use serde_json::json;
use std::collections::HashMap;
use std::io;
use std::sync::{Arc, Mutex};
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// The token the client starts with (in production: from the environment).
const CONFIGURED: &str = "kR7vQm2ZxLp9TfWc4HsYb8NdJg3VuEa6oIt1CqPy5MzXw0BhAeUnGl";
/// The token the reset endpoint issues.
const REISSUED: &str = "Dj4nXo8WqTz1LbKe6SvPm3YcRu9HaGf7iNw2ZxCy5BtVp0MlQsEgOd";
/// The token a successful data reply rotates in (MonSAKTI sends a new one with every reply).
const ROTATED: &str = "pF3hLs9UcVy6ReWb2KnTq8ZdMx1GaJo7iBt4CzNw5EgYm0HuQvPkSl";

/// Everything the tracing subscriber wrote, as one string.
#[derive(Clone, Default)]
struct Captured(Arc<Mutex<Vec<u8>>>);

impl Captured {
    fn text(&self) -> String {
        String::from_utf8_lossy(&self.0.lock().expect("capture lock")).into_owned()
    }
}

impl io::Write for Captured {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.lock().expect("capture lock").extend_from_slice(buf);
        Ok(buf.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl<'a> tracing_subscriber::fmt::MakeWriter<'a> for Captured {
    type Writer = Captured;
    fn make_writer(&'a self) -> Self::Writer {
        self.clone()
    }
}

/// Capture every event of this thread, down to TRACE. `#[tokio::test]` runs the whole test on one
/// thread, so a thread-scoped subscriber sees everything the client logs.
fn capture_logs() -> (Captured, tracing::subscriber::DefaultGuard) {
    let captured = Captured::default();
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE)
        .with_ansi(false)
        .with_writer(captured.clone())
        .finish();
    (captured, tracing::subscriber::set_default(subscriber))
}

/// Fail if `text` holds any 8 consecutive characters of `token`.
fn assert_no_fragment(what: &str, text: &str, token_name: &str, token: &str) {
    let chars: Vec<char> = token.chars().collect();
    for window in chars.windows(8) {
        let fragment: String = window.iter().collect();
        assert!(
            !text.contains(&fragment),
            "{what} contains `{fragment}`, a fragment of the {token_name} token.\n--- {what} ---\n{text}"
        );
    }
}

fn assert_no_credentials(what: &str, text: &str) {
    assert_no_fragment(what, text, "configured", CONFIGURED);
    assert_no_fragment(what, text, "reissued", REISSUED);
    assert_no_fragment(what, text, "rotated", ROTATED);
}

async fn client_for(server: &MockServer) -> MonsaktiClient {
    let mut tokens = HashMap::new();
    tokens.insert("ADM".to_string(), CONFIGURED.to_string());
    let config = Config {
        base_url: server.uri(),
        db_config: None,
        tokens,
        ..Default::default()
    };
    MonsaktiClient::new(config)
        .await
        .expect("a client without a database")
}

#[tokio::test]
async fn an_expired_token_is_reset_and_no_token_reaches_the_log() {
    let server = MockServer::start().await;

    // 1. The configured token is expired.
    Mock::given(method("GET"))
        .and(path("/API/ADM/data/var1"))
        .and(header(
            "Authorization",
            format!("Bearer {CONFIGURED}").as_str(),
        ))
        .respond_with(ResponseTemplate::new(401).set_body_string("Token Expired"))
        .expect(1)
        .mount(&server)
        .await;
    // 2. The reset endpoint issues a new one. Its whole reply is the credential.
    Mock::given(method("GET"))
        .and(path("/resetToken/ADM/tipedata/KL006"))
        .and(header(
            "Authorization",
            format!("Bearer {CONFIGURED}").as_str(),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([{ "TOKEN": REISSUED }])))
        .expect(1)
        .mount(&server)
        .await;
    // 3. The retry carries the new token, and the reply rotates in a third.
    Mock::given(method("GET"))
        .and(path("/API/ADM/data/var1"))
        .and(header(
            "Authorization",
            format!("Bearer {REISSUED}").as_str(),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            [{ "TOKEN": ROTATED }],
            [{ "id": 1, "name": "Item 1" }]
        ])))
        .expect(1)
        .mount(&server)
        .await;

    let mut client = client_for(&server).await;
    let (captured, guard) = capture_logs();
    let response = client.fetch("ADM", "data", vec!["var1".to_string()]).await;
    drop(guard);
    let log = captured.text();

    // The path under test really ran, end to end, with the real tokens on the wire.
    let response = response.unwrap_or_else(|e| panic!("fetch after reset failed: {e:?}\n{log}"));
    assert_eq!(response.new_token.as_deref(), Some(ROTATED));
    server.verify().await;

    // The capture works and saw that path.
    assert!(
        log.contains("/resetToken/ADM/tipedata/KL006"),
        "the log must show the reset request (is the capture working?):\n{log}"
    );
    assert!(
        log.contains("Retry fetching"),
        "the log must show the retry (is the capture working?):\n{log}"
    );

    assert_no_credentials("the log", &log);
}

#[tokio::test]
async fn a_reset_reply_of_the_wrong_shape_does_not_put_its_token_in_the_error() {
    // serde quotes the value it could not use: `invalid type: string "…", expected struct …`.
    // For this endpoint that value is the token, and the caller logs the error with `{:?}`.
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/resetToken/ADM/tipedata/KL006"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([REISSUED])))
        .expect(1)
        .mount(&server)
        .await;

    let mut client = client_for(&server).await;
    let (captured, guard) = capture_logs();
    let result = client.reset_token("ADM", "data", "KL006").await;
    drop(guard);
    let log = captured.text();
    server.verify().await;

    let error = result.expect_err("a reply that is not [{\"TOKEN\": …}] must be an error");
    assert!(
        log.contains("/resetToken/ADM/tipedata/KL006"),
        "the log must show the reset request (is the capture working?):\n{log}"
    );
    assert_no_credentials("the log", &log);
    assert_no_credentials("the error (Display)", &error.to_string());
    assert_no_credentials("the error (Debug)", &format!("{error:?}"));
}

#[tokio::test]
async fn a_reset_reply_that_is_not_json_does_not_put_its_body_in_the_error() {
    // A gateway in front of the API answering in plain text, with the token in it.
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/resetToken/ADM/tipedata/KL006"))
        .respond_with(ResponseTemplate::new(200).set_body_string(format!("TOKEN={REISSUED}")))
        .expect(1)
        .mount(&server)
        .await;

    let mut client = client_for(&server).await;
    let (captured, guard) = capture_logs();
    let result = client.reset_token("ADM", "data", "KL006").await;
    drop(guard);
    let log = captured.text();
    server.verify().await;

    let error = result.expect_err("a reply that is not JSON must be an error");
    // The error still has to be useful to whoever reads it: status and size, not the body.
    let shown = error.to_string();
    assert!(
        shown.contains("200") && shown.contains("bytes"),
        "the error should say what came back without quoting it: {shown}"
    );
    assert_no_credentials("the log", &log);
    assert_no_credentials("the error (Display)", &shown);
    assert_no_credentials("the error (Debug)", &format!("{error:?}"));
}
