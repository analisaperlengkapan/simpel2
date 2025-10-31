/// Unified batch processing module
/// Supports both file-based and database storage strategies
pub mod fetchers;
pub mod orchestrator;

pub use fetchers::*;
pub use orchestrator::*;

/// Konstanta untuk KL Kejaksaan RI
pub const KL_KEJAKSAAN: &str = "006";
