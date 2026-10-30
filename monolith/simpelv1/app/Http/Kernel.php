<?php

namespace App\Http;

use App\Http\Middleware\Authenticate;
use App\Http\Middleware\EncryptCookies;
use App\Http\Middleware\EnforceTokenRevocation;
use App\Http\Middleware\Ensure2FAIsVerified;
use App\Http\Middleware\PreventRequestsDuringMaintenance;
use App\Http\Middleware\RedirectIfAuthenticated;
use App\Http\Middleware\TokenToOAuthMiddleware;
use App\Http\Middleware\TokenToSessionMiddleware;
use App\Http\Middleware\TrimStrings;
use App\Http\Middleware\TrustProxies;
use App\Http\Middleware\ValidateCrossTabSession;
use App\Http\Middleware\ValidateSignature;
use App\Http\Middleware\VerifyCsrfToken;
use Illuminate\Auth\Middleware\AuthenticateWithBasicAuth;
use Illuminate\Auth\Middleware\Authorize;
use Illuminate\Auth\Middleware\EnsureEmailIsVerified;
use Illuminate\Auth\Middleware\RequirePassword;
use Illuminate\Cookie\Middleware\AddQueuedCookiesToResponse;
use Illuminate\Foundation\Http\Kernel as HttpKernel;
use Illuminate\Foundation\Http\Middleware\ConvertEmptyStringsToNull;
use Illuminate\Foundation\Http\Middleware\HandlePrecognitiveRequests;
use Illuminate\Foundation\Http\Middleware\ValidatePostSize;
use Illuminate\Http\Middleware\HandleCors;
use Illuminate\Http\Middleware\SetCacheHeaders;
use Illuminate\Routing\Middleware\SubstituteBindings;
use Illuminate\Routing\Middleware\ThrottleRequests;
use Illuminate\Session\Middleware\AuthenticateSession;
use Illuminate\Session\Middleware\StartSession;
use Illuminate\View\Middleware\ShareErrorsFromSession;

class Kernel extends HttpKernel
{
    /**
     * The application's global HTTP middleware stack.
     *
     * These middleware are run during every request to your application.
     *
     * @var array<int, class-string|string>
     */
    protected $middleware = [
        // \App\Http\Middleware\TrustHosts::class,
        TrustProxies::class,
        HandleCors::class,
        PreventRequestsDuringMaintenance::class,
        ValidatePostSize::class,
        TrimStrings::class,
        ConvertEmptyStringsToNull::class,
    ];

    /**
     * The application's route middleware groups.
     *
     * @var array<string, array<int, class-string|string>>
     */
    protected $middlewareGroups = [
        'web' => [
            EncryptCookies::class,
            AddQueuedCookiesToResponse::class,
            StartSession::class,
            ShareErrorsFromSession::class,
            VerifyCsrfToken::class,
            SubstituteBindings::class,
        ],

        'api' => [
            // \Laravel\Sanctum\Http\Middleware\EnsureFrontendRequestsAreStateful::class,
            ThrottleRequests::class.':api',
            StartSession::class,
            // \App\Http\Middleware\TokenToSessionMiddleware::class,
            SubstituteBindings::class,
        ],
    ];

    /**
     * The application's route middleware priority.
     *
     * `token2oauth` MUST run before `auth`. It is the middleware that, for an
     * unauthenticated browser, starts the Portal OAuth flow (stashing the
     * single-use `state`) instead of letting `auth` 302 to the local login
     * form. Laravel sorts route middleware by this list, and `Authenticate` is
     * on the framework default list — so without the entries below `auth` wins
     * the sort regardless of the order written in the route group, and every
     * protected v1 route answers `302 /auth/login`. The Portal flow then never
     * starts, which is what made the v1↔Portal handoff dead on arrival.
     *
     * `cross-tab-session` and `authenc-revocation` come after `auth` because
     * both read the authenticated session.
     *
     * @var array<int, class-string|string>
     */
    protected $middlewarePriority = [
        HandlePrecognitiveRequests::class,
        EncryptCookies::class,
        AddQueuedCookiesToResponse::class,
        StartSession::class,
        ShareErrorsFromSession::class,
        TokenToOAuthMiddleware::class,
        Authenticate::class,
        Ensure2FAIsVerified::class,
        ValidateCrossTabSession::class,
        EnforceTokenRevocation::class,
        ThrottleRequests::class,
        SubstituteBindings::class,
        Authorize::class,
    ];

    /**
     * The application's middleware aliases.
     *
     * Aliases may be used instead of class names to conveniently assign middleware to routes and groups.
     *
     * @var array<string, class-string|string>
     */
    protected $middlewareAliases = [
        'auth' => Authenticate::class,
        'auth.basic' => AuthenticateWithBasicAuth::class,
        'auth.session' => AuthenticateSession::class,
        'cache.headers' => SetCacheHeaders::class,
        'can' => Authorize::class,
        'guest' => RedirectIfAuthenticated::class,
        'password.confirm' => RequirePassword::class,
        'precognitive' => HandlePrecognitiveRequests::class,
        'signed' => ValidateSignature::class,
        'throttle' => ThrottleRequests::class,
        'verified' => EnsureEmailIsVerified::class,
        'token2session' => TokenToSessionMiddleware::class,
        'token2oauth' => TokenToOAuthMiddleware::class,
        'cross-tab-session' => ValidateCrossTabSession::class,
        'authenc-revocation' => EnforceTokenRevocation::class,
        // '2fa' => \App\Http\Middleware\Ensure2FAIsVerified::class,
        '2fa' => Ensure2FAIsVerified::class,
    ];
}
