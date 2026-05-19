<?php

namespace App\Http\Controllers\Referensi;

use App\Http\Controllers\Controller;
use App\Models\Referensi\Bidang;
use Illuminate\Http\Request;
use Symfony\Component\HttpKernel\Exception\NotFoundHttpException;

class BidangController extends Controller
{
    /**
     * Display a listing of the resource.
     */
    // protected $breadcums = ['Referensi-Kode Barang', 'Golongan'];
    protected $kategoriJudul = 'Kode Barang - Bidang';

    protected $controller = 'referensi/bidang';

    protected $breadcums = ['Master'];

    protected $columns = ['Kode Bidang', 'Deskripsi'];

    protected $defColumns = [0, 1];

    public function __construct()
    {
        $this->breadcums = array_merge($this->breadcums, [['link' => $this->controller, 'title' => 'Kode Barang - Bidang']]);

    }

    public function index()
    {
        $columns = [
            'Kode Bidang',
            'Deskripsi',
        ];
        $defColumns = [0, 1];

        return view('referensi.bidang.bidangV', [
            'tableId' => 'dt-bidang',
            'kategoriJudul' => $this->kategoriJudul,
            'breadcums' => $this->breadcums,
            'columns' => $columns,
            'defColumns' => $defColumns,
            'controller' => $this->controller,
        ]);
    }

    public function gridData(Request $request)
    {
        $model = new Bidang;
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
            $model = Bidang::where('kdbid', $id)->first();
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
