//! Helpdesk — support contact page.

use leptos::prelude::*;

#[component]
pub fn HelpdeskPage() -> impl IntoView {
    let subject = RwSignal::new(String::new());
    let message = RwSignal::new(String::new());
    let submitted = RwSignal::new(false);

    view! {
        <div style="max-width: 800px; margin: 0 auto;">
            <div style="margin-bottom: 28px;">
                <h1 style="font-size: 1.4rem; font-weight: 800; color: #e2e8f0; margin: 0;">"Helpdesk"</h1>
                <p style="font-size: 0.8rem; color: #64748b; margin-top: 4px;">"Hubungi tim dukungan teknis SIMPEL"</p>
            </div>

            // Contact cards
            <div style="display: grid; grid-template-columns: repeat(auto-fill, minmax(220px, 1fr)); gap: 12px; margin-bottom: 28px;">
                <div style="background: rgba(255,255,255,0.03); border: 1px solid rgba(255,255,255,0.06); border-radius: 14px; padding: 20px;">
                    <i class="fas fa-envelope" style="color: #60a5fa; font-size: 1.2rem; margin-bottom: 10px; display: block;"></i>
                    <div style="font-size: 0.82rem; font-weight: 600; color: #e2e8f0; margin-bottom: 4px;">"Email"</div>
                    <div style="font-size: 0.78rem; color: #94a3b8;">"helpdesk-simpel@kejaksaan.go.id"</div>
                </div>
                <div style="background: rgba(255,255,255,0.03); border: 1px solid rgba(255,255,255,0.06); border-radius: 14px; padding: 20px;">
                    <i class="fas fa-phone" style="color: #34d399; font-size: 1.2rem; margin-bottom: 10px; display: block;"></i>
                    <div style="font-size: 0.82rem; font-weight: 600; color: #e2e8f0; margin-bottom: 4px;">"Telepon"</div>
                    <div style="font-size: 0.78rem; color: #94a3b8;">"(021) 123-4567 ext. 890"</div>
                </div>
                <div style="background: rgba(255,255,255,0.03); border: 1px solid rgba(255,255,255,0.06); border-radius: 14px; padding: 20px;">
                    <i class="fas fa-clock" style="color: #fbbf24; font-size: 1.2rem; margin-bottom: 10px; display: block;"></i>
                    <div style="font-size: 0.82rem; font-weight: 600; color: #e2e8f0; margin-bottom: 4px;">"Jam Kerja"</div>
                    <div style="font-size: 0.78rem; color: #94a3b8;">"Senin - Jumat, 08:00 - 16:00"</div>
                </div>
            </div>

            // Contact form
            <div style="background: rgba(255,255,255,0.03); border: 1px solid rgba(255,255,255,0.06); border-radius: 16px; padding: 28px;">
                <h2 style="font-size: 1rem; font-weight: 700; color: #e2e8f0; margin: 0 0 20px 0;">"Kirim Pesan"</h2>

                {move || if submitted.get() {
                    view! {
                        <div style="text-align: center; padding: 24px;">
                            <i class="fas fa-check-circle" style="font-size: 2rem; color: #34d399; margin-bottom: 12px; display: block;"></i>
                            <div style="font-size: 0.9rem; font-weight: 600; color: #e2e8f0; margin-bottom: 6px;">"Pesan Terkirim"</div>
                            <div style="font-size: 0.8rem; color: #94a3b8;">"Tim helpdesk akan merespons dalam 1×24 jam kerja."</div>
                        </div>
                    }.into_any()
                } else {
                    view! {
                        <div>
                            <div style="margin-bottom: 16px;">
                                <label style="display: block; font-size: 0.78rem; font-weight: 600; color: #94a3b8; margin-bottom: 6px;">"Subjek"</label>
                                <input type="text" placeholder="Judul pesan..."
                                    style="width: 100%; padding: 10px 14px; background: rgba(255,255,255,0.04); border: 1px solid rgba(255,255,255,0.08); border-radius: 8px; color: #e2e8f0; font-size: 0.82rem; outline: none;"
                                    on:input=move |ev| subject.set(event_target_value(&ev))
                                />
                            </div>
                            <div style="margin-bottom: 16px;">
                                <label style="display: block; font-size: 0.78rem; font-weight: 600; color: #94a3b8; margin-bottom: 6px;">"Pesan"</label>
                                <textarea placeholder="Jelaskan kendala atau pertanyaan Anda..."
                                    style="width: 100%; padding: 10px 14px; background: rgba(255,255,255,0.04); border: 1px solid rgba(255,255,255,0.08); border-radius: 8px; color: #e2e8f0; font-size: 0.82rem; outline: none; min-height: 120px; resize: vertical;"
                                    on:input=move |ev| message.set(event_target_value(&ev))
                                ></textarea>
                            </div>
                            <button
                                style="padding: 10px 24px; background: linear-gradient(135deg, #d4a843, #facc15); color: #0f172a; font-weight: 700; font-size: 0.82rem; border: none; border-radius: 10px; cursor: pointer;"
                                on:click=move |_| submitted.set(true)
                            >
                                <i class="fas fa-paper-plane" style="margin-right: 6px;"></i> "Kirim Pesan"
                            </button>
                        </div>
                    }.into_any()
                }}
            </div>
        </div>
    }
}
