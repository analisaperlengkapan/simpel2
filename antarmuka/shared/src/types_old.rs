//! # Shared Types for SIMPel Frontend
//!
//! Comprehensive type definitions for all frontend microfrontends.
//! Includes UI components, forms, data structures, and utility types.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use leptos::prelude::*;
use leptos::ev;

// ========================================
// UI COMPONENT TYPES
// ========================================

#[derive(Clone, Debug, PartialEq)]
pub enum ButtonVariant {
    Primary,
    Secondary,
    Success,
    Warning,
    Error,
    Ghost,
    Outline,
}

#[derive(Clone, Debug, PartialEq)]
pub enum InputType {
    Text,
    Email,
    Password,
    Number,
    Tel,
    Url,
}

#[derive(Clone, Debug, PartialEq)]
pub enum AlertType {
    Success,
    Warning,
    Error,
    Info,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ModalSize {
    Small,
    Medium,
    Large,
    XLarge,
}

#[derive(Clone, Debug)]
pub struct SelectOption {
    pub value: String,
    pub label: String,
}

#[derive(Clone, Debug)]
pub struct RadioOption {
    pub value: String,
    pub label: String,
}

#[derive(Clone, Debug)]
pub struct TableColumn {
    pub key: String,
    pub header: String,
    pub sortable: bool,
}

#[derive(Clone)]
pub struct PageAction {
    pub label: String,
    pub icon: String,
    pub variant: ButtonVariant,
    pub on_click: Callback<ev::MouseEvent>,
}

#[derive(Clone)]
pub struct CardAction {
    pub label: String,
    pub icon: String,
    pub variant: ButtonVariant,
    pub on_click: Callback<ev::MouseEvent>,
}

#[derive(Clone)]
pub struct TabItem {
    pub key: String,
    pub label: String,
    pub icon: String,
    pub content: Callback<(), AnyView>,
}

// ========================================
// COMPONENT TYPES - BASIC DATA STRUCTURES
// ========================================

/// Navigation item untuk menu dan navigasi
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavItem {
    pub label: String,
    pub path: String,
    pub icon: String,
    pub children: Option<Vec<NavItem>>,
    pub active: bool,
}

/// Breadcrumb item untuk navigasi hierarkis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreadcrumbItem {
    pub label: String,
    pub path: String,
    pub icon: Option<String>,
}

/// Message type untuk notifikasi
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageType {
    Success,
    Warning,
    Error,
    Info,
}

// ============================================================================
// API RESPONSE TYPES
// ============================================================================

/// Standard API response untuk seluruh sistem Kejaksaan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KejaksaanApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: Option<String>,
    pub errors: Option<Vec<String>>,
    pub meta: Option<ResponseMeta>,
}

/// Generic API response wrapper (backward compatibility)
pub type ApiResponse<T> = KejaksaanApiResponse<T>;

/// Response metadata dengan informasi audit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMeta {
    pub timestamp: DateTime<Utc>,
    pub request_id: Option<String>,
    pub user_id: Option<String>,
    pub unit_code: Option<String>,
    pub pagination: Option<Pagination>,
    pub audit_info: Option<AuditMetadata>,
}

/// Metadata audit untuk setiap response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditMetadata {
    pub action: String,
    pub resource: String,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

/// Pagination dengan informasi lengkap
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pagination {
    pub page: i32,
    pub per_page: i32,
    pub total: i64,
    pub total_pages: i32,
    pub has_next: bool,
    pub has_previous: bool,
}

/// Paginated response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub items: Vec<T>,
    pub pagination: Pagination,
    pub filters_applied: Option<HashMap<String, String>>,
    pub sort_info: Option<SortInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SortInfo {
    pub field: String,
    pub direction: SortDirection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SortDirection {
    Asc,
    Desc,
}

/// Generic service response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceResponse<T> {
    pub data: T,
    pub status: ServiceStatus,
    pub message: Option<String>,
}

