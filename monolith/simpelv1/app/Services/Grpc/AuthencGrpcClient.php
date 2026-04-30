<?php

namespace App\Services\Grpc;

use Illuminate\Support\Facades\Http;
use Illuminate\Support\Facades\Log;
use Throwable;

/**
 * Authenc Gateway Client
 *
 * Talks to the Authenc service via the K8s Sidecar / Rust Gateway Proxy
 * over HTTP/REST. Per `monolith/simpelv1/AGENTS.md`, Laravel MUST NOT
 * open direct gRPC channels to core Rust services — repeated mTLS RPC
 * setup on every php-fpm cycle is prohibitively expensive. The sidecar
 * keeps a long-lived gRPC connection upstream and exposes a local
 * REST endpoint that this client consumes.
 *
 * Class name retained as `AuthencGrpcClient` for backwards compatibility
 * with existing service-container bindings.
 */
class AuthencGrpcClient
{
    private string $baseUrl;
    private float $timeout;

    public function __construct()
    {
        // Resolve the gateway URL via `config()` only. Calling `env()`
        // directly here would return `null` once `php artisan config:cache`
        // has been run (env() only reads $_ENV during config bootstrap),
        // silently falling through to the hardcoded default and ignoring
        // any AUTHENC_GATEWAY_URL set via docker-compose / K8s ConfigMap.
        // The env() → default fallback lives in config/services.php so
        // the binding happens once at config-compile time.
        $this->baseUrl = rtrim(
            (string) config('services.gateway.authenc.url', 'http://127.0.0.1:8081'),
            '/'
        );
        $this->timeout = (float) config('services.gateway.timeout', 5.0);
    }

    /**
     * Verify JWT token via the Authenc gateway.
     */
    public function verifyToken(string $token): ?array
    {
        try {
            $response = Http::timeout($this->timeout)
                ->acceptJson()
                ->asJson()
                ->post("{$this->baseUrl}/v1/tokens/verify", ['token' => $token]);

            if (! $response->successful()) {
                Log::warning('Authenc gateway rejected token', [
                    'status' => $response->status(),
                ]);
                return null;
            }

            $claims = $response->json('claims');
            return is_array($claims) ? $claims : null;
        } catch (Throwable $th) {
            Log::warning("Authenc verifyToken failed: {$th->getMessage()}");
            return null;
        }
    }

    /**
     * Get user details from Authenc gateway.
     */
    public function getUserDetails(string $username): ?array
    {
        try {
            $response = Http::timeout($this->timeout)
                ->acceptJson()
                ->get("{$this->baseUrl}/v1/users/" . rawurlencode($username));

            if (! $response->successful()) {
                return null;
            }

            $user = $response->json();
            return is_array($user) ? $user : null;
        } catch (Throwable $th) {
            Log::warning("Authenc getUserDetails failed: {$th->getMessage()}");
            return null;
        }
    }
}
