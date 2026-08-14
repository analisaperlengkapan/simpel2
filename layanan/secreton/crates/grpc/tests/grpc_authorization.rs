//! Does the gRPC boundary refuse a workload whose policies do not cover the
//! secret path it asked for?
//!
//! mTLS answers *who is calling* — `mtls_enforcement.rs` covers that. This file
//! answers *what they may reach*, which until #129 nothing did: the four secret
//! handlers went straight to `self.storage`, so any workload holding a
//! CA-signed certificate could read, overwrite and delete every secret at every
//! path. A valid `gateway` certificate proves the caller is the gateway; it says
//! nothing about whether the gateway may read `postgres/authenc`.
//!
//! The handlers are driven directly rather than through a TLS listener, because
//! the transport is not what is under test here and a real handshake would make
//! every assertion below depend on certificate plumbing that already has its own
//! suite. The [`PeerIdentity`] is inserted into request extensions exactly where
//! `ClientAuth` puts it in production (`auth.rs`).
//!
//! Reference point for every assertion: a call that gets *through* authorization
//! reaches the handler, which answers `NotFound` for a path that was never
//! written. So `NotFound` means "authorized and dispatched", and anything else
//! means the boundary stopped it — the same discriminator the REST suite in
//! `secreton-api/tests/policy_enforcement.rs` uses.
//!
//! What is faked here is only where rules are *stored*, never how they are
//! *evaluated*: `PolicySet` does the real glob matching in every case below.
//! Faking the decision instead would certify nothing.

use std::collections::HashMap;
use std::sync::Arc;

use secreton_core::error::CoreError;
use secreton_core::models::PolicyRule;
use secreton_core::services::policy::PolicyRuleSource;
use secreton_crypto::transit::TransitEngine;
use secreton_grpc::generated::secreton::v1::secreton_service_server::SecretonService;
use secreton_grpc::generated::secreton::v1::{
    DeleteSecretRequest, GetSecretRequest, ListSecretsRequest, StoreSecretRequest,
};
use secreton_grpc::{GrpcAuthorization, PeerIdentity, SecretonGrpcService};
use secreton_storage::MemoryBackend;
use tonic::{Code, Request};

/// Rules keyed by policy name, resolved the way `PolicyService` resolves them:
/// the union of the rules held by every name the caller presents.
struct StoredPolicies(HashMap<String, Vec<PolicyRule>>);

#[async_trait::async_trait]
impl PolicyRuleSource for StoredPolicies {
    async fn rules_for_policies(
        &self,
        names: &[String],
        _namespace: &str,
    ) -> Result<Vec<PolicyRule>, CoreError> {
        Ok(names
            .iter()
            .filter_map(|n| self.0.get(n))
            .flatten()
            .cloned()
            .collect())
    }
}

/// A policy store that cannot be reached.
///
/// Exists to pin the one behaviour that must never be convenient: when
/// authorization cannot be determined, the answer is "no".
struct UnreachablePolicies;

#[async_trait::async_trait]
impl PolicyRuleSource for UnreachablePolicies {
    async fn rules_for_policies(
        &self,
        _names: &[String],
        _namespace: &str,
    ) -> Result<Vec<PolicyRule>, CoreError> {
        Err(CoreError::Internal {
            message: "policy store is down".to_string(),
            source: None,
        })
    }
}

fn allow(path: &str, action: &str) -> PolicyRule {
    PolicyRule {
        effect: "allow".to_string(),
        action: action.to_string(),
        path: path.to_string(),
        condition: None,
        control_group: None,
        mfa: None,
    }
}

/// The shape the Helm `secretonAuth.policies` block renders: the certificate
/// common name is the policy name, and the policy names resource paths.
fn gateway_may_read_simpelv1() -> Arc<dyn PolicyRuleSource> {
    let mut policies = HashMap::new();
    policies.insert(
        "gateway".to_string(),
        vec![allow("secret/data/simpelv1/*", "read")],
    );
    Arc::new(StoredPolicies(policies))
}

fn service(authorization: GrpcAuthorization) -> SecretonGrpcService {
    SecretonGrpcService::new(
        Arc::new(MemoryBackend::new()),
        Arc::new(TransitEngine::new()),
        None,
        authorization,
    )
}

fn as_peer<T>(message: T, common_name: &str) -> Request<T> {
    let mut request = Request::new(message);
    request.extensions_mut().insert(PeerIdentity {
        common_name: common_name.to_string(),
        serial: "00".to_string(),
    });
    request
}

#[tokio::test]
async fn a_workload_reaches_the_path_its_policy_names() {
    let service = service(GrpcAuthorization::Enforced(gateway_may_read_simpelv1()));

    let status = service
        .get_secret(as_peer(
            GetSecretRequest {
                path: "simpelv1/app".to_string(),
                ..Default::default()
            },
            "gateway",
        ))
        .await
        .expect_err("nothing was ever written at this path");

    assert_eq!(
        status.code(),
        Code::NotFound,
        "the gateway holds `secret/data/simpelv1/*` and asked for \
         `secret/data/simpelv1/app` — it must be dispatched to the handler, \
         which 404s on an unwritten path; got {status:?}"
    );
}

