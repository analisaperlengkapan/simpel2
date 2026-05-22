//! `/admin/templates` — list document templates + render a live preview.
//!
//! Lists templates from `GET /admin/templates`, lets the operator pick one,
//! and renders a PDF/DOCX/HTML preview inline via `POST
//! /admin/templates/{id}/preview`. The preview bytes are wrapped in a blob
//! URL and dropped into an `<iframe>` (PDF/HTML) or offered as a download
//! link (DOCX/XLSX — browsers don't render those inline).

use leptos::prelude::*;
use leptos::task::spawn_local;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{Blob, BlobPropertyBag, Url};

use crate::api::dokumen::{
    list_templates, preview_template, DocumentTemplate, PreviewBody,
};
use crate::api::error::AppError;

const FORMATS: &[(&str, &str)] = &[
    ("pdf", "PDF"),
    ("docx", "DOCX"),
    ("html", "HTML"),
    ("xlsx", "Excel"),
];

#[component]
pub fn AdminTemplatesPage() -> impl IntoView {
    let (templates, set_templates) = signal::<Vec<DocumentTemplate>>(Vec::new());
    let (selected_id, set_selected_id) = signal::<Option<String>>(None);
    let (format, set_format) = signal("pdf".to_string());
    let (preview_url, set_preview_url) = signal::<Option<String>>(None);
    let (preview_blob_mime, set_preview_blob_mime) = signal::<String>("application/pdf".to_string());
    let (loading_list, set_loading_list) = signal(false);
    let (loading_preview, set_loading_preview) = signal(false);
    let (error, set_error) = signal::<Option<String>>(None);

    // Fetch the template list on mount.
    {
        let set_templates = set_templates;
        let set_error = set_error;
        let set_loading_list = set_loading_list;
        Effect::new(move |_| {
            set_loading_list.set(true);
            spawn_local(async move {
                match list_templates(None, Some(true), 1, 100).await {
                    Ok(resp) => set_templates.set(resp.templates),
                    Err(e) => set_error.set(Some(format!("Gagal memuat templates: {}", e))),
                }
                set_loading_list.set(false);
            });
        });
    }

    let trigger_preview = move |template_id: String| {
        set_error.set(None);
        set_loading_preview.set(true);
        // Revoke previous blob URL — browsers leak otherwise.
        if let Some(prev) = preview_url.get_untracked() {
            let _ = Url::revoke_object_url(&prev);
        }
        set_preview_url.set(None);

        let fmt = format.get();
        let mime = mime_for(&fmt).to_string();
        set_preview_blob_mime.set(mime.clone());

        spawn_local(async move {
            match preview_template(&template_id, &fmt, &PreviewBody::default()).await {
                Ok(bytes) => match make_blob_url(&bytes, &mime) {
                    Ok(url) => set_preview_url.set(Some(url)),
                    Err(e) => set_error.set(Some(format!("Gagal membuat blob URL: {}", e))),
                },
                Err(AppError::NotFound(msg)) => {
                    set_error.set(Some(format!("Template / preview tidak ditemukan: {}", msg)))
                }
                Err(e) => set_error.set(Some(format!("Gagal merender preview: {}", e))),
            }
            set_loading_preview.set(false);
        });
    };

    view! {
        <div class="max-w-6xl mx-auto p-6">
            <div class="mb-6">
                <h1 class="text-2xl font-bold text-gray-800">"Document Templates"</h1>
                <p class="text-sm text-gray-500">
                    "Pratinjau langsung template surat / SK / laporan. Pilih template di kiri, atur format, lalu klik "
                    <span class="font-semibold">"Render Preview"</span>
                    "."
                </p>
            </div>

            <Show when=move || error.get().is_some()>
                <div class="mb-4 p-3 bg-red-50 text-red-700 rounded border border-red-200 text-sm">
                    {move || error.get().unwrap_or_default()}
                </div>
            </Show>

            <div class="grid grid-cols-12 gap-4">
                // ── Left: template list ─────────────────────────────────
                <div class="col-span-12 md:col-span-4 bg-white rounded-lg border p-3">
                    <h2 class="font-semibold text-gray-700 mb-2">"Daftar Template"</h2>
                    <Show when=move || loading_list.get()>
                        <p class="text-xs text-gray-400">"Memuat..."</p>
                    </Show>
                    <ul class="space-y-1 max-h-[70vh] overflow-y-auto">
                        <For
                            each=move || templates.get()
                            key=|t| t.id.clone()
                            children=move |t| {
                                let id = t.id.clone();
                                let id_for_select = id.clone();
                                let is_selected = move || {
                                    selected_id
                                        .get()
                                        .as_deref()
                                        .map(|s| s == id.as_str())
                                        .unwrap_or(false)
                                };
                                view! {
                                    <li>
                                        <button
                                            type="button"
                                            class:bg-blue-50=is_selected.clone()
                                            class:text-blue-700=is_selected
                                            class="w-full text-left px-3 py-2 rounded hover:bg-gray-50 text-sm"
                                            on:click=move |_| {
                                                set_selected_id.set(Some(id_for_select.clone()));
                                            }
                                        >
                                            <div class="font-medium">{t.name.clone()}</div>
                                            <div class="text-xs text-gray-500">
                                                {t.template_type.clone()} " · v" {t.version}
                                                {if !t.is_active { " · inactive" } else { "" }}
                                            </div>
                                        </button>
                                    </li>
                                }
                            }
                        />
                    </ul>
                </div>

                // ── Right: preview pane ─────────────────────────────────
                <div class="col-span-12 md:col-span-8 bg-white rounded-lg border p-3">
                    <div class="flex items-center gap-3 mb-3">
                        <label class="text-sm text-gray-600">"Format:"</label>
                        <select
                            class="border rounded px-2 py-1 text-sm"
                            prop:value=move || format.get()
                            on:change=move |ev| set_format.set(event_target_value(&ev))
                        >
                            {FORMATS.iter().map(|(value, label)| {
                                view! {
                                    <option value={*value}>{*label}</option>
                                }
                            }).collect_view()}
                        </select>
                        <button
                            type="button"
                            class="px-3 py-1 bg-blue-600 text-white rounded text-sm hover:bg-blue-700 disabled:opacity-50"
                            disabled=move || selected_id.get().is_none() || loading_preview.get()
                            on:click=move |_| {
                                if let Some(id) = selected_id.get() {
                                    trigger_preview(id);
                                }
                            }
                        >
                            {move || if loading_preview.get() {
                                "Merender..."
                            } else {
                                "Render Preview"
                            }}
                        </button>
                    </div>

                    {move || preview_url.get().map(|url| {
                        let mime = preview_blob_mime.get();
                        let can_inline = mime.starts_with("application/pdf")
                            || mime.starts_with("text/html");
                        if can_inline {
                            view! {
                                <iframe
                                    src=url.clone()
                                    class="w-full border rounded"
                                    style="height: 75vh"
                                />
                            }.into_any()
                        } else {
                            // Browsers don't render .docx / .xlsx inline.
                            view! {
                                <div class="p-6 border-2 border-dashed rounded text-center">
                                    <p class="text-sm text-gray-600 mb-2">
                                        "Format ini tidak bisa dipratinjau inline di browser."
                                    </p>
                                    <a
                                        href=url.clone()
                                        download=true
                                        class="inline-block px-4 py-2 bg-green-600 text-white rounded hover:bg-green-700"
                                    >
                                        "Unduh hasil preview"
                                    </a>
                                </div>
                            }.into_any()
                        }
                    })}

                    <Show when=move || selected_id.get().is_none()>
                        <p class="text-sm text-gray-400 text-center py-12">
                            "Pilih sebuah template untuk melihat pratinjau."
                        </p>
                    </Show>
                </div>
            </div>
        </div>
    }
}

fn mime_for(format: &str) -> &'static str {
    match format {
        "pdf" => "application/pdf",
        "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "xlsx" | "excel" => {
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
        }
        "html" => "text/html",
        _ => "application/octet-stream",
    }
}

fn make_blob_url(bytes: &[u8], mime: &str) -> Result<String, String> {
    // Wrap raw bytes in a `Uint8Array` so `Blob::new_with_u8_array_sequence_…`
    // sees them as a single chunk.
    let array = js_sys::Uint8Array::from(bytes);
    let parts = js_sys::Array::new();
    parts.push(&array);

    let opts = BlobPropertyBag::new();
    opts.set_type(mime);

    let blob = Blob::new_with_u8_array_sequence_and_options(&parts, &opts)
        .map_err(|e: JsValue| format!("Blob ctor: {e:?}"))?;
    Url::create_object_url_with_blob(&blob).map_err(|e: JsValue| format!("createObjectURL: {e:?}"))
}
