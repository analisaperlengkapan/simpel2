use crate::components::layout::Layout;
use leptos::prelude::*;

#[component]
pub fn Dashboard() -> impl IntoView {
    // Dashboard view for supervision statistics
    view! {
        <Layout>
            <h1 class="text-2xl font-bold mb-4">"Dashboard Pengawasan"</h1>
            <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
                <div class="bg-white p-6 rounded shadow">
                    <div class="text-gray-500">"Total Jadwal"</div>
                    <div class="text-3xl font-bold">"12"</div>
                </div>
                <div class="bg-white p-6 rounded shadow">
                    <div class="text-gray-500">"Temuan Bulan Ini"</div>
                    <div class="text-3xl font-bold text-red-500">"5"</div>
                </div>
                <div class="bg-white p-6 rounded shadow">
                    <div class="text-gray-500">"Tindak Lanjut"</div>
                    <div class="text-3xl font-bold text-green-500">"80%"</div>
                </div>
            </div>
        </Layout>
    }
}
