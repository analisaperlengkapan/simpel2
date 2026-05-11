<?php

namespace App\Providers;

use Illuminate\Support\Facades\URL;
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
        // SIMPEL v1 di-mount di subdirectory /perlengkapan/simpel/v1 saat
        // deploy via Istio. Tanpa forceRootUrl, helper Laravel `url()` dan
        // `redirect()` hanya pakai scheme://host (root URL dari Request),
        // sehingga prefix `/perlengkapan/simpel/v1` hilang dari link &
        // Location header. Akibatnya: user login lalu redirect ke
        // `/auth/login` (kehilangan prefix) -> 404 di Istio routing.
        //
        // Set APP_URL=http://host/perlengkapan/simpel/v1 di env, lalu
        // forceRootUrl menarik nilai itu sebagai base URL semua url()/redirect().
        $appUrl = config('app.url');
        if (!empty($appUrl)) {
            URL::forceRootUrl($appUrl);
            // Hormati TLS termination di Istio gateway: APP_URL https://
            // dipakai sebagai sinyal bahwa user-facing scheme adalah HTTPS
            // walau backend serve HTTP.
            if (strpos($appUrl, 'https://') === 0) {
                URL::forceScheme('https');
            }
        }
    }
}
