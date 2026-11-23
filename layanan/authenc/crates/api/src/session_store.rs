//! Temporary storage for the state of WebAuthn registration and authentication
//! ceremonies.
//!
//! A ceremony is two requests, `start` and `finish`, and the state webauthn-rs
//! hands back from `start` has to be there when `finish` arrives. authenc runs
//! more than one replica and the gateway does not pin a client to one of them,
//! so the state cannot live in the memory of the replica that served `start`:
//! with two replicas about half of all ceremonies finished on the other one and
//! died with "session not found or expired". It lives in Postgres instead.
//!
//! `take` is a single `DELETE ... RETURNING`, so a ceremony can be finished
//! exactly once, by whichever replica gets there first, and an expired one is
//! never returned.

use serde::{Serialize, de::DeserializeOwned};

use authenc_storage::Database;
use authenc_types::Result;
use authenc_webauthn::{AuthenticationSession, RegistrationSession};

/// How long a ceremony may stay open. Matches the 5 minute timeout the browser is
/// given, with headroom for a slow authenticator prompt.
const TTL: &str = "10 minutes";

const KIND_REGISTRATION: &str = "registration";
const KIND_AUTHENTICATION: &str = "authentication";

/// Session store for WebAuthn flows
#[derive(Clone)]
pub struct SessionStore {
    db: Database,
}

impl SessionStore {
    /// Create a session store on the shared database.
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    /// Store a registration session.
    pub async fn store_registration_session(
        &self,
        session_id: String,
        session: RegistrationSession,
    ) -> Result<()> {
        self.put(&session_id, KIND_REGISTRATION, &session).await
    }

    /// Take a registration session: it is returned once and removed.
    pub async fn remove_registration_session(
        &self,
        session_id: &str,
    ) -> Result<Option<RegistrationSession>> {
        self.take(session_id, KIND_REGISTRATION).await
    }

    /// Store an authentication session.
    pub async fn store_authentication_session(
        &self,
        session_id: String,
        session: AuthenticationSession,
    ) -> Result<()> {
        self.put(&session_id, KIND_AUTHENTICATION, &session).await
    }

    /// Take an authentication session: it is returned once and removed.
    pub async fn remove_authentication_session(
        &self,
        session_id: &str,
    ) -> Result<Option<AuthenticationSession>> {
        self.take(session_id, KIND_AUTHENTICATION).await
    }

    async fn put<T: Serialize>(&self, session_id: &str, kind: &str, value: &T) -> Result<()> {
        let payload = serde_json::to_string(value).map_err(|e| {
            authenc_types::AuthencError::internal(format!(
                "Failed to serialise the WebAuthn ceremony state: {e}"
            ))
        })?;
        // Expired rows go on every write, so the table never outgrows the traffic
        // of the last few minutes and there is no timer to run or to forget.
        self.db
            .execute(
                "DELETE FROM authenc.webauthn_ceremony_sessions WHERE expires_at <= now()",
                &[],
            )
            .await?;
        self.db
            .execute(
                &format!(
                    "INSERT INTO authenc.webauthn_ceremony_sessions (id, kind, payload, expires_at) \
                     VALUES ($1, $2, $3, now() + interval '{TTL}')"
                ),
                &[&session_id, &kind, &payload],
            )
            .await?;
        Ok(())
    }

