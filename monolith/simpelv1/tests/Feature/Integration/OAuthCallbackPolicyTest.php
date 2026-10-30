<?php

namespace Tests\Feature\Integration;

use Illuminate\Support\Facades\Http;
use Tests\TestCase;

/**
 * `AuthController::oauthCallback` must refuse to create a session while the
 * token still owes an account-policy step.
 *
 * The Portal enforces `require_password_change` / `mfa_setup_required` before
 * it hands a token back, but that is a client-side decision: the callback is a
 * public POST endpoint, so a caller can skip the Portal entirely. Without a
 * server-side check the token creates a full Laravel session and the policy
 * step is bypassed (CWE-841).
 *
 * The gateway is stubbed with `Http::fake()` — this exercises the real
 * controller path up to the policy gate, with no DB and no live stack. Both
 * flags are rejected before any `Pengguna` lookup, so the test needs neither a
 * migrated schema nor a fixture user.
 */
class OAuthCallbackPolicyTest extends TestCase
{
    private const GW = 'http://authenc-gw.test';

    private const VERIFY = self::GW.'/v1/tokens/verify';

    protected function setUp(): void
    {
        parent::setUp();
        config(['services.gateway.authenc.url' => self::GW]);
        config(['services.gateway.timeout' => 5.0]);
    }

    /**
     * Seed a session `oauth_state` so the callback's CSRF check passes and the
     * policy gate is what decides the outcome.
     */
    private function withOauthState(string $state = 'test-state'): void
    {
        $this->withSession(['oauth_state' => $state]);
    }

    private function fakeClaims(array $claims): void
    {
        Http::fake([self::VERIFY => Http::response(['claims' => $claims], 200)]);
    }

    public function test_rejects_token_that_requires_password_change(): void
    {
        $this->withOauthState();
        $this->fakeClaims([
            'nip' => '199203142014031001',
            'require_password_change' => true,
        ]);

        $response = $this->post('/auth/oauth-callback', [
            'token' => 'tok',
            'state' => 'test-state',
        ]);

        $response->assertRedirect(route('login'));
        $response->assertSessionHas('error');
        // No session may be established for a policy-blocked token.
        $this->assertGuest();
    }

    public function test_rejects_token_that_requires_mfa_setup(): void
    {
        $this->withOauthState();
        $this->fakeClaims([
            'nip' => '199203142014031001',
            'mfa_setup_required' => true,
        ]);

        $response = $this->post('/auth/oauth-callback', [
            'token' => 'tok',
            'state' => 'test-state',
        ]);

        $response->assertRedirect(route('login'));
        $response->assertSessionHas('error');
        $this->assertGuest();
    }

    public function test_absent_policy_flags_do_not_reject(): void
    {
        // The gate must be driven by the flags, not by their mere absence of
        // keys: a token without them proceeds past the policy check (it will
        // then fail later on the user lookup, which this test does not reach
        // assertions about — only that it was NOT rejected as a policy error).
        $this->withOauthState();
        $this->fakeClaims(['nip' => '199203142014031001']);

        $response = $this->post('/auth/oauth-callback', [
            'token' => 'tok',
            'state' => 'test-state',
        ]);

        $response->assertRedirect();
        $this->assertNotSame(
            'Anda harus mengganti kata sandi terlebih dahulu melalui Portal',
            session('error'),
        );
        $this->assertNotSame(
            'Anda harus mengaktifkan autentikasi dua faktor terlebih dahulu melalui Portal',
            session('error'),
        );
    }
}
