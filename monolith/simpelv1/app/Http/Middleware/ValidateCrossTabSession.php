<?php

namespace App\Http\Middleware;

use Closure;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\Auth;
use Symfony\Component\HttpFoundation\Response;

/**
 * Cross-Tab Session Validator Middleware
 * 
 * Detects if session was invalidated by logout in another tab
 * by checking if localStorage logout_event was triggered
 */
class ValidateCrossTabSession
{
    /**
     * Handle an incoming request.
     */
    public function handle(Request $request, Closure $next): Response
    {
        // Check if logout event header is present (from client-side storage listener)
        if ($request->header('X-Logout-Event')) {
            // Session was invalidated in another tab
            Auth::logout();
            $request->session()->flush();
            $request->session()->regenerate();

            // This middleware is mounted on the `web` route group
            // (see `routes/web.php`), so requests come from both AJAX
            // callers (which expect JSON) and regular browser
            // navigations (which expect a redirect to the login form).
            // Returning a JSON 401 to a browser navigation produces a
            // raw text page instead of returning the user to login —
            // mirror Laravel's own `Authenticate` middleware and
            // content-negotiate on `expectsJson()`.
            if ($request->expectsJson()) {
                return response()->json([
                    'error' => 'Session invalidated from another tab',
                    'code'  => 'SESSION_INVALIDATED',
                ], 401);
            }

            return redirect()->guest(route('login'))
                ->with('error', 'Sesi Anda telah berakhir karena logout di tab lain');
        }

        return $next($request);
    }
}
