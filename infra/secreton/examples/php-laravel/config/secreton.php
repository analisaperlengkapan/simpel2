<?php

return [
    /*
    |--------------------------------------------------------------------------
    | Secreton Server URL
    |--------------------------------------------------------------------------
    |
    | The base URL of the Secreton server.
    |
    */
    'host' => env('SECRETON_HOST', 'http://127.0.0.1:8200'),

    /*
    |--------------------------------------------------------------------------
    | Authentication
    |--------------------------------------------------------------------------
    |
    | You can provide a pre-existing token (e.g. from env injection) or
    | credentials for auto-login.
    |
    */
    'token' => env('SECRETON_TOKEN'),

    'username' => env('SECRETON_USERNAME'),
    'password' => env('SECRETON_PASSWORD'),

    /*
    |--------------------------------------------------------------------------
    | Timeout
    |--------------------------------------------------------------------------
    |
    | Connection timeout in seconds.
    |
    */
    'timeout' => 5,
];
