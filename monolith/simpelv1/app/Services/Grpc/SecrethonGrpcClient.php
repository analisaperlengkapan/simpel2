<?php

namespace App\Services\Grpc;

use Illuminate\Http\Client\PendingRequest;
use Illuminate\Support\Facades\Http;
use Illuminate\Support\Facades\Log;
use Throwable;

/**
 * Secreton Gateway Client
 *
 * Talks to the Secreton service via the K8s Sidecar / Rust Gateway Proxy
 * over HTTP/REST rather than direct gRPC. See `monolith/simpelv1/AGENTS.md`
 * for the rationale (php-fpm cannot afford to open mTLS gRPC channels
 * per request).
 *
 * Class name retained as `SecrethonGrpcClient` for backwards compatibility.
 */
class SecrethonGrpcClient
{
    private string $baseUrl;
    private float $timeout;

    public function __construct()
    {
        $this->baseUrl = rtrim(
            config('services.gateway.secreton.url', env('SECRETON_GATEWAY_URL', 'http://127.0.0.1:8083')),
            '/'
        );
        $this->timeout = (float) config('services.gateway.timeout', 5.0);
    }

    private function http(): PendingRequest
    {
        return Http::timeout($this->timeout)->acceptJson();
    }

    /**
     * Get secret by name
     */
    public function getSecret(string $secretName): ?string
    {
        try {
            $response = $this->http()->get(
                "{$this->baseUrl}/v1/secrets/" . rawurlencode($secretName)
            );
            if (! $response->successful()) {
                return null;
            }
            $value = $response->json('value');
            return is_string($value) ? $value : null;
        } catch (Throwable $th) {
            Log::error("Failed to retrieve secret {$secretName}: {$th->getMessage()}");
            return null;
        }
    }

    /**
     * Store a new secret
     */
    public function putSecret(string $secretName, string $secretValue, array $metadata = []): bool
    {
        try {
            $response = $this->http()
                ->asJson()
                ->put("{$this->baseUrl}/v1/secrets/" . rawurlencode($secretName), [
                    'value' => $secretValue,
                    'metadata' => $metadata,
                ]);
            return $response->successful();
        } catch (Throwable $th) {
            Log::error("Failed to store secret {$secretName}: {$th->getMessage()}");
            return false;
        }
    }

    /**
     * Delete a secret
     */
    public function deleteSecret(string $secretName): bool
    {
        try {
            $response = $this->http()->delete(
                "{$this->baseUrl}/v1/secrets/" . rawurlencode($secretName)
            );
            return $response->successful();
        } catch (Throwable $th) {
            Log::error("Failed to delete secret {$secretName}: {$th->getMessage()}");
            return false;
        }
    }

    /**
     * Get database credentials
     */
    public function getDatabaseCredentials(string $databaseName): ?array
    {
        try {
            $response = $this->http()->get(
                "{$this->baseUrl}/v1/database-credentials/" . rawurlencode($databaseName)
            );
            if (! $response->successful()) {
                return null;
            }
            $data = $response->json();
            return is_array($data) ? $data : null;
        } catch (Throwable $th) {
            Log::error("Failed to retrieve DB credentials: {$th->getMessage()}");
            return null;
        }
    }

    /**
     * Get API key for external service
     */
    public function getApiKey(string $serviceName): ?string
    {
        try {
            $response = $this->http()->get(
                "{$this->baseUrl}/v1/api-keys/" . rawurlencode($serviceName)
            );
            if (! $response->successful()) {
                return null;
            }
            $key = $response->json('api_key');
            return is_string($key) ? $key : null;
        } catch (Throwable $th) {
            Log::error("Failed to retrieve API key for {$serviceName}: {$th->getMessage()}");
            return null;
        }
    }

    /**
     * Check secrets vault health
     */
    public function health(): bool
    {
        try {
            return $this->http()->get("{$this->baseUrl}/healthz")->successful();
        } catch (Throwable $th) {
            Log::warning("Secreton gateway health check failed: {$th->getMessage()}");
            return false;
        }
    }
}
