//! Master data hub — lists available master-data sources, and provides a
//! CRUD surface for records within the selected source. Consumes
//! `GET /admin/master` and `GET|POST|PUT|DELETE /admin/master/:source`
//! which land in plan commit 19.

use std::sync::Arc;

use leptos::prelude::*;
use lib_ui::components::icon::{AppIcon, icon_from_fa_class};
use phosphor_leptos::{
    ARROW_LEFT, ARROW_RIGHT, CARET_LEFT, CARET_RIGHT, CHECK, CLOCK, MAGNIFYING_GLASS, MINUS, PENCIL,
    PLUS, TRASH, WARNING, X,
};
use leptos::task::spawn_local;
use web_sys::SubmitEvent;

use crate::api::admin::{
    MasterRecord, MasterSource, MasterUpsertRequest, create_master_record, delete_master_record,
    fetch_master_records, fetch_master_sources, update_master_record,
};
use crate::api::error::AppError;
use crate::components::layout::{
    EmptyState, ErrorState, LoadingState, PageBreadcrumb, PageLayout, SectionCard,
};

const PER_PAGE: i32 = 25;

#[component]
pub fn AdminMasterDataPage() -> impl IntoView {
    let (sources, set_sources) = signal::<Vec<MasterSource>>(Vec::new());
    let (sources_loading, set_sources_loading) = signal(true);
    let (sources_error, set_sources_error) = signal::<Option<AppError>>(None);
    let (selected, set_selected) = signal::<Option<MasterSource>>(None);
    let (reload_tick, set_reload_tick) = signal(0_u32);

    Effect::new(move |_| {
        let _ = reload_tick.get();
        set_sources_loading.set(true);
        set_sources_error.set(None);
        spawn_local(async move {
            match fetch_master_sources().await {
                Ok(list) => set_sources.set(list),
                Err(e) => set_sources_error.set(Some(e)),
            }
            set_sources_loading.set(false);
        });
    });

    let breadcrumbs = vec![
        PageBreadcrumb::new("Dashboard", "/perlengkapan/dashboard"),
        PageBreadcrumb::new("Admin", "/perlengkapan/admin/workflow"),
        PageBreadcrumb::leaf("Master Data"),
    ];

    view! {
        <PageLayout
            title="Master Data"
            description="Kelola data referensi sistem: aktivitas BMN, jenis pakaian, spesifikasi, ukuran standar, kode barang."
            icon="fas fa-database"
            breadcrumbs=breadcrumbs
        >
            {move || match selected.get() {
                None => view! {
                    <SectionCard title="Sumber Data" icon="fas fa-layer-group">
                        {move || {
                            if sources_loading.get() {
                                view! { <LoadingState message="Memuat daftar sumber data..." /> }.into_any()
                            } else if let Some(e) = sources_error.get() {
                                let retry: Box<dyn Fn()> = Box::new(move || { set_reload_tick.update(|t| *t += 1); });
                                view! { <ErrorState error=e on_retry=retry /> }.into_any()
                            } else if sources.with(Vec::is_empty) {
                                view! {
                                    <EmptyState
                                        title="Belum ada sumber master data"
                                        description="Backend belum mendaftarkan sumber master data apapun."
                                        icon="fas fa-inbox"
                                    />
                                }.into_any()
                            } else {
                                view! { <SourceGrid sources=sources set_selected=set_selected /> }.into_any()
                            }
                        }}
                    </SectionCard>
                }.into_any(),
                Some(source) => view! {
                    <SourceDetail
                        source=source
                        on_back=Callback::new(move |_| set_selected.set(None))
                    />
                }.into_any(),
            }}
        </PageLayout>
    }
}

