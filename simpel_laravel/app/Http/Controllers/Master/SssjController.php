<?php

namespace App\Http\Controllers\Master;
use Illuminate\Routing\Controller;

use App\Models\Master\MsSssj;
use Illuminate\Http\Request;
use App\Helpers\MyHelper;
use Illuminate\Support\Facades\DB;
use Symfony\Component\HttpKernel\Exception\NotFoundHttpException;

class SssjController extends Controller
{
    /**
     * Display a listing of the resource.
     */
    public function index()
    {
        return view('master.sssj.sssjV');
    }

    public function gridData(Request $request)
    {
        $model = new MsSssj();
        $pagingParams = $request->only(['start', 'length']);
        $searchParams =  $request->only(['search',  'filterBy']);
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
            $model = MsSssj::where('id', $id)->first();
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
        return view('master.sssj.sssjFormV', $data);
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


        $data = new MsSssj();
        $id = isset($request->input()['id']) ? $request->input()['id'] : null;
        if($id){
            $data = MsSssj::find($id);
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
        return view('master.sssj.sssjFormV', $data);
    }

    /**
     * Show the form for editing the specified resource.
     */
    public function edit(MsSssj $sssj)
    {
        //
    }

    /**
     * Update the specified resource in storage.
     */
    public function update(Request $request, MsSssj $sssj)
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
            MsSssj::destroy($id);
            DB::commit();
            return $this->resSuccess('Berhasil Dihapus!');
        } catch (\Throwable $th) {
            DB::rollBack();
        }
    }
}
