<?php

namespace App\Services;

interface LLMProvider
{
    public function getAnswer(string $prompt, array $options = []): string;
}
