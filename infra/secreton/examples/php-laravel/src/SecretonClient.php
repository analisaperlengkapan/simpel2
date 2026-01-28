<?php

namespace App\Services;

use Exception;

class SecretonClient
{
    private string $host;
    private ?string $token = null;
    private array $options = [];

    public function __construct(string $host, ?string $token = null, array $options = [])
    {
        $this->host = rtrim($host, '/');
        $this->token = $token;
        $this->options = $options;
    }

    /**
     * Login to Secreton to obtain an access token.
     */
    public function login(string $username, string $password): string
    {
        $response = $this->request('POST', '/v1/auth/login', [
            'username' => $username,
            'password' => $password,
        ]);

        if (!isset($response['data']['access_token'])) {
            throw new Exception('Secreton: Login failed, no access token returned.');
        }

        $this->token = $response['data']['access_token'];
        return $this->token;
    }

    /**
     * Retrieve a secret from the KV engine.
     */
    public function getSecret(string $path): array
    {
        $path = ltrim($path, '/');
        $response = $this->request('GET', "/v1/secret/data/{$path}");

        return $response['data'] ?? [];
    }

    /**
     * Inject environment variables via Secreton's injection engine.
     */
    public function injectEnv(string $jobId, array $secrets, int $ttl = 3600): array
    {
        $payload = [
            'job_id' => $jobId,
            'secrets' => $secrets,
            'ttl' => $ttl,
        ];

        $response = $this->request('POST', '/v1/sys/inject/env', $payload);

        return $response['data']['env_vars'] ?? [];
    }

    /**
     * Encrypt data using Transit engine.
     */
    public function encrypt(string $keyName, string $plaintext): string
    {
        $payload = [
            'plaintext' => base64_encode($plaintext),
        ];

        $response = $this->request('POST', "/v1/transit/encrypt/{$keyName}", $payload);

        return $response['ciphertext'] ?? '';
    }

    /**
     * Decrypt data using Transit engine.
     */
    public function decrypt(string $keyName, string $ciphertext): string
    {
        $payload = [
            'ciphertext' => $ciphertext,
        ];

        $response = $this->request('POST', "/v1/transit/decrypt/{$keyName}", $payload);

        return base64_decode($response['plaintext'] ?? '');
    }

    private function request(string $method, string $path, array $data = []): array
    {
        $url = $this->host . $path;

        $ch = curl_init();
        curl_setopt($ch, CURLOPT_URL, $url);
        curl_setopt($ch, CURLOPT_RETURNTRANSFER, true);
        curl_setopt($ch, CURLOPT_CUSTOMREQUEST, $method);
        curl_setopt($ch, CURLOPT_HTTPHEADER, $this->buildHeaders());

        // Timeout configuration
        curl_setopt($ch, CURLOPT_TIMEOUT, $this->options['timeout'] ?? 5);
        curl_setopt($ch, CURLOPT_CONNECTTIMEOUT, $this->options['connect_timeout'] ?? 2);

        if (!empty($data)) {
            curl_setopt($ch, CURLOPT_POSTFIELDS, json_encode($data));
        }

        $result = curl_exec($ch);
        $httpCode = curl_getinfo($ch, CURLINFO_HTTP_CODE);
        $error = curl_error($ch);

        curl_close($ch);

        if ($result === false) {
            throw new Exception("Secreton Connection Error: " . $error);
        }

        $decoded = json_decode($result, true);

        if ($httpCode >= 400) {
            $message = $decoded['error'] ?? 'Unknown error';
            if (is_array($message)) {
                $message = json_encode($message);
            }
            throw new Exception("Secreton API Error ({$httpCode}): {$message}");
        }

        return $decoded;
    }

    private function buildHeaders(): array
    {
        $headers = [
            'Content-Type: application/json',
            'Accept: application/json',
            'User-Agent: Secreton-Laravel-Client/1.0',
        ];

        if ($this->token) {
            $headers[] = 'Authorization: Bearer ' . $this->token;
        }

        return $headers;
    }
}
