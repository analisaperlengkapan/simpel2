<?php

namespace App\Http\Controllers\Bantuan;
use Illuminate\Routing\Controller;

use App\Helpers\MyHelper;
use App\Models\Bantuan\Kritik;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\DB;
use Symfony\Component\HttpKernel\Exception\NotFoundHttpException;

class KritikController extends Controller
{
    /**
     * Display a listing of the resource.
     */
    protected $breadcums = ['Support', 'Kritik dan Saran'];
    public function index()
    {
        return view('support.kritik.kritikV', ['tableId' => 'dt-kritik', 'breadcums' => $this->breadcums]);
    }

    public function gridData(Request $request)
    {
        $model = new Kritik();
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
            $model = Kritik::where('id', $id)->first();
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
            'readOnly' => $readOnly,            
            'statusOptions' => MyHelper::generateSelectOptions([
                'data' => $status,
                'text' => 'status',
                'value' => null,
                'selected' => $model['status'] ?? null,
            ]),    
        ];
        return $data;
    }

    public function create()
    {
        $data = $this->getData();
        return view('support.kritik.kritikFormV', $data);
    }

    public function store(Request $request)
    {        
        $request->validate([
            //'kode_tiket' => 'required',
            //'judul' => 'required',
            'kritik' => 'required',
            'saran' => 'required'
        ]);
        
        //dd($request->all());
       
            //$data['kode_tiket'] =$request->kode_tiket;
            //$data['judul']      =$request->judul;
            $data['kritik']     =$request->kritik;
            $data['saran']      =$request->saran;
            if(session('userData.current_role.ms_satker_id_keu')){
                $data['kode_satker'] = session('userData.current_role.ms_satker_id_keu');
            }
        if($request->status){
            $data['status'] =$request->status;
        }else{
            $data['status'] ='Open';
        }
        
        $id = $request->input('id') ?? MyHelper::getPk(date('Ymd'), 'support_kritik_seq');
        $data['kode_tiket'] =$id;
        Kritik::updateOrCreate(['id' => $id],$data);
        return $this->resSuccess();
    }

    public function show(string $id)
    {
        $data = $this->getData($id, true);
        return view('support.kritik.kritikFormV', $data);
    }

    public function edit(string $id)
    {
        $data = $this->getData($id);
        return view('support.kritik.kritikFormV', $data);
    }

    public function update(Request $request, Helpdesk $tanah)
    {
        //
    }

    public function destroy(string $id)
    {
        try {
            DB::beginTransaction();
            Kritik::destroy($id);
            DB::commit();
            return $this->resSuccess('Berhasil Dihapus!');
        } catch (\Throwable $th) {
            DB::rollBack();
        }
    }

}