/// Service status enum
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceStatus {
    Success,
    Warning,
    Error,
}

// ============================================================================
// USER & AUTHENTICATION TYPES
// ============================================================================

/// User profile lengkap sesuai struktur Kejaksaan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub nip: String,
    pub nama_lengkap: String,
    pub email: String,
    pub username: String,
    pub role: UserRole,
    pub jabatan: Jabatan,
    pub unit: InstitusiUnit,
    pub status: UserStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
    pub profile_photo: Option<String>,
}

/// Level jabatan dalam struktur Kejaksaan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UserRole {
    JaksaAgung,
    JaksaAgungMuda,
    Jamwas,
    Jampidsus,
    Jampidum,
    KasubBag,
    JaksaTinggi,
    Jaksa,
    JaksaMuda,
    PegawaiNegeri,
    Admin,
    SuperAdmin,
}

/// Informasi jabatan detail
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Jabatan {
    pub kode_jabatan: String,
    pub nama_jabatan: String,
    pub level: JabatanLevel,
    pub unit_kerja: String,
    pub wewenang: Vec<WewenangLevel>,
    pub atasan_langsung: Option<String>,
}

/// Level jabatan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JabatanLevel {
    Pimpinan,
    Struktural,
    Fungsional,
    Pelaksana,
}

/// Level wewenang dalam sistem
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WewenangLevel {
    SuperAdmin,
    Admin,
    Pimpinan,
    Pelaksana,
    ReadOnly,
    CreateDocument,
    UpdateDocument,
    DeleteDocument,
    ApproveDocument,
    ViewConfidential,
}

/// Unit institusi dalam Kejaksaan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstitusiUnit {
    pub kode_unit: String,
    pub nama_unit: String,
    pub nama_lengkap: String,
    pub jenis_unit: JenisUnit,
    pub level_unit: LevelUnit,
    pub parent_unit: Option<String>,
    pub alamat: Option<Alamat>,
    pub kontak: Option<Kontak>,
}

/// Jenis unit dalam struktur Kejaksaan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JenisUnit {
    KejaksaanAgung,
    KejaksaanTinggi,
    KejaksaanNegeri,
    Cabang,
    UnitKhusus,
    BadanPendukung,
}

/// Level unit organisasi
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LevelUnit {
    Pusat,
    Daerah,
    Cabang,
}

/// Informasi alamat
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alamat {
    pub alamat_lengkap: String,
    pub kota: String,
    pub provinsi: String,
    pub kode_pos: Option<String>,
    pub koordinat: Option<Koordinat>,
}

/// Koordinat geografis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Koordinat {
    pub latitude: f64,
    pub longitude: f64,
}

/// Informasi kontak
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Kontak {
    pub telepon: Option<String>,
    pub fax: Option<String>,
    pub email: Option<String>,
    pub website: Option<String>,
}

/// Status user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UserStatus {
    Aktif,
    NonAktif,
    Suspended,
    Pending,
}

/// Session user yang sedang login
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSession {
    pub user_id: String,
    pub session_id: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub permissions: Vec<String>,
}

// ============================================================================
// AUDIT & LOGGING TYPES
// ============================================================================

/// Log audit untuk setiap aksi dalam sistem
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLog {
    pub id: String,
    pub user_id: String,
    pub action: AuditAction,
    pub resource_type: String,
    pub resource_id: Option<String>,
    pub old_data: Option<serde_json::Value>,
    pub new_data: Option<serde_json::Value>,
    pub timestamp: DateTime<Utc>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub status: AuditStatus,
    pub notes: Option<String>,
}

/// Jenis aksi yang dapat diaudit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditAction {
    Login,
    Logout,
    Create,
    Read,
    Update,
    Delete,
    Approve,
    Reject,
    Submit,
    Archive,
    Restore,
    Export,
    Import,
    PasswordChange,
    PermissionChange,
    SystemConfig,
}

/// Status audit log
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditStatus {
    Success,
    Failed,
    Warning,
}
