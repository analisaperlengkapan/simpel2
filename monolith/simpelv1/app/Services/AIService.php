<?php

namespace App\Services;

use Illuminate\Support\Facades\Log;

class AIService
{
    protected $aiDataService;
    protected $faqService;
    protected $contextService;
    protected $promptService;
    protected $validationService;
    protected $llmProvider;

    public function __construct(
        AIDataService $aiDataService,
        FAQService $faqService,
        ContextService $contextService,
        PromptService $promptService,
        ValidationService $validationService,
        LLMProvider $llmProvider // LLMProvider: service untuk call ke OpenAI/Ollama
    ) {
        $this->aiDataService = $aiDataService;
        $this->faqService = $faqService;
        $this->contextService = $contextService;
        $this->promptService = $promptService;
        $this->validationService = $validationService;
        $this->llmProvider = $llmProvider;
    }

    /**
     * Entry point utama untuk menjawab pertanyaan user.
     * @param string $question
     * @param array $options (opsional: user_id, session, dsb)
     * @return array ['answer' => string, 'source' => string, 'log' => array]
     */
    public function answerQuestion(string $question, array $options = []): array
    {
        $log = [
            'question' => $question,
            'steps' => []
        ];
        try {
            Log::debug('[AIService] Mulai answerQuestion', ['question' => $question, 'options' => $options]);
            // 1. Cari FAQ
            $faq = $this->faqService->getFaqItem($question);
            Log::debug('[AIService] FAQ ditemukan', ['faq' => $faq]);
            $log['steps'][] = ['faq' => $faq];
            // 2. Cari context
            $context = $this->contextService->retrieveContext($question);
            Log::debug('[AIService] Context ditemukan', ['context' => $context]);
            $log['steps'][] = ['context' => $context];
            // 3. Build prompt
            $prompt = $this->promptService->buildPrompt($question, [], $faq, $context);
            Log::debug('[AIService] Prompt dibangun', ['prompt' => $prompt]);
            $log['steps'][] = ['prompt' => $prompt];
            // 4. Panggil LLM
            $aiAnswer = $this->llmProvider->getAnswer($prompt, $options);
            Log::debug('[AIService] Jawaban LLM', ['ai_answer' => $aiAnswer]);
            $log['steps'][] = ['ai_answer' => $aiAnswer];
            // 5. Validasi jawaban
            $validation = $this->validationService->validateAnswer($aiAnswer, $faq['answer'] ?? '', $context);
            Log::debug('[AIService] Validasi jawaban', ['validation' => $validation]);
            $log['steps'][] = ['validation' => $validation];
            // 6. Fallback jika perlu
            $finalAnswer = $aiAnswer;
            $source = 'ai';
            if (!$validation['valid']) {
                $fallback = $this->validationService->generateDirectAnswer($question, $faq['answer'] ?? '', $context);
                Log::debug('[AIService] Fallback digunakan', ['fallback' => $fallback]);
                $log['steps'][] = ['fallback' => $fallback];
                $finalAnswer = $fallback;
                $source = 'fallback';
            }
            // 7. Validasi akronim dan perbaiki jika perlu
            $acronymErrors = $this->validationService->validateAcronyms($finalAnswer);
            if (!empty($acronymErrors)) {
                $fixed = $this->validationService->regenerateAnswerWithAcronymCheck($finalAnswer);
                Log::debug('[AIService] Akronim diperbaiki', ['fixed' => $fixed, 'acronym_errors' => $acronymErrors]);
                $log['steps'][] = ['acronym_fix' => $fixed, 'acronym_errors' => $acronymErrors];
                $finalAnswer = $fixed;
            }
            // 8. Logging akhir
            Log::info('[AIService] Final Answer', [
                'question' => $question,
                'answer' => $finalAnswer,
                'source' => $source,
                'log' => $log
            ]);
            return [
                'answer' => $finalAnswer,
                'source' => $source,
                'log' => $log
            ];
        } catch (\Throwable $e) {
            Log::error('[AIService] ERROR', [
                'question' => $question,
                'error' => $e->getMessage(),
                'trace' => $e->getTraceAsString()
            ]);
            return [
                'answer' => '',
                'source' => 'error',
                'log' => $log,
                'error' => $e->getMessage()
            ];
        }
    }
} 