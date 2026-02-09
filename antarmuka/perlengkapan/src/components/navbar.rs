use leptos::prelude::*;

#[component]
pub fn TopNavBar() -> impl IntoView {
    view! {
        <nav class="bg-gradient-to-r from-red-700 via-red-800 to-red-900 shadow-lg border-b-2 border-yellow-400">
            <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
                <div class="flex items-center justify-between h-16">
                    <div class="flex items-center">
                        <div class="flex-shrink-0 flex items-center space-x-3">
                            <img class="h-10 w-10" src="/logo-kejaksaan.png" alt="Logo Kejaksaan RI"/>
                            <div class="text-white">
                                <div class="text-lg font-bold">SIMPEL PERLENGKAPAN</div>
                                <div class="text-xs text-yellow-300">Sistem Informasi Manajemen Perlengkapan Kejaksaan RI</div>
                            </div>
                        </div>
                    </div>
                    <div class="hidden md:block">
                        <div class="ml-10 flex items-baseline space-x-4">
                            <a href="/dashboard" class="text-yellow-300 hover:text-white hover:bg-red-600 px-3 py-2 rounded-md text-sm font-medium transition-colors duration-200">
                                Dashboard
                            </a>
                            <a href="/bank-aset" class="text-yellow-300 hover:text-white hover:bg-red-600 px-3 py-2 rounded-md text-sm font-medium transition-colors duration-200">
                                Bank Aset
                            </a>
                            <a href="/profile" class="text-yellow-300 hover:text-white hover:bg-red-600 px-3 py-2 rounded-md text-sm font-medium transition-colors duration-200">
                                Profile
                            </a>
                        </div>
                    </div>
                </div>
            </div>
        </nav>
    }
}
