<?php

namespace App\Http\Controllers\Suport;

use App\Http\Controllers\Controller;
use App\Models\BukuPanduan;
use App\Models\Master;
use App\Models\Notifikasi;
use Illuminate\Http\Request;

class BukuPanduanController extends Controller
{
    /**
     * Display a listing of the resource.
     */
    protected $breadcums = ['Support', 'Buku Panduan'];

    private $controller = '/suport/buku-panduan';

    public function index()
    {
        $data = $this->getData();

        return view('suport.bukuPanduanV', $data);
    }

    public function getData($platform = 'WEB', $kategori = null)
    {
        $panduans = BukuPanduan::where('platform', strtoupper($platform))->get();
        $lastUpdate = BukuPanduan::limit(1)->orderBy('created_at', 'desc')->first();
        // $kategori = BukuPanduan::getKategori();
        $mapped = [];
        foreach ($panduans as $key => $panduan) {
            // code...
            $mapped[$panduan->kategori][] = $panduan;
        }
        $data = [
            'controller' => $this->controller,
            'breadcums' => $this->breadcums,
            'panduans' => $mapped,
            'lastUpdate' => $lastUpdate->created_at,
        ];

        return $data;
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
    public function show(Request $request, string $id) {}

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
        if ($pegawai) {
            return $this->resSuccess('ok', null, $pegawai);
        }

        return $this->resError('Pegawai Tidak ditemukan');
    }

    public function getNotif()
    {
        $notifs = Notifikasi::getNotifs();

        return response()->json($notifs, 200);
    }
}
