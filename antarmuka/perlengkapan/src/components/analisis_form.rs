use crate::api::{CreateAnalisisRequest, create_analisis};
use crate::routes;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_meta::Title;
use leptos_router::hooks::use_navigate;
use lib_ui::components::icon::AppIcon;
use lib_ui::hooks::{use_form, use_toast::use_toast};
use phosphor_leptos::{FLOPPY_DISK, SPINNER};

/// Form data for creating an analysis request.
#[derive(Clone, Default)]
struct AnalisisFormData {
    judul: String,
    kategori: String,
    prioritas: String,
    estimasi: String,
}

#[component]
pub fn AnalisisForm() -> impl IntoView {
    let form = use_form(AnalisisFormData {
        prioritas: "sedang".to_string(),
        ..Default::default()
    });
    let toast = use_toast();
    let navigate = use_navigate();

    let on_submit = move |ev: leptos::web_sys::SubmitEvent| {
        ev.prevent_default();
        let data = form.begin_submit();

        let req = CreateAnalisisRequest {
            judul: data.judul,
            kategori: data.kategori,
            deskripsi: None,
            prioritas: data.prioritas,
            estimasi_biaya: data.estimasi.parse::<f64>().ok(),
            justifikasi: None,
        };

        let navigate = navigate.clone();
        spawn_local(async move {
            match create_analisis(req).await {
                Ok(_) => {
                    // Show toast immediately, but keep `submitting=true` during
                    // the navigation delay to prevent double-submission. Only
                    // call `finish_ok()` after the timeout, right before we
                    // navigate away.
                    toast.success("Data analisis berhasil disimpan!");
                    gloo_timers::future::TimeoutFuture::new(1000).await;
                    form.finish_ok();
                    // Pakai relative path (tanpa prefix /perlengkapan/simpel/v2/) supaya
                    // Leptos Router base tidak double-prefix. Konstanta `routes::path::*`
                    // dipakai untuk HTML href/window.location (absolute), bukan nav().
                    navigate("/analitik/roadmap", Default::default());
                }
                Err(e) => {
                    let msg = format!("Gagal menyimpan: {:?}", e);
                    form.finish_err(msg.clone());
                    toast.error(msg);
                }
            }
        });
    };

    view! {
        <Title text="Buat Analisis — SIMPEL Perlengkapan" />
        <div class="mx-auto max-w-2xl rounded-xl border border-white/[0.06] bg-white/[0.04] p-6">
            <h2 class="mb-6 text-xl font-bold text-slate-100">"Buat Analisis Kebutuhan"</h2>

            <form on:submit=on_submit class="space-y-4">
                <div>
                    <label class="mb-1 block text-sm font-medium text-slate-300">
                        "Judul Analisis"
                    </label>
                    <input
                        type="text"
                        class="w-full rounded-lg border border-white/10 bg-white/[0.04] px-4 py-2 text-sm text-slate-100 placeholder-slate-500 outline-none transition focus:border-gold-500/40 focus:ring-2 focus:ring-gold-500/30"
                        placeholder="Contoh: Kebutuhan Server Data Center"
                        prop:value=move || form.get().judul.clone()
                        on:input=move |ev| form.update(|f| f.judul = event_target_value(&ev))
                        required
                    />
                </div>

                <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                    <div>
                        <label class="mb-1 block text-sm font-medium text-slate-300">
                            "Kategori"
                        </label>
                        <select
                            class="w-full rounded-lg border border-white/10 bg-white/[0.04] px-4 py-2 text-sm text-slate-100 placeholder-slate-500 outline-none transition focus:border-gold-500/40 focus:ring-2 focus:ring-gold-500/30"
                            prop:value=move || form.get().kategori.clone()
                            on:change=move |ev| {
                                form.update(|f| f.kategori = event_target_value(&ev))
                            }
                            required
                        >
                            <option value="">"Pilih Kategori"</option>
                            <option value="TIK">"TIK"</option>
                            <option value="Kendaraan">"Kendaraan"</option>
                            <option value="Gedung">"Gedung"</option>
                            <option value="Lainnya">"Lainnya"</option>
                        </select>
                    </div>

                    <div>
                        <label class="mb-1 block text-sm font-medium text-slate-300">
                            "Prioritas"
                        </label>
                        <select
                            class="w-full rounded-lg border border-white/10 bg-white/[0.04] px-4 py-2 text-sm text-slate-100 placeholder-slate-500 outline-none transition focus:border-gold-500/40 focus:ring-2 focus:ring-gold-500/30"
                            prop:value=move || form.get().prioritas.clone()
                            on:change=move |ev| {
                                form.update(|f| f.prioritas = event_target_value(&ev))
                            }
                        >
                            <option value="rendah">"Rendah"</option>
                            <option value="sedang">"Sedang"</option>
                            <option value="tinggi">"Tinggi"</option>
                        </select>
                    </div>
                </div>

                <div>
                    <label class="mb-1 block text-sm font-medium text-slate-300">
                        "Estimasi Biaya (Rp)"
                    </label>
                    <input
                        type="number"
                        class="w-full rounded-lg border border-white/10 bg-white/[0.04] px-4 py-2 text-sm text-slate-100 placeholder-slate-500 outline-none transition focus:border-gold-500/40 focus:ring-2 focus:ring-gold-500/30"
                        placeholder="0"
                        prop:value=move || form.get().estimasi.clone()
                        on:input=move |ev| form.update(|f| f.estimasi = event_target_value(&ev))
                    />
                </div>

                <div class="pt-4 flex justify-end gap-3">
                    <a
                        href=routes::path::ANALITIK_ROADMAP
                        class="rounded-lg border border-white/10 bg-white/[0.04] px-4 py-2 text-sm text-slate-300 transition hover:bg-white/[0.08]"
                    >
                        "Batal"
                    </a>
                    <button
                        type="submit"
                        class="inline-flex items-center gap-2 rounded-lg bg-gold-gradient px-4 py-2 text-sm font-bold text-navy-950 transition hover:opacity-90 disabled:cursor-not-allowed disabled:opacity-50"
                        prop:disabled=move || form.submitting.get()
                    >
                        <Show
                            when=move || form.submitting.get()
                            fallback=|| view! { <AppIcon icon=FLOPPY_DISK /> }
                        >
                            <span class="fa-spin">
                                <AppIcon icon=SPINNER />
                            </span>
                        </Show>
                        "Simpan"
                    </button>
                </div>
            </form>
        </div>
    }
}
