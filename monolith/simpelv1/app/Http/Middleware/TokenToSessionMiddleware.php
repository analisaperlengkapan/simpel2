<?php

namespace App\Http\Middleware;

use App\Models\Pengguna\Pengguna;
use Closure;
use Illuminate\Http\Request;
use Symfony\Component\HttpFoundation\Response;
use Tymon\JWTAuth\Facades\JWTAuth;

class TokenToSessionMiddleware
{
    /**
     * Handle an incoming request.
     *
     * @param  \Closure(\Illuminate\Http\Request): (\Symfony\Component\HttpFoundation\Response)  $next
     */
    public function handle(Request $request, Closure $next): Response
    {

        $user = JWTAuth::parseToken()->user();
        if ($user) {
            $userData = Pengguna::setUserdata($user);
            session()->put('userData', $userData);
        }

        return $next($request);
    }
}
