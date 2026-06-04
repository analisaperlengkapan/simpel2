<?php

return [

    /*
    |--------------------------------------------------------------------------
    | Third Party Services
    |--------------------------------------------------------------------------
    |
    | This file is for storing the credentials for third party services such
    | as Mailgun, Postmark, AWS and more. This file provides the de facto
    | location for this type of information, allowing packages to have
    | a conventional file to locate the various service credentials.
    |
    */

    'mailgun' => [
        'domain' => env('MAILGUN_DOMAIN'),
        'secret' => env('MAILGUN_SECRET'),
        'endpoint' => env('MAILGUN_ENDPOINT', 'api.mailgun.net'),
        'scheme' => 'https',
    ],

    'postmark' => [
        'token' => env('POSTMARK_TOKEN'),
    ],

    'ses' => [
        'key' => env('AWS_ACCESS_KEY_ID'),
        'secret' => env('AWS_SECRET_ACCESS_KEY'),
        'region' => env('AWS_DEFAULT_REGION', 'us-east-1'),
    ],

    /*
    |--------------------------------------------------------------------------
    | Backend Gateway Clients
    |--------------------------------------------------------------------------
    |
    | Per `monolith/simpelv1/AGENTS.md`, Laravel talks to the core Rust
    | services through a K8s sidecar / Rust Gateway Proxy over HTTP/REST
    | (not direct gRPC). These keys are consumed by
    | `App\Services\Grpc\{Authenc,Integrasi,Secrethon}GrpcClient`.
    |
    | The values are read via `config()` so they survive `php artisan
    | config:cache` — calling `env()` directly at runtime from a
    | non-config file returns `null` once the config cache is populated,
    | which is why the clients must resolve URLs through this file
    | rather than calling `env()` in their constructors.
    |
    */
    'gateway' => [
        'timeout' => (float) env('GATEWAY_TIMEOUT', 5.0),
        'authenc' => [
            'url' => env('AUTHENC_GATEWAY_URL', 'http://127.0.0.1:8081'),
            // Throttle window (seconds) for the mid-session revocation re-check
            // in EnforceTokenRevocation. Within this window a session is trusted
            // without re-hitting the gateway. Keep small (30-60s) so a revoke
            // takes effect quickly without a gateway round trip per request.
            'revocation_ttl' => (int) env('AUTHENC_REVOCATION_TTL', 45),
        ],
        'integrasi' => [
            'url' => env('INTEGRASI_GATEWAY_URL', 'http://127.0.0.1:8082'),
        ],
        'secreton' => [
            'url' => env('SECRETON_GATEWAY_URL', 'http://127.0.0.1:8083'),
        ],
    ],

];
