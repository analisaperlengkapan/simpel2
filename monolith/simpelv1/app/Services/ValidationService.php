<?php

namespace App\Services;

use Illuminate\Support\Facades\Log;
use Illuminate\Support\Str;

class ValidationService
{
    protected $aiDataService;

    public function __construct(AIDataService $aiDataService)
    {
        $this->aiDataService = $aiDataService;
    }

    public function validateAnswer($aiAnswer, $faqAnswer = '', $context = ''): array
    {
        $lowerAnswer = Str::lower($aiAnswer);
        $hasDefinitionContent = preg_match('/definisi\s*:|\*\*definisi\*\*/i', $aiAnswer);
        $hasExplanationContent = preg_match('/penjelasan\s*:|\*\*penjelasan\*\*/i', $aiAnswer);
        $explanationValid = false;
        $explanation = '';
        $explanationSentenceCount = 0;
        if ($hasExplanationContent) {
            $explanation = preg_split('/penjelasan\s*:|\*\*penjelasan\*\*/i', $aiAnswer, 2)[1] ?? '';
            $explanation = trim(preg_replace('/definisi\s*:.*/i', '', $explanation));
            $explanationSentenceCount = preg_match_all('/[.!?]\s/', $explanation);
            $explanationValid = strlen($explanation) > 30 && $explanationSentenceCount >= 2;
        } elseif ($hasDefinitionContent) {
            $parts = preg_split('/definisi\s*:|\*\*definisi\*\*/i', $aiAnswer, 2);
            $explanation = count($parts) > 1 ? $parts[1] : '';
            $explanationSentenceCount = preg_match_all('/[.!?]\s/', $explanation);
            $explanationValid = strlen(trim($explanation)) > 30 && $explanationSentenceCount >= 2;
        }
        $faqValidation = ['passed' => true, 'missing' => []];
        if (! empty($faqAnswer)) {
            $keywords = $this->extractKeywords($faqAnswer);
            $foundCount = 0;
            $foundKeywords = [];
            foreach ($keywords as $keyword) {
                if (stripos($lowerAnswer, Str::lower($keyword)) !== false) {
                    $foundCount++;
                    $foundKeywords[] = $keyword;
                }
            }
            $faqValidation['passed'] = $foundCount >= 1;
            $faqValidation['found'] = $foundCount;
            $faqValidation['total'] = count($keywords);
            $faqValidation['found_keywords'] = $foundKeywords;
            $faqValidation['missing_keywords'] = array_diff($keywords, $foundKeywords);
        }
        $disclaimerPattern = '/tidak tahu|tidak dapat membantu|maaf, saya belum punya data|tidak tersedia|tidak ditemukan/i';
        $hasDisclaimer = preg_match($disclaimerPattern, $aiAnswer);
        $valid = $hasDefinitionContent && $hasExplanationContent && $explanationValid && $faqValidation['passed'] && ! $hasDisclaimer;
        $reasons = [];
        if (! $hasDefinitionContent) {
            $reasons[] = 'format_definisi_tidak_ada';
        }
        if (! $hasExplanationContent) {
            $reasons[] = 'format_penjelasan_tidak_ada';
        }
        if (! $explanationValid) {
            $reasons[] = 'penjelasan_tidak_cukup (min 2 kalimat, >30 char)';
        }
        if (! $faqValidation['passed']) {
            $reasons[] = 'kata_kunci_faq_hilang';
        }
        if ($hasDisclaimer) {
            $reasons[] = 'jawaban_mengandung_disclaimer';
        }
        $result = [
            'valid' => $valid,
            'reason' => implode(',', $reasons),
            'details' => [
                'ada_konten_definisi' => $hasDefinitionContent,
                'ada_konten_penjelasan' => $hasExplanationContent,
                'penjelasan_valid' => $explanationValid,
                'penjelasan_sentence_count' => $explanationSentenceCount,
                'panjang_penjelasan' => $explanationValid ? strlen($explanation) : 0,
                'validasi_faq' => $faqValidation,
                'ada_disclaimer' => $hasDisclaimer,
            ],
        ];
        Log::debug('[ValidationService] Validasi Jawaban', $result);

        return $result;
    }

