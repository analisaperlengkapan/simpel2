<?php

namespace App\Http\Controllers;

use App\Helpers\MyHelper;
use App\Models\Files as Model;
use App\Models\Master;
use App\Models\Notifikasi;
use Illuminate\Http\Request;

class MediaManajemenDigitalController extends Controller
{
    /**
     * Display a listing of the resource.
     */
    protected $breadcums = ['Media Manajemen Digital'];

    private $controller = 'media-manajemen-digital';

    public function index(Request $request)
    {
        $data = $this->getData($request);

        // return view('media-manajemen-digitalV', $data);
        return view('media-manajemen-digital-newV', $data);
    }

    public function getData($request, $kategori = null)
    {
        $files = new Model;
        $kategories = $files::getKategori();

        if ($kategori) {
            $searchParams['kategori_slug'] = $kategori;
        }
        $baseLength = 20;
        $page = $request->input('page') ?? 1;
        $pagingParams = [
            'length' => $baseLength,
            'start' => $page == 1 ? 0 : $baseLength * $page,
        ];
        $fotos = $files->getDataGrid($pagingParams, $searchParams ?? []);

        foreach ($fotos['data'] as $key => $value) {
            $isExists = \File::exists($value->path);
            $fotos['data'][$key]->path = $isExists ? $value->path : 'assets/images/no-image.jpeg';
            $fotos['data'][$key]->filetype = $isExists ? $value->filetype : 'jpeg';
        }
        $paging = MyHelper::paginate($fotos['total'], $pagingParams['length'], $page);
        $data = [
            'kategories' => $kategories,
            'aktif' => $kategori,
            'controller' => $this->controller,
            'breadcums' => $this->breadcums,
            'fotos' => $fotos['data'],
            'paginationElms' => $paging,
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
    public function show(Request $request, string $id)
    {
        $data = $this->getData($request, $id);

        return view('media-manajemen-digital-newV', $data);
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
