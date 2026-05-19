<?php

declare(strict_types=1);

namespace Tests\Unit;

use App\Services\AIDataService;
use App\Services\AIService;
use App\Services\ContextService;
use App\Services\FAQService;
use App\Services\LLMProvider;
use App\Services\PromptService;
use App\Services\ValidationService;
use Tests\TestCase;

class AIServiceTest extends TestCase
{
    private function getMockedService(?array $faq = null, $context = null, $aiAnswer = null, $valid = true, $fallback = null, $acronymErrors = [], $acronymFix = null)
    {
        $aiDataService = $this->createMock(AIDataService::class);
        $faqService = $this->createMock(FAQService::class);
        $contextService = $this->createMock(ContextService::class);
        $promptService = $this->createMock(PromptService::class);
        $validationService = $this->createMock(ValidationService::class);
        $llmProvider = $this->createMock(LLMProvider::class);

        $faqService->method('getFaqItem')->willReturn($faq);
        $contextService->method('retrieveContext')->willReturn($context);
        $promptService->method('buildPrompt')->withAnyParameters()->willReturn('PROMPT');
        $llmProvider->method('getAnswer')->willReturn($aiAnswer);
        $validationService->method('validateAnswer')->willReturn([
            'valid' => $valid,
            'reason' => $valid ? '' : 'invalid',
            'details' => [],
        ]);
        $validationService->method('generateDirectAnswer')->willReturn($fallback ?? 'FALLBACK');
        $validationService->method('validateAcronyms')->willReturn($acronymErrors);
        $validationService->method('regenerateAnswerWithAcronymCheck')->willReturn($acronymFix ?? 'FIXED');

        return new AIService(
            $aiDataService,
            $faqService,
            $contextService,
            $promptService,
            $validationService,
            $llmProvider
        );
    }

    public function test_answer_question_valid_ai_answer()
    {
        $faq = ['keyword' => 'BMN', 'answer' => 'Barang Milik Negara'];
        $context = 'BMN adalah aset negara.';
        $aiAnswer = 'DEFINISI: BMN\nPENJELASAN: BMN adalah aset negara.';
        $service = $this->getMockedService($faq, $context, $aiAnswer, true);
        $result = $service->answerQuestion('Apa itu BMN?');
        $this->assertEquals($aiAnswer, $result['answer']);
        $this->assertEquals('ai', $result['source']);
    }

    public function test_answer_question_invalid_ai_answer_fallback()
    {
        $faq = ['keyword' => 'BMN', 'answer' => 'Barang Milik Negara'];
        $context = 'BMN adalah aset negara.';
        $aiAnswer = 'Jawaban tidak valid';
        $fallback = 'DEFINISI: Fallback\nPENJELASAN: Fallback';
        $service = $this->getMockedService($faq, $context, $aiAnswer, false, $fallback);
        $result = $service->answerQuestion('Apa itu BMN?');
        $this->assertEquals($fallback, $result['answer']);
        $this->assertEquals('fallback', $result['source']);
    }

    public function test_answer_question_acronym_fix_applied()
    {
        $faq = ['keyword' => 'BMN', 'answer' => 'Barang Milik Negara'];
        $context = 'BMN adalah aset negara.';
        $aiAnswer = 'DEFINISI: BMN\nPENJELASAN: BMN salah.';
        $acronymErrors = [['kesalahan' => 'BMN salah']];
        $acronymFix = 'DEFINISI: BMN\nPENJELASAN: BMN benar.';
        $service = $this->getMockedService($faq, $context, $aiAnswer, true, null, $acronymErrors, $acronymFix);
        $result = $service->answerQuestion('Apa itu BMN?');
        $this->assertEquals($acronymFix, $result['answer']);
    }

    public function test_answer_question_no_fa_q_no_context()
    {
        $service = $this->getMockedService(null, null, 'AI', true);
        $result = $service->answerQuestion('Pertanyaan tidak dikenal');
        $this->assertEquals('AI', $result['answer']);
    }

