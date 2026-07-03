<?php

namespace Tests\Feature\IntegrationLive;

use App\Services\Grpc\SecrethonGrpcClient;
use Tests\TestCase;

/**
 * simpelv1 ↔ secreton — LIVE (F-GW PR-C).
 *
 * Unlike the `Integration` suite (Http::fake, contract-unit), this runs the real
 * SecrethonGrpcClient against a LIVE gateway → secreton gRPC. The gateway URL is
 * injected via SECRETON_GATEWAY_URL by the `e2e-simpelv1-integration` CI job; if
 * it is unset the whole suite skips, so an accidental full `phpunit` run without a
 * stack does not fail.
 *
 * Seed: `infra/scripts/secreton-ci-bootstrap.sh` init+unseals secreton then seeds
 * `simpelv1-app-key = ci-bootstrap-secret` THROUGH THE GATEWAY (StoreSecret), so
 * the write path matches this read path.
 */
class SecretonLiveTest extends TestCase
{
    protected function setUp(): void
    {
        parent::setUp();
        $gw = getenv('SECRETON_GATEWAY_URL');
        if ($gw === false || $gw === '') {
            $this->markTestSkipped('SECRETON_GATEWAY_URL unset — live gateway stack not present');
        }
        config(['services.gateway.secreton.url' => $gw]);
        config(['services.gateway.timeout' => 10.0]);
    }

    public function test_gateway_health_is_up(): void
    {
        $this->assertTrue((new SecrethonGrpcClient)->health());
    }

    public function test_reads_secret_seeded_by_bootstrap(): void
    {
        // Seeded by secreton-ci-bootstrap.sh via the gateway (StoreSecret).
        $this->assertSame(
            'ci-bootstrap-secret',
            (new SecrethonGrpcClient)->getSecret('simpelv1-app-key')
        );
    }

    public function test_put_then_get_round_trips(): void
    {
        $client = new SecrethonGrpcClient;
        $name = 'simpelv1-live-'.bin2hex(random_bytes(4));

        $this->assertTrue($client->putSecret($name, 'roundtrip-value'));
        $this->assertSame('roundtrip-value', $client->getSecret($name));
    }

    public function test_missing_secret_returns_null(): void
    {
        // gateway maps secreton NotFound → 404 → client returns null.
        $this->assertNull(
            (new SecrethonGrpcClient)->getSecret('does-not-exist-'.bin2hex(random_bytes(4)))
        );
    }
}
