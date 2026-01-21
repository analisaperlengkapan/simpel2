//! Shared health check types and enums
//! Re-exported from lib_common for consistency across the workspace

pub use lib_common::health::{
    HealthStatus,
    DependencyHealth,
    HealthInfo,
    HealthReport,
    ComponentCheck,
    ServiceStatus,
};
