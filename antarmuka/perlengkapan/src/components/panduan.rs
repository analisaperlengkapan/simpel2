//! Panduan Pengguna — user guide page.

use crate::components::layout::{PageLayout, SectionCard};
use leptos::prelude::*;
use lib_ui::components::icon::{AppIcon, icon_from_fa_class};
use phosphor_leptos::CHECK;

#[component]
pub fn PanduanPengguna() -> impl IntoView {
    let guides = vec![
        (
            "Dashboard",
            "fas fa-home",
            "Melihat ringkasan statistik, akses cepat ke modul, dan status aset terkini.",
            vec![
                "Stat cards menampilkan data real-time dari backend",
                "Klik modul untuk navigasi langsung",
                "Ganti role aktif via menu profil di kanan atas",
            ],
        ),
        (
            "Bank Aset",
            "fas fa-boxes",
            "Katalog lengkap BMN dengan pencarian, filter, dan cetak QR Code.",
            vec![
                "Gunakan filter untuk menyaring berdasarkan kategori, kondisi, atau satker",
                "Klik baris aset untuk melihat detail lengkap",
                "Cetak QR Code untuk pelabelan fisik",
            ],
        ),
        (
            "Kebutuhan BMN",
            "fas fa-clipboard-list",
            "Pengajuan analisis kebutuhan barang milik negara.",
            vec![
                "Buat pengajuan baru via menu 'Buat Baru'",
                "Isi formulir analisis kebutuhan dengan lengkap",
                "Pantau status pengajuan di halaman 'Daftar'",
                "Unduh laporan rekap di halaman 'Laporan'",
            ],
        ),
        (
            "Pakaian Dinas",
            "fas fa-tshirt",
            "Pengajuan dan distribusi pakaian dinas pegawai.",
            vec![
                "Data ukuran pegawai dikelola di menu 'Ukuran'",
                "Pengajuan pakaian dinas melalui menu 'Pengajuan'",
                "Lihat jenis pakaian tersedia di menu 'Jenis'",
            ],
        ),
        (
            "Pengelolaan BMN",
            "fas fa-cogs",
            "Pemakaian dan penghapusan BMN.",
            vec![
                "Ajukan izin pemakaian BMN via 'Pemakaian BMN'",
                "Proses penghapusan aset rusak/hilang via 'Penghapusan'",
                "Monitor status persetujuan workflow secara real-time",
            ],
        ),
    ];

    view! {
        <PageLayout
            title="Panduan Pengguna"
            icon="fas fa-book-open"
            description="Dokumentasi penggunaan SIMPEL — Sistem Informasi Manajemen Perlengkapan"
        >
            <div class="flex flex-col gap-4">
                {guides
                    .into_iter()
                    .map(|(title, icon, desc, steps)| {
                        view! {
                            <SectionCard title=title.to_string()>
                                <div class="flex items-start gap-3 mb-3">
                                    <span class="text-gold-400 inline-flex">
                                        <AppIcon icon=icon_from_fa_class(icon) size=14 />
                                    </span>
                                    <p class="text-sm leading-relaxed text-slate-400">{desc}</p>
                                </div>
                                <ul class="flex flex-col gap-1.5">
                                    {steps
                                        .into_iter()
                                        .map(|step| {
                                            view! {
                                                <li class="flex items-start gap-2 text-sm text-slate-200">
                                                    <span class="text-success-400 text-2xs mt-1 shrink-0">
                                                        <AppIcon icon=CHECK />
                                                    </span>
                                                    <span>{step}</span>
                                                </li>
                                            }
                                        })
                                        .collect_view()}
                                </ul>
                            </SectionCard>
                        }
                    })
                    .collect_view()}
            </div>
        </PageLayout>
    }
}
