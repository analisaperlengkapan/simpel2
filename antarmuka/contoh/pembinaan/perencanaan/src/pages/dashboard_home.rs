//! Dashboard Home page for SIMPEL Perencanaan

use leptos::prelude::*;

#[component]
pub fn DashboardHome() -> impl IntoView {
    view! {
        <div class="space-y-6">
            <div class="bg-white rounded-lg shadow p-6">
                <h1 class="text-2xl font-bold text-gray-900 mb-4">"Dashboard Perencanaan BMN"</h1>
                <p class="text-gray-600 mb-6">
                    "Selamat datang di Sistem Informasi Manajemen Perencanaan Barang Milik Negara"
                </p>

                // Stats Cards
                <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
                    <div class="bg-blue-50 p-4 rounded-lg border border-blue-200">
                        <div class="flex items-center">
                            <div class="p-2 bg-blue-500 rounded-lg">
                                <i class="fas fa-shopping-cart text-white"></i>
                            </div>
                            <div class="ml-4">
                                <p class="text-sm font-medium text-gray-600">"Rencana Pengadaan"</p>
                                <p class="text-2xl font-bold text-gray-900">"24"</p>
                            </div>
                        </div>
                    </div>

                    <div class="bg-green-50 p-4 rounded-lg border border-green-200">
                        <div class="flex items-center">
                            <div class="p-2 bg-green-500 rounded-lg">
                                <i class="fas fa-tools text-white"></i>
                            </div>
                            <div class="ml-4">
                                <p class="text-sm font-medium text-gray-600">"Rencana Pemeliharaan"</p>
                                <p class="text-2xl font-bold text-gray-900">"18"</p>
                            </div>
                        </div>
                    </div>

                    <div class="bg-purple-50 p-4 rounded-lg border border-purple-200">
                        <div class="flex items-center">
                            <div class="p-2 bg-purple-500 rounded-lg">
                                <i class="fas fa-chart-line text-white"></i>
                            </div>
                            <div class="ml-4">
                                <p class="text-sm font-medium text-gray-600">"Rencana Pengembangan"</p>
                                <p class="text-2xl font-bold text-gray-900">"12"</p>
                            </div>
                        </div>
                    </div>

                    <div class="bg-orange-50 p-4 rounded-lg border border-orange-200">
                        <div class="flex items-center">
                            <div class="p-2 bg-orange-500 rounded-lg">
                                <i class="fas fa-file-alt text-white"></i>
                            </div>
                            <div class="ml-4">
                                <p class="text-sm font-medium text-gray-600">"Laporan"</p>
                                <p class="text-2xl font-bold text-gray-900">"8"</p>
                            </div>
                        </div>
                    </div>
                </div>

                // Quick Actions
                <div class="bg-gray-50 p-6 rounded-lg">
                    <h2 class="text-lg font-semibold text-gray-900 mb-4">"Aksi Cepat"</h2>
                    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                        <a
                            href="/pengadaan"
                            class="flex items-center p-4 bg-white rounded-lg shadow-sm hover:shadow-md transition-shadow"
                        >
                            <div class="p-2 bg-blue-500 rounded-lg mr-4">
                                <i class="fas fa-plus text-white"></i>
                            </div>
                            <div>
                                <p class="font-medium text-gray-900">"Buat Rencana Pengadaan"</p>
                                <p class="text-sm text-gray-600">"Rencanakan pengadaan baru"</p>
                            </div>
                        </a>

                        <a
                            href="/pemeliharaan"
                            class="flex items-center p-4 bg-white rounded-lg shadow-sm hover:shadow-md transition-shadow"
                        >
                            <div class="p-2 bg-green-500 rounded-lg mr-4">
                                <i class="fas fa-wrench text-white"></i>
                            </div>
                            <div>
                                <p class="font-medium text-gray-900">"Jadwalkan Pemeliharaan"</p>
                                <p class="text-sm text-gray-600">"Atur jadwal pemeliharaan"</p>
                            </div>
                        </a>

                        <a
                            href="/laporan"
                            class="flex items-center p-4 bg-white rounded-lg shadow-sm hover:shadow-md transition-shadow"
                        >
                            <div class="p-2 bg-orange-500 rounded-lg mr-4">
                                <i class="fas fa-chart-bar text-white"></i>
                            </div>
                            <div>
                                <p class="font-medium text-gray-900">"Lihat Laporan"</p>
                                <p class="text-sm text-gray-600">"Akses laporan perencanaan"</p>
                            </div>
                        </a>
                    </div>
                </div>
            </div>
        </div>
    }
}
