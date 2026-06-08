<?php

namespace Tests\Feature\Integration;

use App\Http\Middleware\EnforceTokenRevocation;
use App\Services\Grpc\AuthencGrpcClient;
use Illuminate\Support\Facades\Cache;
use Illuminate\Support\Facades\Http;
use Illuminate\Support\Facades\Route;
use Tests\TestCase;

/**
 * simpelv1 ↔ authenc integration (F5-A, #31).
 *
 * Verifies simpelv1's INTERNAL functions that talk to authenc behave correctly
 * against the authenc gateway contract — NOT business features. The gateway is
 * simulated with Http::fake() so the test is deterministic and needs no live
 * stack or DB (it exercises the real client + middleware code paths).
 *
 * Contract (see AuthencGrpcClient): POST {base}/v1/tokens/verify {token}
 *   200 + {claims:{...}}  → token active / claims
 *   4xx                    → definitively rejected (revoked/expired)
 *   5xx / transport error  → indeterminate
 */
class AuthencIntegrationTest extends TestCase
{
    private const GW = 'http://authenc-gw.test';
    private const VERIFY = self::GW.'/v1/tokens/verify';

    protected function setUp(): void
    {
        parent::setUp();
        config(['services.gateway.authenc.url' => self::GW]);
        config(['services.gateway.timeout' => 5.0]);
        config(['services.gateway.authenc.revocation_ttl' => 45]);
        Cache::flush();
    }

    // ── AuthencGrpcClient ────────────────────────────────────────────────

    public function test_verify_token_returns_claims_on_success(): void
    {
        Http::fake([self::VERIFY => Http::response(['claims' => ['sub' => 'u1', 'sid' => 's1']], 200)]);

        $claims = (new AuthencGrpcClient())->verifyToken('tok');

        $this->assertIsArray($claims);
        $this->assertSame('u1', $claims['sub']);
    }

    public function test_verify_token_returns_null_on_rejection(): void
    {
        Http::fake([self::VERIFY => Http::response(['error' => 'invalid'], 401)]);

        $this->assertNull((new AuthencGrpcClient())->verifyToken('tok'));
    }

    public function test_is_token_active_three_states(): void
    {
        // active
        Http::fake([self::VERIFY => Http::response(['claims' => ['sub' => 'u1']], 200)]);
        $this->assertTrue((new AuthencGrpcClient())->isTokenActive('tok'));

        // definitively revoked/expired (4xx)
        Http::fake([self::VERIFY => Http::response(['error' => 'revoked'], 401)]);
        $this->assertFalse((new AuthencGrpcClient())->isTokenActive('tok'));

        // indeterminate (5xx) → null (fail-open upstream)
        Http::fake([self::VERIFY => Http::response('boom', 503)]);
        $this->assertNull((new AuthencGrpcClient())->isTokenActive('tok'));
    }

    // ── EnforceTokenRevocation middleware ────────────────────────────────

    private function routeWithMiddleware(): void
    {
        Route::middleware(['web', EnforceTokenRevocation::class])
            ->get('/_test/revocation', fn () => response('ok'));
    }

    public function test_revocation_middleware_passes_without_upstream_token(): void
    {
        $this->routeWithMiddleware();
        Http::fake(); // no call expected

        $this->get('/_test/revocation')->assertOk();
        Http::assertNothingSent();
    }

    public function test_revocation_middleware_passes_for_active_token_and_caches(): void
    {
        $this->routeWithMiddleware();
        Http::fake([self::VERIFY => Http::response(['claims' => ['sub' => 'u1']], 200)]);

        $this->withSession(['auth_token' => 'live-token'])
            ->get('/_test/revocation')->assertOk();

        // Second request within TTL must be served from cache (no 2nd gateway call).
        $this->withSession(['auth_token' => 'live-token'])
            ->get('/_test/revocation')->assertOk();
        Http::assertSentCount(1);
    }

    public function test_revocation_middleware_fails_closed_on_revoked_token_web(): void
    {
        $this->routeWithMiddleware();
        Http::fake([self::VERIFY => Http::response(['error' => 'revoked'], 401)]);

        $this->withSession(['auth_token' => 'revoked-token'])
            ->get('/_test/revocation')
            ->assertRedirect(); // guest redirect to login
    }

    public function test_revocation_middleware_fails_closed_on_revoked_token_json(): void
    {
        $this->routeWithMiddleware();
        Http::fake([self::VERIFY => Http::response(['error' => 'revoked'], 401)]);

        $this->withSession(['auth_token' => 'revoked-token'])
            ->getJson('/_test/revocation')
            ->assertStatus(401)
            ->assertJson(['code' => 'TOKEN_REVOKED']);
    }

    public function test_revocation_middleware_fails_open_when_gateway_down(): void
    {
        $this->routeWithMiddleware();
        Http::fake([self::VERIFY => Http::response('down', 503)]);

        // Indeterminate → request still served (fail-open) so a blip doesn't mass-logout.
        $this->withSession(['auth_token' => 'some-token'])
            ->get('/_test/revocation')->assertOk();
    }
}
