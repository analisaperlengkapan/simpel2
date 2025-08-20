use leptos::prelude::*;

#[component]
pub fn IntelFooter() -> impl IntoView {
    view! {
        <footer class="bg-gray-900 text-white mt-12">
            <div class="container mx-auto px-4 py-8">
                <div class="grid grid-cols-1 md:grid-cols-4 gap-8">
                    // Intelligence Operations
                    <div>
                        <h3 class="text-lg font-semibold mb-4 text-blue-400">"Operasi Intelligence"</h3>
                        <ul class="space-y-2 text-sm text-gray-300">
                            <li><a href="#" class="hover:text-blue-300 transition-colors">"Surveillance Operations"</a></li>
                            <li><a href="#" class="hover:text-blue-300 transition-colors">"Counter Intelligence"</a></li>
                            <li><a href="#" class="hover:text-blue-300 transition-colors">"Cyber Security"</a></li>
                            <li><a href="#" class="hover:text-blue-300 transition-colors">"Financial Crime Tracking"</a></li>
                            <li><a href="#" class="hover:text-blue-300 transition-colors">"Terrorism Prevention"</a></li>
                        </ul>
                    </div>

                    // Analysis & Reports
                    <div>
                        <h3 class="text-lg font-semibold mb-4 text-blue-400">"Analisis & Laporan"</h3>
                        <ul class="space-y-2 text-sm text-gray-300">
                            <li><a href="#" class="hover:text-blue-300 transition-colors">"Threat Assessment"</a></li>
                            <li><a href="#" class="hover:text-blue-300 transition-colors">"Intelligence Reports"</a></li>
                            <li><a href="#" class="hover:text-blue-300 transition-colors">"Situation Analysis"</a></li>
                            <li><a href="#" class="hover:text-blue-300 transition-colors">"Risk Evaluation"</a></li>
                            <li><a href="#" class="hover:text-blue-300 transition-colors">"Strategic Intelligence"</a></li>
                        </ul>
                    </div>

                    // Support & Training
                    <div>
                        <h3 class="text-lg font-semibold mb-4 text-blue-400">"Dukungan & Pelatihan"</h3>
                        <ul class="space-y-2 text-sm text-gray-300">
                            <li><a href="#" class="hover:text-blue-300 transition-colors">"Intelligence Training"</a></li>
                            <li><a href="#" class="hover:text-blue-300 transition-colors">"Security Protocols"</a></li>
                            <li><a href="#" class="hover:text-blue-300 transition-colors">"Classification Guidelines"</a></li>
                            <li><a href="#" class="hover:text-blue-300 transition-colors">"Technical Support"</a></li>
                            <li><a href="#" class="hover:text-blue-300 transition-colors">"Equipment Management"</a></li>
                        </ul>
                    </div>

                    // Emergency & Contacts
                    <div>
                        <h3 class="text-lg font-semibold mb-4 text-red-400">"Kontak Darurat"</h3>
                        <div class="space-y-3 text-sm text-gray-300">
                            <div class="p-3 bg-red-900 rounded border border-red-700">
                                <p class="font-semibold text-red-300">"Intel Emergency Line"</p>
                                <p class="text-red-200">"(021) 5255-3500"</p>
                                <p class="text-xs text-red-400">"24/7 Ops Center"</p>
                            </div>
                            <div class="p-3 bg-gray-800 rounded border border-gray-700">
                                <p class="font-semibold">"Cyber Security SOC"</p>
                                <p>"(021) 5255-3600"</p>
                                <p class="text-xs text-gray-400">"24/7 Monitoring"</p>
                            </div>
                            <div class="p-3 bg-gray-800 rounded border border-gray-700">
                                <p class="font-semibold">"Counter Intel Unit"</p>
                                <p>"(021) 5255-3700"</p>
                                <p class="text-xs text-gray-400">"Priority Response"</p>
                            </div>
                        </div>
                    </div>
                </div>

                <div class="border-t border-gray-700 mt-8 pt-6">
                    <div class="flex flex-col md:flex-row justify-between items-center">
                        <div class="flex items-center space-x-4 mb-4 md:mb-0">
                            <div class="flex items-center space-x-2">
                                <div class="w-8 h-8 bg-blue-600 rounded flex items-center justify-center">
                                    <svg class="w-5 h-5 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                                              d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"/>
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                                              d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z"/>
                                    </svg>
                                </div>
                                <span class="font-semibold">"Intel SIMPelv2"</span>
                            </div>
                            <div class="text-gray-400 text-sm">
                                "© 2025 Kejaksaan Agung RI - Intelligence Division"
                            </div>
                        </div>

                        <div class="flex items-center space-x-6 text-sm text-gray-400">
                            <span class="flex items-center space-x-1">
                                <div class="w-2 h-2 bg-green-500 rounded-full"></div>
                                <span>"System Secure"</span>
                            </span>
                            <span class="flex items-center space-x-1">
                                <div class="w-2 h-2 bg-blue-500 rounded-full animate-pulse"></div>
                                <span>"Monitoring Active"</span>
                            </span>
                            <span class="flex items-center space-x-1">
                                <div class="w-2 h-2 bg-yellow-500 rounded-full"></div>
                                <span>"Threat Level: Moderate"</span>
                            </span>
                        </div>
                    </div>

                    // Security Classification Footer
                    <div class="mt-4 text-center">
                        <div class="inline-flex items-center space-x-2 px-4 py-2 bg-red-900 rounded-full border border-red-700">
                            <svg class="w-4 h-4 text-red-300" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                                      d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z"/>
                            </svg>
                            <span class="text-red-300 text-sm font-medium">
                                "CLASSIFIED SYSTEM - AUTHORIZED PERSONNEL ONLY"
                            </span>
                        </div>
                    </div>
                </div>
            </div>
        </footer>
    }
}
