//! Helpdesk — support contact page.

use crate::components::layout::{FormField, PageLayout, SectionCard};
use leptos::prelude::*;
use lib_ui::components::icon::AppIcon;
use phosphor_leptos::{CHECK_CIRCLE, CLOCK, ENVELOPE, PAPER_PLANE_TILT, PHONE};

#[component]
pub fn HelpdeskPage() -> impl IntoView {
    let subject = RwSignal::new(String::new());
    let message = RwSignal::new(String::new());
    let submitted = RwSignal::new(false);

    view! {
        <PageLayout
            title="Helpdesk"
            icon="fas fa-headset"
            description="Hubungi tim dukungan teknis SIMPEL"
        >
            // Contact cards
            <div class="mb-6 grid grid-cols-1 gap-3 sm:grid-cols-3">
                <SectionCard title="Email" dense=true>
                    <div class="flex items-start gap-3">
                        <span class="text-info-400 text-lg"><AppIcon icon=ENVELOPE /></span>
                        <span class="text-sm text-slate-300">"helpdesk-simpel@kejaksaan.go.id"</span>
                    </div>
                </SectionCard>
                <SectionCard title="Telepon" dense=true>
                    <div class="flex items-start gap-3">
                        <span class="text-success-400 text-lg"><AppIcon icon=PHONE /></span>
                        <span class="text-sm text-slate-300">"(021) 123-4567 ext. 890"</span>
                    </div>
                </SectionCard>
                <SectionCard title="Jam Kerja" dense=true>
                    <div class="flex items-start gap-3">
                        <span class="text-gold-400 text-lg"><AppIcon icon=CLOCK /></span>
                        <span class="text-sm text-slate-300">"Senin - Jumat, 08:00 - 16:00"</span>
                    </div>
                </SectionCard>
            </div>

            // Contact form
            <SectionCard title="Kirim Pesan">
                {move || if submitted.get() {
                    view! {
                        <div class="flex flex-col items-center py-6 text-center">
                            <span class="text-3xl text-success-400 mb-3"><AppIcon icon=CHECK_CIRCLE /></span>
                            <p class="text-sm font-semibold text-white">"Pesan Terkirim"</p>
                            <p class="mt-1 text-sm text-slate-400">"Tim helpdesk akan merespons dalam 1×24 jam kerja."</p>
                        </div>
                    }.into_any()
                } else {
                    view! {
                        <div class="flex flex-col gap-5">
                            <FormField label="Subjek" full_width=true>
                                <input
                                    type="text"
                                    placeholder="Judul pesan..."
                                    class="focus-ring w-full rounded-lg border border-white/10 bg-white/[0.04] px-3.5 py-2.5 text-sm text-slate-100 placeholder-slate-500"
                                    on:input=move |ev| subject.set(event_target_value(&ev))
                                />
                            </FormField>
                            <FormField label="Pesan" full_width=true>
                                <textarea
                                    placeholder="Jelaskan kendala atau pertanyaan Anda..."
                                    class="focus-ring min-h-[120px] w-full resize-y rounded-lg border border-white/10 bg-white/[0.04] px-3.5 py-2.5 text-sm text-slate-100 placeholder-slate-500"
                                    on:input=move |ev| message.set(event_target_value(&ev))
                                ></textarea>
                            </FormField>
                            <div class="flex justify-end border-t border-white/[0.04] pt-4">
                                <button
                                    type="button"
                                    class="inline-flex items-center gap-2 rounded-lg bg-gold-gradient px-5 py-2.5 text-sm font-bold text-navy-950 shadow-sm transition hover:opacity-90"
                                    on:click=move |_| submitted.set(true)
                                >
                                    <span class="text-xs"><AppIcon icon=PAPER_PLANE_TILT /></span>
                                    "Kirim Pesan"
                                </button>
                            </div>
                        </div>
                    }.into_any()
                }}
            </SectionCard>
        </PageLayout>
    }
}
