//! Permissions & Scopes Management Page (Admin)
//!
//! Manage authorization resources, scopes, policies, and permissions.
//! Mirrors Keycloak's Authorization Services UI.
//! REQ-PORTAL-018

use crate::components::feedback::EmptyPanel;
use crate::components::layout::main_layout::MainLayout;
use crate::utils::app_state::use_main_layout_session_and_logout;
use leptos::prelude::*;

/// Active sub-section
#[derive(Clone, Copy, PartialEq, Eq)]
enum PermTab {
    Resources,
    Scopes,
    Policies,
    Permissions,
    Evaluate,
}

impl PermTab {
    fn label(&self) -> &'static str {
        match self {
            Self::Resources => "Sumber Daya",
            Self::Scopes => "Cakupan",
            Self::Policies => "Kebijakan",
            Self::Permissions => "Izin",
            Self::Evaluate => "Evaluasi",
        }
    }

    fn icon(&self) -> &'static str {
        match self {
            Self::Resources => "📦",
            Self::Scopes => "🔍",
            Self::Policies => "📜",
            Self::Permissions => "✅",
            Self::Evaluate => "🧪",
        }
    }

    fn all() -> &'static [PermTab] {
        &[
            Self::Resources,
            Self::Scopes,
            Self::Policies,
            Self::Permissions,
            Self::Evaluate,
        ]
    }
}

