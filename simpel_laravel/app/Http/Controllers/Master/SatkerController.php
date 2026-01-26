<?php

namespace App\Http\Controllers\Master;
use Illuminate\Routing\Controller;

use App\Models\Master\MsSatker;
use Illuminate\Http\Request;
use App\Helpers\MyHelper;
use Illuminate\Support\Facades\DB;
use Symfony\Component\HttpKernel\Exception\NotFoundHttpException;

class SatkerController extends Controller
{
    /**
     * Display a listing of the resource.
     */
    //protected $breadcums = ['Master', 'Satker'];
    protected $kategoriJudul = 'Satker';
    protected $controller = 'master/satker';
    protected $breadcums = ['Master'];
    protected $columns = ['Kode Wilayah', 'Kode Satker', 'Nama Satker', 'Kode Monsakti', 'Kode MySimkari', 'Alamat', 'Telepon', 'Fax', 'Jenis', 'Level', 'Kepala'];
    protected $defColumns = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

    public function __construct()
    {
        $this->breadcums = array_merge($this->breadcums, [['link' => $this->controller, 'title' => 'Satker']]);

    }

    public function index()
    {
        $columns = [
            'Kode Wilayah',
            'Kode Satker',
            'Nama Satker',
            'Kode Monsakti',
            'Kode MySimkari',
            'Alamat',
            'Telepon',
            'Fax',
            'Jenis',
            'Level',
            'Kepala',
        ];
        $defColumns = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        return view('master.satker.satkerV', [
            'tableId' => 'dt-satker',
            'kategoriJudul' => $this->kategoriJudul,
            'breadcums' => $this->breadcums,
            'columns' => $columns,
            'defColumns' => $defColumns,
            'controller' => $this->controller
        ]);
    }

    public function gridData(Request $request)
    {
        $model = new MsSatker();
        $pagingParams = $request->only(['start', 'length']);
        //$searchParams =  $request->only(['search',  'filterBy']);
        $searchParams = $request->only(['columns']);
        $data = $model->getDataGrid($pagingParams, $searchParams);
        return response()->json([
            'data' => $data['data'],
            'recordsTotal' => $data['total'],
            'recordsFiltered' => $data['total'],
        ]);
    }

    function getData($id = null)
    {
        $model = [];
        $isNew = true;

        if ($id) {
            $model = MsSatker::where('id', $id)->first();
            if (!$model) {
                throw new NotFoundHttpException('Data Tidak Ditemukan');
            }

            $model = $model->toArray();
            $isNew = false;
        }

        $data = [
            'model' => $model,
            'isNew' => $isNew,
        ];
        return $data;
    }

    /**
     * Show the form for creating a new resource.
     */
    public function create()
    {
        $data = $this->getData();
        return view('master.satker.satkerFormV', $data);
    }

    /**
     * Store a newly created resource in storage.
     */
    public function store(Request $request)
    {
        $request->validate([
            //'inst_satkerinduk' => 'required',
            //'inst_satkerkd' => 'required',
            //'inst_nama' => 'required',
            //'inst_jenis' => 'required'
        ]);


        $data = new MsSatker();
        $id = isset($request->input()['id']) ? $request->input()['id'] : null;
        if ($id) {
            $data = MsSatker::find($id);
        }
        $data->fill($request->input());
        $data->save();
        return $this->resSuccess();
    }

    /**
     * Display the specified resource.
     */
    public function show(string $id)
    {
        $data = $this->getData($id);
        return view('master.satker.satkerFormV', $data);
    }

    /**
     * Show the form for editing the specified resource.
     */
    public function edit(MsSatker $satker)
    {
        //
    }

    /**
     * Update the specified resource in storage.
     */
    public function update(Request $request, MsSatker $satker)
    {
        //
    }

    /**
     * Remove the specified resource from storage.
     */
    public function destroy(string $id)
    {
        try {
            DB::beginTransaction();
            MsSatker::destroy($id);
            DB::commit();
            return $this->resSuccess('Berhasil Dihapus!');
        } catch (\Throwable $th) {
            DB::rollBack();
        }
    }
}
