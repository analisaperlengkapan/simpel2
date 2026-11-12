//! Admin-only pages: audit log viewer, master-data hub, document template
//! manager, and the master-data sync status.

pub mod audit_page;
pub mod master_data_page;
pub mod sync_status_page;
pub mod templates_page;

pub use audit_page::AdminAuditPage;
pub use master_data_page::AdminMasterDataPage;
pub use sync_status_page::AdminSyncStatusPage;
pub use templates_page::AdminTemplatesPage;
