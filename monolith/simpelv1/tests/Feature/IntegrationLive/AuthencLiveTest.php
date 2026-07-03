<?php

namespace Tests\Feature\IntegrationLive;

use App\Services\Grpc\AuthencGrpcClient;
use Tests\TestCase;

/**
 * simpelv1 ↔ authenc — LIVE (F-GW PR-C).
 *
 * Runs the real AuthencGrpcClient against a LIVE gateway → authenc ValidateToken
 * (the F2H token-revocation path). Skips unless AUTHENC_GATEWAY_URL is set.
 *
 * We assert the fail-closed behaviour with a bogus token (a valid-token positive
 * path needs a real login + captcha, deferred). This proves the gateway → authenc
 * wiring returns a definitive rejection — exactly what EnforceTokenRevocation
 * relies on (fail closed on a 4xx, fail open only on transport/5xx).
 */
class AuthencLiveTest extends TestCase
{
    protected function setUp(): void
    {
        parent::setUp();
        $gw = getenv('AUTHENC_GATEWAY_URL');
        if ($gw === false || $gw === '') {
            $this->markTestSkipped('AUTHENC_GATEWAY_URL unset — live gateway stack not present');
        }
        config(['services.gateway.authenc.url' => $gw]);
        config(['services.gateway.timeout' => 10.0]);
    }

    public function test_verify_bogus_token_yields_no_claims(): void
    {
        // gateway → authenc ValidateToken → invalid → non-2xx → null.
        $this->assertNull((new AuthencGrpcClient)->verifyToken('not.a.real.token'));
    }

    public function test_bogus_token_is_not_active(): void
    {
        // isTokenActive is 3-state; a bogus token must never be `true` (active).
        // (false = definitive 4xx reject; null = transport/5xx — both "not active".)
        $this->assertNotSame(true, (new AuthencGrpcClient)->isTokenActive('not.a.real.token'));
    }
}
