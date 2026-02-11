//! Mapping Kodefikasi Proposal Form Component
//!
//! Form for proposing mapping of non-standard codes to standard codes

use leptos::prelude::*;
use leptos::task::spawn_local;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MappingSuggestion {
    pub barang_id: Uuid,
    pub kode_baru: String,
    pub nama_baru: String,
    pub similarity_score: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct MappingProposalRequest {
    pub satker_id: Uuid,
    pub kode_lama: String,
    pub nama_lama: String,
    pub kode_baru_id: Uuid,
    pub catatan: Option<String>,
}

async fn fetch_suggestions(nama: &str) -> Result<Vec<MappingSuggestion>, String> {
    let url = format!(
        "/api/pembinaan/perlengkapan/mapping/suggestions?nama={}",
        urlencoding::encode(nama)
    );

    let response = gloo_net::http::Request::get(&url)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    if !response.ok() {
        return Err(format!("HTTP error: {}", response.status()));
    }

    response
        .json::<Vec<MappingSuggestion>>()
        .await
        .map_err(|e| format!("JSON parse error: {}", e))
}

async fn submit_proposal(request: MappingProposalRequest) -> Result<(), String> {
    let response = gloo_net::http::Request::post("/api/pembinaan/perlengkapan/mapping/proposals")
        .json(&request)
        .map_err(|e| format!("JSON serialization failed: {}", e))?
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    if !response.ok() {
        return Err(format!("HTTP error: {}", response.status()));
    }

    Ok(())
}

#[component]
pub fn MappingKodefikasiForm(
    kode_lama: String,
    nama_lama: String,
    satker_id: Uuid,
) -> impl IntoView {
    let (selected_mapping, set_selected_mapping) = signal::<Option<MappingSuggestion>>(None);
    let (catatan, set_catatan) = signal(String::new());
    let (is_submitting, set_is_submitting) = signal(false);
    let (submit_error, set_submit_error) = signal::<Option<String>>(None);
    let (submit_success, set_submit_success) = signal(false);

    // Fetch suggestions
    let suggestions = LocalResource::new({
        let nama_lama = nama_lama.clone();
        move || {
            let nama = nama_lama.clone();
            async move {
                match fetch_suggestions(&nama).await {
                    Ok(data) => Some(data),
                    Err(e) => {
                        leptos::logging::error!("Failed to fetch suggestions: {}", e);
                        None
                    }
                }
            }
        }
    });

    let kode_lama_for_submit = kode_lama.clone();
    let nama_lama_for_submit = nama_lama.clone();

    let submit_action = Action::new_local(move |_: &()| {
        let mapping = selected_mapping.get();
        let kode = kode_lama_for_submit.clone();
        let nama = nama_lama_for_submit.clone();
        let cat = catatan.get();

        async move {
            if let Some(mapping) = mapping {
                let request = MappingProposalRequest {
                    satker_id,
                    kode_lama: kode,
                    nama_lama: nama,
                    kode_baru_id: mapping.barang_id,
                    catatan: if cat.is_empty() { None } else { Some(cat) },
                };

                match submit_proposal(request).await {
                    Ok(_) => {
                        set_submit_success.set(true);
                        set_is_submitting.set(false);
                    }
                    Err(e) => {
                        set_submit_error.set(Some(e));
                        set_is_submitting.set(false);
                    }
                }
            }
        }
    });

    let handle_submit = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();

        if selected_mapping.get().is_some() {
            set_is_submitting.set(true);
            set_submit_error.set(None);
            submit_action.dispatch(());
        }
    };

    view! {
        <div class="p-6 bg-white rounded-xl shadow-sm border border-gray-100">
            <h1 class="text-2xl font-bold mb-6 text-gray-800">"Propose Mapping"</h1>

            {move || {
                if submit_success.get() {
                    view! {
                        <div class="bg-green-50 border border-green-200 rounded-lg p-4 mb-6">
                            <div class="flex items-center">
                                <span class="text-green-600 text-2xl mr-3">"✓"</span>
                                <div>
                                    <h3 class="text-green-800 font-semibold">"Proposal Submitted"</h3>
                                    <p class="text-green-700 text-sm">
                                        "Your mapping proposal has been submitted for verification."
                                    </p>
                                </div>
                            </div>
                            <div class="mt-4">
                                <a
                                    href="/mapping"
                                    class="text-blue-600 hover:text-blue-800 text-sm font-medium"
                                >
                                    "← Back to Dashboard"
                                </a>
                            </div>
                        </div>
                    }.into_any()
                } else {
                    view! {
                        <form on:submit=handle_submit>
                            // Non-standard code info
                            <div class="bg-gray-50 rounded-lg p-4 mb-6">
                                <h3 class="text-sm font-semibold text-gray-700 mb-2">"Non-Standard Code"</h3>
                                <div class="grid grid-cols-2 gap-4">
                                    <div>
                                        <label class="text-xs text-gray-500">"Kode"</label>
                                        <p class="text-sm font-medium text-gray-900">{kode_lama.clone()}</p>
                                    </div>
                                    <div>
                                        <label class="text-xs text-gray-500">"Nama"</label>
                                        <p class="text-sm font-medium text-gray-900">{nama_lama.clone()}</p>
                                    </div>
                                </div>
                            </div>

                            // Suggestions
                            <div class="mb-6">
                                <label class="block text-sm font-medium text-gray-700 mb-2">
                                    "Select Standard Code Mapping"
                                </label>

                                <Suspense fallback=move || view! {
                                    <div class="flex justify-center py-8">
                                        <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
                                    </div>
                                }>
                                    {move || {
                                        suggestions.get().and_then(|data| data.map(|suggestions| {
                                            if suggestions.is_empty() {
                                                view! {
                                                    <div class="text-center py-8 text-gray-500">
                                                        "No suggestions found. Please search manually."
                                                    </div>
                                                }.into_any()
                                            } else {
                                                view! {
                                                    <div class="space-y-2">
                                                        <For
                                                            each=move || suggestions.clone()
                                                            key=|s| s.barang_id
                                                            children=move |suggestion| {
                                                                let is_selected = move || {
                                                                    selected_mapping.get()
                                                                        .map(|s| s.barang_id == suggestion.barang_id)
                                                                        .unwrap_or(false)
                                                                };

                                                                let suggestion_clone = suggestion.clone();
                                                                let on_click = move |_| {
                                                                    set_selected_mapping.set(Some(suggestion_clone.clone()));
                                                                };

                                                                view! {
                                                                    <div
                                                                        class=move || {
                                                                            let base = "p-4 border rounded-lg cursor-pointer transition-colors";
                                                                            if is_selected() {
                                                                                format!("{} border-blue-500 bg-blue-50", base)
                                                                            } else {
                                                                                format!("{} border-gray-200 hover:border-blue-300", base)
                                                                            }
                                                                        }
                                                                        on:click=on_click
                                                                    >
                                                                        <div class="flex items-center justify-between">
                                                                            <div>
                                                                                <p class="font-medium text-gray-900">
                                                                                    {suggestion.kode_baru.clone()}
                                                                                </p>
                                                                                <p class="text-sm text-gray-600">
                                                                                    {suggestion.nama_baru.clone()}
                                                                                </p>
                                                                            </div>
                                                                            <div class="text-sm text-gray-500">
                                                                                {format!("Match: {:.0}%", suggestion.similarity_score * 100.0)}
                                                                            </div>
                                                                        </div>
                                                                    </div>
                                                                }
                                                            }
                                                        />
                                                    </div>
                                                }.into_any()
                                            }
                                        }))
                                    }}
                                </Suspense>
                            </div>

                            // Notes
                            <div class="mb-6">
                                <label class="block text-sm font-medium text-gray-700 mb-2">
                                    "Notes (Optional)"
                                </label>
                                <textarea
                                    class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                                    rows="3"
                                    placeholder="Add any notes about this mapping..."
                                    on:input=move |ev| {
                                        set_catatan.set(event_target_value(&ev));
                                    }
                                    prop:value=move || catatan.get()
                                />
                            </div>

                            // Error message
                            {move || {
                                submit_error.get().map(|error| {
                                    view! {
                                        <div class="bg-red-50 border border-red-200 rounded-lg p-4 mb-6">
                                            <p class="text-red-800 text-sm">{error}</p>
                                        </div>
                                    }
                                })
                            }}

                            // Submit button
                            <div class="flex justify-end space-x-3">
                                <a
                                    href="/mapping"
                                    class="px-4 py-2 border border-gray-300 rounded-lg text-gray-700 hover:bg-gray-50"
                                >
                                    "Cancel"
                                </a>
                                <button
                                    type="submit"
                                    disabled=move || is_submitting.get() || selected_mapping.get().is_none()
                                    class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:bg-gray-300 disabled:cursor-not-allowed"
                                >
                                    {move || if is_submitting.get() {
                                        "Submitting..."
                                    } else {
                                        "Submit Proposal"
                                    }}
                                </button>
                            </div>
                        </form>
                    }.into_any()
                }
            }}
        </div>
    }
}
