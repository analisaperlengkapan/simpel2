<?php

namespace App\Http\Controllers\Master;
use Illuminate\Routing\Controller;

use App\Models\Master\MsWilayah;
use Illuminate\Http\Request;
use App\Helpers\MyHelper;
use Illuminate\Support\Facades\DB;
use Symfony\Component\HttpKernel\Exception\NotFoundHttpException;

class WilayahController extends Controller
{
    /**
     * Display a listing of the resource.
     */
    protected $breadcums = ['Master', 'Satker'];
    protected $columns = ['Kode Satker', 'Nama Satker', 'Kode Monsakti', 'Kode MySimkari', 'Alamat', 'Telepon', 'Fax', 'Jenis', 'Level', 'Kepala'];
    protected $defColumns = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    public function index()
    {
        return view('master.wilayah.wilayahV', [
            'tableId' => 'dt-kritik',
            'breadcums' => $this->breadcums,
            'defColumns' => $this->defColumns,
            'columns' => $this->columns,
        ]);
    }

    public function gridData(Request $request)
    {
        $model = new MsWilayah();
        $pagingParams = $request->only(['start', 'length']);
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
            $model = MsWilayah::where('id', $id)->first();
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
        return view('master.wilayah.wilayahFormV', $data);
    }

    /**
     * Store a newly created resource in storage.
     */
    public function store(Request $request)
    {
        $request->validate([
            //'inst_wilayahinduk' => 'required',
            //'inst_wilayahkd' => 'required',
            //'inst_nama' => 'required',
            //'inst_jenis' => 'required'
        ]);


        $data = new MsWilayah();
        $id = isset($request->input()['id']) ? $request->input()['id'] : null;
        if ($id) {
            $data = MsWilayah::find($id);
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
        return view('master.wilayah.wilayahFormV', $data);
    }

    /**
     * Show the form for editing the specified resource.
     */
    public function edit(MsWilayah $wilayah)
    {
        //
    }

    /**
     * Update the specified resource in storage.
     */
    public function update(Request $request, MsWilayah $wilayah)
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
            MsWilayah::destroy($id);
            DB::commit();
            return $this->resSuccess('Berhasil Dihapus!');
        } catch (\Throwable $th) {
            DB::rollBack();
        }
    }
}