#[component]
fn SourceGrid(
    sources: ReadSignal<Vec<MasterSource>>,
    set_selected: WriteSignal<Option<MasterSource>>,
) -> impl IntoView {
    view! {
        <div class="grid gap-4 md:grid-cols-2 xl:grid-cols-3">
            {move || sources.get().into_iter().map(|source| {
                let source_for_click = source.clone();
                let icon_class = source.icon.clone().unwrap_or_else(|| "fas fa-table".to_string());
                let updated = source.updated_at.clone().unwrap_or_else(|| "—".to_string());
                view! {
                    <button
                        type="button"
                        class="focus-ring group flex flex-col items-start gap-3 rounded-2xl border border-white/[0.06] bg-white/[0.02] p-5 text-left transition hover:border-gold-500/40 hover:bg-white/[0.04]"
                        on:click=move |_| set_selected.set(Some(source_for_click.clone()))
                    >
                        <div class="flex w-full items-start justify-between gap-3">
                            <span class="flex h-10 w-10 items-center justify-center rounded-xl bg-gold-500/10 text-gold-400 ring-1 ring-gold-500/20">
                                <AppIcon icon=icon_from_fa_class(&icon_class) size=18 />
                            </span>
                            <span class="rounded-lg border border-info-500/30 bg-info-500/10 px-2 py-0.5 text-[0.7rem] font-semibold text-info-300">
                                {format!("{} record", source.record_count)}
                            </span>
                        </div>
                        <div>
                            <h3 class="text-base font-semibold text-white transition group-hover:text-gold-300">
                                {source.label.clone()}
                            </h3>
                            <p class="mt-1 text-sm leading-relaxed text-slate-400">
                                {source.description.clone()}
                            </p>
                        </div>
                        <div class="mt-auto flex w-full items-center justify-between gap-2 text-[0.7rem] text-slate-500">
                            <span>
                                <span class="mr-1 text-[0.65rem]"><AppIcon icon=CLOCK /></span>
                                "Diperbarui: "
                                {updated}
                            </span>
                            <span class="font-semibold text-gold-400">
                                "Kelola"
                                <span class="ml-1 text-[0.65rem]"><AppIcon icon=ARROW_RIGHT /></span>
                            </span>
                        </div>
                    </button>
                }
            }).collect::<Vec<_>>()}
        </div>
    }
}

