use crate::api::{CreateScheduleRequest, create_schedule, fetch_schedules};
use crate::components::layout::Layout;
use chrono::NaiveDateTime;
use leptos::prelude::*;

#[component]
pub fn Schedules() -> impl IntoView {
    let (schedules, set_schedules) = signal(Vec::new());
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal(None::<String>);

    // Form signals
    let (new_title, set_new_title) = signal(String::new());
    let (new_location, set_new_location) = signal(String::new());
    let (new_date, set_new_date) = signal(String::new());

    // Fetch on mount
    Effect::new(move |_| {
        leptos::task::spawn_local(async move {
            match fetch_schedules().await {
                Ok(data) => {
                    set_schedules.set(data);
                    set_loading.set(false);
                }
                Err(e) => {
                    set_error.set(Some(e));
                    set_loading.set(false);
                }
            }
        });
    });

    let on_submit = move |ev: leptos::web_sys::SubmitEvent| {
        ev.prevent_default();
        let title = new_title.get();
        let location = new_location.get();
        let date_str = new_date.get();

        // Basic form validation check
        if title.is_empty() || location.is_empty() || date_str.is_empty() {
            return;
        }

        // Parse date-time-local string to DateTime<Utc>
        // Format from input type="datetime-local" is usually "YYYY-MM-DDTHH:MM"
        let scheduled_at = match NaiveDateTime::parse_from_str(&date_str, "%Y-%m-%dT%H:%M") {
            Ok(dt) => dt.and_utc(),
            Err(_) => {
                set_error.set(Some("Invalid date format".to_string()));
                return;
            }
        };

        leptos::task::spawn_local(async move {
            let req = CreateScheduleRequest {
                title,
                location,
                scheduled_at,
            };

            match create_schedule(req).await {
                Ok(new_schedule) => {
                    set_schedules.update(|s| s.push(new_schedule));
                    set_new_title.set(String::new());
                    set_new_location.set(String::new());
                    set_new_date.set(String::new());
                }
                Err(e) => set_error.set(Some(e)),
            }
        });
    };

    view! {
        <Layout>
            <h1 class="text-2xl font-bold mb-4">"Jadwal Pengawasan"</h1>

            // Create Form
            <div class="bg-white p-4 rounded shadow mb-6">
                <h2 class="text-lg font-semibold mb-2">"Buat Jadwal Baru"</h2>
                <form on:submit=on_submit class="flex gap-4 items-end flex-wrap">
                    <div>
                        <label class="block text-sm text-gray-700">"Judul"</label>
                        <input
                            type="text"
                            class="border rounded px-2 py-1"
                            prop:value=new_title
                            on:input=move |ev| set_new_title.set(event_target_value(&ev))
                        />
                    </div>
                    <div>
                        <label class="block text-sm text-gray-700">"Lokasi"</label>
                        <input
                            type="text"
                            class="border rounded px-2 py-1"
                            prop:value=new_location
                            on:input=move |ev| set_new_location.set(event_target_value(&ev))
                        />
                    </div>
                    <div>
                        <label class="block text-sm text-gray-700">"Waktu"</label>
                        <input
                            type="datetime-local"
                            class="border rounded px-2 py-1"
                            prop:value=new_date
                            on:input=move |ev| set_new_date.set(event_target_value(&ev))
                        />
                    </div>
                    <button type="submit" class="bg-blue-600 text-white px-4 py-1 rounded hover:bg-blue-700">
                        "Tambah"
                    </button>
                </form>
            </div>

            // List
            {move || {
                if loading.get() {
                    view! { <div>"Loading..."</div> }.into_any()
                } else if let Some(e) = error.get() {
                    view! { <div class="text-red-500">{e}</div> }.into_any()
                } else {
                    view! {
                        <div class="bg-white rounded shadow overflow-hidden">
                            <table class="w-full">
                                <thead class="bg-gray-50">
                                    <tr>
                                        <th class="px-4 py-2 text-left">"Judul"</th>
                                        <th class="px-4 py-2 text-left">"Lokasi"</th>
                                        <th class="px-4 py-2 text-left">"Waktu"</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    <For
                                        each=move || schedules.get()
                                        key=|s| s.id
                                        children=move |schedule| {
                                            view! {
                                                <tr class="border-t">
                                                    <td class="px-4 py-2">{schedule.title}</td>
                                                    <td class="px-4 py-2">{schedule.location}</td>
                                                    <td class="px-4 py-2">{schedule.scheduled_at.to_rfc3339()}</td>
                                                </tr>
                                            }
                                        }
                                    />
                                </tbody>
                            </table>
                        </div>
                    }.into_any()
                }
            }}
        </Layout>
    }
}
