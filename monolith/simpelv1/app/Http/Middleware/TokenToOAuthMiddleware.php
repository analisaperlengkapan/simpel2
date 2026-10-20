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
        //
        // `url($request->getRequestUri())` — NOT `$request->fullUrl()`: v1 is
        // mounted under `/perlengkapan/simpel/v1` and the ingress rewrites that
        // prefix away before PHP sees it, so `fullUrl()` produces a path that
        // omits the public prefix. `url()` goes through the root forced by
        // `AppServiceProvider::boot`, which is the only place that knows the
        // prefix.
        $portalUrl = config('app.portal_url', '/portal');
        // Resolve a relative PORTAL_URL against the request's OWN origin.
        //
        // v1 and the portal share one origin in every deployment (prod Istio VS;
        // the cross-app ingress in e2e), so the portal's address is best
        // expressed as the path `/portal`. It cannot simply be handed to
        // `redirect()` as-is, though: `URL::forceRootUrl` has already pinned the
        // root to the v1 mount point, so `redirect('/portal/login')` would emit
        // `/perlengkapan/simpel/v1/portal/login`. Building the absolute URL from
        // the request origin instead keeps the prefix off, and — unlike a
        // hard-coded host — stays correct for whichever address the browser
        // actually used (localhost:3000, cross-app-ingress:8080, the public
        // domain). An absolute PORTAL_URL is still honoured verbatim for
        // deployments that need to point at a different host.
        if (str_starts_with($portalUrl, '/')) {
            $portalUrl = $request->getSchemeAndHttpHost().rtrim($portalUrl, '/');
        }
        $returnTo = urlencode(url($request->getRequestUri()));
        $callback = urlencode(route('auth.oauth-callback'));

        return redirect("{$portalUrl}/login?return_to={$returnTo}&callback={$callback}&state={$state}");
    }
}
