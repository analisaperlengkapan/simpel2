use leptos::prelude::*;

#[component]
// `on_next` is declared for the Next action, wired in F5-C (#33); callers
// already pass it, so it can't be `_`-prefixed (Leptos PropsBuilder).
#[allow(unused_variables)]
pub fn PaginationControls(
    #[prop(into)] current_page: Signal<i32>,
    #[prop(into)] total_pages: Signal<i32>,
    total_items: Option<i64>,
    on_prev: Callback<()>,
    on_next: Callback<()>,
) -> impl IntoView {
    view! {
        <div class="flex items-center justify-between mt-6 pt-4 border-t border-gray-100">
            <div class="text-sm text-gray-500">
                "Menampilkan halaman " <span class="font-medium">{move || current_page.get()}</span>
                " dari " <span class="font-medium">{move || total_pages.get()}</span>
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
                    prop:disabled=move || current_page.get() <= 1
                    on:click=move |_| on_prev.run(())
                >
                    "Sebelumnya"
                </button>
                <button
                    class="px-3 py-1 border rounded hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed"
                    prop:disabled=move || current_page.get() >= total_pages.get()
                    on:click=move |_| on_next.run(())
                >
                    "Selanjutnya"
                </button>
            </div>
        </div>
    }
}