#[component]
fn SourceDetail(source: MasterSource, #[prop(into)] on_back: Callback<()>) -> impl IntoView {
    let source_key = source.key.clone();
    let source_label = source.label.clone();

    let (records, set_records) = signal::<Vec<MasterRecord>>(Vec::new());
    let (total, set_total) = signal(0_i64);
    let (page, set_page) = signal(1_i32);
    let (search, set_search) = signal::<String>(String::new());
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal::<Option<AppError>>(None);
    let (reload_tick, set_reload_tick) = signal(0_u32);
    let (editing, set_editing) = signal::<Option<MasterRecord>>(None);
    let (show_create, set_show_create) = signal(false);
    let (deleting, set_deleting) = signal::<Option<MasterRecord>>(None);
    let (action_msg, set_action_msg) = signal::<Option<(bool, String)>>(None);

    let source_key_effect = source_key.clone();
    Effect::new(move |_| {
        let _ = reload_tick.get();
        let key = source_key_effect.clone();
        let p = page.get();
        let q = search.get();
        let q_opt = if q.trim().is_empty() { None } else { Some(q) };
        set_loading.set(true);
        set_error.set(None);
        spawn_local(async move {
            let q_ref = q_opt.as_deref();
            match fetch_master_records(&key, p, PER_PAGE, q_ref).await {
                Ok(resp) => {
                    set_records.set(resp.data);
                    set_total.set(resp.total);
                }
                Err(e) => set_error.set(Some(e)),
            }
            set_loading.set(false);
        });
    });

    let refresh = move || set_reload_tick.update(|t| *t += 1);

    let on_search_submit = move |ev: SubmitEvent| {
        ev.prevent_default();
        set_page.set(1);
        set_reload_tick.update(|t| *t += 1);
    };

    let source_key_delete = source_key.clone();
    let confirm_delete: Arc<dyn Fn(MasterRecord) + Send + Sync> =
        Arc::new(move |rec: MasterRecord| {
            let key = source_key_delete.clone();
            let id = rec.id.clone();
            spawn_local(async move {
                match delete_master_record(&key, &id).await {
                    Ok(()) => {
                        set_action_msg.set(Some((true, "Record berhasil dihapus.".to_string())));
                        set_deleting.set(None);
                        set_reload_tick.update(|t| *t += 1);
                    }
                    Err(e) => {
                        set_action_msg.set(Some((false, e.user_message())));
                        set_deleting.set(None);
                    }
                }
            });
        });

    let source_key_for_editor = source_key.clone();

    let total_pages = move || {
        let t = total.get();
        if t == 0 {
            1
        } else {
            ((t + PER_PAGE as i64 - 1) / PER_PAGE as i64) as i32
        }
    };

    let label_for_header = source_label.clone();
    view! {
        <SectionCard
            title=label_for_header
            icon=source.icon.clone().unwrap_or_else(|| "fas fa-table".to_string())
            description=source.description.clone()
            actions=Box::new(move || view! {
                <button
                    type="button"
                    class="focus-ring rounded-lg border border-white/[0.08] bg-white/[0.04] px-3 py-2 text-sm font-medium text-slate-200 transition hover:bg-white/[0.08]"
                    on:click=move |_| on_back.run(())
                >
                    <span class="mr-1.5"><AppIcon icon=ARROW_LEFT /></span>
                    "Kembali"
                </button>
                <button
                    type="button"
                    class="focus-ring rounded-lg bg-gold-gradient px-3 py-2 text-sm font-semibold text-navy-950 shadow-card transition hover:brightness-105"
                    on:click=move |_| set_show_create.set(true)
                >
                    <span class="mr-1.5"><AppIcon icon=PLUS /></span>
                    "Tambah"
                </button>
            }.into_any())
        >
            <form class="mb-4 flex gap-2" on:submit=on_search_submit>
                <input
                    type="search"
                    placeholder="Cari kode atau nama..."
                    prop:value=move || search.get()
                    on:input=move |e| set_search.set(event_target_value(&e))
                    class="focus-ring flex-1 rounded-lg border border-white/[0.08] bg-white/[0.04] px-3 py-2 text-sm text-white placeholder:text-slate-500"
                />
                <button
                    type="submit"
                    class="focus-ring rounded-lg border border-white/[0.08] bg-white/[0.04] px-4 py-2 text-sm font-medium text-slate-200 transition hover:bg-white/[0.08]"
                >
                    <AppIcon icon=MAGNIFYING_GLASS />
                </button>
            </form>

            {move || action_msg.get().map(|(ok, msg)| {
                let class = if ok {
                    "rounded-xl border border-success-500/30 bg-success-500/10 p-3 text-xs text-success-400"
                } else {
                    "rounded-xl border border-danger-500/30 bg-danger-500/10 p-3 text-xs text-danger-400"
                };
                view! { <div class=class>{msg}</div> }
            })}

            {move || {
                if loading.get() {
                    view! { <LoadingState message="Memuat record..." /> }.into_any()
                } else if let Some(e) = error.get() {
                    let retry: Box<dyn Fn()> = Box::new(move || { set_reload_tick.update(|t| *t += 1); });
                    view! { <ErrorState error=e on_retry=retry /> }.into_any()
                } else if records.with(Vec::is_empty) {
                    view! {
                        <EmptyState
                            title="Belum ada record"
                            description="Sumber master data ini belum memiliki record apapun."
                            icon="fas fa-inbox"
                        />
                    }.into_any()
                } else {
                    view! {
                        <RecordTable
                            rows=records
                            set_editing=set_editing
                            set_deleting=set_deleting
                        />
                        <PaginationRow
                            page=page
                            set_page=set_page
                            total_pages=Signal::derive(total_pages)
                            on_change=Box::new(refresh)
                        />
                    }.into_any()
                }
            }}

            {
                let source_key_for_create = source_key_for_editor.clone();
                move || show_create.get().then(|| {
                    let source_key = source_key_for_create.clone();
                    view! {
                        <RecordEditorModal
                            source_key=source_key
                            record=None
                            on_close=Callback::new(move |_| set_show_create.set(false))
                            on_saved=Callback::new(move |_| {
                                set_show_create.set(false);
                                set_action_msg.set(Some((true, "Record berhasil disimpan.".to_string())));
                                set_reload_tick.update(|t| *t += 1);
                            })
                        />
                    }
                })
            }

            {move || editing.get().map(|rec| {
                let source_key = source_key_for_editor.clone();
                view! {
                    <RecordEditorModal
                        source_key=source_key
                        record=Some(rec)
                        on_close=Callback::new(move |_| set_editing.set(None))
                        on_saved=Callback::new(move |_| {
                            set_editing.set(None);
                            set_action_msg.set(Some((true, "Record berhasil disimpan.".to_string())));
                            set_reload_tick.update(|t| *t += 1);
                        })
                    />
                }
            })}

            {move || deleting.get().map(|rec| {
                let rec_confirm = rec.clone();
                let rec_for_label = rec.clone();
                let cd = Arc::clone(&confirm_delete);
                view! {
                    <ConfirmDeleteModal
                        record=rec_for_label
                        on_close=Callback::new(move |_| set_deleting.set(None))
                        on_confirm=Callback::new(move |_| cd(rec_confirm.clone()))
                    />
                }
            })}
        </SectionCard>
    }
}

