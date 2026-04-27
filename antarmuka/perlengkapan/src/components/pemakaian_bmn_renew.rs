//! # Pemakaian BMN Renewal Component
//!
//! Form for renewing BMN usage permits.
//! Requirements: REQ-P008

use crate::api::{RenewPermitRequest, fetch_pemakaian_bmn_detail, renew_pemakaian_bmn};
use leptos::prelude::*;
use lib_ui::components::icon::AppIcon;
use phosphor_leptos::{ARROW_CLOCKWISE, CHECK_CIRCLE, SPINNER, WARNING_CIRCLE};
use leptos_router::hooks::{use_navigate, use_params_map};

#[component]
pub fn PemakaianBmnRenew() -> impl IntoView {
    let params = use_params_map();
    let id = move || params.read().get("id").unwrap_or_default();

    // Resource to fetch current permit
    let permit_resource = LocalResource::new(move || {
        let permit_id = id();
        async move {
            match fetch_pemakaian_bmn_detail(&permit_id).await {
                Ok(response) => Some(response.data.izin),
                Err(_) => None,
            }
        }
    });

    // Form state
    let (tanggal_mulai, set_tanggal_mulai) = signal("".to_string());
    let (tanggal_selesai, set_tanggal_selesai) = signal("".to_string());
    let (keperluan, set_keperluan) = signal("".to_string());

    // UI state
    let (loading, set_loading) = signal(false);
    let (error, set_error) = signal(None::<String>);
    let (success, set_success) = signal(false);
    let navigate = use_navigate();

    let on_submit = move |ev: leptos::web_sys::SubmitEvent| {
        ev.prevent_default();

        set_loading.set(true);
        set_error.set(None);
        set_success.set(false);

        let permit_id = id();
        let request = RenewPermitRequest {
            tanggal_mulai: tanggal_mulai.get(),
            tanggal_selesai: tanggal_selesai.get(),
            keperluan: keperluan.get(),
        };

        let navigate = navigate.clone();
        leptos::task::spawn_local(async move {
            match renew_pemakaian_bmn(&permit_id, request).await {
                Ok(_) => {
                    set_success.set(true);
                    set_loading.set(false);

                    // Redirect after success — `navigate` captured at component
                    // level to stay within the reactive scope.
                    gloo_timers::callback::Timeout::new(1500, move || {
                        navigate(
                            &format!("/perlengkapan/pemakaian-bmn/{}", permit_id),
                            Default::default(),
                        );
                    })
                    .forget();
                }
                Err(e) => {
                    set_error.set(Some(format!("Gagal memperpanjang izin: {}", e)));
                    set_loading.set(false);
                }
            }
        });
    };

    view! {
        <div class="max-w-4xl mx-auto p-6">
            <Suspense fallback=move || view! {
                <div class="p-8 text-center">
                    <span class="fa-spin text-2xl text-gray-400 mb-2"><AppIcon icon=SPINNER /></span>
                    <p class="text-gray-600">"Memuat data..."</p>
                </div>
            }>
                {move || {
                    permit_resource.get().flatten().map(|izin| {
                        view! {
                            <div class="bg-white rounded-xl shadow-sm border border-gray-100 p-6">
                                <h2 class="text-2xl font-bold text-gray-800 mb-6">"Perpanjangan Izin Pemakaian BMN"</h2>

                                // Current permit info
                                <div class="mb-6 p-4 bg-blue-50 rounded-lg border border-blue-100">
                                    <h3 class="font-semibold text-blue-900 mb-2">"Izin Saat Ini"</h3>
                                    <div class="grid grid-cols-2 gap-4 text-sm">
                                        <div>
                                            <p class="text-blue-700">"Nomor Izin"</p>
                                            <p class="font-medium text-blue-900">{izin.nomor_izin.clone().unwrap_or_else(|| "-".to_string())}</p>
                                        </div>
                                        <div>
                                            <p class="text-blue-700">"BMN"</p>
                                            <p class="font-medium text-blue-900">{izin.bmn_nama_barang.clone()}</p>
                                        </div>
                                        <div>
                                            <p class="text-blue-700">"Periode Saat Ini"</p>
                                            <p class="font-medium text-blue-900">{izin.tanggal_mulai.clone()} " - " {izin.tanggal_selesai.clone()}</p>
                                        </div>
                                    </div>
                                </div>

                                <Show when=move || success.get()>
                                    <div class="mb-4 p-4 bg-green-50 text-green-700 rounded-lg border border-green-100 flex items-center gap-2">
                                        <AppIcon icon=CHECK_CIRCLE />
                                        "Izin berhasil diperpanjang!"
                                    </div>
                                </Show>

                                <Show when=move || error.get().is_some()>
                                    <div class="mb-4 p-4 bg-red-50 text-red-700 rounded-lg border border-red-100 flex items-center gap-2">
                                        <AppIcon icon=WARNING_CIRCLE />
                                        {error.get()}
                                    </div>
                                </Show>

                                <form on:submit=on_submit class="space-y-6">
                                    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                                        <div>
                                            <label class="block text-sm font-medium text-gray-700 mb-1">"Tanggal Mulai Baru"</label>
                                            <input
                                                type="date"
                                                class="w-full px-4 py-2 border rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none"
                                                prop:value=move || tanggal_mulai.get()
                                                on:input=move |ev| set_tanggal_mulai.set(event_target_value(&ev))
                                                required
                                            />
                                        </div>
                                        <div>
                                            <label class="block text-sm font-medium text-gray-700 mb-1">"Tanggal Selesai Baru"</label>
                                            <input
                                                type="date"
                                                class="w-full px-4 py-2 border rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none"
                                                prop:value=move || tanggal_selesai.get()
                                                on:input=move |ev| set_tanggal_selesai.set(event_target_value(&ev))
                                                required
                                            />
                                        </div>
                                    </div>

                                    <div>
                                        <label class="block text-sm font-medium text-gray-700 mb-1">"Keperluan Perpanjangan"</label>
                                        <textarea
                                            class="w-full px-4 py-2 border rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none"
                                            rows="4"
                                            placeholder="Jelaskan alasan perpanjangan (minimal 10 karakter)"
                                            prop:value=move || keperluan.get()
                                            on:input=move |ev| set_keperluan.set(event_target_value(&ev))
                                            required
                                        ></textarea>
                                    </div>

                                    <div class="pt-6 flex justify-end gap-3 border-t">
                                        <a
                                            href={format!("/perlengkapan/pemakaian-bmn/{}", izin.id)}
                                            class="px-6 py-2 text-gray-700 bg-gray-100 rounded-lg hover:bg-gray-200 transition-colors"
                                        >
                                            "Batal"
                                        </a>
                                        <button
                                            type="submit"
                                            class="px-6 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700 transition-colors disabled:opacity-50 flex items-center gap-2"
                                            prop:disabled=move || loading.get()
                                        >
                                            <Show when=move || loading.get() fallback=|| view! { <AppIcon icon=ARROW_CLOCKWISE /> }>
                                                <span class="fa-spin"><AppIcon icon=SPINNER /></span>
                                            </Show>
                                            "Perpanjang Izin"
                                        </button>
                                    </div>
                                </form>
                            </div>
                        }.into_any()
                    }).unwrap_or_else(|| view! {
                        <div class="p-12 text-center">
                            <span class="text-5xl text-red-300 mb-4"><AppIcon icon=WARNING_CIRCLE /></span>
                            <p class="text-gray-600 text-lg">"Data tidak ditemukan"</p>
                        </div>
                    }.into_any())
                }}
            </Suspense>
        </div>
    }
}
