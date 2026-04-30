<?php

namespace App\Services;

use Illuminate\Support\Str;

class FAQService
{
    protected $aiDataService;

    public function __construct(AIDataService $aiDataService)
    {
        $this->aiDataService = $aiDataService;
    }

    public function getFaqItem($query)
    {
        $faqs = $this->aiDataService->getFaqs();
        $acronymMappings = $this->aiDataService->getAcronyms()['mappings'] ?? [];
        $expandedFaqs = [];
        foreach ($faqs as $item) {
            $expandedFaqs[] = $item;
            if ($item['keyword'] === 'bmn') {
                $expandedFaqs[] = ['keyword' => 'barang milik negara', 'answer' => $item['answer']];
                $expandedFaqs[] = ['keyword' => 'aset negara', 'answer' => $item['answer']];
            }
        }
        usort($expandedFaqs, function($a, $b) {
            $aCount = count(explode(' ', $a['keyword']));
            $bCount = count(explode(' ', $b['keyword']));
            if ($aCount > 1 && $bCount <= 1) return -1;
            if ($bCount > 1 && $aCount <= 1) return 1;
            return strlen($b['keyword']) <=> strlen($a['keyword']);
        });
        $lowerQuery = Str::lower($query);
        // 1. Pencocokan tepat
        foreach ($expandedFaqs as $item) {
            $lowerKeyword = Str::lower($item['keyword']);
            $pattern = '/\\b' . preg_quote($lowerKeyword, '/') . '\\b/';
            if (preg_match($pattern, $lowerQuery)) {
                return $item;
            }
        }
        // 2. Terapkan pemetaan akronim
        $mappedQuery = $lowerQuery;
        foreach ($acronymMappings as $mapping) {
            $short = Str::lower($mapping['short']);
            $long = Str::lower($mapping['long']);
            if (strpos($mappedQuery, $short) !== false) {
                $mappedQuery = str_replace($short, $long, $mappedQuery);
            }
        }
        // 3. Cari dengan query yang sudah dipetakan
        foreach ($expandedFaqs as $item) {
            $lowerKeyword = Str::lower($item['keyword']);
            $pattern = '/\\b' . preg_quote($lowerKeyword, '/') . '\\b/';
            if (preg_match($pattern, $mappedQuery)) {
                return $item;
            }
        }
        return null;
    }
} 