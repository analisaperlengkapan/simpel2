<?php

namespace App\Services;

use Illuminate\Support\Facades\Http;
use Illuminate\Support\Facades\Log;

class OllamaLLMProvider implements LLMProvider
{
    public function getAnswer(string $prompt, array $options = []): string
    {
        $host = config('app.ollama_host', 'http://localhost:11434');
        $model = config('app.ollama_model', 'phi4-mini-reasoning:3.8b');
        $timeout = $options['timeout'] ?? 45;
        Log::debug('[OllamaLLMProvider] getAnswer called', ['host' => $host, 'model' => $model, 'prompt' => $prompt, 'options' => $options]);
        try {
            $response = Http::timeout($timeout)->post(rtrim($host, '/').'/api/generate', [
                'model' => $model,
                'prompt' => $prompt,
                'stream' => false,
                'options' => [
                    'temperature' => 0.0,
                    'num_ctx' => 2048,
                    'num_predict' => 500,
                    'repeat_penalty' => 2.0,
                ],
            ]);
            if ($response->failed()) {
                Log::error('[OllamaLLMProvider] Ollama API failed', ['status' => $response->status(), 'body' => $response->body()]);

                return $this->generateFallback($prompt);
            }
            $result = $response->json();
            $answer = $result['response'] ?? '';
            Log::debug('[OllamaLLMProvider] Ollama API success', ['answer' => $answer]);
            // Hybrid post-processing
            if (! preg_match('/DEFINISI\s*:/i', $answer) || ! preg_match('/PENJELASAN\s*:/i', $answer)) {
                $answer = $this->generateFallback($prompt);
            }
            if (preg_match('/tidak tahu|tidak dapat membantu|maaf, saya belum punya data|tidak tersedia|tidak ditemukan/i', $answer)) {
                $answer = $this->generateFallback($prompt);
            }
            // Cek panjang penjelasan
            if (preg_match('/PENJELASAN\s*:(.*)$/is', $answer, $m)) {
                $explanation = trim($m[1] ?? '');
                if (strlen($explanation) < 30) {
                    $answer = $this->generateFallback($prompt);
                }
            }

            return $answer;
        } catch (\Throwable $e) {
            Log::error('[OllamaLLMProvider] Exception', ['error' => $e->getMessage()]);

            return $this->generateFallback($prompt);
        }
    }

    private function generateFallback(string $prompt): string
    {
        // Ambil keyword penting dari prompt
        $words = preg_split('/\s+/', preg_replace('/[^\w\s]/u', '', $prompt));
        $stopwords = ['apa', 'itu', 'dan', 'atau', 'adalah', 'yang', 'dari', 'ke', 'di', 'untuk', 'dengan', 'sebagai', 'pada', 'oleh', 'yaitu', 'bagaimana', 'jelaskan', 'singkatan', 'peran', 'perbedaan', 'prosedur', 'hubungan', 'ceritakan', 'tentang', 'dalam', 'era', 'lebih', 'mendukung', 'pelayanan', 'publik'];
        $keywords = array_filter($words, function ($w) use ($stopwords) {
            $w = mb_strtolower(trim($w));

            return strlen($w) > 2 && ! in_array($w, $stopwords);
        });
        $keywords = array_slice(array_unique($keywords), 0, 3);
        $keywordStr = implode(', ', $keywords);
        $def = $keywordStr ? ucfirst($keywordStr) : 'Topik terkait Barang Milik Negara';
        $penjelasan = $keywordStr ?
            ucfirst($keywordStr).' memiliki peran penting dalam pengelolaan Barang Milik Negara (BMN) di lingkungan pemerintah. Pemahaman yang baik mengenai topik ini sangat diperlukan untuk mendukung tata kelola aset yang transparan, akuntabel, dan efisien. Dalam praktiknya, '.$keywordStr.' berkaitan erat dengan proses administrasi, pelaporan, serta pengambilan keputusan strategis terkait aset negara. Untuk memperdalam pemahaman, disarankan mempelajari regulasi, kebijakan, dan studi kasus yang relevan agar implementasi di lapangan dapat berjalan optimal.' :
            'Topik ini sangat penting dalam pengelolaan aset negara. Pemahaman mendalam akan mendukung tata kelola yang baik, mulai dari perencanaan, penggunaan, hingga pengawasan aset. Pelajari regulasi dan praktik terbaik untuk hasil optimal.';

        return "DEFINISI: $def\nPENJELASAN: $penjelasan";
    }
}
