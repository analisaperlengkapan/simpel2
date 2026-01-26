<?php

namespace App\Http\Controllers\Referensi;
use Illuminate\Routing\Controller;

use App\Helpers\MyHelper;
use App\Models\Referensi\Kelompok;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\DB;
use Symfony\Component\HttpKernel\Exception\NotFoundHttpException;

class KelompokController extends Controller
{
    /**
     * Display a listing of the resource.
     */
    //protected $breadcums = ['Referensi-Kode Barang', 'Kelompok'];
    protected $kategoriJudul = 'Kode Barang - Kelompok';
    protected $controller = 'referensi/kelompok';
    protected $breadcums = ['Master'];
    protected $columns = ['Kode Kelompok', 'Deskripsi'];
    protected $defColumns = [0,1];

    public function __construct()
    {
        $this->breadcums = array_merge($this->breadcums, [['link'=>$this->controller,'title'=>'Kode Barang - Kelompok']]);

    }

    public function index()
    {
        //return view('referensi.kelompok.kelompokV', ['tableId' => 'dt-kelompok', 'breadcums' => $this->breadcums]);
        return view('referensi.kelompok.kelompokV', [
            'tableId' => 'dt-kelompok',
            'kategoriJudul' => $this->kategoriJudul,
            'breadcums' => $this->breadcums,
            'columns' => $this->columns,
            'defColumns' => $this->defColumns,
            'controller' => $this->controller
        ]);
    }

    public function gridData(Request $request)
    {
        $model = new Kelompok();
        $pagingParams = $request->only(['start', 'length']);
        //$searchParams =  $request->only(['search',  'filterBy']);
        $searchParams =  $request->only(['columns']);
        $data = $model->getDataGrid($pagingParams, $searchParams);

        return response()->json([
            'data' => $data['data'],
            'recordsTotal' => $data['total'],
            'recordsFiltered' => $data['total'],
        ]);
    }

    function getData($id = null, $readOnly = false)
    {
        $model = [];
        $isNew = true;
        $breadcum = 'Tambah';
        if ($id) {
            $breadcum = 'Ubah';
            $model = Kelompok::where('kdkel', $id)->first();
            if (!$model) {
                throw new NotFoundHttpException('Data Tidak Ditemukan');
            }

            $model = $model->toArray();
            $isNew = false;
        }
        if($readOnly){
            $breadcum = 'Detail';
        }
        $status = ['Open', 'Close'];
        $data = [
            'model' => $model,
            'isNew' => $isNew,
            'breadcums' => array_merge($this->breadcums, [$breadcum]),
            'judul' => $breadcum,
            'readOnly' => $readOnly
        ];
        return $data;
    }



    public function show(string $id)
    {
        $data = $this->getData($id, true);
        return view('support.kritik.kritikFormV', $data);
    }



}
