<?php

namespace App\Http\Middleware;

use Illuminate\Foundation\Http\Middleware\VerifyCsrfToken as Middleware;

class VerifyCsrfToken extends Middleware
{
    /**
     * The URIs that should be excluded from CSRF verification.
     *
     * @var array<int, string>
     */
    protected $except = [
        // OAuth callback from Portal. The Portal is an external origin
        // and cannot supply a Laravel-issued CSRF token, so the POST
        // variant of this route would always 419 if left in the `web`
        // CSRF group. Login-CSRF / session-fixation is instead prevented
        // by the `state` parameter (verified with `hash_equals` in
        // `AuthController::oauthCallback`) and JWT signature validation.
        // Without this exemption, the POST route documented in
        // `routes/web.php:136-145` is non-functional.
        'auth/oauth-callback',
    ];
}
