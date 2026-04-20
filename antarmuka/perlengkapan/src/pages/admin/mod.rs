//! Admin-only pages: audit log viewer and master-data hub.

pub mod audit_page;
pub mod master_data_page;

pub use audit_page::AdminAuditPage;
pub use master_data_page::AdminMasterDataPage;
