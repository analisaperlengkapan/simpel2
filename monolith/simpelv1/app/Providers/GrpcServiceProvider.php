<?php

namespace App\Providers;

use App\Services\Grpc\AuthencGrpcClient;
use App\Services\Grpc\IntegrasiGrpcClient;
use App\Services\Grpc\SecrethonGrpcClient;
use Illuminate\Support\ServiceProvider;

/**
 * Service Provider for backend gateway clients.
 *
 * NOTE: The `*GrpcClient` class names are retained for backwards
 * compatibility, but the underlying transport is HTTP/REST against a
 * local K8s sidecar (or a shared Rust Gateway Proxy). Direct gRPC from
 * php-fpm to core Rust services is forbidden by
 * `monolith/simpelv1/AGENTS.md` due to the cost of opening fresh mTLS
 * channels per request.
 */
class GrpcServiceProvider extends ServiceProvider
{
    public function register()
    {
        $this->app->singleton(AuthencGrpcClient::class, function ($app) {
            return new AuthencGrpcClient();
        });

        $this->app->singleton(IntegrasiGrpcClient::class, function ($app) {
            return new IntegrasiGrpcClient();
        });

        $this->app->singleton(SecrethonGrpcClient::class, function ($app) {
            return new SecrethonGrpcClient();
        });

        // Alias for easy access. Laravel's alias() signature is
        // alias($abstract, $alias) — the first argument is the existing
        // binding (the class), the second is the short alias name.
        $this->app->alias(AuthencGrpcClient::class, 'authenc.gateway');
        $this->app->alias(IntegrasiGrpcClient::class, 'integrasi.gateway');
        $this->app->alias(SecrethonGrpcClient::class, 'secreton.gateway');
    }

    public function boot()
    {
        \Illuminate\Support\Facades\Log::debug('Backend gateway clients registered');
    }
}
