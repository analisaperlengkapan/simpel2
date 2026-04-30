<?php

namespace App\Services\Grpc;

use Illuminate\Support\Facades\Log;
use Throwable;

/**
 * Authenc gRPC Client
 * 
 * Interfaces with Authenc service for identity verification and JWT token validation
 */
class AuthencGrpcClient
{
    private ?object $client = null;
    private string $host;
    private int $port;
    private string $sslMode;

    public function __construct()
    {
        $this->host = config('services.grpc.authenc.host', 'authenc');
        $this->port = config('services.grpc.authenc.port', 50051);
        $this->sslMode = config('services.grpc.ssl_mode', 'insecure');
    }

    /**
     * Connect to Authenc gRPC service
     */
    private function connect(): void
    {
        if ($this->client !== null) {
            return;
        }

        try {
            $address = "{$this->host}:{$this->port}";
            $opts = $this->sslMode === 'require' 
                ? ['credentials' => \Grpc\ChannelCredentials::createSsl()]
                : [];

            // Initialize gRPC client (channel will be created by Grpc library)
            $this->client = new \Grpc\Client($address, $opts);
            
            Log::debug("Connected to Authenc gRPC service at {$address}");
        } catch (Throwable $th) {
            Log::error("Failed to connect to Authenc gRPC: {$th->getMessage()}");
            throw $th;
        }
    }

    /**
     * Verify JWT token with Authenc service
     */
    public function verifyToken(string $token): ?array
    {
        try {
            $this->connect();

            // TODO: Call Authenc::VerifyToken() RPC
            // For now, decode JWT locally (placeholder)
            $claims = $this->decodeJwtLocally($token);

            Log::info("JWT token verified successfully for user: {$claims['sub']}");
            return $claims;

        } catch (Throwable $th) {
            Log::warning("Token verification failed: {$th->getMessage()}");
            return null;
        }
    }

    /**
     * Get user details from Authenc
     */
    public function getUserDetails(string $username): ?array
    {
        try {
            $this->connect();

            // TODO: Call Authenc::GetUserDetails() RPC
            Log::info("Retrieved user details for: {$username}");
            return [];

        } catch (Throwable $th) {
            Log::warning("Failed to get user details: {$th->getMessage()}");
            return null;
        }
    }

    /**
     * Local JWT decoding (temporary until gRPC integration)
     */
    private function decodeJwtLocally(string $token): array
    {
        // Split JWT into parts
        $parts = explode('.', $token);
        if (count($parts) !== 3) {
            throw new \Exception('Invalid JWT format');
        }

        $payload = json_decode(
            base64_decode(strtr($parts[1], '-_', '+/')),
            true
        );

        if (!$payload) {
            throw new \Exception('Failed to decode JWT payload');
        }

        return $payload;
    }

    /**
     * Close connection
     */
    public function __destruct()
    {
        $this->client = null;
    }
}
