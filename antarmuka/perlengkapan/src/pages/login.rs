//! Login page for Perlengkapan microfrontend.
//!
//! This page is shown when the user has not authenticated yet.
//! Authentication is delegated to Portal, then redirected back here.

use crate::routes;
use leptos::prelude::*;
use leptos_meta::Title;

#[component]
pub fn LoginPage() -> impl IntoView {
    view! {
        <Title text="Login — SIMPEL Perlengkapan" />
        <div style="min-height: 100vh; display: flex; align-items: center; justify-content: center; padding: 2.5rem 1rem; background: radial-gradient(circle at 18% 20%, rgba(30,64,175,0.34), transparent 42%), radial-gradient(circle at 84% 14%, rgba(15,23,42,0.58), transparent 48%), linear-gradient(180deg, #081229 0%, #091833 55%, #071229 100%);">
            <div style="width: 100%; max-width: 760px; border-radius: 24px; border: 1px solid rgba(51,65,85,0.72); border-top: 3px solid #d4a843; background: linear-gradient(180deg, rgba(4,14,38,0.97) 0%, rgba(3,12,34,0.97) 100%); padding: clamp(1.6rem, 2vw, 2.25rem) clamp(1.25rem, 3vw, 2.25rem) clamp(1.35rem, 2vw, 1.9rem) clamp(1.25rem, 3vw, 2.25rem); box-shadow: 0 22px 70px rgba(0,0,0,0.44), inset 0 1px 0 rgba(255,255,255,0.04);">
                <div style="display: flex; justify-content: center; margin-bottom: 0.8rem;">
                    <img
                        src="/perlengkapan/assets/kejaksaan-logo.png"
                        alt="Kejaksaan RI"
                        style="width: clamp(88px, 15vw, 116px); height: clamp(88px, 15vw, 116px); object-fit: contain; filter: drop-shadow(0 8px 16px rgba(0,0,0,0.34));"
                    />
                </div>

                <div style="margin-bottom: 1.15rem; text-align: center;">
                    <h1 style="font-size: clamp(1.65rem, 3.6vw, 2.35rem); line-height: 1.16; font-weight: 800; color: #f8fafc; margin: 0;">
                        "Sistem Informasi"
                        <br />
                        <span style="color: #d4a843;">"Manajemen Perlengkapan"</span>
                    </h1>
                    <p style="margin: 0.58rem 0 0 0; font-size: 1rem; font-weight: 600; letter-spacing: 0.02em; color: #cbd5e1;">"Kejaksaan Republik Indonesia"</p>
                </div>

                <div style="height: 1px; width: 100%; background: linear-gradient(90deg, transparent 0%, rgba(51,65,85,0.72) 12%, rgba(71,85,105,0.74) 50%, rgba(51,65,85,0.72) 88%, transparent 100%);"></div>

                <p style="max-width: 620px; font-size: 0.98rem; line-height: 1.74; color: #cbd5e1; margin: 1.1rem auto 0 auto; text-align: center;">
                    "Untuk masuk ke aplikasi Perlengkapan, autentikasi dilakukan melalui Portal SIMPEL terlebih dahulu."
                </p>

                <div style="margin-top: 1.45rem; display: flex; justify-content: center; width: 100%;">
                    <a
                        href=routes::path::PORTAL_LOGIN_WITH_REDIRECT
                        style="display: inline-flex; width: min(430px, 100%); align-items: center; justify-content: center; gap: 0.58rem; border-radius: 999px; background: linear-gradient(180deg, #e7c65e 0%, #d4a843 58%, #be9231 100%); padding: 0.9rem 1.2rem; font-size: 1rem; font-weight: 800; color: #0f172a; text-decoration: none; box-shadow: 0 0 0 1px rgba(255,255,255,0.08) inset, 0 9px 24px rgba(212, 168, 67, 0.28); transition: transform 0.16s ease, filter 0.16s ease;"
                        on:mouseenter=move |ev| {
                            let target = leptos::prelude::event_target::<web_sys::HtmlElement>(&ev);
                            let _ = target.style().set_property("transform", "translateY(-1px)");
                            let _ = target.style().set_property("filter", "brightness(1.05)");
                        }
                        on:mouseleave=move |ev| {
                            let target = leptos::prelude::event_target::<web_sys::HtmlElement>(&ev);
                            let _ = target.style().set_property("transform", "translateY(0)");
                            let _ = target.style().set_property("filter", "brightness(1)");
                        }
                    >
                        <i class="fas fa-right-to-bracket" style="font-size: 0.95rem;"></i>
                        "Masuk via Portal"
                    </a>
                </div>

                <p style="margin-top: 0.92rem; text-align: center; font-size: 0.84rem; color: #94a3b8;">
                    "Setelah login berhasil, Anda akan diarahkan kembali ke dashboard Perlengkapan."
                </p>

                <div style="margin-top: 1.2rem; text-align: center; font-size: 0.78rem; color: #64748b;">
                    "Akses resmi internal — SIMPEL"
                </div>
            </div>
        </div>
    }
}
