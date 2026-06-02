//! # Resilience: Circuit Breaker + Retry + Timeout (Fase 2.2)
//!
//! Lapisan tahan-banting untuk panggilan sistem eksternal (SIMAN / MonSAKTI /
//! MySIMKARI lewat `layanan-integrasi` gRPC). Sebelumnya tiap panggilan gRPC
//! langsung `await?` tanpa timeout, retry, atau proteksi cascading-failure —
//! downtime sumber eksternal = request menggantung lalu gagal.
//!
//! Tiga mekanisme digabung di [`guarded`]:
//! 1. **Timeout per percobaan** — panggilan yang menggantung dibatalkan.
//! 2. **Retry backoff eksponensial** — error/timeout sementara dicoba ulang
//!    (aman karena seluruh panggilan integrasi bersifat read-only/idempoten).
//! 3. **Circuit breaker** — setelah N kegagalan beruntun, sirkuit *membuka*
//!    dan panggilan berikut langsung gagal-cepat (`CircuitOpen`) tanpa
//!    menyentuh sumber yang sedang sakit; setelah jeda, masuk *half-open*
//!    untuk satu percobaan pemulihan.

use std::sync::Mutex;
use std::time::{Duration, Instant};

// ============================================================================
// Konfigurasi
// ============================================================================

/// Parameter circuit breaker.
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// Jumlah kegagalan beruntun (state Closed) sebelum sirkuit membuka.
    pub failure_threshold: u32,
    /// Lama sirkuit bertahan Open sebelum diizinkan satu percobaan half-open.
    pub open_duration: Duration,
    /// Jumlah keberhasilan di half-open sebelum sirkuit menutup kembali.
    pub success_threshold: u32,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            open_duration: Duration::from_secs(30),
            success_threshold: 2,
        }
    }
}

/// Parameter retry + timeout per panggilan ter-`guarded`.
#[derive(Debug, Clone)]
pub struct ResiliencePolicy {
    /// Timeout satu percobaan (R5: ~3 detik untuk panggilan integrasi).
    pub timeout: Duration,
    /// Jumlah maksimum percobaan ulang (total percobaan = max_retries + 1).
    pub max_retries: u32,
    /// Basis backoff (digandakan tiap percobaan).
    pub backoff_base: Duration,
    /// Batas atas backoff.
    pub backoff_max: Duration,
}

impl Default for ResiliencePolicy {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(3),
            max_retries: 2,
            backoff_base: Duration::from_millis(100),
            backoff_max: Duration::from_secs(2),
        }
    }
}

// ============================================================================
// Circuit breaker
// ============================================================================

/// Snapshot status sirkuit (untuk health endpoint / banner UI).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

#[derive(Debug)]
enum InnerState {
    Closed,
    Open { since: Instant },
    HalfOpen,
}

#[derive(Debug)]
struct Inner {
    state: InnerState,
    consecutive_failures: u32,
    half_open_successes: u32,
}

/// Circuit breaker thread-safe. Aman di-`clone` lewat `Arc` di pemanggil;
/// state internal dibagi via `Mutex` (seksi kritis kecil, tanpa await).
#[derive(Debug)]
pub struct CircuitBreaker {
    name: String,
    config: CircuitBreakerConfig,
    inner: Mutex<Inner>,
}