/// Permissions & Scopes management page
#[component]
pub fn PermissionsManagementPage() -> impl IntoView {
    let (active_tab, set_active_tab) = signal(PermTab::Resources);
    let (search, set_search) = signal(String::new());
    let (session, on_logout) = use_main_layout_session_and_logout();

    view! {
        <MainLayout user_session=session.clone() on_logout=on_logout>
            <div class="max-w-7xl mx-auto px-4 py-8">
                <nav class="text-sm text-gray-500 mb-1">
                    <a href="/portal/admin" class="hover:text-primary-600">
                        "Admin"
                    </a>
                    " / Izin & Cakupan"
                </nav>

                <div class="flex items-center justify-between mb-6">
                    <div>
                        <h1 class="text-2xl font-bold text-gray-900">"Izin & Cakupan"</h1>
                        <p class="text-sm text-gray-500 mt-1">
                            "Kelola sumber daya, cakupan, kebijakan, dan izin otorisasi"
                        </p>
                    </div>
                </div>

                // Tab bar
                <div class="bg-white rounded-xl border border-gray-200 overflow-hidden">
                    <div class="border-b">
                        <nav class="flex overflow-x-auto px-2">
                            {PermTab::all()
                                .iter()
                                .map(|tab| {
                                    let t = *tab;
                                    view! {
                                        <button
                                            on:click=move |_| set_active_tab.set(t)
                                            class=move || {
                                                if active_tab.get() == t {
                                                    "flex items-center gap-1.5 px-4 py-3 text-sm font-medium text-primary-600 border-b-2 border-primary-600 whitespace-nowrap"
                                                } else {
                                                    "flex items-center gap-1.5 px-4 py-3 text-sm font-medium text-gray-500 hover:text-gray-700 border-b-2 border-transparent whitespace-nowrap"
                                                }
                                            }
                                        >
                                            <span>{t.icon()}</span>
                                            {t.label()}
                                        </button>
                                    }
                                })
                                .collect::<Vec<_>>()}
                        </nav>
                    </div>

                    <div class="p-6">
                        // Honesty banner — the Authorization Services backend
                        // (resource servers, scopes, policies, permissions
                        // evaluation) is a multi-week feature in its own
                        // epic. The UI below is a deliberate preview so admins
                        // know what's coming without thinking the form is live.
                        <div class="mb-4 rounded-md border border-amber-200 bg-amber-50 px-3 py-2 text-xs text-amber-800">
                            "⚠ Pratinjau UI — modul Authorization Services belum diaktifkan. "
                            "Tombol simpan dan kotak input pada halaman ini belum mengirim apa pun ke backend."
                        </div>

                        // Search bar
                        <div class="mb-6">
                            <input
                                type="text"
                                prop:value=search
                                on:input=move |ev| set_search.set(event_target_value(&ev))
                                placeholder="Cari konfigurasi otorisasi..."
                                class="w-full max-w-md px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500 text-sm"
                            />
                        </div>

                        // === Resources ===
                        <Show when=move || active_tab.get() == PermTab::Resources>
                            <div class="space-y-4">
                                <div class="flex items-center justify-between">
                                    <h3 class="font-medium text-gray-900">
                                        "Sumber Daya Otorisasi"
                                    </h3>
                                    <button class="px-3 py-1.5 text-sm bg-primary-600 text-white rounded-lg hover:bg-primary-700">
                                        "＋ Tambah Sumber Daya"
                                    </button>
                                </div>
                                <div class="bg-gray-50 rounded-lg border">
                                    <table class="w-full">
                                        <thead>
                                            <tr class="text-xs text-gray-500 uppercase border-b">
                                                <th class="px-4 py-3 text-left">"Nama"</th>
                                                <th class="px-4 py-3 text-left">"Tipe"</th>
                                                <th class="px-4 py-3 text-left">"URI"</th>
                                                <th class="px-4 py-3 text-left">"Cakupan"</th>
                                            </tr>
                                        </thead>
                                        <tbody>
                                            <tr>
                                                <td
                                                    colspan="4"
                                                    class="px-4 py-8 text-center text-gray-400 text-sm"
                                                >
                                                    "Belum ada sumber daya yang dikonfigurasi."
                                                </td>
                                            </tr>
                                        </tbody>
                                    </table>
                                </div>
                            </div>
                        </Show>

                        // === Scopes ===
                        <Show when=move || active_tab.get() == PermTab::Scopes>
                            <div class="space-y-4">
                                <div class="flex items-center justify-between">
                                    <h3 class="font-medium text-gray-900">"Cakupan Otorisasi"</h3>
                                    <button class="px-3 py-1.5 text-sm bg-primary-600 text-white rounded-lg hover:bg-primary-700">
                                        "＋ Tambah Cakupan"
                                    </button>
                                </div>
                                <div class="bg-gray-50 rounded-lg border">
                                    <table class="w-full">
                                        <thead>
                                            <tr class="text-xs text-gray-500 uppercase border-b">
                                                <th class="px-4 py-3 text-left">"Nama"</th>
                                                <th class="px-4 py-3 text-left">"Deskripsi"</th>
                                                <th class="px-4 py-3 text-left">"Resource"</th>
                                            </tr>
                                        </thead>
                                        <tbody>
                                            <tr>
                                                <td
                                                    colspan="3"
                                                    class="px-4 py-8 text-center text-gray-400 text-sm"
                                                >
                                                    "Belum ada cakupan yang dikonfigurasi."
                                                </td>
                                            </tr>
                                        </tbody>
                                    </table>
                                </div>
                            </div>
                        </Show>

                        // === Policies ===
                        <Show when=move || active_tab.get() == PermTab::Policies>
                            <div class="space-y-4">
                                <div class="flex items-center justify-between">
                                    <h3 class="font-medium text-gray-900">"Kebijakan Otorisasi"</h3>
                                    <button class="px-3 py-1.5 text-sm bg-primary-600 text-white rounded-lg hover:bg-primary-700">
                                        "＋ Buat Kebijakan"
                                    </button>
                                </div>
                                <div class="grid grid-cols-1 md:grid-cols-3 gap-3 mb-4">
                                    <div class="border rounded-lg p-3 text-center hover:bg-gray-50 cursor-pointer">
                                        <p class="text-lg mb-1">"👤"</p>
                                        <p class="text-xs font-medium text-gray-700">
                                            "Kebijakan Peran"
                                        </p>
                                    </div>
                                    <div class="border rounded-lg p-3 text-center hover:bg-gray-50 cursor-pointer">
                                        <p class="text-lg mb-1">"👥"</p>
                                        <p class="text-xs font-medium text-gray-700">
                                            "Kebijakan Grup"
                                        </p>
                                    </div>
                                    <div class="border rounded-lg p-3 text-center hover:bg-gray-50 cursor-pointer">
                                        <p class="text-lg mb-1">"⏰"</p>
                                        <p class="text-xs font-medium text-gray-700">
                                            "Kebijakan Waktu"
                                        </p>
                                    </div>
                                </div>
                                <EmptyPanel
                                    title="Belum ada kebijakan"
                                    message="Tambahkan kebijakan untuk mendefinisikan aturan otorisasi."
                                />
                            </div>
                        </Show>

                        // === Permissions ===
                        <Show when=move || active_tab.get() == PermTab::Permissions>
                            <div class="space-y-4">
                                <div class="flex items-center justify-between">
                                    <h3 class="font-medium text-gray-900">"Izin Otorisasi"</h3>
                                    <button class="px-3 py-1.5 text-sm bg-primary-600 text-white rounded-lg hover:bg-primary-700">
                                        "＋ Buat Izin"
                                    </button>
                                </div>
                                <div class="grid grid-cols-1 md:grid-cols-2 gap-3 mb-4">
                                    <div class="border rounded-lg p-3 hover:bg-gray-50 cursor-pointer">
                                        <p class="text-sm font-medium text-gray-700">
                                            "📦 Izin Berbasis Sumber Daya"
                                        </p>
                                        <p class="text-xs text-gray-500 mt-1">
                                            "Izin berdasarkan sumber daya."
                                        </p>
                                    </div>
                                    <div class="border rounded-lg p-3 hover:bg-gray-50 cursor-pointer">
                                        <p class="text-sm font-medium text-gray-700">
                                            "🔍 Izin Berbasis Cakupan"
                                        </p>
                                        <p class="text-xs text-gray-500 mt-1">
                                            "Izin berdasarkan cakupan."
                                        </p>
                                    </div>
                                </div>
                                <EmptyPanel
                                    title="Belum ada izin"
                                    message="Buat izin untuk menghubungkan resource, scope, dan kebijakan."
                                />
                            </div>
                        </Show>

                        // === Evaluate ===
                        <Show when=move || active_tab.get() == PermTab::Evaluate>
                            <div class="space-y-4">
                                <h3 class="font-medium text-gray-900">"Evaluasi Kebijakan"</h3>
                                <div class="bg-blue-50 border border-blue-200 rounded-lg p-4">
                                    <p class="text-sm text-blue-700">
                                        "Simulasikan permintaan otorisasi untuk menguji kebijakan dan izin Anda."
                                    </p>
                                </div>
                                <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                                    <div>
                                        <label class="block text-sm font-medium text-gray-700 mb-1">
                                            "Pengguna"
                                        </label>
                                        <select class="w-full px-3 py-2 border rounded-lg text-sm">
                                            <option>"Pilih pengguna..."</option>
                                        </select>
                                    </div>
                                    <div>
                                        <label class="block text-sm font-medium text-gray-700 mb-1">
                                            "Sumber Daya"
                                        </label>
                                        <select class="w-full px-3 py-2 border rounded-lg text-sm">
                                            <option>"Pilih sumber daya..."</option>
                                        </select>
                                    </div>
                                </div>
                                <button class="px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 text-sm">
                                    "🧪 Evaluasi"
                                </button>
                            </div>
                        </Show>
                    </div>
                </div>
            </div>
        </MainLayout>
    }
}
