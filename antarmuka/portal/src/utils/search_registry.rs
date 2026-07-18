//! Search registry bootstrap for portal global search.
//!
//! Keeps search data wiring out of app bootstrap and routing code.

use crate::features::microfrontends::MicrofrontendRegistry;
use crate::routes;
use lib_ui::hooks::{SearchCategory, SearchResult, use_search};

/// Register portal applications and pages into global search context.
pub fn setup_search_providers() {
    let search_ctx = use_search();

    // Register applications
    let apps: Vec<SearchResult> = MicrofrontendRegistry::get_all_apps()
        .into_iter()
        .map(|app| SearchResult {
            id: app.id.clone(),
            title: app.name,
            description: app.description,
            category: SearchCategory::Application,
            url: app.url,
            icon: app.icon,
            module: None,
        })
        .collect();

    search_ctx.register_data(apps);

    // Register pages
    let pages = vec![
        SearchResult {
            id: "dashboard".to_string(),
            title: "Dashboard".to_string(),
            description: "Dashboard utama dengan statistik dan aktivitas terbaru".to_string(),
            category: SearchCategory::Page,
            url: routes::path::DASHBOARD.to_string(),
            icon: "📊".to_string(),
            module: Some("Portal".to_string()),
        },
        SearchResult {
            id: "apps".to_string(),
            title: "Aplikasi".to_string(),
            description: "Daftar semua aplikasi SIMPEL yang tersedia".to_string(),
            category: SearchCategory::Page,
            url: routes::path::APPS.to_string(),
            icon: "🚀".to_string(),
            module: Some("Portal".to_string()),
        },
        SearchResult {
            id: "notifications".to_string(),
            title: "Notifikasi".to_string(),
            description: "Semua notifikasi dan pemberitahuan sistem".to_string(),
            category: SearchCategory::Page,
            url: routes::path::NOTIFICATIONS.to_string(),
            icon: "🔔".to_string(),
            module: Some("Portal".to_string()),
        },
        SearchResult {
            id: "settings".to_string(),
            title: "Pengaturan".to_string(),
            description: "Kelola preferensi, tema, dan kustomisasi tampilan".to_string(),
            category: SearchCategory::Page,
            url: routes::path::SETTINGS.to_string(),
            icon: "⚙️".to_string(),
            module: Some("Portal".to_string()),
        },
        SearchResult {
            id: "profile".to_string(),
            title: "Profil Saya".to_string(),
            description: "Kelola informasi profil dan data pribadi".to_string(),
            category: SearchCategory::Page,
            url: routes::path::PROFILE.to_string(),
            icon: "👤".to_string(),
            module: Some("Portal".to_string()),
        },
        SearchResult {
            id: "passkeys".to_string(),
            title: "Passkey".to_string(),
            description: "Kelola kunci keamanan dan autentikasi biometrik".to_string(),
            category: SearchCategory::Page,
            url: routes::path::PASSKEYS.to_string(),
            icon: "🔐".to_string(),
            module: Some("Portal".to_string()),
        },
        SearchResult {
            id: "password".to_string(),
            title: "Ubah Kata Sandi".to_string(),
            description: "Perbarui kata sandi akun Anda".to_string(),
            category: SearchCategory::Page,
            url: routes::path::PASSWORD.to_string(),
            icon: "🔒".to_string(),
            module: Some("Portal".to_string()),
        },
        SearchResult {
            id: "sessions".to_string(),
            title: "Sesi Aktif".to_string(),
            description: "Kelola sesi login dan perangkat aktif".to_string(),
            category: SearchCategory::Page,
            url: routes::path::SESSIONS.to_string(),
            icon: "📱".to_string(),
            module: Some("Portal".to_string()),
        },
        SearchResult {
            id: "admin".to_string(),
            title: "Dasbor Administrasi".to_string(),
            description: "Administrasi identitas dan manajemen akses".to_string(),
            category: SearchCategory::Page,
            url: routes::path::ADMIN.to_string(),
            icon: "🛡️".to_string(),
            module: Some("Admin".to_string()),
        },
        SearchResult {
            id: "admin-users".to_string(),
            title: "Manajemen Pengguna".to_string(),
            description: "Kelola akun pengguna, status, dan atribut".to_string(),
            category: SearchCategory::Page,
            url: routes::path::ADMIN_USERS.to_string(),
            icon: "👥".to_string(),
            module: Some("Admin".to_string()),
        },
        SearchResult {
            id: "admin-clients".to_string(),
            title: "Manajemen Klien OAuth2".to_string(),
            description: "Kelola aplikasi klien dan kredensial integrasi".to_string(),
            category: SearchCategory::Page,
            url: routes::path::ADMIN_CLIENTS.to_string(),
            icon: "🔑".to_string(),
            module: Some("Admin".to_string()),
        },
        SearchResult {
            id: "admin-roles".to_string(),
            title: "Manajemen Peran".to_string(),
            description: "Kelola peran dan hak akses".to_string(),
            category: SearchCategory::Page,
            url: routes::path::ADMIN_ROLES.to_string(),
            icon: "🛡️".to_string(),
            module: Some("Admin".to_string()),
        },
        SearchResult {
            id: "admin-audit".to_string(),
            title: "Audit Log".to_string(),
            description: "Tinjau aktivitas keamanan dan jejak audit".to_string(),
            category: SearchCategory::Page,
            url: routes::path::ADMIN_AUDIT.to_string(),
            icon: "📋".to_string(),
            module: Some("Admin".to_string()),
        },
    ];

    search_ctx.register_data(pages);
}
