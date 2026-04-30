<?php

namespace App\Services;

use Illuminate\Support\Facades\Cache;
use Illuminate\Support\Facades\Log;
use Illuminate\Support\Str;

class AIDataService
{
    public function load()
    {
        return Cache::rememberForever('ai_data', function() {
            $path = storage_path('app/data/ai_data.json');
            if (!file_exists($path)) {
                Log::warning('[Data AI] File tidak ditemukan', ['path' => $path]);
                return [];
            }
            $data = json_decode(file_get_contents($path), true);
            if (!is_array($data)) {
                Log::error('[Data AI] Format tidak valid', ['contoh' => Str::limit(file_get_contents($path), 100)]);
                return [];
            }
            Log::debug('[Data AI] Berhasil dimuat', ['jumlah_faq' => count($data['faqs'] ?? [])]);
            return $data;
        });
    }

    public function getFaqs() { return $this->load()['faqs'] ?? []; }
    public function getAcronyms() { return $this->load()['acronyms'] ?? []; }
    public function getPromptTemplates() { return $this->load()['prompt_templates'] ?? []; }
    public function getContextPatterns() { return $this->load()['context_patterns'] ?? []; }
    public function getPriorityKeywords() { return $this->load()['priority_keywords'] ?? []; }
    public function getRaw() { return $this->load(); }
} 