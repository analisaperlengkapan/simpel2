<?php

namespace App\Http\Middleware;

use Closure;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\Auth;
use Illuminate\Support\Str;
use Symfony\Component\HttpFoundation\Response;

/**
 * Middleware untuk OAuth flow dengan Portal
 *
 * Jika user belum terautentikasi, redirect ke Portal login
 * Portal akan mengirimkan JWT token kembali ke oauth-callback endpoint.
 *
 * Generates a random `state` parameter bound to the user's session and
 * passes it to the Portal so the callback handler can detect login-CSRF
 * attempts: an attacker who tricks the victim into hitting
 * `/auth/oauth-callback?token=<attacker_jwt>` directly will not have a
 * matching `state` in the victim's session, and the callback will refuse
 * to log them in.
 */
class TokenToOAuthMiddleware
{
    /**
     * Handle an incoming request.
     *
     * @param  Closure(Request): (Response)  $next
     */
    public function handle(Request $request, Closure $next): Response
    {
        // Jika user sudah terautentikasi, lanjutkan
        if (Auth::check()) {
            return $next($request);
        }

        // Cek apakah token JWT ada di query parameter atau header
        $token = $request->query('token') ?? $request->bearerToken();

        if ($token) {
            // Jika ada token, redirect ke oauth-callback untuk diproses.
            // Forward the original `state` (if any) so the callback can
            // verify it against the value stored in session.
            return redirect()->route('auth.oauth-callback', array_filter([
                'token' => $token,
                'state' => $request->query('state'),
            ]));
        }

        // Generate a single-use OAuth state and stash it in session so
        // the callback can verify the token belongs to a flow this
        // browser actually initiated (login-CSRF / session-fixation
        // protection).
        $state = Str::random(40);
        $request->session()->put('oauth_state', $state);

        // Jika tidak ada token dan user belum login, redirect ke Portal login.
        // Preserve the originally requested URL so the user lands back where they
        // tried to go, not just at the v1 root.
        $portalUrl = config('app.portal_url', 'http://localhost:3000/portal');
        $returnTo = urlencode($request->fullUrl());
        $callback = urlencode(route('auth.oauth-callback'));

        return redirect("{$portalUrl}/login?return_to={$returnTo}&callback={$callback}&state={$state}");
    }
}
