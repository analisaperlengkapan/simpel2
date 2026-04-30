<?php

namespace App\Http\Middleware;

use Closure;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\Auth;
use Symfony\Component\HttpFoundation\Response;

/**
 * Middleware untuk OAuth flow dengan Portal
 * 
 * Jika user belum terautentikasi, redirect ke Portal login
 * Portal akan mengirimkan JWT token kembali ke oauth-callback endpoint
 */
class TokenToOAuthMiddleware
{
    /**
     * Handle an incoming request.
     *
     * @param  \Closure(\Illuminate\Http\Request): (\Symfony\Component\HttpFoundation\Response)  $next
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
            // Jika ada token, redirect ke oauth-callback untuk diproses
            return redirect()->route('auth.oauth-callback', ['token' => $token]);
        }

        // Jika tidak ada token dan user belum login, redirect ke Portal login.
        // Preserve the originally requested URL so the user lands back where they
        // tried to go, not just at the v1 root.
        $portalUrl = config('app.portal_url', 'http://localhost:3000/portal');
        $returnTo = urlencode($request->fullUrl());
        $callback = urlencode(route('auth.oauth-callback'));

        return redirect("{$portalUrl}/login?return_to={$returnTo}&callback={$callback}");
    }
}
