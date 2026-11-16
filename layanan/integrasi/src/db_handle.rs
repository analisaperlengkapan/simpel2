//! A database handle that outlives its connection.
//!
//! `tokio_postgres::connect` returns a `Client` and a `Connection` future. When that future ends —
//! the peer closed the socket, Postgres restarted, a sidecar enforced its idle timeout — the
//! `Client` is dead for good: every later call fails with `connection closed`, forever. The
//! gRPC/HTTP server used to hold exactly one such `Client` for the whole life of the process.
//!
//! Seen on staging (rc46, 2026-11-11): the pod started at 22:39:38; at 23:39:38 — one hour later,
//! to the second, with no traffic in between — the log said `Database connection error:
//! connection closed`; from then on `GET /api/monitoring/health` answered 503
//! `{"database":"disconnected","error":"connection closed"}` and every pegawai/satker/foto query
//! failed until the pod was restarted. The pod stayed `Ready` (its probes are TCP-only), and 218
//! e2e tests passed, because none of them ran after an idle hour. Production meets the same wall
//! at the first quiet hour, and at every Postgres restart (upgrade, node drain).
//!
//! `DbHandle` keeps the URL and swaps in a fresh connection when the current one is closed. It
//! offers only the four calls the server makes, all reads. They retry ONCE on a fresh connection
//! when the first attempt died with the connection: a read is safe to repeat, a write whose reply
//! was lost is not, so there is deliberately no `execute` here.

use std::future::Future;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::RwLock;
use tokio_postgres::types::ToSql;
use tokio_postgres::{Client, Config, Error, NoTls, Row, SimpleQueryMessage, ToStatement};
use tracing::{info, warn};

/// How long one attempt to open a connection may take. `tokio_postgres` has no default: a
/// Postgres that is down (restarting) would otherwise hold every caller for the OS connect timeout.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);

pub struct DbHandle {
    config: Config,
    current: RwLock<Arc<Client>>,
}

/// Did this error come from the connection going away, rather than from the statement?
///
/// When the SERVER ends a session (`pg_terminate_backend`, a restart, a shutdown) it first sends
/// an ErrorResponse — `57P01 terminating connection due to administrator command` — and only then
/// closes the socket. The caller sees that error while `Client::is_closed()` is still `false`:
/// the connection task has not noticed yet. Checking `is_closed()` alone would let exactly the
/// restart case through (found by the first run of the tests below). A sidecar or a firewall that
/// just drops the socket gives `is_closed()` instead.
fn connection_lost(client: &Client, error: &Error) -> bool {
    client.is_closed()
        || error.is_closed()
        || error.code().is_some_and(|c| {
            let code = c.code();
            // 57P01 admin_shutdown, 57P02 crash_shutdown, 57P03 cannot_connect_now; class 08 is
            // "connection exception".
            matches!(code, "57P01" | "57P02" | "57P03") || code.starts_with("08")
        })
}

impl DbHandle {
    /// Connect once, now; later calls reconnect on demand. Fails if the first connection fails,
    /// so a wrong `DATABASE_URL` still stops the process at start instead of at the first request.
    pub async fn connect(url: &str) -> Result<Arc<Self>, Error> {
        let mut config: Config = url.parse()?;
        if config.get_connect_timeout().is_none() {
            config.connect_timeout(CONNECT_TIMEOUT);
        }
        let client = open(&config).await?;
        Ok(Arc::new(Self {
            config,
            current: RwLock::new(client),
        }))
    }

    /// A client whose connection is alive, reconnecting if the last one is closed.
    pub async fn client(&self) -> Result<Arc<Client>, Error> {
        {
            let current = self.current.read().await;
            if !current.is_closed() {
                return Ok(current.clone());
            }
        }
        let mut current = self.current.write().await;
        // Another caller may have reconnected while this one waited for the write lock.
        if current.is_closed() {
            warn!("database connection is closed; reconnecting");
            *current = open(&self.config).await?;
            info!("database reconnected");
        }
        Ok(current.clone())
    }

    /// Replace `failed` with a fresh connection, unless another caller already did.
    ///
    /// Used after an error that says the connection is going away while it is not yet marked
    /// closed: asking `client()` again would hand back the same doomed client.
    async fn replace(&self, failed: &Arc<Client>) -> Result<Arc<Client>, Error> {
        let mut current = self.current.write().await;
        if Arc::ptr_eq(&current, failed) {
            warn!("database connection was lost; reconnecting");
            *current = open(&self.config).await?;
            info!("database reconnected");
        }
        Ok(current.clone())
    }

    /// Run a read on a live connection; if it fails because the connection was lost, run it ONCE
    /// more on a fresh one. A read is safe to repeat; a write whose reply was lost is not, which
    /// is why this type has no `execute`.
    async fn read<T, F, Fut>(&self, op: F) -> Result<T, Error>
    where
        F: Fn(Arc<Client>) -> Fut,
        Fut: Future<Output = Result<T, Error>>,
    {
        let client = self.client().await?;
        match op(client.clone()).await {
            Err(e) if connection_lost(&client, &e) => {
                warn!("database read failed because the connection was lost ({e}); retrying once");
                let fresh = self.replace(&client).await?;
                op(fresh).await
            }
            other => other,
        }
    }

    /// `Client::query` on a live connection. Reads only.
    pub async fn query<T>(
        &self,
        statement: &T,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<Vec<Row>, Error>
    where
        T: ?Sized + ToStatement + Sync,
    {
        self.read(|c| async move { c.query(statement, params).await })
            .await
    }

    /// `Client::query_one` on a live connection. Reads only.
    pub async fn query_one<T>(
        &self,
        statement: &T,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<Row, Error>
    where
        T: ?Sized + ToStatement + Sync,
    {
        self.read(|c| async move { c.query_one(statement, params).await })
            .await
    }

    /// `Client::query_opt` on a live connection. Reads only.
    pub async fn query_opt<T>(
        &self,
        statement: &T,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<Option<Row>, Error>
    where
        T: ?Sized + ToStatement + Sync,
    {
        self.read(|c| async move { c.query_opt(statement, params).await })
            .await
    }

    /// `Client::simple_query` on a live connection. Reads only.
    pub async fn simple_query(&self, query: &str) -> Result<Vec<SimpleQueryMessage>, Error> {
        self.read(|c| async move { c.simple_query(query).await })
            .await
    }
}

async fn open(config: &Config) -> Result<Arc<Client>, Error> {
    let (client, connection) = config.connect(NoTls).await?;
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            warn!("database connection ended: {e}");
        }
    });
    Ok(Arc::new(client))
}
