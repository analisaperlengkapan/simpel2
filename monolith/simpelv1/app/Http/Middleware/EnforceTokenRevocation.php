<?php

namespace App\Http\Middleware;

use App\Services\Grpc\AuthencGrpcClient;
use Closure;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\Auth;
use Illuminate\Support\Facades\Cache;
use Symfony\Component\HttpFoundation\Response;

/**
 * Enforce upstream (Authenc) token revocation mid-session.
 *
 * After OAuth login the Authenc-issued token is parked in
 * `session('auth_token')` while the request runs on a Laravel session. Authenc
 * can revoke that token before its `exp` (logout elsewhere, role change,
 * account deactivation); without a re-check the v1 session would outlive the
 * revocation — a security gap.
 *
 * We re-validate the parked token via the Authenc gateway (per
 * `monolith/simpelv1/AGENTS.md`, through the K8s sidecar / Rust Gateway Proxy —
 * never a direct gRPC channel from php-fpm), but **throttled** through the cache
 * (TTL ~30-60s keyed by the token) so we add at most one gateway round trip per
 * window per session rather than one per request.
 *
 * Fail closed on a definitive rejection (revoked/expired) — tear the session
 * down. Fail open on an indeterminate result (gateway down) so a transient blip
 * doesn't mass-logout every active user; a short backoff avoids hammering the
 * gateway during an outage.
 */
class EnforceTokenRevocation
{
    public function __construct(private AuthencGrpcClient $authenc) {}

    public function handle(Request $request, Closure $next): Response
    {
        $token = $request->session()->get('auth_token');

        // Only OAuth/Authenc-backed sessions carry an upstream token. Password
        // logins (local JWT) and guest requests are out of scope.
        if (! is_string($token) || $token === '') {
            return $next($request);
        }

        $cacheKey = 'authenc:revchk:'.hash('sha256', $token);

        // Recently confirmed valid → trust within the throttle window.
        if (Cache::get($cacheKey) === true) {
            return $next($request);
        }

        $ttl = (int) config('services.gateway.authenc.revocation_ttl', 45);
        $active = $this->authenc->isTokenActive($token);

        if ($active === true) {
            Cache::put($cacheKey, true, $ttl);

            return $next($request);
        }

        if ($active === null) {
            // Indeterminate (gateway unreachable) → fail open, but back off so
            // we don't call the gateway on every request during an outage.
            Cache::put($cacheKey, true, min(10, $ttl));

            return $next($request);
        }

        // $active === false → definitively revoked/expired. Tear down the
        // session, mirroring ValidateCrossTabSession's content negotiation.
        Auth::logout();
        $request->session()->flush();
        $request->session()->regenerate();

        if ($request->expectsJson()) {
            return response()->json([
                'error' => 'Sesi tidak lagi valid (token dicabut)',
                'code' => 'TOKEN_REVOKED',
            ], 401);
        }

        return redirect()->guest(route('login'))
            ->with('error', 'Sesi Anda telah berakhir, silakan login ulang');
    }
}