impl CircuitBreaker {
    pub fn new(name: impl Into<String>, config: CircuitBreakerConfig) -> Self {
        Self {
            name: name.into(),
            config,
            inner: Mutex::new(Inner {
                state: InnerState::Closed,
                consecutive_failures: 0,
                half_open_successes: 0,
            }),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    /// Apakah panggilan diizinkan? Melakukan transisi Open→HalfOpen bila
    /// `open_duration` sudah terlewati. `false` = gagal-cepat.
    pub fn allow(&self) -> bool {
        let mut inner = self.inner.lock().unwrap();
        match inner.state {
            InnerState::Closed | InnerState::HalfOpen => true,
            InnerState::Open { since } => {
                if since.elapsed() >= self.config.open_duration {
                    inner.state = InnerState::HalfOpen;
                    inner.half_open_successes = 0;
                    tracing::info!(breaker = %self.name, "circuit breaker: Open → HalfOpen (percobaan pemulihan)");
                    true
                } else {
                    false
                }
            }
        }
    }

    /// Catat keberhasilan satu operasi ter-guarded.
    pub fn on_success(&self) {
        let mut inner = self.inner.lock().unwrap();
        match inner.state {
            InnerState::Closed => {
                inner.consecutive_failures = 0;
            }
            InnerState::HalfOpen => {
                inner.half_open_successes += 1;
                if inner.half_open_successes >= self.config.success_threshold {
                    inner.state = InnerState::Closed;
                    inner.consecutive_failures = 0;
                    inner.half_open_successes = 0;
                    tracing::info!(breaker = %self.name, "circuit breaker: HalfOpen → Closed (pulih)");
                }
            }
            InnerState::Open { .. } => {
                // Tidak lazim (allow() seharusnya mengubah ke HalfOpen dulu);
                // perlakukan sebagai pemulihan.
                inner.state = InnerState::Closed;
                inner.consecutive_failures = 0;
                inner.half_open_successes = 0;
            }
        }
    }

    /// Catat kegagalan satu operasi ter-guarded.
    pub fn on_failure(&self) {
        let mut inner = self.inner.lock().unwrap();
        match inner.state {
            InnerState::Closed => {
                inner.consecutive_failures += 1;
                if inner.consecutive_failures >= self.config.failure_threshold {
                    inner.state = InnerState::Open {
                        since: Instant::now(),
                    };
                    tracing::warn!(
                        breaker = %self.name,
                        failures = inner.consecutive_failures,
                        "circuit breaker: Closed → Open (gagal-cepat aktif)"
                    );
                }
            }
            InnerState::HalfOpen => {
                inner.state = InnerState::Open {
                    since: Instant::now(),
                };
                inner.half_open_successes = 0;
                tracing::warn!(breaker = %self.name, "circuit breaker: HalfOpen → Open (percobaan pemulihan gagal)");
            }
            InnerState::Open { .. } => {
                inner.state = InnerState::Open {
                    since: Instant::now(),
                };
            }
        }
    }

    /// Status sirkuit saat ini (untuk health/banner).
    pub fn state(&self) -> CircuitState {
        let inner = self.inner.lock().unwrap();
        match inner.state {
            InnerState::Closed => CircuitState::Closed,
            InnerState::Open { .. } => CircuitState::Open,
            InnerState::HalfOpen => CircuitState::HalfOpen,
        }
    }
}

// ============================================================================
// Error
// ============================================================================

/// Hasil error dari [`guarded`].
#[derive(Debug)]
pub enum ResilienceError<E> {
    /// Sirkuit terbuka — gagal-cepat tanpa memanggil operasi.
    CircuitOpen { name: String },
    /// Seluruh percobaan habis karena timeout.
    Timeout { name: String, timeout: Duration },
    /// Operasi mengembalikan error (setelah retry habis).
    Operation(E),
}

impl<E: std::fmt::Display> std::fmt::Display for ResilienceError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResilienceError::CircuitOpen { name } => {
                write!(f, "sirkuit '{}' terbuka — sumber eksternal sedang tidak tersedia", name)
            }
            ResilienceError::Timeout { name, timeout } => {
                write!(f, "panggilan '{}' timeout setelah {:?}", name, timeout)
            }
            ResilienceError::Operation(e) => write!(f, "{}", e),
        }
    }
}

impl<E: std::fmt::Display + std::fmt::Debug> std::error::Error for ResilienceError<E> {}

// ============================================================================
// guarded()
// ============================================================================

