//! Placeholder page — elegant "coming soon" page for unimplemented modules.

use leptos::prelude::*;

#[component]
pub fn PlaceholderPage(
    title: &'static str,
    icon: &'static str,
    description: &'static str,
) -> impl IntoView {
    view! {
        <div style="max-width: 600px; margin: 60px auto; text-align: center;">
            <div style="background: rgba(255,255,255,0.03); border: 1px solid rgba(255,255,255,0.08); border-radius: 24px; padding: 48px 32px;">
                // Icon
                <div style="width: 72px; height: 72px; display: flex; align-items: center; justify-content: center; margin: 0 auto 20px; background: rgba(212,168,67,0.1); border-radius: 20px; border: 1px solid rgba(212,168,67,0.15);">
                    <i class=icon style="font-size: 1.8rem; color: #d4a843;"></i>
                </div>

                // Title
                <h1 style="font-size: 1.5rem; font-weight: 800; color: #ffffff; margin: 0 0 8px 0;">{title}</h1>

                // Description
                <p style="font-size: 0.88rem; color: #94a3b8; line-height: 1.5; margin: 0 0 24px 0; max-width: 400px; margin-left: auto; margin-right: auto;">
                    {description}
                </p>

                // Status badge
                <div style="display: inline-flex; align-items: center; gap: 8px; padding: 8px 20px; background: rgba(255,255,255,0.04); border: 1px solid rgba(255,255,255,0.08); border-radius: 999px;">
                    <div style="width: 8px; height: 8px; background: #fbbf24; border-radius: 50%; animation: pulse 2s ease-in-out infinite;"></div>
                    <span style="font-size: 0.75rem; font-weight: 600; color: #94a3b8; letter-spacing: 0.05em;">"DALAM PENGEMBANGAN"</span>
                </div>

                // Back link
                <div style="margin-top: 28px;">
                    <a
                        href="/perlengkapan/dashboard"
                        style="display: inline-flex; align-items: center; gap: 8px; font-size: 0.82rem; color: #d4a843; text-decoration: none; font-weight: 600;"
                    >
                        <i class="fas fa-arrow-left" style="font-size: 0.75rem;"></i>
                        "Kembali ke Dashboard"
                    </a>
                </div>
            </div>
        </div>
    }
}
