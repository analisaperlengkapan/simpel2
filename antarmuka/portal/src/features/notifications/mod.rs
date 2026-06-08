//! Notifications history page
//!
//! Full page view of all user notifications

use crate::components::layout::MainLayout;
use leptos::prelude::*;
use lib_ui::components::NotificationList;

/// Notifications history page component — reads session from context
#[component]
pub fn NotificationsPage() -> impl IntoView {
    view! {
        <MainLayout>
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
