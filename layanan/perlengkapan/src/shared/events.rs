// ============================================================================
// shared/events.rs — In-process Event Bus (Fase 1.4)
// ============================================================================
//
// Sebelum: WorkflowEngine.transition() memanggil audit_sink + notifier
// langsung inline. Tight coupling = sulit menambah subscriber baru (mis.
// websocket push, search index update, metrics) tanpa menyentuh engine.
//
// Sesudah: EventBus tipis di atas `tokio::sync::broadcast`. Engine
// mem-publish `DomainEvent` setelah transisi committed. Subscriber apa
// pun (audit, notif, real-time monitoring, integrasi luar) tap-in via
// `bus.subscribe()` dan jalan di task sendiri — kegagalan satu subscriber
// tidak men-stop yg lain.
//
// In-process saja — untuk pub-sub lintas-service pakai gRPC streaming
// atau message broker. Out of scope di plan ini.
//
// Backward-compat: WorkflowEngine tetap menerima `audit_sink` langsung
// (Fase 0.10). EventBus adalah TAMBAHAN, bukan pengganti — agar migrasi
// inkremental.
// ============================================================================

use chrono::{DateTime, Utc};
use std::sync::Arc;
use tokio::sync::broadcast;
use uuid::Uuid;

/// Default capacity broadcast channel. Subscriber lambat akan menerima
/// `Err(RecvError::Lagged)` jika ringbuffer penuh — penanganan
/// terstandarisasi di tiap subscriber (skip + log + continue).
const DEFAULT_CAPACITY: usize = 1024;

/// Domain event dipublish ke seluruh subscriber via `EventBus`.
///
/// Variant harus tetap `Clone + Send + Sync` agar bisa lewat
/// broadcast channel. Field besar (mis. dokumen biner) HARUS disimpan
/// di storage dgn URL, bukan di-embed sebagai bytes.
#[derive(Debug, Clone)]
pub enum DomainEvent {
    /// Workflow transition berhasil committed.
    WorkflowTransitioned {
        entity_type: String,
        entity_id: Uuid,
        from_state: String,
        to_state: String,
        user_id: Uuid,
        catatan: Option<String>,
        ip_address: String,
        timestamp: DateTime<Utc>,
    },
    /// Dokumen (DOCX/PDF) ter-generate dan tersimpan di DocumentStorage.
    DocumentGenerated {
        entity_type: String,
        entity_id: Uuid,
        document_type: String,
        url: String,
        timestamp: DateTime<Utc>,
    },
    /// Izin pemakaian BMN aktif (post Approver Satker approve).
    PermitActivated {
        permit_id: Uuid,
        nomor_izin: String,
        pegawai_nip: String,
        timestamp: DateTime<Utc>,
    },
    /// Izin pemakaian BMN dicabut.
    PermitRevoked {
        permit_id: Uuid,
        reason: String,
        revoked_by: Uuid,
        timestamp: DateTime<Utc>,
    },
}

impl DomainEvent {
    /// Tag pendek utk logging — tidak meng-clone payload.
    pub fn kind(&self) -> &'static str {
        match self {
            Self::WorkflowTransitioned { .. } => "workflow.transitioned",
            Self::DocumentGenerated { .. } => "document.generated",
            Self::PermitActivated { .. } => "permit.activated",
            Self::PermitRevoked { .. } => "permit.revoked",
        }
    }
}

/// In-process pub-sub bus untuk `DomainEvent`. Aman di-clone (channel
/// sender adalah refcounted) → cukup `Arc<EventBus>` di state app.
#[derive(Debug, Clone)]
pub struct EventBus {
    sender: broadcast::Sender<DomainEvent>,
}

impl EventBus {
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self { sender }
    }

    pub fn with_default_capacity() -> Self {
        Self::new(DEFAULT_CAPACITY)
    }

    /// Publish event ke seluruh subscriber. Return jumlah subscriber yg
    /// dapat menerima (0 jika tidak ada subscriber — bukan error).
    pub fn publish(&self, event: DomainEvent) -> usize {
        // `send` mengembalikan Err hanya jika tidak ada subscriber.
        // Itu BUKAN kondisi error — banyak deployment dev tidak punya
        // subscriber. Log debug saja.
        self.sender.send(event).unwrap_or_default()
    }

    /// Subscribe — return Receiver baru. Subscriber baru mulai dari event
    /// berikutnya (tidak replay event lama; gunakan database utk historis).
    pub fn subscribe(&self) -> broadcast::Receiver<DomainEvent> {
        self.sender.subscribe()
    }

    /// Jumlah subscriber aktif. Berguna utk health/metrics.
    pub fn subscriber_count(&self) -> usize {
        self.sender.receiver_count()
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::with_default_capacity()
    }
}

