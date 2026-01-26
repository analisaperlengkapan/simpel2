<?php

namespace App\Http\Controllers\Dashboard;
use Illuminate\Routing\Controller;

use App\Models\Master;
use Illuminate\Http\Request;

class DashboardController extends Controller
{
    /**
     * Display a listing of the resource.
     */
    public function index(Request $request)
    {
        // $create  = User::create(['name' => 'test', 'email' => 'test', 'password' => 'password', 'nip' => '1231231']);
        // $lastId = $create->value('id');
        // dd($lastId);
        // $data = $request->session()->all();

        return view('dashboard');
    }


    /**
     * Show the form for creating a new resource.
     */
    public function create()
    {
        //
    }

    /**
     * Store a newly created resource in storage.
     */
    public function store(Request $request)
    {
        //
    }

    /**
     * Display the specified resource.
     */
    public function show(string $id)
    {
        //
    }

    /**
     * Show the form for editing the specified resource.
     */
    public function edit(string $id)
    {
        //
    }

    /**
     * Update the specified resource in storage.
     */
    public function update(Request $request, string $id)
    {
        //
    }

    /**
     * Remove the specified resource from storage.
     */
    public function destroy(string $id)
    {
        //
    }

    public function error($code, $msg)
    {
        return view('error', ['errCode' => $code, 'errMsg' => $msg]);
    }

    public function searchPegawai(string $nip)
    {
        $pegawai = Master::getPegawaiByNip($nip);
        if ($pegawai)
            return $this->resSuccess('ok', null, $pegawai);
        return $this->resError('Pegawai Tidak ditemukan');
    }
}