    public function test_answer_question_contains_definition_and_explanation()
    {
        $faq = ['keyword' => 'BMN', 'answer' => 'Barang Milik Negara adalah aset milik pemerintah.'];
        $context = 'BMN digunakan untuk pengelolaan aset negara.';
        $aiAnswer = 'DEFINISI: BMN\nPENJELASAN: BMN adalah aset negara.';
        $service = $this->getMockedService($faq, $context, $aiAnswer, true);
        $result = $service->answerQuestion('Apa itu BMN?');
        $this->assertStringContainsString('DEFINISI', $result['answer']);
        $this->assertStringContainsString('PENJELASAN', $result['answer']);
    }

    public function test_answer_question_contains_faq_keyword()
    {
        $faq = ['keyword' => 'BMN', 'answer' => 'Barang Milik Negara adalah aset milik pemerintah.'];
        $context = 'BMN digunakan untuk pengelolaan aset negara.';
        $aiAnswer = 'DEFINISI: BMN\nPENJELASAN: BMN adalah aset negara.';
        $service = $this->getMockedService($faq, $context, $aiAnswer, true);
        $result = $service->answerQuestion('Apa itu BMN?');
        $this->assertStringContainsString('Barang Milik Negara', $faq['answer']);
        $this->assertStringContainsString('BMN', $result['answer']);
    }

    public function test_answer_question_fallback_if_ai_answer_empty()
    {
        $faq = ['keyword' => 'BMN', 'answer' => 'Barang Milik Negara adalah aset milik pemerintah.'];
        $context = 'BMN digunakan untuk pengelolaan aset negara.';
        $aiAnswer = '';
        $fallback = 'DEFINISI: Fallback\nPENJELASAN: Fallback';
        $service = $this->getMockedService($faq, $context, $aiAnswer, false, $fallback);
        $result = $service->answerQuestion('Apa itu BMN?');
        $this->assertEquals($fallback, $result['answer']);
    }

    public function test_answer_question_no_disclaimer_if_data_exists()
    {
        $faq = ['keyword' => 'BMN', 'answer' => 'Barang Milik Negara adalah aset milik pemerintah.'];
        $context = 'BMN digunakan untuk pengelolaan aset negara.';
        $aiAnswer = 'DEFINISI: BMN\nPENJELASAN: BMN adalah aset negara.';
        $service = $this->getMockedService($faq, $context, $aiAnswer, true);
        $result = $service->answerQuestion('Apa itu BMN?');
        $this->assertStringNotContainsString('tidak tahu', strtolower($result['answer']));
        $this->assertStringNotContainsString('tidak dapat membantu', strtolower($result['answer']));
    }

    public function test_answer_question_acronym_is_correct()
    {
        $faq = ['keyword' => 'BMN', 'answer' => 'Barang Milik Negara adalah aset milik pemerintah.'];
        $context = 'BMN digunakan untuk pengelolaan aset negara.';
        $aiAnswer = 'DEFINISI: bmn\nPENJELASAN: bmn salah.';
        $acronymErrors = [['kesalahan' => 'bmn salah']];
        $acronymFix = 'DEFINISI: BMN\nPENJELASAN: BMN benar.';
        $service = $this->getMockedService($faq, $context, $aiAnswer, true, null, $acronymErrors, $acronymFix);
        $result = $service->answerQuestion('Apa itu BMN?');
        $this->assertStringContainsString('BMN benar', $result['answer']);
    }

    public function test_answer_question_format_always_correct()
    {
        $faq = ['keyword' => 'BMN', 'answer' => 'Barang Milik Negara adalah aset milik pemerintah.'];
        $context = 'BMN digunakan untuk pengelolaan aset negara.';
        $aiAnswer = 'DEFINISI: BMN\nPENJELASAN: BMN adalah aset negara.';
        $service = $this->getMockedService($faq, $context, $aiAnswer, true);
        $result = $service->answerQuestion('Apa itu BMN?');
        $this->assertMatchesRegularExpression('/^DEFINISI:.*PENJELASAN:/s', $result['answer']);
    }

    public function test_answer_question_explanation_length()
    {
        $faq = ['keyword' => 'BMN', 'answer' => 'Barang Milik Negara adalah aset milik pemerintah.'];
        $context = 'BMN digunakan untuk pengelolaan aset negara.';
        $aiAnswer = 'DEFINISI: BMN\nPENJELASAN: Penjelasan ini cukup panjang untuk lolos validasi.';
        $service = $this->getMockedService($faq, $context, $aiAnswer, true);
        $result = $service->answerQuestion('Apa itu BMN?');
        preg_match('/PENJELASAN:(.*)$/s', $result['answer'], $matches);
        $this->assertGreaterThanOrEqual(30, strlen(trim($matches[1] ?? '')));
    }

