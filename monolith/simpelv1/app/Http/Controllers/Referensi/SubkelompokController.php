<?php

namespace App\Http\Controllers\Referensi;

use App\Http\Controllers\Controller;
use App\Models\Referensi\Subkelompok;
use Illuminate\Http\Request;
use Symfony\Component\HttpKernel\Exception\NotFoundHttpException;

class SubkelompokController extends Controller
{
    /**
     * Display a listing of the resource.
     */
    // protected $breadcums = ['Referensi-Kode Barang', 'Sub Kelompok'];
    protected $kategoriJudul = 'Kode Barang - Sub Kelompok';

    protected $controller = 'referensi/subkelompok';

    protected $breadcums = ['Master'];

    protected $columns = ['Kode Sub Kelompok', 'Deskripsi'];

    protected $defColumns = [0, 1];

    public function __construct()
    {
        $this->breadcums = array_merge($this->breadcums, [['link' => $this->controller, 'title' => 'Kode Barang - Sub Kelompok']]);

    }

    public function index()
    {
        // return view('referensi.subkelompok.subkelompokV', ['tableId' => 'dt-subkelompok', 'breadcums' => $this->breadcums]);
        return view('referensi.subkelompok.subkelompokV', [
            'tableId' => 'dt-subkelompok',
            'kategoriJudul' => $this->kategoriJudul,
            'breadcums' => $this->breadcums,
            'columns' => $this->columns,
            'defColumns' => $this->defColumns,
            'controller' => $this->controller,
        ]);
    }

    public function gridData(Request $request)
    {
        $model = new Subkelompok;
        $pagingParams = $request->only(['start', 'length']);
        // $searchParams =  $request->only(['search',  'filterBy']);
        $searchParams = $request->only(['columns']);
        $data = $model->getDataGrid($pagingParams, $searchParams);

        return response()->json([
            'data' => $data['data'],
            'recordsTotal' => $data['total'],
            'recordsFiltered' => $data['total'],
        ]);
    }

    public function getData($id = null, $readOnly = false)
    {
        $model = [];
        $isNew = true;
        $breadcum = 'Tambah';
        if ($id) {
            $breadcum = 'Ubah';
            $model = Subkelompok::where('kdskel', $id)->first();
            if (! $model) {
                throw new NotFoundHttpException('Data Tidak Ditemukan');
            }

            $model = $model->toArray();
            $isNew = false;
        }
        if ($readOnly) {
            $breadcum = 'Detail';
        }
        $status = ['Open', 'Close'];
        $data = [
            'model' => $model,
            'isNew' => $isNew,
            'breadcums' => array_merge($this->breadcums, [$breadcum]),
            'judul' => $breadcum,
            'readOnly' => $readOnly,
        ];

        return $data;
    }

    public function show(string $id)
    {
        $data = $this->getData($id, true);

        return view('suport.kritik.kritikFormV', $data);
    }
}
