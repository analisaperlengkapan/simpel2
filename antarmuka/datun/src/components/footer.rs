//! Footer component for Datun Criminal Prosecution System

use leptos::prelude::*;
use shared_microfrontend::prelude::*;

/// Footer component khusus untuk sistem Datun
#[component]
pub fn DatunFooter() -> impl IntoView {
    view! {
        <footer class="bg-kejaksaan-secondary text-white mt-16">
            <div class="container mx-auto px-4 py-12">
                <div class="grid grid-cols-1 md:grid-cols-4 gap-8">

                    // Criminal Prosecution Info
                    <div class="space-y-4">
                        <div class="flex items-center space-x-3">
                            <div class="w-10 h-10 bg-kejaksaan-primary rounded-lg flex items-center justify-center">
                                <span class="text-xl">"⚖️"</span>
                            </div>
                            <div>
                                <h3 class="text-lg font-bold">"Datun"</h3>
                                <p class="text-sm text-gray-300">"Justice Through Law"</p>
                            </div>
                        </div>
                        <p class="text-sm text-gray-300 leading-relaxed">
                            "Direktorat Tindak Pidana Umum Kejaksaan Agung RI yang berkomitmen "
                            "menegakkan keadilan melalui penuntutan pidana yang professional."
                        </p>
                        <div class="flex space-x-3">
                            <SocialLink icon="📧" href="mailto:datun@kejaksaan.go.id" label="Email" />
                            <SocialLink icon="📞" href="tel:+622178050001" label="Phone" />
                            <SocialLink icon="🌐" href="https://datun.kejaksaan.go.id" label="Website" />
                        </div>
                    </div>

                    // Criminal Categories
                    <div>
                        <h4 class="text-lg font-semibold mb-4">"🎯 Kategori Tindak Pidana"</h4>
                        <ul class="space-y-2 text-sm text-gray-300">
                            <FooterLink href="/economic-crime" text="Tindak Pidana Ekonomi" />
                            <FooterLink href="/environmental-crime" text="Tindak Pidana Lingkungan" />
                            <FooterLink href="/narcotics" text="Tindak Pidana Narkotika" />
                            <FooterLink href="/cybercrime" text="Tindak Pidana Siber" />
                            <FooterLink href="/violence" text="Tindak Pidana Kekerasan" />
                            <FooterLink href="/property-crime" text="Tindak Pidana Properti" />
                        </ul>
                    </div>

                    // Legal Resources
                    <div>
                        <h4 class="text-lg font-semibold mb-4">"📚 Sumber Hukum"</h4>
                        <ul class="space-y-2 text-sm text-gray-300">
                            <FooterLink href="/legal-database" text="Database Hukum" />
                            <FooterLink href="/case-precedents" text="Yurisprudensi" />
                            <FooterLink href="/legal-analysis" text="Analisis Hukum" />
                            <FooterLink href="/prosecution-guidelines" text="Pedoman Penuntutan" />
                            <FooterLink href="/evidence-management" text="Manajemen Barang Bukti" />
                            <FooterLink href="/legal-training" text="Pelatihan Hukum" />
                        </ul>
                    </div>

                    // Support & Contact
                    <div>
                        <h4 class="text-lg font-semibold mb-4">"📞 Kontak & Dukungan"</h4>
                        <div class="space-y-3 text-sm text-gray-300">
                            <ContactInfo icon="📍" label="Alamat"
                                        value="Jl. Sultan Hasanudin No. 1, Kebayoran Baru, Jakarta Selatan" />
                            <ContactInfo icon="📞" label="Telpon" value="(021) 7805001 ext. 234" />
                            <ContactInfo icon="📠" label="Fax" value="(021) 7805234" />
                            <ContactInfo icon="📧" label="Email" value="datun@kejaksaan.go.id" />
                            <ContactInfo icon="🕒" label="Jam Operasional"
                                        value="Senin - Jumat: 08:00 - 16:00 WIB" />
                        </div>

                        // Emergency Contact
                        <div class="mt-4 p-3 bg-kejaksaan-primary rounded-lg">
                            <h5 class="font-medium mb-2">"🚨 Kontak Darurat"</h5>
                            <p class="text-sm text-gray-200">"Hotline 24/7: 110"</p>
                            <p class="text-sm text-gray-200">"SMS: +62 812-1000-110"</p>
                            <p class="text-sm text-gray-200">"Email: emergency@datun.kejaksaan.go.id"</p>
                        </div>
                    </div>
                </div>

                // Prosecution Statistics Bar
                <div class="border-t border-gray-700 mt-8 pt-6">
                    <div class="grid grid-cols-2 md:grid-cols-4 gap-4 text-center">
                        <StatItem icon="⚖️" value="2,847" label="Perkara Selesai" />
                        <StatItem icon="👨‍⚖️" value="127" label="Jaksa Aktif" />
                        <StatItem icon="📊" value="94%" label="Tingkat Keberhasilan" />
                        <StatItem icon="🏆" value="98%" label="Kepuasan Publik" />
                    </div>
                </div>

                // Footer Bottom
                <div class="border-t border-gray-700 mt-8 pt-6 flex flex-col md:flex-row justify-between items-center">
                    <div class="text-sm text-gray-400 mb-4 md:mb-0">
                        "© 2025 Datun Kejaksaan Agung RI. Seluruh hak cipta dilindungi."
                    </div>
                    <div class="flex space-x-6 text-sm text-gray-400">
                        <FooterLink href="/privacy" text="Kebijakan Privasi" />
                        <FooterLink href="/terms" text="Syarat Layanan" />
                        <FooterLink href="/accessibility" text="Aksesibilitas" />
                        <FooterLink href="/sitemap" text="Peta Situs" />
                    </div>
                </div>

                // Government Compliance
                <div class="border-t border-gray-700 mt-6 pt-6 text-center">
                    <div class="flex justify-center items-center space-x-6 text-gray-400 text-xs">
                        <div class="flex items-center space-x-2">
                            <span>"⚖️"</span>
                            <span>"Kejaksaan Agung RI"</span>
                        </div>
                        <div class="flex items-center space-x-2">
                            <span>"🛡️"</span>
                            <span>"ISO 27001:2013"</span>
                        </div>
                        <div class="flex items-center space-x-2">
                            <span>"♿"</span>
                            <span>"WCAG 2.1 AA"</span>
                        </div>
                        <div class="flex items-center space-x-2">
                            <span>"🇮🇩"</span>
                            <span>"Bahasa Indonesia"</span>
                        </div>
                    </div>
                </div>
            </div>
        </footer>
    }
}

