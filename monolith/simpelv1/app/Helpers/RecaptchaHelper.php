<?php

namespace App\Helpers;

use Illuminate\Support\Facades\Http;

class RecaptchaHelper
{
    /**
     * Verifikasi token reCAPTCHA v3.
     * Jika RECAPTCHA_ENABLED=false → bypass.
     */
    public static function verify(string $token): array
    {
        if (! config('app.recaptcha_enabled')) {
            return [
                'success' => true,
                'score'   => 1.0,
                'action'  => 'bypass',
            ];
        }

        $response = Http::asForm()->post('https://www.google.com/recaptcha/api/siteverify', [
            'secret'   => config('app.recaptcha_secret_key'),
            'response' => $token,
        ]);

        return $response->json();
    }
}