#[tokio::test]
async fn a_workload_cannot_reach_another_workloads_secrets() {
    let service = service(GrpcAuthorization::Enforced(gateway_may_read_simpelv1()));

    // This is the defect the whole task is about: before #129 this call
    // succeeded, because the handler never asked whether the caller was
    // allowed to be there.
    let status = service
        .get_secret(as_peer(
            GetSecretRequest {
                path: "postgres/authenc".to_string(),
                ..Default::default()
            },
            "gateway",
        ))
        .await
        .expect_err("the gateway has no policy covering authenc's database credentials");

    assert_eq!(
        status.code(),
        Code::PermissionDenied,
        "a workload reached outside its policy; got {status:?}"
    );
}

#[tokio::test]
async fn a_caller_with_no_certificate_is_refused() {
    let service = service(GrpcAuthorization::Enforced(gateway_may_read_simpelv1()));

    // No `PeerIdentity` in extensions: the call arrived without a client
    // certificate. There is no principal to authorize, so there is nothing that
    // could make this an allow.
    let status = service
        .get_secret(Request::new(GetSecretRequest {
            path: "simpelv1/app".to_string(),
            ..Default::default()
        }))
        .await
        .expect_err("an anonymous caller must not reach the handler");

    assert_eq!(
        status.code(),
        Code::Unauthenticated,
        "anonymous call was not refused; got {status:?}"
    );
}

/// Read access is not write access. Worth its own test because the four
/// handlers each pass a different action, and a copy-paste that sent `"read"`
/// from `store_secret` would turn every reader into a writer while every
/// read-path test stayed green.
#[tokio::test]
async fn read_access_does_not_confer_write_or_delete() {
    let service = service(GrpcAuthorization::Enforced(gateway_may_read_simpelv1()));

    let store = service
        .store_secret(as_peer(
            StoreSecretRequest {
                path: "simpelv1/app".to_string(),
                ..Default::default()
            },
            "gateway",
        ))
        .await
        .expect_err("a read-only policy must not authorize a write");
    assert_eq!(
        store.code(),
        Code::PermissionDenied,
        "read-only policy authorized a store; got {store:?}"
    );

    let delete = service
        .delete_secret(as_peer(
            DeleteSecretRequest {
                path: "simpelv1/app".to_string(),
            },
            "gateway",
        ))
        .await
        .expect_err("a read-only policy must not authorize a delete");
    assert_eq!(
        delete.code(),
        Code::PermissionDenied,
        "read-only policy authorized a delete; got {delete:?}"
    );
}

/// An unfiltered List is a request to enumerate every secret in the store.
#[tokio::test]
async fn an_unscoped_list_is_not_granted_by_a_subtree_policy() {
    let service = service(GrpcAuthorization::Enforced(gateway_may_read_simpelv1()));

    let status = service
        .list_secrets(as_peer(
            ListSecretsRequest {
                prefix: None,
                ..Default::default()
            },
            "gateway",
        ))
        .await
        .expect_err("listing the whole store is not covered by a subtree policy");

    assert_eq!(
        status.code(),
        Code::PermissionDenied,
        "an unscoped list enumerated the store; got {status:?}"
    );
}

/// A policy store that cannot be consulted must deny, not admit.
///
/// This is the failure mode that turns an outage into a breach, and it is
/// invisible in normal operation — the only time it runs is the one time it
/// matters.
#[tokio::test]
async fn an_unreachable_policy_store_denies_rather_than_admits() {
    let service = service(GrpcAuthorization::Enforced(Arc::new(UnreachablePolicies)));

    let status = service
        .get_secret(as_peer(
            GetSecretRequest {
                path: "simpelv1/app".to_string(),
                ..Default::default()
            },
            "gateway",
        ))
        .await
        .expect_err("policies could not be loaded, so the caller is not authorized");

    assert_ne!(
        status.code(),
        Code::NotFound,
        "a caller reached the handler while the policy store was unreachable"
    );
    assert_eq!(
        status.code(),
        Code::Internal,
        "expected the refusal to surface as an internal error; got {status:?}"
    );
}

/// The escape hatch, asserted rather than assumed.
///
/// `GRPC_ALLOW_INSECURE` really does serve every caller without a check — that
/// is what it is for, and staging currently depends on it (#128). Pinning it
/// here means the opt-out is a tested, deliberate behaviour rather than
/// something a reader has to infer, and it will go red the day someone changes
/// what "insecure" means without changing its name.
#[tokio::test]
async fn the_anonymous_opt_out_serves_callers_without_any_policy() {
    let service = service(GrpcAuthorization::AnonymousOptOut {
        reason: "test: no client certificates issued yet".to_string(),
    });

    let status = service
        .get_secret(Request::new(GetSecretRequest {
            path: "postgres/authenc".to_string(),
            ..Default::default()
        }))
        .await
        .expect_err("nothing was ever written at this path");

    assert_eq!(
        status.code(),
        Code::NotFound,
        "the opt-out is supposed to dispatch unauthenticated callers straight to \
         the handler; got {status:?}"
    );
}
