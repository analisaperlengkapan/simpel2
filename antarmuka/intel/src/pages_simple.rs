use leptos::prelude::*;

#[component]
pub fn IntelDashboard() -> impl IntoView {
    view! {
        <div class="space-y-6">
            <div class="bg-white p-6 rounded-lg shadow">
                <h2 class="text-2xl font-bold text-gray-900 mb-4">
                    "Dashboard Intelligence"
                </h2>
                <p class="text-gray-600">
                    "Sistem Intelligence untuk monitoring dan analisis data kejaksaan."
                </p>
            </div>

            <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
                <div class="bg-white p-6 rounded-lg shadow">
                    <h3 class="font-semibold text-gray-700">"Operasi Aktif"</h3>
                    <p class="text-3xl font-bold text-blue-600">"24"</p>
                </div>

                <div class="bg-white p-6 rounded-lg shadow">
                    <h3 class="font-semibold text-gray-700">"Data Terkumpul"</h3>
                    <p class="text-3xl font-bold text-green-600">"1,847"</p>
                </div>

                <div class="bg-white p-6 rounded-lg shadow">
                    <h3 class="font-semibold text-gray-700">"Laporan"</h3>
                    <p class="text-3xl font-bold text-purple-600">"156"</p>
                </div>
            </div>
        </div>
    }
}