#[component]
fn RecordTable(
    rows: ReadSignal<Vec<MasterRecord>>,
    set_editing: WriteSignal<Option<MasterRecord>>,
    set_deleting: WriteSignal<Option<MasterRecord>>,
) -> impl IntoView {
    view! {
        <div class="overflow-x-auto rounded-xl border border-white/[0.06]">
            <table class="min-w-full border-collapse">
                <thead>
                    <tr class="bg-white/[0.02]">
                        <th class="border-b border-white/[0.06] px-3 py-2 text-left text-[0.65rem] font-semibold uppercase tracking-wider text-slate-400">"Kode"</th>
                        <th class="border-b border-white/[0.06] px-3 py-2 text-left text-[0.65rem] font-semibold uppercase tracking-wider text-slate-400">"Nama"</th>
                        <th class="border-b border-white/[0.06] px-3 py-2 text-left text-[0.65rem] font-semibold uppercase tracking-wider text-slate-400">"Deskripsi"</th>
                        <th class="border-b border-white/[0.06] px-3 py-2 text-center text-[0.65rem] font-semibold uppercase tracking-wider text-slate-400">"Aktif"</th>
                        <th class="border-b border-white/[0.06] px-3 py-2 text-right text-[0.65rem] font-semibold uppercase tracking-wider text-slate-400">"Aksi"</th>
                    </tr>
                </thead>
                <tbody>
                    {move || rows.get().into_iter().map(|rec| {
                        let rec_edit = rec.clone();
                        let rec_del = rec.clone();
                        let code = rec.code.clone().unwrap_or_else(|| "—".to_string());
                        let desc = rec.description.clone().unwrap_or_default();
                        let active = rec.active;
                        view! {
                            <tr class="border-b border-white/[0.04] text-xs">
                                <td class="px-3 py-2 font-mono text-slate-300">{code}</td>
                                <td class="px-3 py-2 font-semibold text-white">{rec.name.clone()}</td>
                                <td class="px-3 py-2 text-slate-400">{desc}</td>
                                <td class="px-3 py-2 text-center">
                                    {if active {
                                        view! {
                                            <span class="inline-flex h-6 w-6 items-center justify-center rounded-md border border-success-500/30 bg-success-500/15 text-success-400">
                                                <span class="text-[0.65rem]"><AppIcon icon=CHECK /></span>
                                            </span>
                                        }.into_any()
                                    } else {
                                        view! {
                                            <span class="inline-flex h-6 w-6 items-center justify-center rounded-md border border-white/[0.06] bg-white/[0.02] text-slate-500">
                                                <span class="text-[0.65rem]"><AppIcon icon=MINUS /></span>
                                            </span>
                                        }.into_any()
                                    }}
                                </td>
                                <td class="px-3 py-2 text-right">
                                    <div class="inline-flex gap-1">
                                        <button
                                            type="button"
                                            class="focus-ring rounded-md border border-info-500/30 bg-info-500/10 px-2 py-1 text-[0.7rem] font-semibold text-info-300 transition hover:bg-info-500/20"
                                            title="Edit"
                                            on:click=move |_| set_editing.set(Some(rec_edit.clone()))
                                        >
                                            <AppIcon icon=PENCIL />
                                        </button>
                                        <button
                                            type="button"
                                            class="focus-ring rounded-md border border-danger-500/30 bg-danger-500/10 px-2 py-1 text-[0.7rem] font-semibold text-danger-400 transition hover:bg-danger-500/20"
                                            title="Hapus"
                                            on:click=move |_| set_deleting.set(Some(rec_del.clone()))
                                        >
                                            <AppIcon icon=TRASH />
                                        </button>
                                    </div>
                                </td>
                            </tr>
                        }
                    }).collect::<Vec<_>>()}
                </tbody>
            </table>
        </div>
    }
}

