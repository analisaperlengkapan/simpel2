<?php

namespace App\Http\Controllers\Pengaturan\MenuTema;

use App\Http\Controllers\Controller;
use App\Models\Files;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\DB;

class TemaController extends Controller
{
    /**
     * Display a listing of the resource.
     */
    protected $breadcums = ['Pengaturan', 'Menu dan Tema Aplikasi', 'Tema Aplikasi'];

    protected $controller = '/pengaturan/tema';

    public function index()
    {
        $logo = DB::table('ms_setting_qr as lg')->where('lg.id', '=', 1)->first();
        $logonya = (array) $logo;

        // echo "<pre>";print_r($logonya);exit;

        return view('pengaturan.tema.mainV', ['breadcums' => $this->breadcums, 'controller' => $this->controller, 'logotema' => $logonya]);
    }

    public function gridData(Request $request)
    {
        $model = new Model;
        $pagingParams = $request->only(['start', 'length']);
        $searchParams = $request->only(['search',  'filterBy']);
        $data = $model->getDataGrid($pagingParams, $searchParams);

        return response()->json([
            'data' => $data['data'],
            'recordsTotal' => $data['total'],
            'recordsFiltered' => $data['total'],
        ]);
    }

    public function getData($id = null, $readOnly = false) {}

    /**
     * Show the form for creating a new resource.
     */
    public function create() {}

    /**
     * Store a newly created resource in storage.
     */
    public function store(Request $request)
    {
        $customMessages = [];
        $validasi = [];
        if ($request->hasFile('logo')) {
            $validasi['logo'] = 'mimes:jpeg,png,jpg|max:2048';
        }
        if ($request->hasFile('logo_dark')) {
            $validasi['logo_dark'] = 'mimes:jpeg,png,jpg|max:2048';
        }
        $request->validate($validasi, $customMessages);
        try {
            DB::beginTransaction();
            $id = 1;
            $post = [];
            // Ambil field lain dari request
            if ($request->has('copyright')) {
                $post['copyright'] = $request->input('copyright');
            }
            if ($request->has('footer_text')) {
                $post['footer_text'] = $request->input('footer_text');
            }
            // ...tambahkan field lain sesuai kebutuhan
            $params = [
                'kategori' => 'Logo Aplikasi',
                'dir' => 'setting/logo-aplikasi',
                'fileKey' => 'logo',
                'pkey' => $id,
            ];
            $fotos = Files::upload($request, $params);
            if (! empty($fotos)) {
                $post['logo_aplikasi'] = $fotos['path'];
            }

            $params_dark = [
                'kategori' => 'Logo Aplikasi',
                'dir' => 'setting/logo-aplikasi',
                'fileKey' => 'logo_dark',
                'pkey' => $id,
            ];
            $fotos_dark = Files::upload($request, $params_dark);
            if (! empty($fotos_dark)) {
                $post['logo_aplikasi_dark'] = $fotos_dark['path'];
            }

            $params_kecil = [
                'kategori' => 'Logo Aplikasi',
                'dir' => 'setting/logo-aplikasi',
                'fileKey' => 'logo_kecil',
                'pkey' => $id,
            ];
            $fotos_kecil = Files::upload($request, $params_kecil);
            if (! empty($fotos_kecil)) {
                $post['logo_kecil'] = $fotos_kecil['path'];
            }

            DB::table('ms_setting_qr')->where('id', '1')->update($post);

            DB::commit();

            return $this->resSuccess(
                'Berhasil Disimpan!'
            );
        } catch (\Throwable $th) {
            dd($th->getMessage());

            return $this->resError('Gagal menyimpan data');
        }
    }

    /**
     * Display the specified resource.
     */
    public function show(string $id) {}

    /**
     * Show the form for editing the specified resource.
     */
    public function edit(string $id) {}

    /**
     * Update the specified resource in storage.
     */
    public function update(Request $request, Model $model)
    {
        //
    }

    /**
     * Remove the specified resource from storage.
     */
    public function destroy(string $id) {}
}
