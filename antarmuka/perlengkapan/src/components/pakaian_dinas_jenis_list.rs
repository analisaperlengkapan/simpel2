//! Jenis Pakaian Dinas List Component
//!
//! Displays list of official uniform types with CRUD operations.

use crate::api::{
    AppError, CreateJenisPakaianDinasRequest, JenisPakaianDinas, create_jenis_pakaian_dinas,
    delete_jenis_pakaian_dinas, fetch_jenis_pakaian_dinas,
};
use crate::components::layout::{
    EmptyState, ErrorState, FormField, LoadingState, PageLayout, SectionCard,
};
use leptos::prelude::*;
use leptos::task::spawn_local;

#[component]
pub fn PakaianDinasJenisList() -> impl IntoView {
    let (page, set_page) = signal(1);
    let (show_form, set_show_form) = signal(false);
    let (form_nama, set_form_nama) = signal(String::new());
    let (form_keterangan, set_form_keterangan) = signal(String::new());
    let (is_loading, set_is_loading) = signal(false);
    let (error_message, set_error_message) = signal(Option::<String>::None);
    let refresh_trigger = RwSignal::new(0);

    let data_resource = LocalResource::new(move || {
        let p = page.get();
        let _trigger = refresh_trigger.get();
        async move { fetch_jenis_pakaian_dinas(p, 20).await }
    });

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_is_loading.set(true);
        set_error_message.set(None);

        let nama = form_nama.get();
        let keterangan = form_keterangan.get();

        spawn_local(async move {
            let request = CreateJenisPakaianDinasRequest {
                nama,
                keterangan: if keterangan.is_empty() {
                    None
                } else {
                    Some(keterangan)
                },
            };

            match create_jenis_pakaian_dinas(request).await {
                Ok(_) => {
                    set_form_nama.set(String::new());
                    set_form_keterangan.set(String::new());
                    set_show_form.set(false);
                    refresh_trigger.update(|v| *v += 1);
                }
                Err(e) => {
                    set_error_message.set(Some(e.user_message()));
                }
            }
            set_is_loading.set(false);
        });
    };

    view! {
        <PageLayout
            title="Jenis Pakaian Dinas"
            icon="fas fa-tshirt"
            description="Kelola master data jenis pakaian dinas"
        >
            // Action bar
            <div class="mb-4 flex justify-end">
                <button
                    class="inline-flex items-center gap-2 rounded-lg bg-gold-gradient px-4 py-2.5 text-sm font-bold text-navy-950 shadow-sm transition hover:opacity-90"
                    on:click=move |_| set_show_form.update(|v| *v = !*v)
                >
                    <i class="fas fa-plus text-xs"></i>
                    "Tambah Jenis"
                </button>
            </div>

            // Create form
            <Show when=move || show_form.get()>
                <SectionCard title="Tambah Jenis Pakaian Dinas Baru">
                    <Show when=move || error_message.get().is_some()>
                        <div class="mb-4 flex items-center gap-2 rounded-xl border border-danger-500/30 bg-danger-500/[0.08] px-4 py-3 text-sm text-danger-300">
                            <i class="fas fa-exclamation-circle"></i>
                            {move || error_message.get()}
                        </div>
                    </Show>

                    <form on:submit=on_submit>
                        <div class="grid grid-cols-1 gap-5 sm:grid-cols-2">
                            <FormField label="Nama Jenis">
                                <input
                                    type="text"
                                    class="focus-ring w-full rounded-lg border border-white/10 bg-white/[0.04] px-3.5 py-2.5 text-sm text-slate-100 placeholder-slate-500"
                                    placeholder="Contoh: PDH, PDL, Toga"
                                    prop:value=move || form_nama.get()
                                    on:input=move |ev| set_form_nama.set(event_target_value(&ev))
                                    required=true
                                />
                            </FormField>
                            <FormField label="Keterangan">
                                <input
                                    type="text"
                                    class="focus-ring w-full rounded-lg border border-white/10 bg-white/[0.04] px-3.5 py-2.5 text-sm text-slate-100 placeholder-slate-500"
                                    placeholder="Keterangan (opsional)"
                                    prop:value=move || form_keterangan.get()
                                    on:input=move |ev| set_form_keterangan.set(event_target_value(&ev))
                                />
                            </FormField>
                        </div>
                        <div class="mt-5 flex gap-3 border-t border-white/[0.04] pt-4">
                            <button
                                type="submit"
                                class="inline-flex items-center gap-2 rounded-lg bg-gold-gradient px-5 py-2.5 text-sm font-bold text-navy-950 shadow-sm transition hover:opacity-90 disabled:opacity-50"
                                prop:disabled=move || is_loading.get()
                            >
                                {move || if is_loading.get() { "Menyimpan..." } else { "Simpan" }}
                            </button>
                            <button
                                type="button"
                                class="rounded-lg border border-white/10 bg-white/[0.04] px-5 py-2.5 text-sm text-slate-300 transition hover:bg-white/[0.08]"
                                on:click=move |_| set_show_form.set(false)
                            >
                                "Batal"
                            </button>
                        </div>
                    </form>
                </SectionCard>
            </Show>

            // Data table
            <Suspense fallback=move || view! { <LoadingState /> }>
                {move || match data_resource.get() {
                    None => view! { <LoadingState /> }.into_any(),
                    Some(Err(e)) => view! { <ErrorState error=e /> }.into_any(),
                    Some(Ok(response)) => {
                        if response.data.is_empty() {
                            view! {
                                <EmptyState
                                    icon="fas fa-tshirt"
                                    title="Belum Ada Data"
                                    description="Klik \"Tambah Jenis\" untuk menambahkan jenis pakaian dinas."
                                />
                            }.into_any()
                        } else {
                            render_jenis_table(response.data, response.page, response.total, response.total_pages, page, set_page, refresh_trigger)
                        }
                    }
                }}
            </Suspense>
        </PageLayout>
    }
}

