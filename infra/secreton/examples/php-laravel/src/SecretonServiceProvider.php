<?php

namespace App\Providers;

use Illuminate\Support\ServiceProvider;
use App\Services\SecretonClient;
use Illuminate\Support\Facades\Log;

class SecretonServiceProvider extends ServiceProvider
{
    /**
     * Register any application services.
     */
    public function register(): void
    {
        $this->mergeConfigFrom(
            __DIR__.'/../config/secreton.php', 'secreton'
        );

        $this->app->singleton(SecretonClient::class, function ($app) {
            $config = $app['config']['secreton'];
            $host = $config['host'];
            $token = $config['token'] ?? null;

            // If we have a token from env injection, use it.
            // Otherwise, try to login if credentials are provided.

            $client = new SecretonClient($host, $token, [
                'timeout' => $config['timeout'] ?? 5,
            ]);

            if (!$token && !empty($config['username']) && !empty($config['password'])) {
                try {
                    $client->login($config['username'], $config['password']);
                } catch (\Exception $e) {
                    // Log error but don't crash app unless critical
                    Log::error("Secreton Auto-Login failed: " . $e->getMessage());
                }
            }

            return $client;
        });
    }

    /**
     * Bootstrap any application services.
     */
    public function boot(): void
    {
        $this->publishes([
            __DIR__.'/../config/secreton.php' => config_path('secreton.php'),
        ], 'secreton-config');
    }
}
