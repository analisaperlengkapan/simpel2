use leptos::prelude::*;

/// Column descriptor for `DataTable`. `header` is the column title,
/// `align` picks the body alignment, and `cell` builds the per-row view.
pub struct DataTableColumn<T> {
    pub header: &'static str,
    pub align: &'static str,
    pub cell: Box<dyn Fn(&T) -> AnyView + Send + Sync>,
}

impl<T> DataTableColumn<T> {
    pub fn new(header: &'static str, cell: impl Fn(&T) -> AnyView + Send + Sync + 'static) -> Self {
        Self {
            header,
            align: "text-left",
            cell: Box::new(cell),
        }
    }

    pub fn align(mut self, align: &'static str) -> Self {
        self.align = align;
        self
    }
}

/// Responsive data table with sticky header, zebra-striped rows, and a
/// consistent empty-state footer slot. Callers supply columns + row data;
/// the component takes care of styling.
#[component]
pub fn DataTable<T>(
    columns: Vec<DataTableColumn<T>>,
    rows: Vec<T>,
    #[prop(optional, into)] empty_message: Option<String>,
) -> impl IntoView
where
    T: 'static,
{
    let empty_label = empty_message.unwrap_or_else(|| "Tidak ada data.".to_string());
    let is_empty = rows.is_empty();
    let col_count = columns.len();

    let header_cells = columns
        .iter()
        .map(|col| {
            let align = col.align;
            let label = col.header;
            view! {
                <th class=format!("px-4 py-3 text-xs font-semibold uppercase tracking-wide text-slate-400 {}", align)
                    scope="col">
                    {label}
                </th>
            }
        })
        .collect_view();

    let body_rows = rows
        .iter()
        .enumerate()
        .map(|(idx, row)| {
            let cells = columns
                .iter()
                .map(|col| {
                    let align = col.align;
                    let content = (col.cell)(row);
                    view! {
                        <td class=format!("px-4 py-3 text-sm text-slate-200 {}", align)>
                            {content}
                        </td>
                    }
                })
                .collect_view();
            let bg = if idx % 2 == 0 {
                "bg-transparent"
            } else {
                "bg-white/[0.015]"
            };
            view! {
                <tr class=format!("border-b border-white/[0.04] {}", bg)>
                    {cells}
                </tr>
            }
        })
        .collect_view();

    view! {
        <div class="overflow-hidden rounded-2xl border border-white/[0.06] bg-surface-panel">
            <div class="overflow-x-auto">
                <table class="min-w-full divide-y divide-white/[0.04]">
                    <thead class="bg-white/[0.02]">
                        <tr>{header_cells}</tr>
                    </thead>
                    <tbody>
                        {body_rows}
                        {is_empty.then(|| view! {
                            <tr>
                                <td
                                    class="px-4 py-8 text-center text-sm text-slate-500"
                                    colspan=col_count.to_string()
                                >
                                    {empty_label}
                                </td>
                            </tr>
                        })}
                    </tbody>
                </table>
            </div>
        </div>
    }
}
