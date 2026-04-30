<?php

namespace App\Providers;

use Illuminate\Support\ServiceProvider;
use App\Services\LLMProvider;
use App\Services\OllamaLLMProvider;

class AppServiceProvider extends ServiceProvider
{
    /**
     * Register any application services.
     */
    public function register(): void
    {
        $this->app->bind(LLMProvider::class, OllamaLLMProvider::class);
        $this->app->singleton(\App\Services\AIDataService::class);
    }

    /**
     * Bootstrap any application services.
     */
    public function boot(): void
    {
        //
    }
}
