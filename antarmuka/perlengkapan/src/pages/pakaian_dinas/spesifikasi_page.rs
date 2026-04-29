//! Spesifikasi page — lists specifications for a given jenis pakaian dinas.
//!
//! Replaces the placeholder route `/pakaian-dinas/jenis/:id/spesifikasi`.

use leptos::prelude::*;
use leptos_fetch::QueryClient;
use leptos_router::hooks::use_params_map;

use crate::api::PaginatedResponse;
use crate::api::pakaian_dinas::{SpesifikasiPakaianDinas, fetch_spesifikasi_pakaian};
use crate::components::layout::{
    DataTable, DataTableColumn, EmptyState, ErrorState, LoadingState, PageLayout, SectionCard,
};

/// Spesifikasi query keyed by jenis id (`String`). Each jenis
/// caches independently so navigating between jenis pages keeps
/// already-loaded specs warm.
async fn query_spesifikasi(
    jenis_id: String,
) -> Result<PaginatedResponse<SpesifikasiPakaianDinas>, crate::api::AppError> {
    fetch_spesifikasi_pakaian(1, 100, Some(jenis_id)).await
}

/// Render the list of spesifikasi for a single jenis pakaian dinas.
fn render_spesifikasi_table(items: Vec<SpesifikasiPakaianDinas>) -> AnyView {
    let jenis_nama = items
        .first()
        .and_then(|s| s.jenis_pakaian_nama.clone())
        .unwrap_or_else(|| "Pakaian Dinas".to_string());

    let columns: Vec<DataTableColumn<SpesifikasiPakaianDinas>> = vec![
        DataTableColumn::new("Nama", |item: &SpesifikasiPakaianDinas| {
            let nama = item.nama.clone();
            view! { <span class="font-medium text-slate-100">{nama}</span> }.into_any()
        }),
        DataTableColumn::new("Keterangan", |item: &SpesifikasiPakaianDinas| {
            let text = item.keterangan.clone().unwrap_or_else(|| "-".to_string());
            view! { <span class="text-slate-400">{text}</span> }.into_any()
        }),
        DataTableColumn::new("Foto", |item: &SpesifikasiPakaianDinas| match &item.foto {
            Some(url) if !url.is_empty() => {
                let src = url.clone();
                view! {
                    <img src=src alt="Foto spesifikasi" class="h-10 w-10 rounded-lg object-cover" />
                }
                .into_any()
            }
            _ => view! {
                <span class="text-slate-500 text-xs">"Tidak ada"</span>
            }
            .into_any(),
        }),
        DataTableColumn::new("Tanggal Dibuat", |item: &SpesifikasiPakaianDinas| {
            let date = item.created_at.chars().take(10).collect::<String>();
            view! { <span class="text-xs text-slate-400">{date}</span> }.into_any()
        }),
    ];

    view! {
        <SectionCard title=jenis_nama>
            <DataTable columns=columns rows=items />
        </SectionCard>
    }
    .into_any()
}

#[component]
pub fn SpesifikasiPage() -> impl IntoView {
    let params = use_params_map();
    let jenis_id = move || params.with(|p| p.get("id").unwrap_or_default().to_string());

    let client: QueryClient = expect_context();
    let data = client.local_resource(query_spesifikasi, move || jenis_id());

    let content = move || -> AnyView {
        let result = data.get();
        match result {
            None => view! { <LoadingState /> }.into_any(),
            Some(Err(e)) => view! { <ErrorState error=e /> }.into_any(),
            Some(Ok(response)) => {
                let items = response.data;
                if items.is_empty() {
                    view! {
                        <EmptyState
                            icon="fas fa-list"
                            title="Belum Ada Spesifikasi"
                            description="Belum ada spesifikasi untuk jenis pakaian dinas ini."
                        />
                    }
                    .into_any()
                } else {
                    render_spesifikasi_table(items)
                }
            }
        }
    };

    view! {
        <PageLayout
            title="Spesifikasi Pakaian Dinas"
            icon="fas fa-list"
            description="Daftar spesifikasi untuk jenis pakaian dinas yang dipilih."
        >
            <Suspense fallback=move || view! { <LoadingState /> }>
                {content}
            </Suspense>
        </PageLayout>
    }
}
