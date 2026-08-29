use leptos::prelude::*;

#[component]
// The `#[allow(unused_variables)]` here said `on_next` was merely "declared for
// a future wiring". It was in fact wired and then broken: a split `>=` turned
// this button's `on:click` into a text node, so "Selanjutnya" did nothing and
// the literal Rust source rendered on the page. The allow silenced the one
// warning that was pointing at it.
pub fn PaginationControls(
    #[prop(into)] current_page: Signal<i32>,
    #[prop(into)] total_pages: Signal<i32>,
    total_items: Option<i64>,
    on_prev: Callback<()>,
    on_next: Callback<()>,
) -> impl IntoView {
    view! {
        <div class="mt-6 flex items-center justify-between border-t border-white/[0.06] pt-4">
            <div class="text-xs text-slate-400">
                "Menampilkan halaman " <span class="font-medium text-slate-200">{move || current_page.get()}</span>
                " dari " <span class="font-medium text-slate-200">{move || total_pages.get()}</span>
                {total_items
                    .map(|total| {
                        view! { <>" (" <span class="font-medium text-slate-200">{total}</span> " total)"</> }
                    })}
            </div>
            <div class="flex gap-2">
                <button
                    class="rounded-lg border border-white/10 bg-white/[0.04] px-3 py-1 text-xs text-slate-300 transition hover:bg-white/[0.08] disabled:cursor-not-allowed disabled:opacity-40"
                    prop:disabled=move || current_page.get() <= 1
                    on:click=move |_| on_prev.run(())
                >
                    "Sebelumnya"
                </button>
                <button
                    class="rounded-lg border border-white/10 bg-white/[0.04] px-3 py-1 text-xs text-slate-300 transition hover:bg-white/[0.08] disabled:cursor-not-allowed disabled:opacity-40"
                    prop:disabled=move || { current_page.get() >= total_pages.get() }
                    on:click=move |_| on_next.run(())
                >
                    "Selanjutnya"
                </button>
            </div>
        </div>
    }
}