    async fn take<T: DeserializeOwned>(&self, session_id: &str, kind: &str) -> Result<Option<T>> {
        let row = self
            .db
            .query_opt(
                "DELETE FROM authenc.webauthn_ceremony_sessions \
                 WHERE id = $1 AND kind = $2 AND expires_at > now() \
                 RETURNING payload",
                &[&session_id, &kind],
            )
            .await?;
        let Some(row) = row else {
            return Ok(None);
        };
        let payload: String = row.get("payload");
        match serde_json::from_str(&payload) {
            Ok(value) => Ok(Some(value)),
            Err(e) => {
                // A row this build cannot read (written by a different version).
                // Treat it as a ceremony that is gone: the user starts again.
                tracing::warn!(error = %e, kind, "Unreadable WebAuthn ceremony state, discarding");
                Ok(None)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    //! These need a Postgres with the authenc migrations applied (006 creates the
    //! table), so they are `#[ignore]`d in a plain `cargo test` and run with
    //!
    //! ```text
    //! AUTHENC_TEST_DATABASE_URL=postgres://… cargo test -p authenc-api \
    //!     session_store -- --ignored
    //! ```
    //!
    //! They exercise the SQL, which no unit test can: single use, expiry, the
    //! `kind` guard, and two replicas racing to finish the same ceremony.

    use super::*;
    use serde_json::{Value, json};

    async fn store() -> SessionStore {
        let url = std::env::var("AUTHENC_TEST_DATABASE_URL")
            .expect("AUTHENC_TEST_DATABASE_URL must point at a migrated database");
        SessionStore::new(Database::new(&url, 4).await.expect("connect"))
    }

    fn id() -> String {
        uuid::Uuid::new_v4().to_string()
    }

    #[tokio::test]
    #[ignore = "needs a migrated Postgres: see the module doc"]
    async fn a_ceremony_is_returned_once_and_only_once() {
        let s = store().await;
        let sid = id();
        s.put(&sid, KIND_AUTHENTICATION, &json!({"n": 1}))
            .await
            .unwrap();

        let first: Option<Value> = s.take(&sid, KIND_AUTHENTICATION).await.unwrap();
        assert_eq!(first, Some(json!({"n": 1})));
        let second: Option<Value> = s.take(&sid, KIND_AUTHENTICATION).await.unwrap();
        assert_eq!(
            second, None,
            "a finished ceremony must not be finishable twice"
        );
    }

    #[tokio::test]
    #[ignore = "needs a migrated Postgres: see the module doc"]
    async fn an_expired_ceremony_is_never_returned() {
        let s = store().await;
        let sid = id();
        s.put(&sid, KIND_REGISTRATION, &json!({"n": 2}))
            .await
            .unwrap();
        s.db.execute(
            "UPDATE authenc.webauthn_ceremony_sessions \
                 SET expires_at = now() - interval '1 second' WHERE id = $1",
            &[&sid],
        )
        .await
        .unwrap();

        let got: Option<Value> = s.take(&sid, KIND_REGISTRATION).await.unwrap();
        assert_eq!(got, None);
    }

    #[tokio::test]
    #[ignore = "needs a migrated Postgres: see the module doc"]
    async fn a_registration_cannot_be_finished_as_an_authentication() {
        let s = store().await;
        let sid = id();
        s.put(&sid, KIND_REGISTRATION, &json!({"n": 3}))
            .await
            .unwrap();

        let wrong: Option<Value> = s.take(&sid, KIND_AUTHENTICATION).await.unwrap();
        assert_eq!(wrong, None);
        // The wrong-kind attempt did not consume it.
        let right: Option<Value> = s.take(&sid, KIND_REGISTRATION).await.unwrap();
        assert_eq!(right, Some(json!({"n": 3})));
    }

    #[tokio::test]
    #[ignore = "needs a migrated Postgres: see the module doc"]
    async fn two_replicas_racing_to_finish_one_ceremony_get_one_winner() {
        let s = store().await;
        let sid = id();
        s.put(&sid, KIND_AUTHENTICATION, &json!({"n": 4}))
            .await
            .unwrap();

        // Two stores on separate connection pools stand in for two replicas.
        let other = store().await;
        let (a, b) = tokio::join!(
            s.take::<Value>(&sid, KIND_AUTHENTICATION),
            other.take::<Value>(&sid, KIND_AUTHENTICATION),
        );
        let winners = [a.unwrap(), b.unwrap()]
            .into_iter()
            .filter(Option::is_some)
            .count();
        assert_eq!(winners, 1, "exactly one replica may finish the ceremony");
    }

    #[tokio::test]
    #[ignore = "needs a migrated Postgres: see the module doc"]
    async fn a_row_this_build_cannot_read_is_treated_as_gone() {
        let s = store().await;
        let sid = id();
        s.db.execute(
            "INSERT INTO authenc.webauthn_ceremony_sessions (id, kind, payload, expires_at) \
                 VALUES ($1, $2, 'not json', now() + interval '1 minute')",
            &[&sid, &KIND_AUTHENTICATION],
        )
        .await
        .unwrap();

        let got: Option<Value> = s.take(&sid, KIND_AUTHENTICATION).await.unwrap();
        assert_eq!(got, None);
    }
}
