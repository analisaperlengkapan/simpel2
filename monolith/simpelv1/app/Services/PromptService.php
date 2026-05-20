<?php

namespace App\Services;

use Illuminate\Support\Str;

class PromptService
{
    protected $aiDataService;

    public function __construct(AIDataService $aiDataService)
    {
        $this->aiDataService = $aiDataService;
    }

    public function buildPrompt($userPrompt, $history, $faq, $context)
    {
        $aiData = $this->aiDataService->getRaw();
        $prompt = "Anda adalah asisten AI SIMPEL Kejaksaan RI. Jawaban Anda WAJIB mengikuti format dan aturan berikut (jangan pernah melanggar!):\n\n";
        $prompt .= "1. Jawaban WAJIB terdiri dari dua bagian:\n   a. DEFINISI: [definisi singkat, gunakan keyword utama jika ada]\n   b. PENJELASAN: [penjelasan minimal 2 kalimat, >30 karakter, WAJIB mengandung keyword penting dari pertanyaan/FAQ/konteks, tidak boleh mengulang definisi, tidak boleh mengandung disclaimer seperti 'tidak tahu', 'tidak dapat membantu', 'tidak ditemukan', dsb.]\n";
        $prompt .= "2. Jika tidak ada data, tetap buat jawaban terstruktur dan informatif, JANGAN PERNAH menjawab 'tidak tahu', 'tidak ditemukan', atau sejenisnya.\n";
        $prompt .= "3. Gunakan bahasa Indonesia yang jelas, padat, dan profesional.\n";
        $prompt .= "4. Jawaban WAJIB mengandung minimal satu keyword penting dari pertanyaan/FAQ/konteks.\n";
        $prompt .= "5. Format WAJIB:\nDEFINISI: ...\nPENJELASAN: ...\n";
        $prompt .= "6. SEBELUM menulis jawaban akhir, lakukan penalaran (reasoning) langkah demi langkah secara internal, lalu tuliskan jawaban akhir dengan format di atas.\n";
        $prompt .= "7. Jika pertanyaan membutuhkan analisis, uraikan secara ringkas alasan atau proses berpikir Anda dalam PENJELASAN.\n";
        $prompt .= "\nContoh reasoning:\nPertanyaan: Apa itu BMN?\nReasoning: BMN sering disebut dalam konteks aset negara. Berdasarkan peraturan, BMN adalah ...\nJawaban:\nDEFINISI: BMN adalah Barang Milik Negara.\nPENJELASAN: BMN merupakan aset milik pemerintah yang digunakan untuk mendukung tugas dan fungsi instansi. Pengelolaan BMN diatur dalam peraturan perundang-undangan.\n";
        $prompt .= "DEFINISI: SIMPEL adalah Sistem Informasi Manajemen Pengelolaan Barang Milik Negara Kejaksaan RI.\nPENJELASAN: SIMPEL digunakan untuk mencatat, memantau, dan mengelola aset Kejaksaan secara terintegrasi. Sistem ini membantu transparansi dan akuntabilitas pengelolaan BMN.\n";
        if (! empty($aiData['prompt_templates']['acronym_rules'])) {
            $prompt .= "\n\nAturan akronim: ".implode(', ', array_slice($aiData['prompt_templates']['acronym_rules'], 0, 3));
        }
        // Tambahkan history
        $historyContext = $this->formatHistory($history);
        // Tambahkan FAQ dan context
        $faqText = $faq ? ('FAQ: '.$faq['answer']) : '';
        $contextText = $context ? ("Konteks: $context") : '';
        $promptText = "$prompt\n$historyContext\nPertanyaan: $userPrompt\n$faqText\n$contextText";

        return Str::limit($promptText, 3000, ' [...]');
    }

    private function formatHistory($history)
    {
        if (empty($history)) {
            return '';
        }
        $recent = array_slice($history, -2);
        $str = "\n### RIWAYAT PERCAKAPAN TERAKHIR:\n";
        foreach ($recent as $entry) {
            $role = $entry['role'] === 'user' ? 'User' : 'AI';
            $str .= "$role: {$entry['content']}\n";
        }

        return Str::limit($str, 300, ' [...]');
    }
}
