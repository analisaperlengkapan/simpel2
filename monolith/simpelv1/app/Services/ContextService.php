<?php

namespace App\Services;

use Illuminate\Support\Facades\Cache;
use Illuminate\Support\Facades\Log;
use Illuminate\Support\Str;
use Illuminate\Support\Facades\Http;
use PhpOffice\PhpWord\IOFactory as WordIOFactory;
use PhpOffice\PhpSpreadsheet\IOFactory as ExcelIOFactory;
use Smalot\PdfParser\Parser as PdfParser;

class ContextService
{
    protected $aiDataService;

    public function __construct(AIDataService $aiDataService)
    {
        $this->aiDataService = $aiDataService;
    }

    public function retrieveContext($query)
    {
        Log::debug('[ContextService] Ambil Konteks', ['query' => $query]);
        $dir = storage_path('app/data');
        if (!is_dir($dir)) return '';
        $context = '';
        $fileCount = 0;
        $matchCount = 0;
        $queryKeywords = array_filter(
            preg_split('/\s+/', preg_replace('/[^a-z0-9\s]/i', '', $query))
        );
        $priorityKeywords = $this->aiDataService->getPriorityKeywords();
        $allKeywords = array_unique(array_merge($queryKeywords, $priorityKeywords));
        $files = array_slice(scandir($dir), 0, 20);
        foreach ($files as $file) {
            if ($file === 'ai_data.json' || $file === '.' || $file === '..') continue;
            $path = "$dir/$file";
            if (!is_file($path)) continue;
            $fileCount++;
            $ext = strtolower(pathinfo($file, PATHINFO_EXTENSION));
            $text = '';
            try {
                switch ($ext) {
                    case 'txt': case 'md':
                        $text = file_get_contents($path);
                        break;
                    case 'csv':
                        $rows = array_map('str_getcsv', file($path));
                        $header = array_shift($rows);
                        $limit = 50; $count = 0;
                        foreach ($rows as $row) {
                            if (count($header) !== count($row)) continue;
                            $text .= implode(' ', array_combine($header, $row)) . "\n";
                            if (++$count >= $limit) break;
                        }
                        break;
                    case 'pdf':
                        $parser = new PdfParser();
                        $pdf = $parser->parseFile($path);
                        $text = $pdf->getText();
                        break;
                    case 'docx':
                        $phpWord = WordIOFactory::load($path);
                        foreach ($phpWord->getSections() as $sec) {
                            foreach ($sec->getElements() as $el) {
                                if (method_exists($el, 'getText')) {
                                    $text .= $el->getText() . "\n";
                                }
                            }
                        }
                        break;
                    case 'rtf':
                        $text = strip_tags(file_get_contents($path));
                        break;
                    case 'xlsx': case 'xls':
                        $reader = ExcelIOFactory::createReaderForFile($path);
                        $reader->setReadDataOnly(true);
                        $spreadsheet = $reader->load($path);
                        $sheet = $spreadsheet->getActiveSheet();
                        $rows = $sheet->toArray();
                        $limit = 50; $count = 0;
                        foreach ($rows as $row) {
                            $text .= implode(' ', $row) . "\n";
                            if (++$count >= $limit) break;
                        }
                        $spreadsheet->disconnectWorksheets();
                        unset($spreadsheet);
                        break;
                }
                $sentences = preg_split('/(?<=[.!?])\s+/', $text, -1, PREG_SPLIT_NO_EMPTY);
                $matchedSentences = [];
                foreach ($sentences as $sentence) {
                    $sentenceLower = Str::lower($sentence);
                    foreach ($allKeywords as $keyword) {
                        $lowerKeyword = Str::lower($keyword);
                        if (stripos($sentenceLower, $lowerKeyword) !== false) {
                            $matchedSentences[] = $sentence;
                            break;
                        }
                    }
                    if (count($matchedSentences) >= 2) break;
                }
                if (!empty($matchedSentences)) {
                    $context .= "\n---\n" . implode(" ", $matchedSentences);
                    $matchCount++;
                }
            } catch (\Throwable $e) {
                Log::error('[ContextService] Error File', ['file' => $file, 'error' => $e->getMessage()]);
            }
        }
        $result = trim(Str::limit($context, 500));
        Log::debug('[ContextService] Selesai', ['file_diproses' => $fileCount, 'file_cocok' => $matchCount, 'panjang_konteks' => strlen($result)]);
        return $result;
    }

    public function searchOnline($query)
    {
        $apiKey = config('app.google_api_key');
        $cseId = config('app.google_cse_id');
        if (empty($apiKey) || empty($cseId)) {
            Log::warning('[ContextService] Google API tidak dikonfigurasi');
            return '';
        }
        $cacheKey = 'google_search_' . md5($query);
        return Cache::remember($cacheKey, now()->addHours(24), function () use ($apiKey, $cseId, $query) {
            try {
                $response = Http::timeout(10)->get('https://www.googleapis.com/customsearch/v1', [
                    'key' => $apiKey,
                    'cx' => $cseId,
                    'q' => $query,
                    'num' => 2
                ]);
                $data = $response->json();
                $items = $data['items'] ?? [];
                if (empty($items)) return '';
                $context = '';
                foreach ($items as $item) {
                    $context .= "### {$item['title']}\n{$item['snippet']}\n\n";
                }
                return trim(Str::limit($context, 300));
            } catch (\Throwable $e) {
                Log::error('[ContextService] Error Google Search', ['error' => $e->getMessage()]);
                return '';
            }
        });
    }

    public function mergeContexts($localContext, $onlineContext)
    {
        $maxContextLength = 500;
        $merged = trim($localContext . "\n\n[Sumber Online]\n" . $onlineContext);
        if (strlen($merged) > $maxContextLength) {
            $localLength = strlen($localContext);
            $allowedOnlineLength = $maxContextLength - $localLength - 50;
            if ($allowedOnlineLength > 0) {
                $onlineContext = Str::limit($onlineContext, $allowedOnlineLength, ' [...]');
                $merged = trim($localContext . "\n\n[Sumber Online]\n" . $onlineContext);
            } else {
                $merged = Str::limit($localContext, $maxContextLength, ' [...]');
            }
        }
        return $merged;
    }
} 