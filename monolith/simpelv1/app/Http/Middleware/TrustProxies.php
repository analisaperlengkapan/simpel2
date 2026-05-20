<?php

namespace App\Http\Middleware;

use Illuminate\Http\Middleware\TrustProxies as Middleware;
use Illuminate\Http\Request;

class TrustProxies extends Middleware
{
    /**
     * The trusted proxies for this application.
     *
     * Trust ALL upstream proxies ("*") karena simpelv1 deploy di belakang
     * Istio ingress gateway + sidecar yang sah-sah saja kirim X-Forwarded-*
     * headers. Tanpa setting ini, Symfony Request abaikan X-Forwarded-Proto
     * sehingga $request->getScheme() selalu return 'http' (raw connection
     * dari sidecar), bukan 'https' yang user pakai di browser.
     *
     * Side-effect: URL::to(), redirect()->to(), helper url() akan generate
     * Location header dengan scheme HTTPS bila user akses lewat HTTPS,
     * fix issue Location: http://... saat browser akses https://...
     *
     * @var array<int, string>|string|null
     */
    protected $proxies = '*';

    /**
     * The headers that should be used to detect proxies.
     *
     * @var int
     */
    protected $headers =
        Request::HEADER_X_FORWARDED_FOR |
        Request::HEADER_X_FORWARDED_HOST |
        Request::HEADER_X_FORWARDED_PORT |
        Request::HEADER_X_FORWARDED_PROTO |
        Request::HEADER_X_FORWARDED_AWS_ELB;
}
