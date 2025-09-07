use leptos::prelude::*;

#[component]
pub fn BankAsetContent() -> impl IntoView {
    view! {
        <div>
            <h1 class="text-2xl font-bold text-gray-900 mb-6">"Bank Aset"</h1>
            <div class="bg-white p-6 rounded-lg shadow-md">
                <h3 class="text-lg font-semibold text-gray-900 mb-4">"Manajemen Bank Aset"</h3>
                <p class="text-gray-600">"Fitur bank aset untuk mengelola katalog, kategori, dan spesifikasi aset."</p>
                <p class="text-sm text-gray-500 mt-2">"Implementasi backend dan integrasi API akan ditambahkan setelah frontend selesai."</p>
            </div>
        </div>
    }
}

#[component]
pub fn AnalisisKebutuhanContent() -> impl IntoView {
    view! {
        <div>
            <h1 class="text-2xl font-bold text-gray-900 mb-6">"Analisis Kebutuhan"</h1>
            <div class="bg-white p-6 rounded-lg shadow-md">
                <h3 class="text-lg font-semibold text-gray-900 mb-4">"Analisis Kebutuhan Aset"</h3>
                <p class="text-gray-600">"Modul untuk analisis kebutuhan, prioritas, dan pelaporan kebutuhan aset."</p>
                <p class="text-sm text-gray-500 mt-2">"Implementasi backend dan integrasi API akan ditambahkan setelah frontend selesai."</p>
            </div>
        </div>
    }
}

#[component]
pub fn PengadaanContent() -> impl IntoView {
    view! {
        <div>
            <h1 class="text-2xl font-bold text-gray-900 mb-6">"Pengadaan"</h1>
            <div class="bg-white p-6 rounded-lg shadow-md">
                <h3 class="text-lg font-semibold text-gray-900 mb-4">"Manajemen Pengadaan"</h3>
                <p class="text-gray-600">"Sistem pengadaan meliputi perencanaan, tender, kontrak, dan monitoring."</p>
                <p class="text-sm text-gray-500 mt-2">"Implementasi backend dan integrasi API akan ditambahkan setelah frontend selesai."</p>
            </div>
        </div>
    }
}

#[component]
pub fn PengelolaanBmnContent() -> impl IntoView {
    view! {
        <div>
            <h1 class="text-2xl font-bold text-gray-900 mb-6">"Pengelolaan BMN"</h1>
            <div class="bg-white p-6 rounded-lg shadow-md">
                <h3 class="text-lg font-semibold text-gray-900 mb-4">"Pengelolaan Barang Milik Negara"</h3>
                <p class="text-gray-600">"Manajemen inventaris, pemeliharaan, mutasi, dan penghapusan BMN."</p>
                <p class="text-sm text-gray-500 mt-2">"Implementasi backend dan integrasi API akan ditambahkan setelah frontend selesai."</p>
            </div>
        </div>
    }
}

#[component]
pub fn PenggunaContent() -> impl IntoView {
    view! {
        <div>
            <h1 class="text-2xl font-bold text-gray-900 mb-6">"Manajemen Pengguna"</h1>
            <div class="bg-white p-6 rounded-lg shadow-md">
                <h3 class="text-lg font-semibold text-gray-900 mb-4">"Pengelolaan Pengguna dan Akses"</h3>
                <p class="text-gray-600">"Daftar pengguna, manajemen peran, dan kontrol akses sistem."</p>
                <p class="text-sm text-gray-500 mt-2">"Implementasi backend dan integrasi API akan ditambahkan setelah frontend selesai."</p>
            </div>
        </div>
    }
}

#[component]
pub fn BantuanContent() -> impl IntoView {
    view! {
        <div>
            <h1 class="text-2xl font-bold text-gray-900 mb-6">"Bantuan"</h1>
            <div class="bg-white p-6 rounded-lg shadow-md">
                <h3 class="text-lg font-semibold text-gray-900 mb-4">"Pusat Bantuan"</h3>
                <p class="text-gray-600">"Panduan penggunaan, FAQ, dan informasi kontak dukungan."</p>
                <p class="text-sm text-gray-500 mt-2">"Implementasi backend dan integrasi API akan ditambahkan setelah frontend selesai."</p>
            </div>
        </div>
    }
}
