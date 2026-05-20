<?php

namespace App\Http\Controllers;

use Illuminate\Foundation\Auth\Access\AuthorizesRequests;
use Illuminate\Foundation\Validation\ValidatesRequests;
use Illuminate\Routing\Controller as BaseController;

class Controller extends BaseController
{
    use AuthorizesRequests, ValidatesRequests;

    public function resSuccess($msg = 'Ok', $callback = null, $body = [])
    {
        return response([
            'status' => 'success',
            'body' => $body,
            'message' => $msg,
            'callback' => $callback,
        ], 200);
    }

    public function resError($msg = 'Gagal', $callback = null)
    {
        return response([
            'status' => 'danger',
            'message' => $msg,
            'callback' => $callback,
        ], 400);
    }
}