    public function validateAcronyms(string $text): array
    {
        $errors = [];
        $rules = $this->aiDataService->getAcronyms()['validation'] ?? [];
        foreach ($rules as $rule) {
            $term = $rule['term'] ?? '';
            $correct = $rule['correct'] ?? $term;
            if (empty($term)) {
                continue;
            }
            foreach ($rule['incorrect'] as $incorrect) {
                if (stripos($text, $incorrect) !== false) {
                    $errors[] = [
                        'kesalahan' => "Akronim salah: '$incorrect'",
                        'koreksi' => "Harus: '$correct'",
                        'aturan' => $rule,
                    ];
                }
            }
            if (stripos($text, $term) === false && stripos($text, $correct) === false) {
                $errors[] = [
                    'kesalahan' => "Akronim resmi '$term' tidak digunakan",
                    'aturan' => $rule,
                ];
            }
        }

        return $errors;
    }

    public function regenerateAnswerWithAcronymCheck(string $answer): string
    {
        $original = $answer;
        foreach ($this->aiDataService->getAcronyms()['validation'] ?? [] as $rule) {
            foreach ($rule['incorrect'] as $incorrect) {
                $answer = str_ireplace($incorrect, $rule['correct'], $answer);
            }
        }
        foreach ($this->aiDataService->getAcronyms()['mappings'] ?? [] as $mapping) {
            $answer = str_ireplace($mapping['long'], $mapping['short'], $answer);
        }
        if ($original !== $answer) {
            Log::debug('[ValidationService] Perbaikan Akronim', ['sebelum' => Str::limit($original, 100), 'sesudah' => Str::limit($answer, 100)]);
        }

        return $answer;
    }

    private function extractKeywords(string $text): array
    {
        $words = preg_split('/\s+/', $text);
        $stopwords = ['adalah', 'dari', 'yang', 'dengan', 'untuk', 'pada', 'sebagai', 'dalam', 'oleh', 'yaitu'];
        $keywords = array_filter($words, function ($word) use ($stopwords) {
            $clean = trim($word, " .,;:!?\"'()[]{}");

            return strlen($clean) > 3 && ! in_array(Str::lower($clean), $stopwords);
        });

        return array_values(array_unique($keywords));
    }

    public function generateDirectAnswer(string $prompt, ?string $faqAnswer, string $context): string
    {
        $definition = '';
        $explanation = '';
        if (! empty($faqAnswer)) {
            $definition = $faqAnswer;
            $explanation = $this->findExplanationInContext($context, $prompt) ?: 'Penjelasan tidak ditemukan';
        } else {
            $definition = $this->findDefinitionInContext($context, $prompt);
            $explanation = $this->findExplanationInContext($context, $prompt);
        }
        $definitionPatterns = $this->aiDataService->getContextPatterns()['definition_patterns'] ?? [];
        if (! $definition && $context && ! empty($definitionPatterns)) {
            foreach ($definitionPatterns as $pattern) {
                if (preg_match("/{$pattern}/i", $context, $matches)) {
                    $definition = trim($matches[0]);
                    break;
                }
            }
        }

        return 'DEFINISI: '.($definition ?: 'Informasi tidak ditemukan')."\n".
               'PENJELASAN: '.($explanation ?: 'Silakan merujuk ke dokumen resmi terkait');
    }

    private function findDefinitionInContext(string $context, string $prompt): string
    {
        if (empty($context)) {
            return '';
        }
        $keywords = $this->extractKeywords($prompt);
        $sentences = preg_split('/(?<=[.?!])\s+/', $context, -1, PREG_SPLIT_NO_EMPTY);
        foreach ($sentences as $sentence) {
            foreach ($keywords as $keyword) {
                if (stripos($sentence, $keyword) !== false) {
                    return $sentence;
                }
            }
        }

        return '';
    }

    private function findExplanationInContext(string $context, string $prompt): string
    {
        if (empty($context)) {
            return '';
        }
        $sentences = preg_split('/(?<=[.!?])\s+/', $context, -1, PREG_SPLIT_NO_EMPTY);
        $keywords = $this->extractKeywords($prompt);
        $explanation = '';
        $count = 0;
        foreach ($sentences as $sentence) {
            $hasKeyword = false;
            foreach ($keywords as $keyword) {
                if (stripos($sentence, $keyword) !== false) {
                    $hasKeyword = true;
                    break;
                }
            }
            if ($hasKeyword) {
                $explanation .= $sentence.' ';
                $count++;
                if ($count >= 2) {
                    break;
                }
            }
        }

        return trim($explanation) ?: 'Penjelasan lebih lanjut tersedia di dokumen resmi';
    }
}
