//! FAQ — Frequently Asked Questions page.

use leptos::prelude::*;

#[component]
pub fn FaqPage() -> impl IntoView {
    let faqs = vec![
        (
            "Bagaimana cara mengajukan kebutuhan BMN?",
            "Buka menu 'Kebutuhan BMN' → 'Buat Baru', isi formulir analisis kebutuhan, lalu submit. Pengajuan akan diproses sesuai alur persetujuan (Operator → Validator Wilayah → Validator Pusat).",
        ),
        (
            "Bagaimana cara mengganti role aktif?",
            "Klik ikon profil di pojok kanan atas header, lalu pilih role yang diinginkan dari daftar yang tersedia. Perubahan langsung berlaku tanpa perlu reload.",
        ),
        (
            "Apa saja role yang tersedia?",
            "Ada 4 role: Operator Satker (input data operasional), Validator Wilayah (verifikasi tingkat wilayah), Validator Pusat (persetujuan & penerbitan SK), dan Admin (pengelolaan sistem).",
        ),
        (
            "Bagaimana cara cetak QR Code aset?",
            "Buka menu 'Bank Aset' → 'Cetak QR Code'. Pilih aset yang ingin dicetak QR-nya dengan checkbox, lalu klik tombol 'Cetak'.",
        ),
        (
            "Bagaimana proses penghapusan BMN?",
            "Buka menu 'Pengelolaan BMN' → 'Penghapusan' → 'Buat'. Isi formulir penghapusan, lampirkan dokumen pendukung. Pengajuan akan melalui alur workflow sampai terbit SK penghapusan.",
        ),
        (
            "Bagaimana cara melihat laporan?",
            "Setiap modul memiliki sub-menu 'Laporan'. Laporan bisa di-filter berdasarkan tahun dan status, serta di-export ke format XLSX atau PDF.",
        ),
        (
            "Data SIMAN tidak sinkron, bagaimana?",
            "Sinkronisasi data SIMAN dilakukan secara otomatis oleh sistem. Jika ada ketidaksesuaian, hubungi tim support melalui menu 'Bantuan' → 'Helpdesk'.",
        ),
        (
            "Bagaimana cara mengajukan pakaian dinas?",
            "Pastikan data ukuran pegawai sudah terisi di menu 'Pakaian Dinas' → 'Ukuran'. Kemudian buat pengajuan melalui menu 'Pengajuan'.",
        ),
    ];

    let open_idx = RwSignal::new(None::<usize>);

    view! {
        <div style="max-width: 800px; margin: 0 auto;">
            <div style="margin-bottom: 28px;">
                <h1 style="font-size: 1.4rem; font-weight: 800; color: #e2e8f0; margin: 0;">"FAQ"</h1>
                <p style="font-size: 0.8rem; color: #64748b; margin-top: 4px;">"Pertanyaan yang sering diajukan tentang SIMPEL"</p>
            </div>

            {faqs.into_iter().enumerate().map(|(i, (question, answer))| {
                view! {
                    <div style="border-bottom: 1px solid rgba(255,255,255,0.06);">
                        <button
                            on:click=move |_| open_idx.update(|o| *o = if *o == Some(i) { None } else { Some(i) })
                            style="width: 100%; display: flex; align-items: center; justify-content: space-between; padding: 16px 0; background: none; border: none; cursor: pointer; text-align: left;"
                        >
                            <span style="font-size: 0.88rem; font-weight: 600; color: #e2e8f0;">{question}</span>
                            <i class="fas fa-chevron-down" style=move || format!(
                                "color: #475569; font-size: 0.7rem; transform: rotate({}deg);",
                                if open_idx.get() == Some(i) { 180 } else { 0 }
                            )></i>
                        </button>
                        <div style=move || format!(
                            "overflow: hidden; max-height: {};",
                            if open_idx.get() == Some(i) { "200px" } else { "0" }
                        )>
                            <p style="font-size: 0.82rem; color: #94a3b8; line-height: 1.6; padding: 0 0 16px 0; margin: 0;">{answer}</p>
                        </div>
                    </div>
                }
            }).collect_view()}
        </div>
    }
}
