<?php

/**
 * Secreton Bootstrap Script for Laravel
 *
 * This script runs before Laravel boots to inject secrets from Secreton
 * directly into the environment variables.
 */

// Basic configuration from environment
$host = getenv('SECRETON_HOST');
$username = getenv('SECRETON_USERNAME');
$password = getenv('SECRETON_PASSWORD');
$jobId = getenv('SECRETON_JOB_ID') ?: 'laravel-boot-' . uniqid();

// Skip if configuration is missing
if (!$host || !$username || !$password) {
    // Optional: log warning to stderr
    // fwrite(STDERR, "Secreton: Skipping injection (missing credentials)\n");
    return;
}

// Load SecretonClient manually if not autoloaded
if (!class_exists('App\Services\SecretonClient')) {
    require_once __DIR__ . '/src/SecretonClient.php';
}

use App\Services\SecretonClient;

try {
    $client = new SecretonClient($host);

    // 1. Login
    $client->login($username, $password);

    // 2. Define secrets to inject
    // You can customize this list or load it from a separate config file
    $secretsToInject = [
        [
            'path' => 'secret/data/myapp/database',
            'key' => 'password',
            'env_name' => 'DB_PASSWORD'
        ],
        [
            'path' => 'secret/data/myapp/database',
            'key' => 'username',
            'env_name' => 'DB_USERNAME'
        ],
        [
            'path' => 'secret/data/myapp/app',
            'key' => 'key',
            'env_name' => 'APP_KEY'
        ],
        // Add more secrets here...
    ];

    // 3. Perform injection
    $envVars = $client->injectEnv($jobId, $secretsToInject, 3600); // 1 hour TTL for session

    // 4. Populate environment
    foreach ($envVars as $key => $value) {
        putenv("$key=$value");
        $_ENV[$key] = $value;
        $_SERVER[$key] = $value;
    }

    // fwrite(STDERR, "Secreton: Successfully injected " . count($envVars) . " secrets.\n");

} catch (Exception $e) {
    // In production, you might want to exit if secrets fail to load
    fwrite(STDERR, "Secreton Error: " . $e->getMessage() . "\n");
    // exit(1);
}