/// Jalankan `op` dengan proteksi circuit breaker + retry + timeout.
///
/// Breaker dicek satu kali di awal (gagal-cepat bila Open). Bila lolos, `op`
/// dijalankan dengan timeout per percobaan dan diulang sampai `max_retries`.
/// Hasil terminal (sukses/gagal) dicatat ke breaker satu kali.
pub async fn guarded<T, E, F, Fut>(
    breaker: &CircuitBreaker,
    policy: &ResiliencePolicy,
    mut op: F,
) -> Result<T, ResilienceError<E>>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
    E: std::fmt::Display,
{
    if !breaker.allow() {
        tracing::warn!(breaker = %breaker.name, "circuit breaker terbuka — gagal-cepat");
        return Err(ResilienceError::CircuitOpen {
            name: breaker.name.clone(),
        });
    }

    let mut attempt: u32 = 0;
    loop {
        match tokio::time::timeout(policy.timeout, op()).await {
            Ok(Ok(value)) => {
                breaker.on_success();
                return Ok(value);
            }
            Ok(Err(err)) => {
                if attempt >= policy.max_retries {
                    breaker.on_failure();
                    return Err(ResilienceError::Operation(err));
                }
                tracing::warn!(breaker = %breaker.name, attempt, error = %err, "resilience: retry setelah error");
            }
            Err(_elapsed) => {
                if attempt >= policy.max_retries {
                    breaker.on_failure();
                    return Err(ResilienceError::Timeout {
                        name: breaker.name.clone(),
                        timeout: policy.timeout,
                    });
                }
                tracing::warn!(breaker = %breaker.name, attempt, "resilience: retry setelah timeout");
            }
        }

        attempt += 1;
        let backoff = policy
            .backoff_base
            .saturating_mul(2u32.saturating_pow(attempt - 1))
            .min(policy.backoff_max);
        tokio::time::sleep(backoff).await;
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    fn cfg() -> CircuitBreakerConfig {
        CircuitBreakerConfig {
            failure_threshold: 3,
            open_duration: Duration::from_millis(40),
            success_threshold: 2,
        }
    }

    #[test]
    fn opens_after_consecutive_failures() {
        let cb = CircuitBreaker::new("test", cfg());
        assert_eq!(cb.state(), CircuitState::Closed);
        assert!(cb.allow());
        cb.on_failure();
        cb.on_failure();
        assert_eq!(cb.state(), CircuitState::Closed, "belum mencapai ambang");
        cb.on_failure(); // ke-3 → buka
        assert_eq!(cb.state(), CircuitState::Open);
        assert!(!cb.allow(), "Open harus gagal-cepat");
    }

    #[test]
    fn success_resets_failure_count() {
        let cb = CircuitBreaker::new("test", cfg());
        cb.on_failure();
        cb.on_failure();
        cb.on_success(); // reset
        cb.on_failure();
        cb.on_failure();
        assert_eq!(cb.state(), CircuitState::Closed, "sukses mereset hitungan");
    }

    #[test]
    fn half_open_then_close_on_success() {
        let cb = CircuitBreaker::new("test", cfg());
        cb.on_failure();
        cb.on_failure();
        cb.on_failure();
        assert_eq!(cb.state(), CircuitState::Open);
        // tunggu open_duration → allow() memicu HalfOpen
        std::thread::sleep(Duration::from_millis(55));
        assert!(cb.allow());
        assert_eq!(cb.state(), CircuitState::HalfOpen);
        cb.on_success(); // 1 dari 2
        assert_eq!(cb.state(), CircuitState::HalfOpen);
        cb.on_success(); // 2 dari 2 → menutup
        assert_eq!(cb.state(), CircuitState::Closed);
    }

    #[test]
    fn half_open_reopens_on_failure() {
        let cb = CircuitBreaker::new("test", cfg());
        cb.on_failure();
        cb.on_failure();
        cb.on_failure();
        std::thread::sleep(Duration::from_millis(55));
        assert!(cb.allow());
        assert_eq!(cb.state(), CircuitState::HalfOpen);
        cb.on_failure(); // gagal di half-open → buka lagi
        assert_eq!(cb.state(), CircuitState::Open);
    }

    #[tokio::test]
    async fn guarded_retries_then_succeeds() {
        let cb = CircuitBreaker::new("test", cfg());
        let policy = ResiliencePolicy {
            timeout: Duration::from_millis(200),
            max_retries: 3,
            backoff_base: Duration::from_millis(1),
            backoff_max: Duration::from_millis(5),
        };
        let calls = Cell::new(0);
        let result: Result<i32, ResilienceError<&str>> = guarded(&cb, &policy, || {
            let n = calls.get() + 1;
            calls.set(n);
            async move {
                if n < 3 {
                    Err("transient")
                } else {
                    Ok(42)
                }
            }
        })
        .await;
        assert_eq!(result.unwrap(), 42);
        assert_eq!(calls.get(), 3, "gagal 2x lalu sukses");
        assert_eq!(cb.state(), CircuitState::Closed);
    }

    #[tokio::test]
    async fn guarded_fast_fails_when_open() {
        let cb = CircuitBreaker::new("test", cfg());
        cb.on_failure();
        cb.on_failure();
        cb.on_failure();
        assert_eq!(cb.state(), CircuitState::Open);
        let policy = ResiliencePolicy::default();
        let calls = Cell::new(0);
        let result: Result<i32, ResilienceError<&str>> = guarded(&cb, &policy, || {
            calls.set(calls.get() + 1);
            async move { Ok(1) }
        })
        .await;
        assert!(matches!(result, Err(ResilienceError::CircuitOpen { .. })));
        assert_eq!(calls.get(), 0, "operasi tidak dipanggil saat Open");
    }
}
