<?php

namespace App\Http\Middleware;

use Closure;
use Illuminate\Http\Request;
use Symfony\Component\HttpFoundation\Response;

class TokenMiddleware
{
 public function handle(Request $request, Closure $next)
    {
        $token = $request->bearerToken();
        if ($token !== 'TOKENRAHASIA123') {
            return response()->json(['message' => 'Unauthorized'], 401);
        }

        return $next($request);
    }
}
