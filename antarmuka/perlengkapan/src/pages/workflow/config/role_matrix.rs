//! Role matrix — which role owns each state, with quick summary by role.

use leptos::prelude::*;
use lib_ui::components::icon::AppIcon;
use phosphor_leptos::{USER_LIST};
use std::collections::BTreeMap;

use crate::api::workflow::WorkflowStep;

#[component]
pub fn RoleMatrix(steps: Vec<WorkflowStep>) -> impl IntoView {
    if steps.is_empty() {
        return view! {
            <div class="rounded-xl border border-dashed border-white/[0.08] bg-white/[0.02] p-6 text-center text-sm text-slate-400">
                "Belum ada langkah pada workflow ini."
            </div>
        }
        .into_any();
    }

    let mut groups: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for step in &steps {
        let role = step
            .required_role
            .clone()
            .unwrap_or_else(|| "(tanpa role)".to_string());
        groups
            .entry(role)
            .or_default()
            .push(step.state_name.clone());
    }

    let rows = steps
        .into_iter()
        .map(|step| {
            let role = step
                .required_role
                .clone()
                .unwrap_or_else(|| "—".to_string());
            let has_role = step.required_role.is_some();
            let state_label = step.state_name.clone();
            view! {
                <tr class="border-b border-white/[0.04]">
                    <td class="px-3 py-2 text-xs font-semibold text-slate-200">{state_label}</td>
                    <td class="px-3 py-2">
                        <span class=if has_role {
                            "inline-flex items-center gap-1.5 rounded-lg border border-info-500/30 bg-info-500/10 px-2.5 py-1 text-[0.7rem] font-semibold text-info-300"
                        } else {
                            "inline-flex items-center gap-1.5 rounded-lg border border-white/[0.06] bg-white/[0.02] px-2.5 py-1 text-[0.7rem] font-semibold text-slate-500"
                        }>
                            <span class="text-[0.6rem]"><AppIcon icon=USER_LIST /></span>
                            {role}
                        </span>
                    </td>
                </tr>
            }
        })
        .collect::<Vec<_>>();

    let summary = groups
        .into_iter()
        .map(|(role, states)| {
            view! {
                <div class="rounded-xl border border-white/[0.06] bg-white/[0.02] p-3">
                    <div class="mb-1 text-[0.65rem] font-semibold uppercase tracking-wider text-slate-500">
                        {role}
                    </div>
                    <div class="text-xs text-slate-200">
                        {format!("{} langkah — {}", states.len(), states.join(", "))}
                    </div>
                </div>
            }
        })
        .collect::<Vec<_>>();

    view! {
        <div class="flex flex-col gap-4">
            <div class="overflow-x-auto rounded-xl border border-white/[0.06] bg-white/[0.02]">
                <table class="min-w-full border-collapse">
                    <thead>
                        <tr>
                            <th class="border-b border-white/[0.06] px-3 py-2 text-left text-[0.65rem] font-semibold uppercase tracking-wider text-slate-400">
                                "State"
                            </th>
                            <th class="border-b border-white/[0.06] px-3 py-2 text-left text-[0.65rem] font-semibold uppercase tracking-wider text-slate-400">
                                "Required Role"
                            </th>
                        </tr>
                    </thead>
                    <tbody>{rows}</tbody>
                </table>
            </div>

            <div>
                <div class="mb-2 text-xs font-semibold text-slate-300">"Rangkuman per Role"</div>
                <div class="grid gap-2 sm:grid-cols-2">{summary}</div>
            </div>
        </div>
    }
    .into_any()
}
