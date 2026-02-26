//! Notifications history page
//!
//! Full page view of all user notifications

use crate::components::layout::MainLayout;
use crate::features::auth::UserSession;
use leptos::prelude::*;
use lib_ui::components::NotificationList;

/// Notifications history page component - displays all user notifications
#[component]
pub fn NotificationsPage(
    /// Current user session data
    user_session: UserSession,
    /// Callback function to handle user logout
    on_logout: Box<dyn Fn()>,
) -> impl IntoView {
    view! {
        <MainLayout user_session=user_session on_logout=on_logout>
            <div class="container mx-auto px-4 py-8">
                // Page Header
                <div class="mb-8">
                    <h1 class="text-3xl font-bold text-gray-900 dark:text-white mb-2">
                        "Notifikasi"
                    </h1>
                    <p class="text-gray-600 dark:text-gray-400">
                        "Semua notifikasi dan pemberitahuan sistem"
                    </p>
                </div>

                // Use shared NotificationList component with filters
                <NotificationList show_filters=true />
            </div>
        </MainLayout>
    }
}