    public function test_answer_question_contains_at_least_one_keyword()
    {
        $faq = ['keyword' => 'BMN', 'answer' => 'Barang Milik Negara adalah aset milik pemerintah.'];
        $context = 'BMN digunakan untuk pengelolaan aset negara.';
        $aiAnswer = 'DEFINISI: BMN\nPENJELASAN: BMN adalah aset negara.';
        $service = $this->getMockedService($faq, $context, $aiAnswer, true);
        $result = $service->answerQuestion('Apa itu BMN?');
        $keywords = ['BMN', 'aset', 'negara'];
        $found = false;
        foreach ($keywords as $kw) {
            if (stripos($result['answer'], $kw) !== false) {
                $found = true;
                break;
            }
        }
        $this->assertTrue($found, 'Jawaban tidak mengandung keyword penting.');
    }

    public function test_answer_question_fallback_is_relevant()
    {
        $faq = ['keyword' => 'BMN', 'answer' => 'Barang Milik Negara adalah aset milik pemerintah.'];
        $context = 'BMN digunakan untuk pengelolaan aset negara.';
        $aiAnswer = '';
        $fallback = 'DEFINISI: Barang Milik Negara adalah aset milik pemerintah.\nPENJELASAN: BMN digunakan untuk pengelolaan aset negara.';
        $service = $this->getMockedService($faq, $context, $aiAnswer, false, $fallback);
        $result = $service->answerQuestion('Apa itu BMN?');
        $this->assertStringContainsString('Barang Milik Negara', $result['answer']);
        $this->assertStringContainsString('pengelolaan aset negara', $result['answer']);
    }

    public function test_answer_question_acronym_is_consistent()
    {
        $faq = ['keyword' => 'BMN', 'answer' => 'Barang Milik Negara adalah aset milik pemerintah.'];
        $context = 'BMN digunakan untuk pengelolaan aset negara.';
        $aiAnswer = 'DEFINISI: bmn\nPENJELASAN: bmn salah.';
        $acronymErrors = [['kesalahan' => 'bmn salah']];
        $acronymFix = 'DEFINISI: BMN\nPENJELASAN: BMN benar.';
        $service = $this->getMockedService($faq, $context, $aiAnswer, true, null, $acronymErrors, $acronymFix);
        $result = $service->answerQuestion('Apa itu BMN?');
        $this->assertMatchesRegularExpression('/DEFINISI: BMN.*PENJELASAN: BMN benar\./s', $result['answer']);
    }

    public function test_answer_question_ambiguous_question_still_structured()
    {
        $faq = null;
        $context = '';
        $aiAnswer = 'DEFINISI: Informasi tidak ditemukan\nPENJELASAN: Silakan merujuk ke dokumen resmi.';
        $service = $this->getMockedService($faq, $context, $aiAnswer, true);
        $result = $service->answerQuestion('asdfghjkl');
        $this->assertMatchesRegularExpression('/^DEFINISI:.*PENJELASAN:/s', $result['answer']);
    }

    public function test_answer_question_repeat_question_not_identical()
    {
        $faq = ['keyword' => 'BMN', 'answer' => 'Barang Milik Negara adalah aset milik pemerintah.'];
        $context = 'BMN digunakan untuk pengelolaan aset negara.';
        $aiAnswer1 = 'DEFINISI: BMN\nPENJELASAN: BMN adalah aset negara.';
        $aiAnswer2 = 'DEFINISI: BMN\nPENJELASAN: BMN adalah aset negara dan digunakan oleh pemerintah.';
        $service = $this->getMockedService($faq, $context, $aiAnswer1, true);
        $result1 = $service->answerQuestion('Apa itu BMN?');
        $service2 = $this->getMockedService($faq, $context, $aiAnswer2, true);
        $result2 = $service2->answerQuestion('Apa itu BMN?');
        $this->assertNotEquals($result1['answer'], $result2['answer']);
    }
}