/// Component untuk social media link
#[component]
fn SocialLink(
    /// Icon emoji
    icon: &'static str,
    /// URL tujuan
    href: &'static str,
    /// Label untuk accessibility
    label: &'static str,
) -> impl IntoView {
    view! {
        <a href=href
           class="w-8 h-8 bg-kejaksaan-primary rounded-lg flex items-center justify-center hover:bg-kejaksaan-primary-light transition-colors"
           title=label>
            <span class="text-sm">{icon}</span>
        </a>
    }
}

/// Component untuk footer link
#[component]
fn FooterLink(
    /// URL tujuan
    href: &'static str,
    /// Teks link
    text: &'static str,
) -> impl IntoView {
    view! {
        <li>
            <a href=href class="hover:text-kejaksaan-primary-light transition-colors duration-200">
                {text}
            </a>
        </li>
    }
}

/// Component untuk informasi kontak
#[component]
fn ContactInfo(
    /// Icon emoji
    icon: &'static str,
    /// Label informasi
    label: &'static str,
    /// Nilai informasi
    value: &'static str,
) -> impl IntoView {
    view! {
        <div class="flex items-start space-x-2">
            <span class="text-kejaksaan-primary-light mt-0.5">{icon}</span>
            <div>
                <span class="font-medium">{label}": "</span>
                <span>{value}</span>
            </div>
        </div>
    }
}

/// Component untuk statistik item
#[component]
fn StatItem(
    /// Icon emoji
    icon: &'static str,
    /// Nilai statistik
    value: &'static str,
    /// Label statistik
    label: &'static str,
) -> impl IntoView {
    view! {
        <div class="text-center">
            <div class="text-2xl mb-1">{icon}</div>
            <div class="text-xl font-bold text-kejaksaan-primary-light">{value}</div>
            <div class="text-sm text-gray-400">{label}</div>
        </div>
    }
}
