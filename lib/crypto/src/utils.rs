pub mod crypto_monitor {
    pub struct CryptoMonitor;

    impl CryptoMonitor {
        pub fn monitor_rsa_operation<F, T>(_op_name: &str, f: F) -> T
        where
            F: FnOnce() -> T,
        {
            // Simple pass-through for now, can add logging/metrics later
            f()
        }
    }
}
