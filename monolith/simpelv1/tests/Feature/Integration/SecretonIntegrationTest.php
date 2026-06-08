<?php

namespace Tests\Feature\Integration;

use App\Services\Grpc\SecrethonGrpcClient;
use Illuminate\Support\Facades\Http;
use Tests\TestCase;

/**
 * simpelv1 ↔ secreton integration (F5-A, #31).
 *
 * Verifies simpelv1's INTERNAL secret-fetch functions speak the Secreton
 * gateway contract correctly (URL shape, response parsing, fail-safe nulls) —
 * not business features. Http::fake() simulates the gateway; no live stack/DB.
 *
 * Contract (see SecrethonGrpcClient):
 *   GET {base}/v1/secrets/{name}              → {value}
 *   GET {base}/v1/database-credentials/{name} → {...}
 *   GET {base}/v1/api-keys/{name}             → {api_key}
 *   GET {base}/healthz                        → 2xx
 */
class SecretonIntegrationTest extends TestCase
{
    private const GW = 'http://secreton-gw.test';

    protected function setUp(): void
    {
        parent::setUp();
        config(['services.gateway.secreton.url' => self::GW]);
        config(['services.gateway.timeout' => 5.0]);
    }

    public function test_get_secret_returns_value_on_success(): void
    {
        Http::fake([self::GW.'/v1/secrets/*' => Http::response(['value' => 's3cr3t'], 200)]);

        $this->assertSame('s3cr3t', (new SecrethonGrpcClient)->getSecret('db/password'));
    }

    public function test_get_secret_returns_null_when_missing(): void
    {
        Http::fake([self::GW.'/v1/secrets/*' => Http::response(['error' => 'not found'], 404)]);

        $this->assertNull((new SecrethonGrpcClient)->getSecret('nope'));
    }

    public function test_get_database_credentials_returns_array(): void
    {
        Http::fake([self::GW.'/v1/database-credentials/*' => Http::response(
            ['username' => 'u', 'password' => 'p', 'ttl' => 3600], 200
        )]);

        $creds = (new SecrethonGrpcClient)->getDatabaseCredentials('dbsimpelv1');
        $this->assertIsArray($creds);
        $this->assertSame('u', $creds['username']);
    }

    public function test_get_api_key_returns_key_or_null(): void
    {
        Http::fake([self::GW.'/v1/api-keys/*' => Http::response(['api_key' => 'k-123'], 200)]);
        $this->assertSame('k-123', (new SecrethonGrpcClient)->getApiKey('maps'));

        Http::fake([self::GW.'/v1/api-keys/*' => Http::response('', 500)]);
        $this->assertNull((new SecrethonGrpcClient)->getApiKey('maps'));
    }

    public function test_health_reflects_gateway_status(): void
    {
        Http::fake([self::GW.'/healthz' => Http::response('ok', 200)]);
        $this->assertTrue((new SecrethonGrpcClient)->health());

        Http::fake([self::GW.'/healthz' => Http::response('down', 503)]);
        $this->assertFalse((new SecrethonGrpcClient)->health());
    }
}
