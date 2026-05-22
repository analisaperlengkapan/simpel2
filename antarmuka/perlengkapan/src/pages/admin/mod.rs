//! Admin-only pages: audit log viewer, master-data hub, and document
//! template manager.

pub mod audit_page;
pub mod master_data_page;
pub mod templates_page;

pub use audit_page::AdminAuditPage;
pub use master_data_page::AdminMasterDataPage;
pub use templates_page::AdminTemplatesPage;
