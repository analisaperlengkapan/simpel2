<?php

namespace App\Http\Controllers\AnalisisKebutuhan\PakaianDinas;

use App\Http\Controllers\Controller;
use App\Models\Master;
use App\Models\PegawaiPakaianDinas;
use Illuminate\Http\Request;
use Illuminate\Support\Arr;

class UkuranPakaianPegawaiController extends Controller
{
    protected $breadcums = ['Analisis Kebutuhan', 'Pakaian Dinas', 'Ukuran Pakaian Pegawai'];

    private $controller = '/analisis-kebutuhan/pakaian-dinas/ukuran-pakaian-pegawai';

    public function getData()
    {
        $ukurans = Master::getMsUkuranGroup(true);
        $groups = Arr::pluck($ukurans, 'group');
        $default = PegawaiPakaianDinas::where('nip', session('userData.username'))->first();

        if ($default) {
            $default = $default->toArray();
            $selectedUkurans = [
                'BAJU' => $default['ukuran_baju'],
                'CELANA' => $default['ukuran_celana'],
                'SEPATU' => $default['ukuran_sepatu'],
            ];

        }
        $type = request()->wantsJson() ? 'raw' : null;
        $ukuranOptions = Master::getMsUkuranGroupMapped($groups, $selectedUkurans ?? null, $type);
        $data = [
            'breadcums' => $this->breadcums,
            'controller' => $this->controller,
            'ukurans' => $ukuranOptions,
            'title' => 'Ukuran Pakaian Pegawai',
            'default' => $default,
        ];

        if (request()->wantsJson()) {
            return response()->json($data);
        }

        return $data;
    }

    public function index()
    {
        $data = $this->getData();

        return view('analisis_kebutuhan.pakaian_dinas.ukuranPakaianPegawaiV', $data);
    }

    public function store(Request $request)
    {
        $validasi = [
            'ukuran_baju' => 'required',
            'ukuran_celana' => 'required',
            'ukuran_sepatu' => 'required',
        ];
        $request->validate($validasi);
        try {
            $inputan = $request->only(['ukuran_baju', 'ukuran_celana', 'ukuran_sepatu']);
            $inputan['with_hijab'] = $request->input('with_hijab', 0);
            PegawaiPakaianDinas::updateOrCreate(['nip' => session('userData.username')], $inputan);

            return $this->resSuccess('Ok', ['type' => 'redirect', 'url' => $this->controller]);
        } catch (\Throwable $th) {
            $msg = $th->getMessage();

            return $this->resError($msg);
        }

    }
}
