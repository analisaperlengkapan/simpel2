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

        // Alias for easy access
        $this->app->alias('authenc.grpc', AuthencGrpcClient::class);
        $this->app->alias('integrasi.grpc', IntegrasiGrpcClient::class);
        $this->app->alias('secreton.grpc', SecrethonGrpcClient::class);
    }

    public function boot()
    {
        // Log service registration
        \Illuminate\Support\Facades\Log::debug('gRPC services registered');
    }
}
