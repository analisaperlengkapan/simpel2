//! Transition matrix — visual grid of allowed transitions: from-state × to-state.

use leptos::prelude::*;
use lib_ui::components::icon::AppIcon;
use phosphor_leptos::{CHECK, CIRCLE, FLAG_CHECKERED};

use crate::api::workflow::WorkflowStep;

#[component]
pub fn TransitionMatrix(steps: Vec<WorkflowStep>) -> impl IntoView {
    let state_names: Vec<String> = steps.iter().map(|s| s.state_name.clone()).collect();
    if state_names.is_empty() {
        return view! {
            <div class="rounded-xl border border-dashed border-white/[0.08] bg-white/[0.02] p-6 text-center text-sm text-slate-400">
                "Belum ada langkah pada workflow ini."
            </div>
        }
        .into_any();
    }

    let header_cells = state_names
        .iter()
        .map(|name| {
            view! {
                <th class="whitespace-nowrap border-b border-white/[0.06] px-3 py-2 text-left text-[0.65rem] font-semibold uppercase tracking-wider text-slate-400">
                    {name.clone()}
                </th>
            }
        })
        .collect::<Vec<_>>();

    let state_names_for_rows = state_names.clone();
    let rows = steps
        .into_iter()
        .map(|step| {
            let row_label = step.state_name.clone();
            let is_terminal = step.is_terminal;
            let next_states = step.next_states.clone();
            let cells = state_names_for_rows
                .iter()
                .map(|target| {
                    let allowed = next_states.iter().any(|n| n == target);
                    let cell_class = if allowed {
                        "border-b border-white/[0.04] px-3 py-2 text-center"
                    } else {
                        "border-b border-white/[0.04] px-3 py-2 text-center text-slate-600"
                    };
                    if allowed {
                        view! {
                            <td class=cell_class>
                                <span class="inline-flex h-6 w-6 items-center justify-center rounded-full bg-success-500/15 text-[0.7rem] text-success-400">
                                    <AppIcon icon=CHECK />
                                </span>
                            </td>
                        }
                        .into_any()
                    } else {
                        view! {
                            <td class=cell_class>
                                <span class="text-[0.7rem] text-slate-600">"—"</span>
                            </td>
                        }
                        .into_any()
                    }
                })
                .collect::<Vec<_>>();
            view! {
                <tr>
                    <th class="whitespace-nowrap border-b border-white/[0.04] bg-white/[0.02] px-3 py-2 text-left text-xs font-semibold text-slate-200">
                        <div class="flex items-center gap-2">
                            <span class=if is_terminal { "inline-flex text-danger-400" } else { "inline-flex text-info-400" }>
                                <AppIcon icon=if is_terminal { FLAG_CHECKERED } else { CIRCLE } size=10 />
                            </span>
                            {row_label}
                        </div>
                    </th>
                    {cells}
                </tr>
            }
        })
        .collect::<Vec<_>>();

    view! {
        <div class="overflow-x-auto rounded-xl border border-white/[0.06] bg-white/[0.02]">
            <table class="min-w-full border-collapse text-xs">
                <thead>
                    <tr>
                        <th class="whitespace-nowrap border-b border-white/[0.06] bg-white/[0.02] px-3 py-2 text-left text-[0.65rem] font-semibold uppercase tracking-wider text-slate-500">
                            "Dari ↓ / Ke →"
                        </th>
                        {header_cells}
                    </tr>
                </thead>
                <tbody>
                    {rows}
                </tbody>
            </table>
        </div>
    }
    .into_any()
}
