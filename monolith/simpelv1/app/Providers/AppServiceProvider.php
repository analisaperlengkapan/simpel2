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
        // Skip URL setup di CLI/artisan (request() tidak tersedia,
        // dan TIDAK perlu untuk migrate/seed/cache:clear).
        if (php_sapi_name() === 'cli') {
            return;
        }

        // SIMPEL v1 di-mount di subdirectory /perlengkapan/simpel/v1 saat
        // deploy via Istio. Tanpa forceRootUrl, helper `url()` & `redirect()`
        // hanya pakai scheme://host dari Request, sehingga prefix
        // /perlengkapan/simpel/v1 hilang dari Location header.
        //
        // Scheme HARUS dinamis (HTTP atau HTTPS) berdasarkan request user,
        // bukan hardcoded dari APP_URL config. TrustProxies middleware harus
        // trust upstream (Istio sidecar) supaya $request->getScheme()
        // baca X-Forwarded-Proto correctly. Hasil:
        //   - User akses https://10.1.7.121/.../v1/... → redirect ke
        //     https://10.1.7.121/.../v1/auth/login (preserve scheme HTTPS)
        //   - User akses http://...  → redirect ke http://...
        //
        // Sebelumnya: APP_URL=http://... → forceRootUrl selalu HTTP →
        // user di HTTPS dapat Location: http://... → mixed-content / HSTS
        // bermasalah → ERR_CONNECTION_REFUSED.
        $appUrl = config('app.url');
        if (empty($appUrl)) {
            return;
        }

        $parsed = parse_url($appUrl);
        $path = $parsed['path'] ?? '';

        try {
            $request = request();
            $scheme = $request->getScheme() ?: ($parsed['scheme'] ?? 'http');
            $host = $request->getHost() ?: ($parsed['host'] ?? 'localhost');
        } catch (\Throwable $e) {
            // Fallback ke APP_URL kalau request() not bound (edge case).
            $scheme = $parsed['scheme'] ?? 'http';
            $host = $parsed['host'] ?? 'localhost';
        }

        URL::forceRootUrl("{$scheme}://{$host}{$path}");
        if ($scheme === 'https') {
            URL::forceScheme('https');
        }
    }
}
