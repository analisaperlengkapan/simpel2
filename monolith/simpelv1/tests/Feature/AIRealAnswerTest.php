<?php

declare(strict_types=1);

namespace Tests\Feature;

use App\Models\Pengguna\Pengguna;
use Tests\TestCase;

class AIRealAnswerTest extends TestCase
{
    /**
     * @dataProvider questionProvider
     */
    public function test_ai_real_answer_quality($question, $minKeywords, $desc)
    {
        // Login user (pastikan ada user dengan email 'admin@example.com' atau sesuaikan)
        $user = Pengguna::first();
        if (! $user) {
            $user = Pengguna::create([
                'name' => 'Test User',
                'username' => 'testuser',
                'email' => 'testuser@example.com',
                'password' => bcrypt('password'),
            ]);
        }
        $this->actingAs($user);
        $response = $this->post('/ai/chat', [
            'prompt' => $question,
        ]);
        $response->assertStatus(200);
        $data = $response->json();
        $answer = $data['response'] ?? '';
        $this->assertMatchesRegularExpression('/^DEFINISI:.*PENJELASAN:/s', $answer, "[$desc] Format tidak sesuai");
        preg_match('/PENJELASAN:(.*)$/s', $answer, $matches);
        $explanation = trim($matches[1] ?? '');
        $this->assertGreaterThanOrEqual(30, strlen($explanation), "[$desc] Penjelasan terlalu pendek");
        $this->assertFalse(
            preg_match('/tidak tahu|tidak dapat membantu|maaf, saya belum punya data|tidak tersedia|tidak ditemukan/i', $answer),
            "[$desc] Jawaban mengandung disclaimer"
        );
        $found = false;
        foreach ($minKeywords as $kw) {
            if (stripos($answer, $kw) !== false) {
                $found = true;
                break;
            }
        }
        $this->assertTrue($found, "[$desc] Jawaban tidak mengandung minimal satu keyword penting: ".implode(', ', $minKeywords));
    }

    public static function questionProvider()
    {
        return [
            // Mudah
            ['Apa itu BMN?', ['BMN', 'Barang Milik Negara'], 'Definisi BMN'],
            ['Jelaskan singkatan SIMPEL.', ['SIMPEL'], 'Singkatan SIMPEL'],
            ['Apa tugas Kejaksaan RI?', ['Kejaksaan'], 'Tugas Kejaksaan'],
            // Menengah
            ['Bagaimana pengelolaan BMN di instansi pemerintah?', ['BMN', 'pengelolaan'], 'Pengelolaan BMN'],
            ['Apa perbedaan aset tetap dan BMN?', ['aset', 'BMN'], 'Perbedaan aset tetap dan BMN'],
            ['Bagaimana prosedur penghapusan aset negara?', ['penghapusan', 'aset'], 'Prosedur penghapusan aset'],
            // Sulit
            ['Bagaimana jika BMN rusak berat dan tidak tercatat?', ['BMN', 'rusak'], 'BMN rusak berat'],
            ['Apa hubungan antara BMN, SIMPEL, dan audit internal?', ['BMN', 'SIMPEL', 'audit'], 'Hubungan BMN, SIMPEL, audit'],
            ['Jelaskan BMN dalam konteks digitalisasi dan keamanan data.', ['BMN', 'digitalisasi', 'keamanan'], 'BMN dan digitalisasi'],
            // Ambigu
            ['asdfghjkl', ['definisi', 'penjelasan'], 'Pertanyaan tidak jelas'],
            ['?', ['definisi', 'penjelasan'], 'Pertanyaan tanda tanya'],
            ['Ceritakan sesuatu.', ['definisi', 'penjelasan'], 'Pertanyaan sangat umum'],
            // Edge-case
            ['Apa itu XYZ123 yang tidak ada di data?', ['definisi', 'penjelasan'], 'Istilah tidak ada di data'],
            ['Jelaskan peran BMN dalam era revolusi industri 4.0.', ['BMN', 'industri'], 'BMN dan industri 4.0'],
            ['Bagaimana BMN mendukung pelayanan publik?', ['BMN', 'pelayanan'], 'BMN dan pelayanan publik'],
        ];
    }
}
