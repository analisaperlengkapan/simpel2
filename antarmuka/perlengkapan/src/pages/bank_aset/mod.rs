//! Bank Aset pages — unified read-only façade over SIMAN data.

pub mod dashboard_page;
pub mod detail_page;
pub mod list_page;
pub mod qrcode_page;
pub mod sebaran_page;

pub use dashboard_page::BankAsetDashboardPage;
pub use detail_page::BankAsetDetailPage;
pub use list_page::BankAsetListPage;
pub use qrcode_page::BankAsetQrCodePage;
pub use sebaran_page::BankAsetSebaranPage;
