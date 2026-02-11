//! Mapping Kodefikasi Verification Component
//!
//! Interface for Admin Pusat to verify mapping proposals

use leptos::prelude::*;
use leptos::task::spawn_local;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MappingProposal {
    pub id: Uuid,
    pub satker_id: Uuid,
    pub kode_barang_lama: String,
    pub nama_barang_lama: String,
    pub kode_barang_baru_id: Option<Uuid>,
    pub status_mapping: String,
    pub catatan_mapping: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct VerifyMappingRequest {
    pub approved: bool,
    pub catatan_verifikasi: Option<String>,
}

async fn fetch_proposals() -> Result<Vec<MappingProposal>, String> {
    let response = gloo_net::http::Request::get("/api/pembinaan/perlengkapan/mapping/proposals")
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    if !response.ok() {
        return Err(format!("HTTP error: {}", response.status()));
    }

    response
        .json::<Vec<MappingProposal>>()
        .await
        .map_err(|e| format!("JSON parse error: {}", e))
}

async fn verify_proposal(
    proposal_id: Uuid,
    request: VerifyMappingRequest,
) -> Result<(), String> {
    let url = format!(
        "/api/pembinaan/perlengkapan/mapping/proposals/{}/verify",
        proposal_id
    );

    let response = gloo_net::http::Request::put(&url)
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
pub fn MappingKodefikasiVerification() -> impl IntoView {
    let (refresh_trigger, set_refresh_trigger) = signal(0);
    let (verifying_id, set_verifying_id) = signal::<Option<Uuid>>(None);
    let (verification_notes, set_verification_notes) = signal(String::new());

    let proposals = LocalResource::new(move || {
        let _ = refresh_trigger.get();
        async move {
            match fetch_proposals().await {
                Ok(data) => Some(data),
                Err(e) => {
                    leptos::logging::error!("Failed to fetch proposals: {}", e);
                    None
                }
            }
        }
    });

    let handle_verify = move |proposal_id: Uuid, approved: bool| {
        let notes = verification_notes.get();
        let request = VerifyMappingRequest {
            approved,
            catatan_verifikasi: if notes.is_empty() { None } else { Some(notes) },
        };

        spawn_local(async move {
            match verify_proposal(proposal_id, request).await {
                Ok(_) => {
                    set_verifying_id.set(None);
                    set_verification_notes.set(String::new());
                    set_refresh_trigger.update(|v| *v += 1);
                }
                Err(e) => {
                    leptos::logging::error!("Failed to verify proposal: {}", e);
                }
            }
        });
    };

    view! {
        <div class="p-6 bg-white rounded-xl shadow-sm border border-gray-100">
            <h1 class="text-2xl font-bold mb-6 text-gray-800">"Mapping Verification"</h1>

            <Suspense fallback=move || view! {
                <div class="flex justify-center items-center py-12">
                    <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600"></div>
                </div>
            }>
                {move || {
                    proposals.get().and_then(|data| data.map(|proposals| {
                        let pending_proposals: Vec<_> = proposals
                            .into_iter()
                            .filter(|p| p.status_mapping == "PROPOSED")
                            .collect();

                        if pending_proposals.is_empty() {
                            view! {
                                <div class="text-center py-12 text-gray-500">
                                    <p class="text-lg">"No pending proposals"</p>
                                    <p class="text-sm mt-2">"All mapping proposals have been verified."</p>
                                </div>
                            }.into_any()
                        } else {
                            view! {
                                <div class="space-y-4">
                                    <For
                                        each=move || pending_proposals.clone()
                                        key=|p| p.id
                                        children=move |proposal| {
                                            let is_verifying = move || {
                                                verifying_id.get() == Some(proposal.id)
                                            };

                                            let proposal_id = proposal.id;
                                            let show_verification = move |_| {
                                                set_verifying_id.set(Some(proposal_id));
                                                set_verification_notes.set(String::new());
                                            };

                                            let cancel_verification = move |_| {
                                                set_verifying_id.set(None);
                                                set_verification_notes.set(String::new());
                                            };

                                            let approve = move |_| {
                                                handle_verify(proposal_id, true);
                                            };

                                            let reject = move |_| {
                                                handle_verify(proposal_id, false);
                                            };

                                            view! {
                                                <div class="border border-gray-200 rounded-lg p-6">
                                                    <div class="grid grid-cols-2 gap-4 mb-4">
                                                        <div>
                                                            <label class="text-xs text-gray-500">"Old Code"</label>
                                                            <p class="text-sm font-medium text-gray-900">
                                                                {proposal.kode_barang_lama.clone()}
                                                            </p>
                                                        </div>
                                                        <div>
                                                            <label class="text-xs text-gray-500">"Old Name"</label>
                                                            <p class="text-sm font-medium text-gray-900">
                                                                {proposal.nama_barang_lama.clone()}
                                                            </p>
                                                        </div>
                                                    </div>

                                                    {proposal.catatan_mapping.clone().map(|notes| {
                                                        view! {
                                                            <div class="mb-4">
                                                                <label class="text-xs text-gray-500">"Proposer Notes"</label>
                                                                <p class="text-sm text-gray-700">{notes}</p>
                                                            </div>
                                                        }
                                                    })}

                                                    {move || {
                                                        if is_verifying() {
                                                            view! {
                                                                <div class="mt-4 space-y-4">
                                                                    <div>
                                                                        <label class="block text-sm font-medium text-gray-700 mb-2">
                                                                            "Verification Notes"
                                                                        </label>
                                                                        <textarea
                                                                            class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500"
                                                                            rows="3"
                                                                            placeholder="Add verification notes..."
                                                                            on:input=move |ev| {
                                                                                set_verification_notes.set(event_target_value(&ev));
                                                                            }
                                                                            prop:value=move || verification_notes.get()
                                                                        />
                                                                    </div>

                                                                    <div class="flex justify-end space-x-3">
                                                                        <button
                                                                            type="button"
                                                                            on:click=cancel_verification
                                                                            class="px-4 py-2 border border-gray-300 rounded-lg text-gray-700 hover:bg-gray-50"
                                                                        >
                                                                            "Cancel"
                                                                        </button>
                                                                        <button
                                                                            type="button"
                                                                            on:click=reject
                                                                            class="px-4 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700"
                                                                        >
                                                                            "Reject"
                                                                        </button>
                                                                        <button
                                                                            type="button"
                                                                            on:click=approve
                                                                            class="px-4 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700"
                                                                        >
                                                                            "Approve"
                                                                        </button>
                                                                    </div>
                                                                </div>
                                                            }.into_any()
                                                        } else {
                                                            view! {
                                                                <div class="flex justify-end mt-4">
                                                                    <button
                                                                        type="button"
                                                                        on:click=show_verification
                                                                        class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700"
                                                                    >
                                                                        "Verify"
                                                                    </button>
                                                                </div>
                                                            }.into_any()
                                                        }
                                                    }}
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
    }
}
