<?php

namespace App\Providers;

use App\Services\Grpc\AuthencGrpcClient;
use App\Services\Grpc\IntegrasiGrpcClient;
use App\Services\Grpc\SecrethonGrpcClient;
use Illuminate\Support\ServiceProvider;

/**
 * Service Provider for gRPC Clients
 */
class GrpcServiceProvider extends ServiceProvider
{
    public function register()
    {
        // Register Authenc gRPC Client
        $this->app->singleton(AuthencGrpcClient::class, function ($app) {
            return new AuthencGrpcClient();
        });

        // Register Integrasi gRPC Client
        $this->app->singleton(IntegrasiGrpcClient::class, function ($app) {
            return new IntegrasiGrpcClient();
        });

        // Register Secreton gRPC Client
        $this->app->singleton(SecrethonGrpcClient::class, function ($app) {
            return new SecrethonGrpcClient();
        });

        // Alias for easy access. Laravel's alias() signature is
        // alias($abstract, $alias) — the first argument is the existing
        // binding (the class), the second is the short alias name.
        $this->app->alias(AuthencGrpcClient::class, 'authenc.grpc');
        $this->app->alias(IntegrasiGrpcClient::class, 'integrasi.grpc');
        $this->app->alias(SecrethonGrpcClient::class, 'secreton.grpc');
    }

    public function boot()
    {
        // Log service registration
        \Illuminate\Support\Facades\Log::debug('gRPC services registered');
    }
}
