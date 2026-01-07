use crate::components::layout::MainLayout;
use crate::features::auth::UserSession;
use crate::types::AuditLogEntry;
use crate::utils::api::fetch_audit_logs;
use leptos::prelude::*;
use leptos::task::spawn_local;

#[component]
pub fn MonitoringPage(
    user_session: UserSession,
    on_logout: Box<dyn Fn()>,
) -> impl IntoView {
    // Resource for audit logs
    let access_token = user_session.access_token.clone();
    let (logs_data, set_logs_data) = signal(Option::<Vec<AuditLogEntry>>::None);
    let (is_loading, set_is_loading) = signal(true);

    Effect::new(move |_| {
        let token = access_token.clone();
        if let Some(token) = token {
            spawn_local(async move {
                if let Ok(data) = fetch_audit_logs(&token, Some(50)).await {
                    set_logs_data.set(Some(data));
                }
                set_is_loading.set(false);
            });
        } else {
            set_is_loading.set(false);
        }
    });

    view! {
        <MainLayout user_session=user_session.clone() on_logout=on_logout>
            <div class="container mx-auto px-4 py-8">
                <div class="mb-8">
                    <h1 class="text-3xl font-bold text-gray-900 dark:text-white">"System Monitoring"</h1>
                    <p class="text-gray-600 dark:text-gray-400">"Real-time audit logs and system events"</p>
                </div>

                <div class="bg-white dark:bg-gray-800 rounded-lg shadow overflow-hidden">
                    <div class="px-6 py-4 border-b border-gray-200 dark:border-gray-700">
                        <h2 class="text-xl font-semibold text-gray-800 dark:text-gray-200">"Audit Logs"</h2>
                    </div>
                    <div class="overflow-x-auto">
                        <table class="min-w-full divide-y divide-gray-200 dark:divide-gray-700">
                            <thead class="bg-gray-50 dark:bg-gray-700">
                                <tr>
                                    <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-300 uppercase tracking-wider">
                                        "Time"
                                    </th>
                                    <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-300 uppercase tracking-wider">
                                        "User"
                                    </th>
                                    <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-300 uppercase tracking-wider">
                                        "Action"
                                    </th>
                                    <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-300 uppercase tracking-wider">
                                        "Resource"
                                    </th>
                                    <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-300 uppercase tracking-wider">
                                        "Status"
                                    </th>
                                </tr>
                            </thead>
                            <tbody class="bg-white dark:bg-gray-800 divide-y divide-gray-200 dark:divide-gray-700">
                                {move || {
                                    if is_loading.get() {
                                        view! { <tr><td colspan="5" class="px-6 py-4 text-center">"Loading..."</td></tr> }.into_any()
                                    } else {
                                        match logs_data.get() {
                                            Some(logs) if !logs.is_empty() => {
                                                logs.into_iter().map(|log| {
                                                    let status_color = if log.success { "text-green-600" } else { "text-red-600" };
                                                    let status_text = if log.success { "Success" } else { "Failed" };

                                                    view! {
                                                        <tr class="hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors">
                                                            <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500 dark:text-gray-400">
                                                                {log.timestamp}
                                                            </td>
                                                            <td class="px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900 dark:text-white">
                                                                {log.user_id}
                                                            </td>
                                                            <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500 dark:text-gray-400">
                                                                {log.action}
                                                            </td>
                                                            <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500 dark:text-gray-400">
                                                                {log.resource}
                                                            </td>
                                                            <td class="px-6 py-4 whitespace-nowrap text-sm">
                                                                <span class={format!("px-2 inline-flex text-xs leading-5 font-semibold rounded-full bg-opacity-10 {}", status_color)}>
                                                                    {status_text}
                                                                </span>
                                                            </td>
                                                        </tr>
                                                    }
                                                }).collect_view().into_any()
                                            }
                                            _ => view! {
                                                <tr><td colspan="5" class="px-6 py-4 text-center text-gray-500">"No logs found"</td></tr>
                                            }.into_any()
                                        }
                                    }
                                }}
                            </tbody>
                        </table>
                    </div>
                </div>
            </div>
        </MainLayout>
    }
}
