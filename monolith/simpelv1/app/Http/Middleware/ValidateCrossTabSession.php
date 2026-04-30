<?php

namespace App\Http\Middleware;

use Closure;
use Illuminate\Http\Request;
use Illuminate\Http\Response;
use Illuminate\Support\Facades\Auth;

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

            return response()->json([
                'error' => 'Session invalidated from another tab',
                'code' => 'SESSION_INVALIDATED',
            ], 401);
        }

        return $next($request);
    }
}
