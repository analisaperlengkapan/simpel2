//! Workflow Configuration Management Page
//!
//! CRUD UI for Admin Pusat to manage workflow definitions and their steps.

use crate::api::workflow::{
    WorkflowDefinition, WorkflowDefinitionDetail, WorkflowStep, fetch_workflow_definition_detail,
    fetch_workflow_definitions, format_sla, create_workflow_definition, update_workflow_definition,
    delete_workflow_definition, upsert_workflow_step, delete_workflow_step,
    CreateWorkflowRequest, UpdateWorkflowRequest, UpsertStepRequest,
};
use leptos::prelude::*;
use leptos::task::spawn_local;

// ═══════════════════════════════════════════════════════════════════════════
// Workflow Edit Form Modal Component
// ═══════════════════════════════════════════════════════════════════════════

#[component]
fn WorkflowEditModal(
    workflow: Option<WorkflowDefinition>,
    #[prop(into)] on_close: Callback<()>,
    #[prop(into)] on_save: Callback<()>,
) -> impl IntoView {
    let is_create = workflow.is_none();
    let title = if is_create { "Buat Workflow Baru" } else { "Edit Workflow" };

    let (name, set_name) = signal(workflow.as_ref().map(|w| w.name.clone()).unwrap_or_default());
    let (description, set_description) = signal(workflow.as_ref().map(|w| w.description.clone()).unwrap_or_default());
    let (version, set_version) = signal(workflow.as_ref().map(|w| w.version.clone()).unwrap_or_else(|| "1.0".to_string()));
    let (status, set_status) = signal(workflow.as_ref().map(|w| w.status.clone()).unwrap_or_else(|| "draft".to_string()));
    let (parallel_approval, set_parallel_approval) = signal(workflow.as_ref().map(|w| w.supports_parallel_approval).unwrap_or(false));
    let (saving, set_saving) = signal(false);
    let (error, set_error) = signal::<Option<String>>(None);

    let on_submit = move |_| {
        set_saving.set(true);
        set_error.set(None);

        let name_val = name.get();
        let desc_val = description.get();
        let ver_val = version.get();
        let stat_val = status.get();
        let parallel_val = parallel_approval.get();

        spawn_local(async move {
            let result = if is_create {
                create_workflow_definition(CreateWorkflowRequest {
                    name: name_val,
                    description: desc_val,
                    version: ver_val,
                    status: stat_val,
                    supports_parallel_approval: parallel_val,
                }).await
            } else {
                update_workflow_definition(&name_val, UpdateWorkflowRequest {
                    description: Some(desc_val),
                    version: Some(ver_val),
                    status: Some(stat_val),
                    supports_parallel_approval: Some(parallel_val),
                }).await
            };

            set_saving.set(false);

            match result {
                Ok(_) => on_save.run(()),
                Err(e) => set_error.set(Some(e.to_string())),
            }
        });
    };

    view! {
        <div
            style="position: fixed; inset: 0; background: rgba(0,0,0,0.7); backdrop-filter: blur(4px); z-index: 1000; display: flex; align-items: center; justify-content: center; padding: 20px;"
            on:click=move |e| {
                if e.target() == e.current_target() {
                    on_close.run(());
                }
            }
        >
            <div style="background: linear-gradient(180deg, #0f172a 0%, #1e293b 100%); border: 1px solid rgba(255,255,255,0.1); border-radius: 20px; max-width: 600px; width: 100%; max-height: 90vh; overflow: hidden; display: flex; flex-direction: column;">
                // Header
                <div style="padding: 24px; border-bottom: 1px solid rgba(255,255,255,0.08);">
                    <div style="display: flex; justify-content: space-between; align-items: center;">
                        <div>
                            <h2 style="font-size: 1.3rem; font-weight: 800; color: #e2e8f0; margin: 0 0 6px 0;">
                                {title}
                            </h2>
                            <p style="font-size: 0.8rem; color: #94a3b8; margin: 0;">
                                {if is_create { "Buat workflow definition baru" } else { "Edit workflow definition" }}
                            </p>
                        </div>
                        <button
                            on:click=move |_| on_close.run(())
                            style="width: 36px; height: 36px; display: flex; align-items: center; justify-content: center; background: rgba(255,255,255,0.04); border: 1px solid rgba(255,255,255,0.08); border-radius: 10px; color: #94a3b8; cursor: pointer; transition: all 0.2s;"
                            class="hover:bg-white/10"
                        >
                            <i class="fas fa-times"></i>
                        </button>
                    </div>
                </div>

                // Content
                <div style="flex: 1; overflow-y: auto; padding: 24px;">
                    {move || error.get().map(|e| view! {
                        <div style="background: rgba(248,113,113,0.1); border: 1px solid rgba(248,113,113,0.3); border-radius: 12px; padding: 12px 16px; margin-bottom: 20px; display: flex; align-items: start; gap: 12px;">
                            <i class="fas fa-exclamation-triangle" style="color: #f87171; font-size: 1rem; margin-top: 2px;"></i>
                            <div style="flex: 1;">
                                <div style="font-size: 0.85rem; font-weight: 600; color: #f87171; margin-bottom: 4px;">
                                    "Error"
                                </div>
                                <div style="font-size: 0.8rem; color: #fca5a5;">
                                    {e.to_string()}
                                </div>
                            </div>
                        </div>
                    })}

                    <div style="display: flex; flex-direction: column; gap: 20px;">
                        // Name field
                        <div>
                            <label style="display: block; font-size: 0.85rem; font-weight: 600; color: #e2e8f0; margin-bottom: 8px;">
                                "Nama Workflow" <span style="color: #f87171;">"*"</span>
                            </label>
                            <input
                                type="text"
                                prop:value=move || name.get()
                                on:input=move |e| set_name.set(event_target_value(&e))
                                prop:disabled=move || !is_create || saving.get()
                                placeholder="kebutuhan_bmn"
                                style="width: 100%; padding: 12px; background: rgba(255,255,255,0.04); border: 1px solid rgba(255,255,255,0.1); border-radius: 10px; color: #e2e8f0; font-size: 0.85rem;"
                            />
                            <p style="font-size: 0.75rem; color: #64748b; margin: 6px 0 0 0;">
                                "Nama unik untuk workflow (tidak bisa diubah setelah dibuat)"
                            </p>
                        </div>

                        // Description field
                        <div>
                            <label style="display: block; font-size: 0.85rem; font-weight: 600; color: #e2e8f0; margin-bottom: 8px;">
                                "Deskripsi" <span style="color: #f87171;">"*"</span>
                            </label>
                            <textarea
                                prop:value=move || description.get()
                                on:input=move |e| set_description.set(event_target_value(&e))
                                prop:disabled=move || saving.get()
                                placeholder="Workflow untuk proses persetujuan kebutuhan BMN"
                                rows="3"
                                style="width: 100%; padding: 12px; background: rgba(255,255,255,0.04); border: 1px solid rgba(255,255,255,0.1); border-radius: 10px; color: #e2e8f0; font-size: 0.85rem; resize: vertical;"
                            ></textarea>
                        </div>

                        // Version field
                        <div>
                            <label style="display: block; font-size: 0.85rem; font-weight: 600; color: #e2e8f0; margin-bottom: 8px;">
                                "Versi" <span style="color: #f87171;">"*"</span>
                            </label>
                            <input
                                type="text"
                                prop:value=move || version.get()
                                on:input=move |e| set_version.set(event_target_value(&e))
                                prop:disabled=move || saving.get()
                                placeholder="1.0"
                                style="width: 100%; padding: 12px; background: rgba(255,255,255,0.04); border: 1px solid rgba(255,255,255,0.1); border-radius: 10px; color: #e2e8f0; font-size: 0.85rem;"
                            />
                        </div>

                        // Status field
                        <div>
                            <label style="display: block; font-size: 0.85rem; font-weight: 600; color: #e2e8f0; margin-bottom: 8px;">
                                "Status" <span style="color: #f87171;">"*"</span>
                            </label>
                            <select
                                prop:value=move || status.get()
                                on:change=move |e| set_status.set(event_target_value(&e))
                                prop:disabled=move || saving.get()
                                style="width: 100%; padding: 12px; background: rgba(255,255,255,0.04); border: 1px solid rgba(255,255,255,0.1); border-radius: 10px; color: #e2e8f0; font-size: 0.85rem;"
                            >
                                <option value="draft">"Draft"</option>
                                <option value="active">"Active"</option>
                                <option value="inactive">"Inactive"</option>
                            </select>
                        </div>

                        // Parallel approval checkbox
                        <div>
                            <label style="display: flex; align-items: center; gap: 10px; cursor: pointer;">
                                <input
                                    type="checkbox"
                                    prop:checked=move || parallel_approval.get()
                                    on:change=move |e| set_parallel_approval.set(event_target_checked(&e))
                                    prop:disabled=move || saving.get()
                                    style="width: 18px; height: 18px; cursor: pointer;"
                                />
                                <span style="font-size: 0.85rem; font-weight: 600; color: #e2e8f0;">
                                    "Dukungan Parallel Approval"
                                </span>
                            </label>
                            <p style="font-size: 0.75rem; color: #64748b; margin: 6px 0 0 28px;">
                                "Izinkan beberapa approver untuk menyetujui secara bersamaan"
                            </p>
                        </div>
                    </div>
                </div>

                // Footer
                <div style="padding: 20px 24px; border-top: 1px solid rgba(255,255,255,0.08); display: flex; justify-content: flex-end; gap: 12px;">
                    <button
                        on:click=move |_| on_close.run(())
                        prop:disabled=move || saving.get()
                        style="padding: 10px 20px; background: rgba(255,255,255,0.06); border: 1px solid rgba(255,255,255,0.1); border-radius: 10px; color: #e2e8f0; font-size: 0.85rem; font-weight: 600; cursor: pointer; transition: all 0.2s;"
                        class="hover:bg-white/10"
                    >
                        "Batal"
                    </button>
                    <button
                        on:click=on_submit
                        prop:disabled=move || saving.get() || name.get().is_empty() || description.get().is_empty()
                        style="padding: 10px 20px; background: linear-gradient(135deg, #60a5fa, #3b82f6); border: none; border-radius: 10px; color: white; font-size: 0.85rem; font-weight: 600; cursor: pointer; transition: all 0.2s;"
                        class="hover:opacity-90"
                    >
                        {move || if saving.get() {
                            view! { <><i class="fas fa-spinner fa-spin" style="margin-right: 6px;"></i>"Menyimpan..."</> }.into_any()
                        } else {
                            view! { <><i class="fas fa-save" style="margin-right: 6px;"></i>"Simpan"</> }.into_any()
                        }}
                    </button>
                </div>
            </div>
        </div>
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Delete Confirmation Modal Component
// ═══════════════════════════════════════════════════════════════════════════

#[component]
fn DeleteConfirmModal(
    workflow_name: String,
    #[prop(into)] on_close: Callback<()>,
    #[prop(into)] on_confirm: Callback<()>,
) -> impl IntoView {
    let (deleting, set_deleting) = signal(false);
    let (error, set_error) = signal::<Option<String>>(None);

    let name = workflow_name.clone();
    let on_delete = move |_| {
        set_deleting.set(true);
        set_error.set(None);

        let name_val = name.clone();
        spawn_local(async move {
            match delete_workflow_definition(&name_val).await {
                Ok(_) => on_confirm.run(()),
                Err(e) => {
                    set_error.set(Some(e.to_string()));
                    set_deleting.set(false);
                }
            }
        });
    };

    view! {
        <div
            style="position: fixed; inset: 0; background: rgba(0,0,0,0.7); backdrop-filter: blur(4px); z-index: 1000; display: flex; align-items: center; justify-content: center; padding: 20px;"
            on:click=move |e| {
                if e.target() == e.current_target() {
                    on_close.run(());
                }
            }
        >
            <div style="background: linear-gradient(180deg, #0f172a 0%, #1e293b 100%); border: 1px solid rgba(255,255,255,0.1); border-radius: 20px; max-width: 500px; width: 100%; padding: 24px;">
                <div style="text-align: center; margin-bottom: 20px;">
                    <div style="width: 64px; height: 64px; margin: 0 auto 16px; display: flex; align-items: center; justify-content: center; background: rgba(248,113,113,0.1); border: 2px solid rgba(248,113,113,0.3); border-radius: 50%;">
                        <i class="fas fa-exclamation-triangle" style="font-size: 1.8rem; color: #f87171;"></i>
                    </div>
                    <h2 style="font-size: 1.2rem; font-weight: 800; color: #e2e8f0; margin: 0 0 8px 0;">
                        "Hapus Workflow?"
                    </h2>
                    <p style="font-size: 0.85rem; color: #94a3b8; margin: 0;">
                        "Anda yakin ingin menghapus workflow "
                        <strong style="color: #e2e8f0;">{workflow_name.clone()}</strong>
                        "? Tindakan ini tidak dapat dibatalkan."
                    </p>
                </div>

                {move || error.get().map(|e| view! {
                    <div style="background: rgba(248,113,113,0.1); border: 1px solid rgba(248,113,113,0.3); border-radius: 12px; padding: 12px 16px; margin-bottom: 20px; display: flex; align-items: start; gap: 12px;">
                        <i class="fas fa-exclamation-triangle" style="color: #f87171; font-size: 1rem; margin-top: 2px;"></i>
                        <div style="flex: 1;">
                            <div style="font-size: 0.85rem; font-weight: 600; color: #f87171; margin-bottom: 4px;">
                                "Error"
                            </div>
                            <div style="font-size: 0.8rem; color: #fca5a5;">
                                {e.to_string()}
                            </div>
                        </div>
                    </div>
                })}

                <div style="display: flex; gap: 12px;">
                    <button
                        on:click=move |_| on_close.run(())
                        prop:disabled=move || deleting.get()
                        style="flex: 1; padding: 12px; background: rgba(255,255,255,0.06); border: 1px solid rgba(255,255,255,0.1); border-radius: 10px; color: #e2e8f0; font-size: 0.85rem; font-weight: 600; cursor: pointer; transition: all 0.2s;"
                        class="hover:bg-white/10"
                    >
                        "Batal"
                    </button>
                    <button
                        on:click=on_delete
                        prop:disabled=move || deleting.get()
                        style="flex: 1; padding: 12px; background: linear-gradient(135deg, #f87171, #ef4444); border: none; border-radius: 10px; color: white; font-size: 0.85rem; font-weight: 600; cursor: pointer; transition: all 0.2s;"
                        class="hover:opacity-90"
                    >
                        {move || if deleting.get() {
                            view! { <><i class="fas fa-spinner fa-spin" style="margin-right: 6px;"></i>"Menghapus..."</> }.into_any()
                        } else {
                            view! { <><i class="fas fa-trash" style="margin-right: 6px;"></i>"Hapus"</> }.into_any()
                        }}
                    </button>
                </div>
            </div>
        </div>
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Workflow Card Component
// ═══════════════════════════════════════════════════════════════════════════

#[component]
fn WorkflowCard(
    workflow: WorkflowDefinition,
    #[prop(into)] on_view_details: Callback<String>,
    #[prop(into)] on_edit: Callback<String>,
    #[prop(into)] on_delete: Callback<String>,
) -> impl IntoView {
    let name = workflow.name.clone();
    let name_for_edit = workflow.name.clone();
    let name_for_delete = workflow.name.clone();
    let status_color = match workflow.status.as_str() {
        "active" => "#22c55e",
        "draft" => "#fb923c",
        _ => "#64748b",
    };

    let status_label = match workflow.status.as_str() {
        "active" => "Aktif",
        "draft" => "Draft",
        _ => "Tidak Aktif",
    };

    view! {
        <div style="background: rgba(255,255,255,0.04); border: 1px solid rgba(255,255,255,0.08); border-radius: 16px; padding: 20px; transition: all 0.2s;">
            <div style="display: flex; justify-content: space-between; align-items: start; margin-bottom: 16px;">
                <div style="flex: 1;">
                    <div style="display: flex; align-items: center; gap: 10px; margin-bottom: 8px;">
                        <h3 style="font-size: 1rem; font-weight: 700; color: #e2e8f0; margin: 0;">
                            {workflow.name.clone()}
                        </h3>
                        <span style=format!(
                            "display: inline-flex; align-items: center; gap: 4px; padding: 3px 10px; background: {}15; border: 1px solid {}40; border-radius: 999px; font-size: 0.7rem; font-weight: 600; color: {};",
                            status_color, status_color, status_color
                        )>
                            <span style=format!("width: 6px; height: 6px; background: {}; border-radius: 50%;", status_color)></span>
                            {status_label}
                        </span>
                    </div>
                    <p style="font-size: 0.8rem; color: #94a3b8; margin: 0 0 12px 0; line-height: 1.4;">
                        {workflow.description.clone()}
                    </p>
                    <div style="display: flex; flex-wrap: wrap; gap: 8px;">
                        <div style="display: flex; align-items: center; gap: 6px; padding: 4px 10px; background: rgba(255,255,255,0.04); border: 1px solid rgba(255,255,255,0.06); border-radius: 8px; font-size: 0.7rem; color: #94a3b8;">
                            <i class="fas fa-code-branch" style="color: #60a5fa; font-size: 0.65rem;"></i>
                            "v" {workflow.version.clone()}
                        </div>
                        {workflow.supports_parallel_approval.then(|| view! {
                            <div style="display: flex; align-items: center; gap: 6px; padding: 4px 10px; background: rgba(255,255,255,0.04); border: 1px solid rgba(255,255,255,0.06); border-radius: 8px; font-size: 0.7rem; color: #94a3b8;">
                                <i class="fas fa-users" style="color: #34d399; font-size: 0.65rem;"></i>
                                "Parallel Approval"
                            </div>
                        })}
                    </div>
                </div>
            </div>
            <div style="display: flex; gap: 8px;">
                <button
                    on:click=move |_| on_view_details.run(name.clone())
                    style="flex: 1; padding: 10px; background: rgba(96,165,250,0.1); border: 1px solid rgba(96,165,250,0.3); border-radius: 10px; color: #60a5fa; font-size: 0.8rem; font-weight: 600; cursor: pointer; transition: all 0.2s;"
                    class="hover:bg-blue-500/20"
                >
                    <i class="fas fa-eye" style="margin-right: 6px;"></i>
                    "Lihat"
                </button>
                <button
                    on:click=move |_| on_edit.run(name_for_edit.clone())
                    style="flex: 1; padding: 10px; background: rgba(251,146,60,0.1); border: 1px solid rgba(251,146,60,0.3); border-radius: 10px; color: #fb923c; font-size: 0.8rem; font-weight: 600; cursor: pointer; transition: all 0.2s;"
                    class="hover:bg-orange-500/20"
                >
                    <i class="fas fa-edit" style="margin-right: 6px;"></i>
                    "Edit"
                </button>
                <button
                    on:click=move |_| on_delete.run(name_for_delete.clone())
                    style="padding: 10px 14px; background: rgba(248,113,113,0.1); border: 1px solid rgba(248,113,113,0.3); border-radius: 10px; color: #f87171; font-size: 0.8rem; font-weight: 600; cursor: pointer; transition: all 0.2s;"
                    class="hover:bg-red-500/20"
                >
                    <i class="fas fa-trash"></i>
                </button>
            </div>
        </div>
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// SLA Configuration Modal Component
// ═══════════════════════════════════════════════════════════════════════════

/// Time unit for SLA configuration
#[derive(Debug, Clone, Copy, PartialEq)]
enum TimeUnit {
    Minutes,
    Hours,
    Days,
}

impl TimeUnit {
    fn to_string(self) -> &'static str {
        match self {
            TimeUnit::Minutes => "minutes",
            TimeUnit::Hours => "hours",
            TimeUnit::Days => "days",
        }
    }

    fn label(self) -> &'static str {
        match self {
            TimeUnit::Minutes => "Menit",
            TimeUnit::Hours => "Jam",
            TimeUnit::Days => "Hari",
        }
    }

    /// Convert value in this unit to minutes
    fn to_minutes(self, value: u32) -> u32 {
        match self {
            TimeUnit::Minutes => value,
            TimeUnit::Hours => value * 60,
            TimeUnit::Days => value * 1440,
        }
    }

    /// Convert minutes to this unit (returns None if not evenly divisible)
    fn from_minutes(minutes: u32) -> Option<(u32, Self)> {
        if minutes.is_multiple_of(1440) {
            Some((minutes / 1440, TimeUnit::Days))
        } else if minutes.is_multiple_of(60) {
            Some((minutes / 60, TimeUnit::Hours))
        } else {
            Some((minutes, TimeUnit::Minutes))
        }
    }
}

/// Determine best time unit for displaying minutes
fn best_time_unit(minutes: u32) -> (u32, TimeUnit) {
    if minutes.is_multiple_of(1440) {
        (minutes / 1440, TimeUnit::Days)
    } else if minutes.is_multiple_of(60) {
        (minutes / 60, TimeUnit::Hours)
    } else {
        (minutes, TimeUnit::Minutes)
    }
}

#[component]
fn SlaConfigModal(
    workflow_name: String,
    step: WorkflowStep,
    #[prop(into)] on_close: Callback<()>,
    #[prop(into)] on_save: Callback<()>,
) -> impl IntoView {
    // Clone values needed in closures
    let step_state_name = step.state_name.clone();

    // Initialize SLA values
    let (sla_enabled, set_sla_enabled) = signal(step.sla_minutes.is_some());
    let initial_unit_value = step.sla_minutes.map(best_time_unit).unwrap_or((1, TimeUnit::Hours));
    let (sla_value, set_sla_value) = signal(initial_unit_value.0);
    let (time_unit, set_time_unit) = signal(initial_unit_value.1);
    let (escalation_enabled, set_escalation_enabled) = signal(step.escalation_enabled);
    let (saving, set_saving) = signal(false);
    let (error, set_error) = signal::<Option<String>>(None);

    // Calculate SLA in minutes for preview
    let sla_minutes = move || {
        if sla_enabled.get() {
            Some(time_unit.get().to_minutes(sla_value.get()))
        } else {
            None
        }
    };

    let on_submit = move |_| {
        set_saving.set(true);
        set_error.set(None);

        let workflow = workflow_name.clone();
        let request = UpsertStepRequest {
            state_name: step.state_name.clone(),
            state_code: step.state_code,
            required_role: step.required_role.clone(),
            sla_minutes: sla_minutes(),
            next_states: step.next_states.clone(),
            escalation_enabled: escalation_enabled.get(),
        };

        spawn_local(async move {
            match upsert_workflow_step(&workflow, request).await {
                Ok(_) => on_save.run(()),
                Err(e) => {
                    set_error.set(Some(e.to_string()));
                    set_saving.set(false);
                }
            }
        });
    };

    view! {
        <div
            style="position: fixed; inset: 0; background: rgba(0,0,0,0.7); backdrop-filter: blur(4px); z-index: 1000; display: flex; align-items: center; justify-content: center; padding: 20px;"
            on:click=move |e| {
                if e.target() == e.current_target() {
                    on_close.run(());
                }
            }
        >
            <div style="background: linear-gradient(180deg, #0f172a 0%, #1e293b 100%); border: 1px solid rgba(255,255,255,0.1); border-radius: 20px; max-width: 600px; width: 100%; max-height: 90vh; overflow: hidden; display: flex; flex-direction: column;">
                // Header
                <div style="padding: 24px; border-bottom: 1px solid rgba(255,255,255,0.08);">
                    <div style="display: flex; justify-content: space-between; align-items: center;">
                        <div>
                            <h2 style="font-size: 1.3rem; font-weight: 800; color: #e2e8f0; margin: 0 0 6px 0;">
                                "Konfigurasi SLA"
                            </h2>
                            <p style="font-size: 0.8rem; color: #94a3b8; margin: 0;">
                                "Atur batas waktu dan eskalasi untuk " <strong>{step_state_name.clone()}</strong>
                            </p>
                        </div>
                        <button
                            on:click=move |_| on_close.run(())
                            style="width: 36px; height: 36px; display: flex; align-items: center; justify-content: center; background: rgba(255,255,255,0.04); border: 1px solid rgba(255,255,255,0.08); border-radius: 10px; color: #94a3b8; cursor: pointer; transition: all 0.2s;"
                            class="hover:bg-white/10"
                        >
                            <i class="fas fa-times"></i>
                        </button>
                    </div>
                </div>

                // Content
                <div style="flex: 1; overflow-y: auto; padding: 24px;">
                    {move || error.get().map(|e| view! {
                        <div style="background: rgba(248,113,113,0.1); border: 1px solid rgba(248,113,113,0.3); border-radius: 12px; padding: 12px 16px; margin-bottom: 20px; display: flex; align-items: start; gap: 12px;">
                            <i class="fas fa-exclamation-triangle" style="color: #f87171; font-size: 1rem; margin-top: 2px;"></i>
                            <div style="flex: 1;">
                                <div style="font-size: 0.85rem; font-weight: 600; color: #f87171; margin-bottom: 4px;">
                                    "Error"
                                </div>
                                <div style="font-size: 0.8rem; color: #fca5a5;">
                                    {e.to_string()}
                                </div>
                            </div>
                        </div>
                    })}

                    <div style="display: flex; flex-direction: column; gap: 24px;">
                        // Enable SLA Toggle
                        <div style="background: rgba(255,255,255,0.04); border: 1px solid rgba(255,255,255,0.08); border-radius: 12px; padding: 16px;">
                            <label style="display: flex; align-items: center; gap: 12px; cursor: pointer;">
                                <input
                                    type="checkbox"
                                    prop:checked=move || sla_enabled.get()
                                    on:change=move |e| set_sla_enabled.set(event_target_checked(&e))
                                    prop:disabled=move || saving.get()
                                    style="width: 20px; height: 20px; cursor: pointer;"
                                />
                                <div style="flex: 1;">
                                    <div style="font-size: 0.9rem; font-weight: 600; color: #e2e8f0;">
                                        "Aktifkan SLA Tracking"
                                    </div>
                                    <div style="font-size: 0.75rem; color: #64748b; margin-top: 4px;">
                                        "Pantau dan eskalasi jika langkah ini melebihi batas waktu"
                                    </div>
                                </div>
                            </label>
                        </div>

                        // SLA Time Configuration
                        {move || sla_enabled.get().then(|| view! {
                            <div>
                                <label style="display: block; font-size: 0.85rem; font-weight: 600; color: #e2e8f0; margin-bottom: 8px;">
                                    "Batas Waktu SLA" <span style="color: #f87171;">"*"</span>
                                </label>
                                <div style="display: flex; gap: 12px;">
                                    <input
                                        type="number"
                                        prop:value=move || sla_value.get()
                                        on:input=move |e| {
                                            if let Ok(val) = event_target_value(&e).parse::<u32>() {
                                                set_sla_value.set(val);
                                            }
                                        }
                                        prop:disabled=move || saving.get()
                                        min="1"
                                        placeholder="1"
                                        style="flex: 1; padding: 12px; background: rgba(255,255,255,0.04); border: 1px solid rgba(255,255,255,0.1); border-radius: 10px; color: #e2e8f0; font-size: 0.85rem;"
                                    />
                                    <select
                                        prop:value=move || time_unit.get().to_string()
                                        on:change=move |e| {
                                            let val = event_target_value(&e);
                                            let unit = match val.as_str() {
                                                "hours" => TimeUnit::Hours,
                                                "days" => TimeUnit::Days,
                                                _ => TimeUnit::Minutes,
                                            };
                                            set_time_unit.set(unit);
                                        }
                                        prop:disabled=move || saving.get()
                                        style="padding: 12px; background: rgba(255,255,255,0.04); border: 1px solid rgba(255,255,255,0.1); border-radius: 10px; color: #e2e8f0; font-size: 0.85rem; min-width: 120px;"
                                    >
                                        <option value="minutes">"Menit"</option>
                                        <option value="hours">"Jam"</option>
                                        <option value="days">"Hari"</option>
                                    </select>
                                </div>
                                <p style="font-size: 0.75rem; color: #64748b; margin: 6px 0 0 0;">
                                    "Waktu maksimal untuk menyelesaikan langkah ini"
                                </p>
                            </div>

                            // SLA Preview
                            <div style="background: rgba(96,165,250,0.1); border: 1px solid rgba(96,165,250,0.3); border-radius: 12px; padding: 16px;">
                                <div style="display: flex; align-items: center; gap: 12px; margin-bottom: 12px;">
                                    <i class="fas fa-info-circle" style="color: #60a5fa; font-size: 1.2rem;"></i>
                                    <div style="font-size: 0.85rem; font-weight: 600; color: #60a5fa;">
                                        "Preview SLA"
                                    </div>
                                </div>
                                <div style="font-size: 0.8rem; color: #93c5fd; line-height: 1.6;">
                                    "Batas waktu: " <strong style="color: #e0f2fe;">{move || format_sla(sla_minutes().unwrap_or(0))}</strong>
                                    <br/>
                                    "Total: " <strong style="color: #e0f2fe;">{move || sla_minutes().unwrap_or(0)} " menit"</strong>
                                </div>
                            </div>

                            // Escalation Configuration
                            <div style="background: rgba(255,255,255,0.04); border: 1px solid rgba(255,255,255,0.08); border-radius: 12px; padding: 16px;">
                                <label style="display: flex; align-items: center; gap: 12px; cursor: pointer;">
                                    <input
                                        type="checkbox"
                                        prop:checked=move || escalation_enabled.get()
                                        on:change=move |e| set_escalation_enabled.set(event_target_checked(&e))
                                        prop:disabled=move || saving.get()
                                        style="width: 20px; height: 20px; cursor: pointer;"
                                    />
                                    <div style="flex: 1;">
                                        <div style="font-size: 0.9rem; font-weight: 600; color: #e2e8f0;">
                                            "Aktifkan Eskalasi Otomatis"
                                        </div>
                                        <div style="font-size: 0.75rem; color: #64748b; margin-top: 4px;">
                                            "Kirim notifikasi eskalasi ke supervisor jika SLA dilanggar"
                                        </div>
                                    </div>
                                </label>
                            </div>

                            // Validation Warning
                            {move || {
                                let minutes = sla_minutes().unwrap_or(0);
                                if minutes < 5 {
                                    Some(view! {
                                        <div style="background: rgba(251,146,60,0.1); border: 1px solid rgba(251,146,60,0.3); border-radius: 12px; padding: 12px 16px; display: flex; align-items: start; gap: 12px;">
                                            <i class="fas fa-exclamation-triangle" style="color: #fb923c; font-size: 1rem; margin-top: 2px;"></i>
                                            <div style="flex: 1;">
                                                <div style="font-size: 0.85rem; font-weight: 600; color: #fb923c; margin-bottom: 4px;">
                                                    "Peringatan"
                                                </div>
                                                <div style="font-size: 0.8rem; color: #fdba74;">
                                                    "SLA terlalu pendek. Disarankan minimal 5 menit."
                                                </div>
                                            </div>
                                        </div>
                                    })
                                } else if minutes > 43200 {
                                    Some(view! {
                                        <div style="background: rgba(251,146,60,0.1); border: 1px solid rgba(251,146,60,0.3); border-radius: 12px; padding: 12px 16px; display: flex; align-items: start; gap: 12px;">
                                            <i class="fas fa-exclamation-triangle" style="color: #fb923c; font-size: 1rem; margin-top: 2px;"></i>
                                            <div style="flex: 1;">
                                                <div style="font-size: 0.85rem; font-weight: 600; color: #fb923c; margin-bottom: 4px;">
                                                    "Peringatan"
                                                </div>
                                                <div style="font-size: 0.8rem; color: #fdba74;">
                                                    "SLA terlalu panjang (lebih dari 30 hari). Pertimbangkan untuk mengurangi."
                                                </div>
                                            </div>
                                        </div>
                                    })
                                } else {
                                    None
                                }
                            }}
                        })}
                    </div>
                </div>

                // Footer
                <div style="padding: 20px 24px; border-top: 1px solid rgba(255,255,255,0.08); display: flex; justify-content: flex-end; gap: 12px;">
                    <button
                        on:click=move |_| on_close.run(())
                        prop:disabled=move || saving.get()
                        style="padding: 10px 20px; background: rgba(255,255,255,0.06); border: 1px solid rgba(255,255,255,0.1); border-radius: 10px; color: #e2e8f0; font-size: 0.85rem; font-weight: 600; cursor: pointer; transition: all 0.2s;"
                        class="hover:bg-white/10"
                    >
                        "Batal"
                    </button>
                    <button
                        on:click=on_submit
                        prop:disabled=move || saving.get() || (sla_enabled.get() && sla_value.get() == 0)
                        style="padding: 10px 20px; background: linear-gradient(135deg, #60a5fa, #3b82f6); border: none; border-radius: 10px; color: white; font-size: 0.85rem; font-weight: 600; cursor: pointer; transition: all 0.2s;"
                        class="hover:opacity-90"
                    >
                        {move || if saving.get() {
                            view! { <><i class="fas fa-spinner fa-spin" style="margin-right: 6px;"></i>"Menyimpan..."</> }.into_any()
                        } else {
                            view! { <><i class="fas fa-save" style="margin-right: 6px;"></i>"Simpan"</> }.into_any()
                        }}
                    </button>
                </div>
            </div>
        </div>
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Workflow Step Row Component
// ═══════════════════════════════════════════════════════════════════════════

#[component]
fn WorkflowStepRow(
    step: WorkflowStep,
    workflow_name: String,
    #[prop(into)] on_sla_config: Callback<WorkflowStep>,
) -> impl IntoView {
    let sla_display = step
        .sla_minutes
        .map(|m| format_sla(m))
        .unwrap_or_else(|| "—".to_string());

    let has_sla = step.sla_minutes.is_some();
    let sla_color = if has_sla { "#fb923c" } else { "#64748b" };

    let role_display = step
        .required_role
        .clone()
        .unwrap_or_else(|| "—".to_string());

    let next_states_display = if step.next_states.is_empty() {
        "Terminal".to_string()
    } else {
        step.next_states.join(", ")
    };

    let state_icon = if step.is_terminal {
        "fas fa-flag-checkered"
    } else {
        "fas fa-circle"
    };

    let state_color = if step.is_terminal {
        "#f87171"
    } else {
        "#60a5fa"
    };

    let step_for_config = step.clone();

    view! {
        <tr style="border-bottom: 1px solid rgba(255,255,255,0.06);">
            <td style="padding: 14px 12px;">
                <div style="display: flex; align-items: center; gap: 10px;">
                    <div style=format!(
                        "width: 32px; height: 32px; display: flex; align-items: center; justify-content: center; border-radius: 8px; background: {}15; border: 1px solid {}40;",
                        state_color, state_color
                    )>
                        <i class=state_icon style=format!("font-size: 0.75rem; color: {};", state_color)></i>
                    </div>
                    <div>
                        <div style="font-size: 0.85rem; font-weight: 600; color: #e2e8f0;">
                            {step.state_name.clone()}
                        </div>
                        {step.state_code.map(|code| view! {
                            <div style="font-size: 0.7rem; color: #64748b;">
                                "Kode: " {code}
                            </div>
                        })}
                    </div>
                </div>
            </td>
            <td style="padding: 14px 12px;">
                <span style="display: inline-flex; align-items: center; gap: 6px; padding: 4px 10px; background: rgba(255,255,255,0.04); border: 1px solid rgba(255,255,255,0.06); border-radius: 8px; font-size: 0.75rem; color: #94a3b8;">
                    <i class="fas fa-user-tag" style="color: #a78bfa; font-size: 0.7rem;"></i>
                    {role_display}
                </span>
            </td>
            <td style="padding: 14px 12px;">
                <div style="display: flex; align-items: center; gap: 8px;">
                    <span style=format!(
                        "display: inline-flex; align-items: center; gap: 6px; padding: 4px 10px; background: rgba(255,255,255,0.04); border: 1px solid rgba(255,255,255,0.06); border-radius: 8px; font-size: 0.75rem; color: {};",
                        if has_sla { "#fb923c" } else { "#94a3b8" }
                    )>
                        <i class="fas fa-clock" style=format!("color: {}; font-size: 0.7rem;", sla_color)></i>
                        {sla_display}
                    </span>
                    <button
                        on:click=move |_| on_sla_config.run(step_for_config.clone())
                        style="padding: 4px 8px; background: rgba(96,165,250,0.1); border: 1px solid rgba(96,165,250,0.3); border-radius: 6px; color: #60a5fa; font-size: 0.7rem; cursor: pointer; transition: all 0.2s;"
                        class="hover:bg-blue-500/20"
                        title="Konfigurasi SLA"
                    >
                        <i class="fas fa-cog"></i>
                    </button>
                </div>
            </td>
            <td style="padding: 14px 12px;">
                <div style="font-size: 0.75rem; color: #94a3b8; line-height: 1.4;">
                    {next_states_display}
                </div>
            </td>
            <td style="padding: 14px 12px; text-align: center;">
                {if step.escalation_enabled {
                    view! {
                        <span style="display: inline-flex; align-items: center; justify-content: center; width: 24px; height: 24px; background: rgba(34,197,94,0.15); border: 1px solid rgba(34,197,94,0.3); border-radius: 6px;">
                            <i class="fas fa-check" style="font-size: 0.7rem; color: #22c55e;"></i>
                        </span>
                    }
                } else {
                    view! {
                        <span style="display: inline-flex; align-items: center; justify-content: center; width: 24px; height: 24px; background: rgba(100,116,139,0.15); border: 1px solid rgba(100,116,139,0.3); border-radius: 6px;">
                            <i class="fas fa-minus" style="font-size: 0.7rem; color: #64748b;"></i>
                        </span>
                    }
                }}
            </td>
        </tr>
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Workflow Detail Modal Component
// ═══════════════════════════════════════════════════════════════════════════

#[component]
fn WorkflowDetailModal(
    workflow_name: String,
    #[prop(into)] on_close: Callback<()>,
) -> impl IntoView {
    let workflow_name_for_modal = workflow_name.clone();
    let workflow_name_for_view = workflow_name.clone();

    let detail_resource = LocalResource::new(move || {
        let name = workflow_name.clone();
        async move { fetch_workflow_definition_detail(&name).await }
    });

    let (sla_config_step, set_sla_config_step) = signal::<Option<WorkflowStep>>(None);

    let on_sla_config = Callback::new(move |step: WorkflowStep| {
        set_sla_config_step.set(Some(step));
    });

    let on_close_sla_config = Callback::new(move |_: ()| {
        set_sla_config_step.set(None);
    });

    let on_save_sla_config = Callback::new(move |_: ()| {
        set_sla_config_step.set(None);
        detail_resource.refetch();
    });

    view! {
        <div
            style="position: fixed; inset: 0; background: rgba(0,0,0,0.7); backdrop-filter: blur(4px); z-index: 1000; display: flex; align-items: center; justify-content: center; padding: 20px;"
            on:click=move |e| {
                if e.target() == e.current_target() {
                    on_close.run(());
                }
            }
        >
            <div style="background: linear-gradient(180deg, #0f172a 0%, #1e293b 100%); border: 1px solid rgba(255,255,255,0.1); border-radius: 20px; max-width: 1000px; width: 100%; max-height: 90vh; overflow: hidden; display: flex; flex-direction: column;">
                // Header
                <div style="padding: 24px; border-bottom: 1px solid rgba(255,255,255,0.08);">
                    <div style="display: flex; justify-content: space-between; align-items: center;">
                        <div>
                            <h2 style="font-size: 1.3rem; font-weight: 800; color: #e2e8f0; margin: 0 0 6px 0;">
                                "Detail Workflow"
                            </h2>
                            <p style="font-size: 0.8rem; color: #94a3b8; margin: 0;">
                                "Konfigurasi langkah dan transisi workflow"
                            </p>
                        </div>
                        <button
                            on:click=move |_| on_close.run(())
                            style="width: 36px; height: 36px; display: flex; align-items: center; justify-content: center; background: rgba(255,255,255,0.04); border: 1px solid rgba(255,255,255,0.08); border-radius: 10px; color: #94a3b8; cursor: pointer; transition: all 0.2s;"
                            class="hover:bg-white/10"
                        >
                            <i class="fas fa-times"></i>
                        </button>
                    </div>
                </div>

                // Content
                <div style="flex: 1; overflow-y: auto; padding: 24px;">
                    <Suspense fallback=move || view! {
                        <div style="text-align: center; padding: 40px;">
                            <i class="fas fa-spinner fa-spin" style="font-size: 2rem; color: #60a5fa;"></i>
                            <p style="margin-top: 16px; color: #94a3b8; font-size: 0.85rem;">"Memuat detail workflow..."</p>
                        </div>
                    }>
                        {move || {
                            detail_resource.get().map(|result| match result {
                                Ok(response) => {
                                    let detail = response.data;
                                    view! {
                                        <div>
                                            // Workflow Info
                                            <div style="background: rgba(255,255,255,0.04); border: 1px solid rgba(255,255,255,0.08); border-radius: 12px; padding: 20px; margin-bottom: 24px;">
                                                <h3 style="font-size: 1rem; font-weight: 700; color: #e2e8f0; margin: 0 0 12px 0;">
                                                    {detail.definition.name.clone()}
                                                </h3>
                                                <p style="font-size: 0.85rem; color: #94a3b8; margin: 0 0 16px 0;">
                                                    {detail.definition.description.clone()}
                                                </p>
                                                <div style="display: flex; flex-wrap: wrap; gap: 10px;">
                                                    <div style="display: flex; align-items: center; gap: 8px; padding: 6px 12px; background: rgba(255,255,255,0.04); border: 1px solid rgba(255,255,255,0.06); border-radius: 8px; font-size: 0.75rem; color: #94a3b8;">
                                                        <i class="fas fa-code-branch" style="color: #60a5fa;"></i>
                                                        "Version: " <strong style="color: #e2e8f0;">{detail.definition.version.clone()}</strong>
                                                    </div>
                                                    <div style="display: flex; align-items: center; gap: 8px; padding: 6px 12px; background: rgba(255,255,255,0.04); border: 1px solid rgba(255,255,255,0.06); border-radius: 8px; font-size: 0.75rem; color: #94a3b8;">
                                                        <i class="fas fa-info-circle" style="color: #22c55e;"></i>
                                                        "Status: " <strong style="color: #e2e8f0;">{detail.definition.status.clone()}</strong>
                                                    </div>
                                                    {detail.definition.supports_parallel_approval.then(|| view! {
                                                        <div style="display: flex; align-items: center; gap: 8px; padding: 6px 12px; background: rgba(255,255,255,0.04); border: 1px solid rgba(255,255,255,0.06); border-radius: 8px; font-size: 0.75rem; color: #94a3b8;">
                                                            <i class="fas fa-users" style="color: #34d399;"></i>
                                                            "Parallel Approval Enabled"
                                                        </div>
                                                    })}
                                                </div>
                                            </div>

                                            // Workflow Steps Table
                                            <div style="background: rgba(255,255,255,0.04); border: 1px solid rgba(255,255,255,0.08); border-radius: 12px; overflow: hidden;">
                                                <div style="padding: 16px 20px; border-bottom: 1px solid rgba(255,255,255,0.08);">
                                                    <h4 style="font-size: 0.9rem; font-weight: 700; color: #e2e8f0; margin: 0;">
                                                        "Langkah Workflow"
                                                    </h4>
                                                </div>
                                                <div style="overflow-x: auto;">
                                                    <table style="width: 100%; border-collapse: collapse;">
                                                        <thead>
                                                            <tr style="background: rgba(255,255,255,0.02); border-bottom: 1px solid rgba(255,255,255,0.08);">
                                                                <th style="padding: 12px; text-align: left; font-size: 0.75rem; font-weight: 600; color: #94a3b8; text-transform: uppercase; letter-spacing: 0.05em;">"State"</th>
                                                                <th style="padding: 12px; text-align: left; font-size: 0.75rem; font-weight: 600; color: #94a3b8; text-transform: uppercase; letter-spacing: 0.05em;">"Role"</th>
                                                                <th style="padding: 12px; text-align: left; font-size: 0.75rem; font-weight: 600; color: #94a3b8; text-transform: uppercase; letter-spacing: 0.05em;">"SLA"</th>
                                                                <th style="padding: 12px; text-align: left; font-size: 0.75rem; font-weight: 600; color: #94a3b8; text-transform: uppercase; letter-spacing: 0.05em;">"Next States"</th>
                                                                <th style="padding: 12px; text-align: center; font-size: 0.75rem; font-weight: 600; color: #94a3b8; text-transform: uppercase; letter-spacing: 0.05em;">"Escalation"</th>
                                                            </tr>
                                                        </thead>
                                                        <tbody>
                                                            {detail.steps.into_iter().map(|step| view! {
                                                                <WorkflowStepRow
                                                                    step=step
                                                                    workflow_name=workflow_name_for_view.clone()
                                                                    on_sla_config=on_sla_config
                                                                />
                                                            }).collect::<Vec<_>>()}
                                                        </tbody>
                                                    </table>
                                                </div>
                                            </div>
                                        </div>
                                    }.into_any()
                                }
                                Err(e) => view! {
                                    <div style="text-align: center; padding: 40px;">
                                        <i class="fas fa-exclamation-triangle" style="font-size: 2rem; color: #f87171;"></i>
                                        <p style="margin-top: 16px; color: #f87171; font-size: 0.85rem;">{e.to_string()}</p>
                                    </div>
                                }.into_any()
                            })
                        }}
                    </Suspense>
                </div>

                // Footer
                <div style="padding: 20px 24px; border-top: 1px solid rgba(255,255,255,0.08); display: flex; justify-content: flex-end;">
                    <button
                        on:click=move |_| on_close.run(())
                        style="padding: 10px 20px; background: rgba(255,255,255,0.06); border: 1px solid rgba(255,255,255,0.1); border-radius: 10px; color: #e2e8f0; font-size: 0.85rem; font-weight: 600; cursor: pointer; transition: all 0.2s;"
                        class="hover:bg-white/10"
                    >
                        "Tutup"
                    </button>
                </div>
            </div>

            // SLA Configuration Modal
            {move || sla_config_step.get().map(|step| view! {
                <SlaConfigModal
                    workflow_name=workflow_name_for_modal.clone()
                    step=step
                    on_close=on_close_sla_config
                    on_save=on_save_sla_config
                />
            })}
        </div>
    }
}

#[component]
pub fn WorkflowConfigManagement() -> impl IntoView {
    let workflows_resource = LocalResource::new(|| fetch_workflow_definitions());
    let (selected_workflow, set_selected_workflow) = signal::<Option<String>>(None);
    let (edit_workflow, set_edit_workflow) = signal::<Option<WorkflowDefinition>>(None);
    let (delete_workflow_name, set_delete_workflow_name) = signal::<Option<String>>(None);
    let (show_create, set_show_create) = signal(false);

    let on_view_details = Callback::new(move |name: String| {
        set_selected_workflow.set(Some(name));
    });

    let on_close_modal = Callback::new(move |_: ()| {
        set_selected_workflow.set(None);
    });

    let on_edit = Callback::new(move |name: String| {
        // Find the workflow to edit
        if let Some(Ok(response)) = workflows_resource.get() {
            if let Some(workflow) = response.data.iter().find(|w| w.name == name) {
                set_edit_workflow.set(Some(workflow.clone()));
            }
        }
    });

    let on_delete = Callback::new(move |name: String| {
        set_delete_workflow_name.set(Some(name));
    });

    let on_close_edit = Callback::new(move |_: ()| {
        set_edit_workflow.set(None);
        set_show_create.set(false);
    });

    let on_save = Callback::new(move |_: ()| {
        set_edit_workflow.set(None);
        set_show_create.set(false);
        workflows_resource.refetch();
    });

    let on_close_delete = Callback::new(move |_: ()| {
        set_delete_workflow_name.set(None);
    });

    let on_confirm_delete = Callback::new(move |_: ()| {
        set_delete_workflow_name.set(None);
        workflows_resource.refetch();
    });

    let on_create_new = move |_| {
        set_show_create.set(true);
    };

    view! {
        <div style="max-width: 1400px; margin: 0 auto;">
            // Header
            <div style="margin-bottom: 28px;">
                <div style="display: flex; align-items: center; justify-content: space-between; margin-bottom: 12px;">
                    <div style="display: flex; align-items: center; gap: 12px;">
                        <div style="width: 4px; height: 32px; background: linear-gradient(180deg, #60a5fa, #3b82f6); border-radius: 2px;"></div>
                        <h1 style="font-size: 1.5rem; font-weight: 800; color: #e2e8f0; margin: 0;">
                            "Konfigurasi Workflow"
                        </h1>
                    </div>
                    <button
                        on:click=on_create_new
                        style="padding: 12px 20px; background: linear-gradient(135deg, #60a5fa, #3b82f6); border: none; border-radius: 12px; color: white; font-size: 0.85rem; font-weight: 600; cursor: pointer; transition: all 0.2s; display: flex; align-items: center; gap: 8px;"
                        class="hover:opacity-90"
                    >
                        <i class="fas fa-plus"></i>
                        "Buat Workflow Baru"
                    </button>
                </div>
                <p style="font-size: 0.9rem; color: #94a3b8; margin: 0 0 0 16px;">
                    "Kelola dan pantau konfigurasi workflow untuk proses persetujuan BMN"
                </p>
            </div>

            // Workflows Grid
            <Suspense fallback=move || view! {
                <div style="display: grid; grid-template-columns: repeat(auto-fill, minmax(320px, 1fr)); gap: 16px;">
                    {(0..3).map(|_| view! {
                        <div style="background: rgba(255,255,255,0.04); border: 1px solid rgba(255,255,255,0.08); border-radius: 16px; padding: 20px; height: 200px;">
                            <div style="width: 60%; height: 20px; background: rgba(255,255,255,0.06); border-radius: 4px; margin-bottom: 12px;"></div>
                            <div style="width: 100%; height: 40px; background: rgba(255,255,255,0.06); border-radius: 4px; margin-bottom: 16px;"></div>
                            <div style="width: 40%; height: 16px; background: rgba(255,255,255,0.06); border-radius: 4px;"></div>
                        </div>
                    }).collect::<Vec<_>>()}
                </div>
            }>
                {move || {
                    workflows_resource.get().map(|result| match result {
                        Ok(response) => {
                            if response.data.is_empty() {
                                view! {
                                    <div style="text-align: center; padding: 60px 20px; background: rgba(255,255,255,0.04); border: 1px solid rgba(255,255,255,0.08); border-radius: 16px;">
                                        <i class="fas fa-project-diagram" style="font-size: 3rem; color: #64748b; margin-bottom: 16px;"></i>
                                        <h3 style="font-size: 1.1rem; font-weight: 700; color: #e2e8f0; margin: 0 0 8px 0;">
                                            "Tidak Ada Workflow"
                                        </h3>
                                        <p style="font-size: 0.85rem; color: #94a3b8; margin: 0 0 20px 0;">
                                            "Belum ada workflow yang dikonfigurasi dalam sistem."
                                        </p>
                                        <button
                                            on:click=on_create_new
                                            style="padding: 12px 24px; background: linear-gradient(135deg, #60a5fa, #3b82f6); border: none; border-radius: 12px; color: white; font-size: 0.85rem; font-weight: 600; cursor: pointer; transition: all 0.2s;"
                                            class="hover:opacity-90"
                                        >
                                            <i class="fas fa-plus" style="margin-right: 8px;"></i>
                                            "Buat Workflow Pertama"
                                        </button>
                                    </div>
                                }.into_any()
                            } else {
                                view! {
                                    <div style="display: grid; grid-template-columns: repeat(auto-fill, minmax(320px, 1fr)); gap: 16px;">
                                        {response.data.into_iter().map(|workflow| view! {
                                            <WorkflowCard
                                                workflow=workflow
                                                on_view_details=on_view_details
                                                on_edit=on_edit
                                                on_delete=on_delete
                                            />
                                        }).collect::<Vec<_>>()}
                                    </div>
                                }.into_any()
                            }
                        }
                        Err(e) => view! {
                            <div style="text-align: center; padding: 60px 20px; background: rgba(255,255,255,0.04); border: 1px solid rgba(255,255,255,0.08); border-radius: 16px;">
                                <i class="fas fa-exclamation-triangle" style="font-size: 3rem; color: #f87171; margin-bottom: 16px;"></i>
                                <h3 style="font-size: 1.1rem; font-weight: 700; color: #e2e8f0; margin: 0 0 8px 0;">
                                    "Gagal Memuat Data"
                                </h3>
                                <p style="font-size: 0.85rem; color: #94a3b8; margin: 0;">
                                    {e.to_string()}
                                </p>
                            </div>
                        }.into_any()
                    })
                }}
            </Suspense>

            // Detail Modal
            {move || selected_workflow.get().map(|name| view! {
                <WorkflowDetailModal workflow_name=name on_close=on_close_modal />
            })}

            // Edit Modal
            {move || edit_workflow.get().map(|workflow| view! {
                <WorkflowEditModal workflow=Some(workflow) on_close=on_close_edit on_save=on_save />
            })}

            // Create Modal
            {move || show_create.get().then(|| view! {
                <WorkflowEditModal workflow=None on_close=on_close_edit on_save=on_save />
            })}

            // Delete Confirmation Modal
            {move || delete_workflow_name.get().map(|name| view! {
                <DeleteConfirmModal workflow_name=name on_close=on_close_delete on_confirm=on_confirm_delete />
            })}
        </div>
    }
}
