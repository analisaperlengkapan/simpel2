<?php

namespace App\Http\Controllers;

use App\Http\Requests\AskAIRequest;
use App\Services\AIService;

class AIController extends Controller
{
    protected $aiService;

    public function __construct(AIService $aiService)
    {
        $this->aiService = $aiService;
    }

    /**
     * Endpoint utama untuk chat/ask AI
     *
     * @param  Request  $request
     * @return \Illuminate\Http\JsonResponse
     */
    public function ask(AskAIRequest $request)
    {
        $question = $request->input('question');
        $options = $request->only(['user_id', 'session']);
        $result = $this->aiService->answerQuestion($question, $options);

        return response()->json([
            'success' => true,
            'answer' => $result['answer'],
            'source' => $result['source'],
            'log' => $result['log'],
        ]);
    }

    /**
     * Endpoint untuk chat-widget (web) - menerima 'prompt', return 'response'
     */
    public function chat(\Illuminate\Http\Request $request)
    {
        try {
            \Log::debug('[AIController] chat called', ['input' => $request->all()]);
            $request->validate([
                'prompt' => 'required|string|min:3',
            ]);
            $question = $request->input('prompt');
            $options = $request->only(['user_id', 'session']);
            $result = $this->aiService->answerQuestion($question, $options);
            if (isset($result['error'])) {
                \Log::error('[AIController] chat error', ['error' => $result['error'], 'result' => $result]);

                return response()->json([
                    'message' => $result['error'],
                    'response' => $result['answer'] ?? '',
                    'source' => $result['source'] ?? 'error',
                    'log' => $result['log'] ?? [],
                ], 500);
            }
            \Log::debug('[AIController] chat response', ['result' => $result]);

            return response()->json([
                'response' => $result['answer'],
                'source' => $result['source'],
                'log' => $result['log'],
            ]);
        } catch (\Throwable $e) {
            \Log::error('[AIController] chat exception', [
                'error' => $e->getMessage(),
                'trace' => $e->getTraceAsString(),
                'input' => $request->all(),
            ]);

            return response()->json([
                'message' => $e->getMessage(),
                'response' => '',
                'source' => 'exception',
                'log' => [],
            ], 500);
        }
    }
}
