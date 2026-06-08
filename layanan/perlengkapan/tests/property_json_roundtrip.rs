//! Property 1 — JSON round-trip stability for the wire DTOs.
//!
//! Anything we put on the wire MUST survive `serde_json::to_string(...)`
//! followed by `serde_json::from_str(...)` unchanged. proptest generates
//! random instances of each DTO and asserts equality after the round-trip,
//! so any future change to a `Serialize` / `Deserialize` derive that
//! breaks symmetry (rename without `#[serde(rename)]`, missing default on
//! a removed field, etc.) is caught here before it ships.

use layanan_perlengkapan::contracts::{
    DocumentArtifact, DocumentFormat, DocumentRequest, NotificationChannel, NotificationMessage,
    NotificationPriority, NotificationReceipt, StorageHandle,
};
use lib_perlengkapan::audit::{AuditAction, AuditEvent};
use lib_perlengkapan::pagination::PageParams;
use lib_perlengkapan::response::{ApiResponse, ErrorBody, PageMeta, PaginatedResponse};
use proptest::prelude::*;

// ── Strategy helpers ─────────────────────────────────────────────────────

fn arb_uuid() -> impl Strategy<Value = uuid::Uuid> {
    any::<[u8; 16]>().prop_map(uuid::Uuid::from_bytes)
}

fn arb_format() -> impl Strategy<Value = DocumentFormat> {
    prop_oneof![
        Just(DocumentFormat::Pdf),
        Just(DocumentFormat::Excel),
        Just(DocumentFormat::Docx),
        Just(DocumentFormat::Html),
        Just(DocumentFormat::Csv),
    ]
}

fn arb_channel() -> impl Strategy<Value = NotificationChannel> {
    prop_oneof![
        Just(NotificationChannel::Email),
        Just(NotificationChannel::Sms),
        Just(NotificationChannel::Whatsapp),
        Just(NotificationChannel::Push),
        Just(NotificationChannel::InApp),
    ]
}

fn arb_priority() -> impl Strategy<Value = NotificationPriority> {
    prop_oneof![
        Just(NotificationPriority::Low),
        Just(NotificationPriority::Medium),
        Just(NotificationPriority::High),
        Just(NotificationPriority::Critical),
    ]
}

fn arb_audit_action() -> impl Strategy<Value = AuditAction> {
    prop_oneof![
        Just(AuditAction::Create),
        Just(AuditAction::Read),
        Just(AuditAction::Update),
        Just(AuditAction::Delete),
        Just(AuditAction::Login),
        Just(AuditAction::Logout),
        Just(AuditAction::Approve),
        Just(AuditAction::Reject),
        Just(AuditAction::Submit),
        Just(AuditAction::Cancel),
        Just(AuditAction::Export),
        Just(AuditAction::Import),
        Just(AuditAction::Custom),
    ]
}

// proptest::collection::vec wants a `Strategy<Value=T>`, and stringy
// payloads benefit from a constrained ASCII alphabet so generated text is
// human-readable in failure output.
fn arb_short_string() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9 ]{0,32}".prop_map(|s| s.to_string())
}

// ── Property tests ───────────────────────────────────────────────────────

fn roundtrip<T>(value: &T)
where
    T: serde::Serialize + serde::de::DeserializeOwned + PartialEq + std::fmt::Debug,
{
    let json = serde_json::to_string(value).expect("serialize");
    let back: T = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(value, &back);
}

proptest! {
    #![proptest_config(ProptestConfig {
        // ~64 cases is enough to catch the common breakage classes
        // (rename / missing default / variant order) without blowing up
        // CI time.
        cases: 64,
        .. ProptestConfig::default()
    })]

    #[test]
    fn api_response_roundtrips(
        data in any::<i64>(),
        message in arb_short_string(),
    ) {
        let value = ApiResponse::<i64>::success(data, message);
        roundtrip(&value);
    }

    #[test]
    fn paginated_response_roundtrips(
        data in proptest::collection::vec(any::<i32>(), 0..16),
        total in 0i64..1_000_000,
        page in 1i32..1_000,
        per_page in 1i32..200,
        message in arb_short_string(),
    ) {
        let value = PaginatedResponse::<i32>::new(data, total, page, per_page, message);
        roundtrip(&value);
    }

    #[test]
    fn page_meta_roundtrips(
        page in 1u32..1_000,
        per_page in 1u32..200,
        total in 0u64..1_000_000,
    ) {
        let value = PageMeta::new(page, per_page, total);
        roundtrip(&value);
    }

    #[test]
    fn page_params_roundtrips(
        page in proptest::option::of(any::<u32>()),
        per_page in proptest::option::of(any::<u32>()),
    ) {
        let value = PageParams { page, per_page };
        roundtrip(&value);
    }

    #[test]
    fn error_body_roundtrips(
        code in arb_short_string(),
        message in arb_short_string(),
    ) {
        let value = ErrorBody::new(code, message);
        roundtrip(&value);
    }

    #[test]
    fn document_request_roundtrips(
        template_id in arb_short_string(),
        format in arb_format(),
        locale in proptest::option::of(arb_short_string()),
        requested_by in proptest::option::of(arb_uuid()),
    ) {
        let value = DocumentRequest {
            template_id,
            format,
            data: serde_json::json!({"k": "v"}),
            locale,
            requested_by,
        };
        roundtrip(&value);
    }

    #[test]
    fn document_artifact_roundtrips(
        document_id in arb_uuid(),
        filename in arb_short_string(),
        content_type in arb_short_string(),
        size_bytes in 0u64..1_000_000_000,
        storage_key in arb_short_string(),
    ) {
        let value = DocumentArtifact {
            document_id,
            filename,
            content_type,
            size_bytes,
            storage_key,
            generated_at: chrono::Utc::now(),
        };
        roundtrip(&value);
    }

    #[test]
    fn storage_handle_roundtrips(
        key in arb_short_string(),
        size_bytes in 0u64..1_000_000_000,
        content_type in arb_short_string(),
        etag in proptest::option::of(arb_short_string()),
    ) {
        let value = StorageHandle {
            key,
            size_bytes,
            content_type,
            etag,
            stored_at: chrono::Utc::now(),
        };
        roundtrip(&value);
    }

    #[test]
    fn notification_message_roundtrips(
        event in arb_short_string(),
        recipient_user_id in arb_uuid(),
        channels in proptest::collection::vec(arb_channel(), 0..5),
        priority in arb_priority(),
        title in arb_short_string(),
        body in arb_short_string(),
        deeplink in proptest::option::of(arb_short_string()),
    ) {
        let value = NotificationMessage {
            event,
            recipient_user_id,
            channels,
            priority,
            title,
            body,
            variables: Some(serde_json::json!({"k": "v"})),
            deeplink,
            recipient_email: None,
        };
        roundtrip(&value);
    }

    #[test]
    fn notification_receipt_roundtrips(
        notification_id in arb_uuid(),
        channels in proptest::collection::vec(arb_channel(), 0..5),
    ) {
        let value = NotificationReceipt {
            notification_id,
            queued_at: chrono::Utc::now(),
            channels_dispatched: channels,
        };
        roundtrip(&value);
    }

    #[test]
    fn audit_event_roundtrips(
        module in arb_short_string(),
        action in arb_audit_action(),
        resource_type in arb_short_string(),
        resource_id in proptest::option::of(arb_short_string()),
        actor_user_id in proptest::option::of(arb_uuid()),
    ) {
        let mut value = AuditEvent::new(module, action, resource_type);
        value.resource_id = resource_id;
        value.actor_user_id = actor_user_id;
        roundtrip(&value);
    }
}