#[component]
fn PaginationRow(
    page: ReadSignal<i32>,
    set_page: WriteSignal<i32>,
    total_pages: Signal<i32>,
    on_change: Box<dyn Fn()>,
) -> impl IntoView {
    use std::rc::Rc;
    let on_change = Rc::new(on_change);
    let prev_handler = {
        let on_change = Rc::clone(&on_change);
        move |_: leptos::ev::MouseEvent| {
            let cur = page.get();
            if cur > 1 {
                set_page.set(cur - 1);
                on_change();
            }
        }
    };
    let next_handler = {
        let on_change = Rc::clone(&on_change);
        move |_: leptos::ev::MouseEvent| {
            let cur = page.get();
            if cur < total_pages.get() {
                set_page.set(cur + 1);
                on_change();
            }
        }
    };
    view! {
        <div class="mt-3 flex items-center justify-between text-xs text-slate-400">
            <div>
                "Halaman " <strong class="text-white">{move || page.get()}</strong>
                " dari "
                <strong class="text-white">{move || total_pages.get()}</strong>
            </div>
            <div class="flex gap-2">
                <button
                    type="button"
                    class="focus-ring rounded-lg border border-white/[0.08] bg-white/[0.04] px-3 py-1.5 text-xs font-medium text-slate-200 transition hover:bg-white/[0.08] disabled:opacity-40"
                    prop:disabled=move || page.get() <= 1
                    on:click=prev_handler
                >
                    <span class="mr-1"><AppIcon icon=CARET_LEFT /></span>
                    "Sebelumnya"
                </button>
                <button
                    type="button"
                    class="focus-ring rounded-lg border border-white/[0.08] bg-white/[0.04] px-3 py-1.5 text-xs font-medium text-slate-200 transition hover:bg-white/[0.08] disabled:opacity-40"
                    prop:disabled=move || page.get() >= total_pages.get()
                    on:click=next_handler
                >
                    "Berikutnya"
                    <span class="ml-1"><AppIcon icon=CARET_RIGHT /></span>
                </button>
            </div>
        </div>
    }
}

