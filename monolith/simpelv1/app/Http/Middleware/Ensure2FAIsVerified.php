<?php

namespace App\Http\Middleware;

use Closure;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\Auth;

class Ensure2FAIsVerified
{
    public function handle(Request $request, Closure $next)
    {
        if (
            Auth::check() &&
            Auth::user()->google2fa_secret &&
            ! session('2fa_verified')
        ) {
            return redirect()->route('2fa.index');
        }

        return $next($request);
    }
}