/// Trait subscriber yg dapat dipasang via `spawn_subscriber()`.
/// Implement utk component spesifik (audit_sink, notif_dispatcher, dll).
#[async_trait::async_trait]
pub trait EventSubscriber: Send + Sync {
    fn name(&self) -> &'static str;
    async fn handle(&self, event: DomainEvent);
}

/// Spawn satu subscriber jadi task background. Task akan jalan sampai
/// `EventBus` sender di-drop atau program shutdown. Lagging diberitahu
/// via warning log lalu loop di-restart agar tidak putus permanen.
pub fn spawn_subscriber(bus: &EventBus, subscriber: Arc<dyn EventSubscriber>) {
    let mut rx = bus.subscribe();
    let name = subscriber.name();
    tokio::spawn(async move {
        loop {
            match rx.recv().await {
                Ok(event) => subscriber.handle(event).await,
                Err(broadcast::error::RecvError::Lagged(n)) => {
                    tracing::warn!(
                        subscriber = name,
                        dropped = n,
                        "EventBus subscriber tertinggal — event skipped"
                    );
                    // Loop terus — receiver sudah otomatis sync di skip
                    // ke event terbaru.
                }
                Err(broadcast::error::RecvError::Closed) => {
                    tracing::info!(subscriber = name, "EventBus closed — subscriber exit");
                    return;
                }
            }
        }
    });
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tokio::time::{Duration, sleep};

    fn sample_event() -> DomainEvent {
        DomainEvent::WorkflowTransitioned {
            entity_type: "pemakaian_bmn".into(),
            entity_id: Uuid::nil(),
            from_state: "DRAFT".into(),
            to_state: "SUBMITTED".into(),
            user_id: Uuid::nil(),
            catatan: None,
            ip_address: "127.0.0.1".into(),
            timestamp: Utc::now(),
        }
    }

    #[tokio::test]
    async fn publish_with_no_subscriber_is_noop() {
        let bus = EventBus::with_default_capacity();
        let n = bus.publish(sample_event());
        assert_eq!(n, 0);
    }

    #[tokio::test]
    async fn multi_subscriber_fan_out() {
        let bus = EventBus::with_default_capacity();
        let mut rx1 = bus.subscribe();
        let mut rx2 = bus.subscribe();
        assert_eq!(bus.subscriber_count(), 2);

        let n = bus.publish(sample_event());
        assert_eq!(n, 2);

        let ev1 = rx1.recv().await.unwrap();
        let ev2 = rx2.recv().await.unwrap();
        assert_eq!(ev1.kind(), "workflow.transitioned");
        assert_eq!(ev2.kind(), "workflow.transitioned");
    }

    struct CountingSubscriber {
        counter: Arc<AtomicUsize>,
    }
    #[async_trait::async_trait]
    impl EventSubscriber for CountingSubscriber {
        fn name(&self) -> &'static str {
            "counter"
        }
        async fn handle(&self, _event: DomainEvent) {
            self.counter.fetch_add(1, Ordering::SeqCst);
        }
    }

    #[tokio::test]
    async fn spawn_subscriber_receives_events() {
        let bus = EventBus::with_default_capacity();
        let counter = Arc::new(AtomicUsize::new(0));
        spawn_subscriber(
            &bus,
            Arc::new(CountingSubscriber {
                counter: counter.clone(),
            }),
        );
        // Yield agar task subscriber sempat subscribe.
        sleep(Duration::from_millis(10)).await;

        bus.publish(sample_event());
        bus.publish(sample_event());
        bus.publish(sample_event());

        // Beri kesempatan task untuk konsumsi.
        sleep(Duration::from_millis(20)).await;
        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn event_kind_tags() {
        let activated = DomainEvent::PermitActivated {
            permit_id: Uuid::nil(),
            nomor_izin: "IP/2026/05/0001".into(),
            pegawai_nip: "199001012010012001".into(),
            timestamp: Utc::now(),
        };
        assert_eq!(activated.kind(), "permit.activated");
    }
}