/// Renders the jenis pakaian dinas table with pagination.
fn render_jenis_table(
    data: Vec<JenisPakaianDinas>,
    current_page: i32,
    total: i64,
    total_pages: i32,
    page: ReadSignal<i32>,
    set_page: WriteSignal<i32>,
    refresh_trigger: RwSignal<i32>,
) -> AnyView {
    view! {
        <SectionCard title="Daftar Jenis">
            <div class="overflow-hidden rounded-2xl border border-white/[0.06] bg-surface-panel">
                <div class="overflow-x-auto">
                    <table class="min-w-full divide-y divide-white/[0.04]">
                        <thead class="bg-white/[0.02]">
                            <tr>
                                <th class="px-4 py-3 text-xs font-semibold uppercase tracking-wide text-slate-400">"No"</th>
                                <th class="px-4 py-3 text-left text-xs font-semibold uppercase tracking-wide text-slate-400">"Nama Jenis"</th>
                                <th class="px-4 py-3 text-left text-xs font-semibold uppercase tracking-wide text-slate-400">"Keterangan"</th>
                                <th class="px-4 py-3 text-left text-xs font-semibold uppercase tracking-wide text-slate-400">"Dibuat"</th>
                                <th class="px-4 py-3 text-left text-xs font-semibold uppercase tracking-wide text-slate-400">"Aksi"</th>
                            </tr>
                        </thead>
                        <tbody>
                            {data.into_iter().enumerate().map(|(idx, item)| {
                                let item_id = item.id.clone();
                                let item_id_for_delete = item_id.clone();
                                let num = ((page.get() - 1) * 20 + idx as i32 + 1).to_string();
                                let ket = item.keterangan.unwrap_or_else(|| "-".to_string());
                                let date = item.created_at.chars().take(10).collect::<String>();
                                let bg = if idx % 2 == 0 { "bg-transparent" } else { "bg-white/[0.015]" };
                                view! {
                                    <tr class=format!("border-b border-white/[0.04] {}", bg)>
                                        <td class="px-4 py-3 text-sm text-slate-400">{num}</td>
                                        <td class="px-4 py-3 text-sm font-semibold text-gold-400">{item.nama}</td>
                                        <td class="px-4 py-3 text-sm text-slate-400">{ket}</td>
                                        <td class="px-4 py-3 text-xs text-slate-500">{date}</td>
                                        <td class="px-4 py-3">
                                            <div class="flex items-center gap-3">
                                                <a
                                                    href=format!("/perlengkapan/pakaian-dinas/jenis/{}/spesifikasi", item_id)
                                                    class="text-success-400 transition hover:text-success-300"
                                                    title="Lihat Spesifikasi"
                                                >
                                                    <i class="fas fa-list text-xs"></i>
                                                </a>
                                                <button
                                                    class="text-danger-400 transition hover:text-danger-300"
                                                    title="Hapus"
                                                    on:click=move |_| {
                                                        let id = item_id_for_delete.clone();
                                                        let confirmed = web_sys::window()
                                                            .and_then(|w| w.confirm_with_message("Yakin ingin menghapus?").ok())
                                                            .unwrap_or(false);
                                                        if confirmed {
                                                            spawn_local(async move {
                                                                match delete_jenis_pakaian_dinas(id).await {
                                                                    Ok(_) => refresh_trigger.update(|v| *v += 1),
                                                                    Err(e) => leptos::logging::error!("Failed to delete: {}", e.user_message()),
                                                                }
                                                            });
                                                        }
                                                    }
                                                >
                                                    <i class="fas fa-trash text-xs"></i>
                                                </button>
                                            </div>
                                        </td>
                                    </tr>
                                }
                            }).collect_view()}
                        </tbody>
                    </table>
                </div>
            </div>

            // Pagination
            <div class="mt-4 flex items-center justify-between border-t border-white/[0.04] pt-4">
                <p class="text-xs text-slate-400">
                    "Halaman "
                    <span class="font-medium text-slate-200">{current_page}</span>
                    " dari "
                    <span class="font-medium text-slate-200">{total_pages}</span>
                    " (" <span class="font-medium text-slate-200">{total}</span> " data)"
                </p>
                <div class="flex gap-2">
                    <button
                        class="rounded-lg border border-white/10 bg-white/[0.04] px-3 py-1.5 text-xs text-slate-300 transition hover:bg-white/[0.08] disabled:opacity-40"
                        prop:disabled=move || page.get() <= 1
                        on:click=move |_| set_page.update(|p| *p -= 1)
                    >
                        "Sebelumnya"
                    </button>
                    <button
                        class="rounded-lg border border-white/10 bg-white/[0.04] px-3 py-1.5 text-xs text-slate-300 transition hover:bg-white/[0.08] disabled:opacity-40"
                        prop:disabled=move || page.get() >= total_pages
                        on:click=move |_| set_page.update(|p| *p += 1)
                    >
                        "Selanjutnya"
                    </button>
                </div>
            </div>
        </SectionCard>
    }.into_any()
}
