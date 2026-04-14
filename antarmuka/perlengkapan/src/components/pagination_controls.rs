use leptos::prelude::*;

#[component]
pub fn PaginationControls(
    current_page: i32,
    total_pages: i32,
    total_items: Option<i64>,
    on_prev: Callback<()>,
    on_next: Callback<()>,
) -> impl IntoView {
    let prev_disabled = current_page <= 1;
    let next_disabled = current_page >= total_pages;

    view! {
        <div class="flex items-center justify-between mt-6 pt-4 border-t border-gray-100">
            <div class="text-sm text-gray-500">
                "Menampilkan halaman " <span class="font-medium">{current_page}</span>
                " dari " <span class="font-medium">{total_pages}</span>
                {total_items.map(|total| {
                    view! {
                        <>
                            " (" <span class="font-medium">{total}</span> " total)"
                        </>
                    }
                })}
            </div>
            <div class="flex gap-2">
                <button
                    class="px-3 py-1 border rounded hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed"
                    disabled=prev_disabled
                    on:click=move |_| on_prev.run(())
                >
                    "Sebelumnya"
                </button>
                <button
                    class="px-3 py-1 border rounded hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed"
                    disabled=next_disabled
                    on:click=move |_| on_next.run(())
                >
                    "Selanjutnya"
                </button>
            </div>
        </div>
    }
}
