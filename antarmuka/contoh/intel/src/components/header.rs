use leptos::prelude::*;

#[component]
pub fn IntelHeader() -> impl IntoView {
    view! {
        <header class="bg-gradient-to-r from-gray-900 to-gray-700 text-white shadow-lg">
            <div class="container mx-auto px-4">
                <div class="flex items-center justify-between py-4">
                    <div class="flex items-center space-x-4">
                        <div class="flex items-center space-x-3">
                            // Intelligence Icon
                            <div class="w-10 h-10 bg-blue-600 rounded-lg flex items-center justify-center">
                                <svg class="w-6 h-6 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                                          d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"/>
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                                          d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z"/>
                                </svg>
                            </div>
                            <div>
                                <h1 class="text-xl font-bold">"Intel"</h1>
                                <p class="text-sm text-gray-300">"Sistem Intelligence"</p>
                            </div>
                        </div>
                    </div>

                    <nav class="hidden md:flex items-center space-x-6">
                        <a href="#dashboard" class="hover:text-blue-300 transition-colors">"Dashboard"</a>
                        <a href="#operations" class="hover:text-blue-300 transition-colors">"Operasi"</a>
                        <a href="#reports" class="hover:text-blue-300 transition-colors">"Laporan"</a>
                        <a href="#analysis" class="hover:text-blue-300 transition-colors">"Analisis"</a>
                        <a href="#surveillance" class="hover:text-blue-300 transition-colors">"Surveillance"</a>
                        <a href="#threats" class="hover:text-blue-300 transition-colors">"Threat Assessment"</a>
                    </nav>

                    <div class="flex items-center space-x-4">
                        // Classification Indicator
                        <div class="hidden lg:flex items-center space-x-2">
                            <div class="w-3 h-3 bg-red-500 rounded-full animate-pulse"></div>
                            <span class="text-sm font-medium">"CLASSIFIED"</span>
                        </div>

                        // User Menu
                        <div class="flex items-center space-x-2">
                            <div class="w-8 h-8 bg-gray-600 rounded-full flex items-center justify-center">
                                <svg class="w-5 h-5 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                                          d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z"/>
                                </svg>
                            </div>
                            <span class="hidden sm:inline text-sm">"Intel Officer"</span>
                        </div>
                    </div>
                </div>

                // Security Notice Bar
                <div class="bg-red-800 text-center py-2 text-sm">
                    <span class="font-medium">"🔒 SISTEM RAHASIA - AKSES TERBATAS"</span>
                    <span class="mx-2">"|"</span>
                    <span>"Semua aktivitas dimonitor dan dicatat"</span>
                </div>
            </div>
        </header>
    }
}
