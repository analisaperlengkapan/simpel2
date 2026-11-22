//! FAQ — Frequently Asked Questions page.

use crate::components::layout::PageLayout;
use leptos::prelude::*;
use lib_ui::components::icon::AppIcon;
use phosphor_leptos::CARET_DOWN;

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
            "Ada 6 role: Operator Satker (mengelola perlengkapan di satuan kerja), Validator Satker (memverifikasi usulan izin pemakaian BMN di satuan kerja), Approver Satker (kuasa pengguna barang: menyetujui izin pemakaian BMN), Validator Wilayah (verifikasi tingkat Kejaksaan Tinggi), Validator Pusat (verifikasi akhir di Kejaksaan Agung), dan Administrator (data master, template, audit log, pengguna, dan role). Rincian hak akses tiap role ada di Admin → Otorisasi.",
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
        <PageLayout
            title="FAQ"
            icon="fas fa-circle-question"
            description="Pertanyaan yang sering diajukan tentang SIMPEL"
        >
            <div class="overflow-hidden rounded-2xl border border-white/[0.06] bg-surface-panel">
                {faqs
                    .into_iter()
                    .enumerate()
                    .map(|(i, (question, answer))| {
                        view! {
                            <div class="border-b border-white/[0.04] last:border-b-0">
                                <button
                                    type="button"
                                    on:click=move |_| {
                                        open_idx
                                            .update(|o| *o = if *o == Some(i) { None } else { Some(i) })
                                    }
                                    class="flex w-full items-center justify-between px-5 py-4 text-left transition hover:bg-white/[0.02]"
                                >
                                    <span class="text-sm font-semibold text-slate-100">
                                        {question}
                                    </span>
                                    <span
                                        class="inline-flex text-slate-500 transition-transform"
                                        class:rotate-180=move || open_idx.get() == Some(i)
                                    >
                                        <AppIcon icon=CARET_DOWN size=10 />
                                    </span>
                                </button>
                                // The open panel takes the height of its answer (a grid row going from
                                // `0fr` to `1fr`), not a fixed `max-h-52`: 208px held about eight lines, so
                                // an answer longer than that was cut off on a phone with no way to scroll
                                // to the rest.
                                <div class=move || {
                                    if open_idx.get() == Some(i) {
                                        "grid grid-rows-[1fr] transition-[grid-template-rows] duration-200"
                                    } else {
                                        "grid grid-rows-[0fr] transition-[grid-template-rows] duration-200"
                                    }
                                }>
                                    <div class="overflow-hidden">
                                        <p class="px-5 pb-4 text-sm leading-relaxed text-slate-400">
                                            {answer}
                                        </p>
                                    </div>
                                </div>
                            </div>
                        }
                    })
                    .collect_view()}
            </div>
        </PageLayout>
    }
}
