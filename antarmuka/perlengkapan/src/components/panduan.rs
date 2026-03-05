//! Panduan Pengguna — user guide page.

use leptos::prelude::*;

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
        <div style="max-width: 900px; margin: 0 auto;">
            <div style="margin-bottom: 28px;">
                <h1 style="font-size: 1.4rem; font-weight: 800; color: #e2e8f0; margin: 0;">"Panduan Pengguna"</h1>
                <p style="font-size: 0.8rem; color: #64748b; margin-top: 4px;">"Dokumentasi penggunaan SIMPEL — Sistem Informasi Manajemen Perlengkapan"</p>
            </div>

            {guides.into_iter().map(|(title, icon, desc, steps)| {
                view! {
                    <div style="background: rgba(255,255,255,0.03); border: 1px solid rgba(255,255,255,0.06); border-radius: 16px; padding: 24px; margin-bottom: 16px;">
                        <div style="display: flex; align-items: center; gap: 12px; margin-bottom: 12px;">
                            <i class=icon style="color: #d4a843; font-size: 1rem; width: 20px; text-align: center;"></i>
                            <h2 style="font-size: 1rem; font-weight: 700; color: #e2e8f0; margin: 0;">{title}</h2>
                        </div>
                        <p style="font-size: 0.82rem; color: #94a3b8; margin-bottom: 14px; line-height: 1.5;">{desc}</p>
                        <ul style="list-style: none; padding: 0; margin: 0;">
                            {steps.into_iter().map(|step| view! {
                                <li style="display: flex; align-items: flex-start; gap: 8px; padding: 6px 0; font-size: 0.8rem; color: #cbd5e1;">
                                    <i class="fas fa-check" style="color: #34d399; font-size: 0.65rem; margin-top: 4px; flex-shrink: 0;"></i>
                                    <span>{step}</span>
                                </li>
                            }).collect_view()}
                        </ul>
                    </div>
                }
            }).collect_view()}
        </div>
    }
}
