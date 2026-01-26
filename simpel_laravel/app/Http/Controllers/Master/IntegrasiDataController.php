<?php

namespace App\Http\Controllers\Master;
use Illuminate\Routing\Controller;

use App\Helpers\MyHelper;
use App\Models\Master\MsIntegrasiData as Model;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\DB;
use Symfony\Component\HttpKernel\Exception\NotFoundHttpException;

class IntegrasiDataController extends Controller
{
    /**
     * Display a listing of the resource.
     */
    protected $breadcums = ['Master', 'Integrasi Data'];
    public function index()
    {
        return view('master.integrasi-data.gridV', ['tableId' => 'dt-interasi', 'breadcums' => $this->breadcums]);
    }

    public function gridData(Request $request)
    {
        $model = new Model();
        $pagingParams = $request->only(['start', 'length']);
        $searchParams =  $request->only(['search',  'filterBy']);
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
            $model = Model::where('id', $id)->first();
            if (!$model) {
                throw new NotFoundHttpException('Data Tidak Ditemukan');
            }

            $model = $model->toArray();
            $isNew = false;
        }
        if($readOnly){
            $breadcum = 'Detail';
        }
        $data = [
            'model' => $model,
            'isNew' => $isNew,
            'breadcums' => array_merge($this->breadcums, [$breadcum]),
            'judul' => $breadcum,
            'readOnly' => $readOnly,
        ];
        //dd($data['satkerOptions']);
        return $data;
    }

    /**
     * Show the form for creating a new resource.
     */
    public function create()
    {
        $data = $this->getData();
        return view('master.integrasi-data.formV', $data);
    }

    /**
     * Store a newly created resource in storage.
     */
    public function store(Request $request)
    {
        $customMessages = [
            'host.required' => 'Host harus diisi ',
            'username.required' => 'Username harus diisi ',
            'password.required' => 'Password harus diisi ',
            'nama_aplikasi.required' => 'Nama Aplikasi harus diisi ',
        ];
        $request->validate([
            'host' => 'required',
            'username' => 'required',
            'password' => 'required',
            'nama_aplikasi' => 'required'
        ], $customMessages);
        $id = $request->input('id') ?? MyHelper::getPk(date('Ymd'), 'ms_integrasi_data_seq');
        Model::updateOrCreate(['id' => $id],$request->input());
        return $this->resSuccess();
    }

    /**
     * Display the specified resource.
     */
    public function show(string $id)
    {
        $data = $this->getData($id);
        return view('master.integrasi-data.formV', $data);
    }

    /**
     * Show the form for editing the specified resource.
     */
    public function edit(string $id)
    {
    }

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
    public function destroy(string $id)
    {
        try {
            DB::beginTransaction();
            Model::destroy($id);
            DB::commit();
            return $this->resSuccess('Berhasil Dihapus!');
        } catch (\Throwable $th) {
            DB::rollBack();
        }
    }
}