#[component]
fn RecordEditorModal(
    source_key: String,
    record: Option<MasterRecord>,
    #[prop(into)] on_close: Callback<()>,
    #[prop(into)] on_saved: Callback<()>,
) -> impl IntoView {
    let is_create = record.is_none();
    let title = if is_create {
        "Tambah Record"
    } else {
        "Edit Record"
    };

    let (code, set_code) = signal(
        record
            .as_ref()
            .and_then(|r| r.code.clone())
            .unwrap_or_default(),
    );
    let (name, set_name) = signal(record.as_ref().map(|r| r.name.clone()).unwrap_or_default());
    let (description, set_description) = signal(
        record
            .as_ref()
            .and_then(|r| r.description.clone())
            .unwrap_or_default(),
    );
    let (active, set_active) = signal(record.as_ref().map(|r| r.active).unwrap_or(true));
    let (saving, set_saving) = signal(false);
    let (error, set_error) = signal::<Option<String>>(None);

    let source_key = source_key.clone();
    let record_id = record.as_ref().map(|r| r.id.clone());

    let on_submit = move |ev: SubmitEvent| {
        ev.prevent_default();
        set_saving.set(true);
        set_error.set(None);

        let req = MasterUpsertRequest {
            code: Some(code.get()).filter(|s| !s.trim().is_empty()),
            name: name.get(),
            description: Some(description.get()).filter(|s| !s.trim().is_empty()),
            active: active.get(),
            extra: serde_json::Value::Null,
        };
        let key = source_key.clone();
        let id = record_id.clone();

        spawn_local(async move {
            let result = if let Some(id) = id {
                update_master_record(&key, &id, &req).await.map(|_| ())
            } else {
                create_master_record(&key, &req).await.map(|_| ())
            };
            match result {
                Ok(()) => on_saved.run(()),
                Err(e) => {
                    set_error.set(Some(e.user_message()));
                    set_saving.set(false);
                }
            }
        });
    };

    view! {
        <div
            class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 p-4 backdrop-blur-sm"
            on:click=move |e| {
                if e.target() == e.current_target() {
                    on_close.run(());
                }
            }
        >
            <form
                class="flex max-h-[92vh] w-full max-w-lg flex-col overflow-hidden rounded-2xl border border-white/[0.08] bg-surface-panel shadow-panel"
                on:submit=on_submit
            >
                <header class="flex items-start justify-between gap-4 border-b border-white/[0.06] p-6">
                    <h2 class="text-lg font-bold text-white">{title}</h2>
                    <button
                        type="button"
                        class="focus-ring rounded-lg border border-white/[0.08] bg-white/[0.04] p-2 text-slate-300 transition hover:bg-white/[0.08]"
                        on:click=move |_| on_close.run(())
                    >
                        <AppIcon icon=X />
                    </button>
                </header>

                <div class="flex-1 overflow-y-auto p-6">
                    {move || error.get().map(|msg| view! {
                        <div class="mb-4 flex items-start gap-3 rounded-xl border border-danger-500/30 bg-danger-500/10 p-3">
                            <span class="mt-0.5 text-danger-400"><AppIcon icon=WARNING /></span>
                            <div class="text-xs text-danger-100">{msg}</div>
                        </div>
                    })}

                    <div class="flex flex-col gap-4">
                        <div>
                            <label class="mb-2 block text-sm font-semibold text-white">"Kode"</label>
                            <input
                                type="text"
                                prop:value=move || code.get()
                                prop:disabled=move || saving.get()
                                on:input=move |e| set_code.set(event_target_value(&e))
                                class="focus-ring w-full rounded-lg border border-white/[0.08] bg-white/[0.04] px-3 py-2 text-sm text-white placeholder:text-slate-500"
                            />
                        </div>
                        <div>
                            <label class="mb-2 block text-sm font-semibold text-white">
                                "Nama "
                                <span class="text-danger-400">"*"</span>
                            </label>
                            <input
                                type="text"
                                required=true
                                prop:value=move || name.get()
                                prop:disabled=move || saving.get()
                                on:input=move |e| set_name.set(event_target_value(&e))
                                class="focus-ring w-full rounded-lg border border-white/[0.08] bg-white/[0.04] px-3 py-2 text-sm text-white placeholder:text-slate-500"
                            />
                        </div>
                        <div>
                            <label class="mb-2 block text-sm font-semibold text-white">"Deskripsi"</label>
                            <textarea
                                rows="3"
                                prop:value=move || description.get()
                                prop:disabled=move || saving.get()
                                on:input=move |e| set_description.set(event_target_value(&e))
                                class="focus-ring w-full rounded-lg border border-white/[0.08] bg-white/[0.04] px-3 py-2 text-sm text-white placeholder:text-slate-500"
                            ></textarea>
                        </div>
                        <label class="flex items-center gap-3 rounded-xl border border-white/[0.06] bg-white/[0.02] p-3">
                            <input
                                type="checkbox"
                                prop:checked=move || active.get()
                                prop:disabled=move || saving.get()
                                on:change=move |e| set_active.set(event_target_checked(&e))
                                class="h-4 w-4 cursor-pointer accent-gold-400"
                            />
                            <div>
                                <div class="text-sm font-semibold text-white">"Aktif"</div>
                                <div class="text-xs text-slate-400">
                                    "Nonaktifkan untuk menyembunyikan record dari dropdown tanpa menghapusnya."
                                </div>
                            </div>
                        </label>
                    </div>
                </div>

                <footer class="flex justify-end gap-2 border-t border-white/[0.06] p-4">
                    <button
                        type="button"
                        class="focus-ring rounded-lg border border-white/[0.08] bg-white/[0.04] px-4 py-2 text-sm font-medium text-slate-200 transition hover:bg-white/[0.08]"
                        prop:disabled=move || saving.get()
                        on:click=move |_| on_close.run(())
                    >
                        "Batal"
                    </button>
                    <button
                        type="submit"
                        class="focus-ring rounded-lg bg-gold-gradient px-4 py-2 text-sm font-semibold text-navy-950 shadow-card transition hover:brightness-105 disabled:opacity-60"
                        prop:disabled=move || saving.get() || name.get().trim().is_empty()
                    >
                        {move || if saving.get() { "Menyimpan..." } else { "Simpan" }}
                    </button>
                </footer>
            </form>
        </div>
    }
}

