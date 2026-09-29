//! Portal navigation menu model.
//!
//! Provides a centralized RBAC-aware menu tree consumed by navbar and sidebar.

use crate::features::auth::UserSession;
use crate::routes;

#[derive(Clone, Debug)]
pub enum MenuVisibility {
    Public,
    Authenticated,
    AdminOnly,
    AnyPermission(&'static [&'static str]),
}

#[derive(Clone, Debug)]
pub struct PortalMenuItem {
    pub id: &'static str,
    pub label: &'static str,
    pub href: &'static str,
    pub visibility: MenuVisibility,
    pub children: Vec<PortalMenuItem>,
}

#[derive(Clone, Debug)]
pub struct PortalMenuSection {
    pub title: &'static str,
    pub items: Vec<PortalMenuItem>,
}

impl PortalMenuItem {
    fn leaf(
        id: &'static str,
        label: &'static str,
        href: &'static str,
        visibility: MenuVisibility,
    ) -> Self {
        Self {
            id,
            label,
            href,
            visibility,
            children: Vec::new(),
        }
    }

    fn parent(
        id: &'static str,
        label: &'static str,
        href: &'static str,
        visibility: MenuVisibility,
        children: Vec<PortalMenuItem>,
    ) -> Self {
        Self {
            id,
            label,
            href,
            visibility,
            children,
        }
    }
}

pub fn resolve_menu_sections(session: Option<&UserSession>) -> Vec<PortalMenuSection> {
    let sections = vec![
        PortalMenuSection {
            title: "Utama",
            items: vec![
                PortalMenuItem::leaf(
                    "dashboard",
                    "Dashboard",
                    routes::path::DASHBOARD,
                    MenuVisibility::Authenticated,
                ),
                PortalMenuItem::leaf(
                    "apps",
                    "Aplikasi",
                    routes::path::APPS,
                    MenuVisibility::Authenticated,
                ),
                PortalMenuItem::leaf(
                    "notifications",
                    "Notifikasi",
                    routes::path::NOTIFICATIONS,
                    MenuVisibility::Authenticated,
                ),
                PortalMenuItem::leaf(
                    "settings",
                    "Pengaturan",
                    routes::path::SETTINGS,
                    MenuVisibility::Authenticated,
                ),
            ],
        },
        PortalMenuSection {
            title: "Akun",
            items: vec![
                PortalMenuItem::leaf(
                    "profile",
                    "Profil",
                    routes::path::PROFILE,
                    MenuVisibility::Authenticated,
                ),
                PortalMenuItem::leaf(
                    "passkeys",
                    "Passkey",
                    routes::path::PASSKEYS,
                    MenuVisibility::Authenticated,
                ),
                PortalMenuItem::leaf(
                    "password",
                    "Ubah Kata Sandi",
                    routes::path::PASSWORD,
                    MenuVisibility::Authenticated,
                ),
                PortalMenuItem::leaf(
                    "sessions",
                    "Sesi Aktif",
                    routes::path::SESSIONS,
                    MenuVisibility::Authenticated,
                ),
            ],
        },
        PortalMenuSection {
            title: "Administrasi",
            items: vec![PortalMenuItem::parent(
                "admin",
                "Panel Administrasi",
                routes::path::ADMIN,
                MenuVisibility::AdminOnly,
                vec![
                    PortalMenuItem::leaf(
                        "admin-users",
                        "Pengguna",
                        routes::path::ADMIN_USERS,
                        MenuVisibility::AdminOnly,
                    ),
                    PortalMenuItem::leaf(
                        "admin-roles",
                        "Peran",
                        routes::path::ADMIN_ROLES,
                        MenuVisibility::AdminOnly,
                    ),
                    PortalMenuItem::leaf(
                        "admin-clients",
                        "Klien",
                        routes::path::ADMIN_CLIENTS,
                        MenuVisibility::AdminOnly,
                    ),
                    PortalMenuItem::leaf(
                        "admin-audit",
                        "Audit Log",
                        routes::path::ADMIN_AUDIT,
                        MenuVisibility::AdminOnly,
                    ),
                ],
            )],
        },
    ];

    sections
        .into_iter()
        .filter_map(|section| {
            let filtered_items: Vec<PortalMenuItem> = section
                .items
                .into_iter()
                .filter_map(|item| filter_item(item, session))
                .collect();

            if filtered_items.is_empty() {
                None
            } else {
                Some(PortalMenuSection {
                    title: section.title,
                    items: filtered_items,
                })
            }
        })
        .collect()
}

pub fn topbar_items(session: Option<&UserSession>) -> Vec<PortalMenuItem> {
    resolve_menu_sections(session)
        .into_iter()
        .find(|section| section.title == "Utama")
        .map(|section| section.items)
        .unwrap_or_default()
}

fn filter_item(item: PortalMenuItem, session: Option<&UserSession>) -> Option<PortalMenuItem> {
    if !is_visible(&item.visibility, session) {
        return None;
    }

    let children = item
        .children
        .into_iter()
        .filter_map(|child| filter_item(child, session))
        .collect();

    Some(PortalMenuItem { children, ..item })
}

fn is_visible(visibility: &MenuVisibility, session: Option<&UserSession>) -> bool {
    match visibility {
        MenuVisibility::Public => true,
        MenuVisibility::Authenticated => session.is_some(),
        // The admin menu is the IAM console: exact `admin`, like its guard.
        MenuVisibility::AdminOnly => session.is_some_and(|s| s.can_administer_iam()),
        // `AnyPermission` tests realm roles, not permission strings — this
        // system has no separate permission vocabulary, and the old
        // implementation compared against `"user:read"`/`"admin:*"` strings
        // that no issuer ever minted. Admin is NOT an implicit superset any more:
        // the backend dropped that bypass, and the menu must not offer what the API
        // would refuse.
        MenuVisibility::AnyPermission(required) => session.is_some_and(|s| {
            let authz = lib_core::authz::Authorization::from_slice(&s.roles);
            required.iter().any(|target| authz.roles().has(target))
        }),
    }
}