#[component]
fn ConfirmDeleteModal(
    record: MasterRecord,
    #[prop(into)] on_close: Callback<()>,
    #[prop(into)] on_confirm: Callback<()>,
) -> impl IntoView {
    let name = record.name.clone();
    view! {
        <div
            class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 p-4 backdrop-blur-sm"
            on:click=move |e| {
                if e.target() == e.current_target() {
                    on_close.run(());
                }
            }
        >
            <div class="w-full max-w-md rounded-2xl border border-white/[0.08] bg-surface-panel p-6 shadow-panel">
                <div class="mb-5 flex flex-col items-center text-center">
                    <div class="mb-4 flex h-14 w-14 items-center justify-center rounded-full border-2 border-danger-500/30 bg-danger-500/10 text-danger-400">
                        <span class="text-xl"><AppIcon icon=WARNING /></span>
                    </div>
                    <h2 class="text-lg font-bold text-white">"Hapus Record?"</h2>
                    <p class="mt-2 text-sm text-slate-400">
                        "Anda yakin ingin menghapus "
                        <strong class="text-white">{name}</strong>
                        "? Tindakan ini tidak dapat dibatalkan."
                    </p>
                </div>
                <div class="flex gap-2">
                    <button
                        type="button"
                        class="focus-ring flex-1 rounded-lg border border-white/[0.08] bg-white/[0.04] px-4 py-2 text-sm font-medium text-slate-200 transition hover:bg-white/[0.08]"
                        on:click=move |_| on_close.run(())
                    >
                        "Batal"
                    </button>
                    <button
                        type="button"
                        class="focus-ring flex-1 rounded-lg bg-danger-500 px-4 py-2 text-sm font-semibold text-white transition hover:bg-danger-600"
                        on:click=move |_| on_confirm.run(())
                    >
                        "Hapus"
                    </button>
                </div>
            </div>
        </div>
    }
}
