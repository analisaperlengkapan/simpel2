<?php

namespace App\Http\Controllers\Suport;

use App\Helpers\MyHelper;
use App\Http\Controllers\Controller;
use App\Models\Suport\Topik;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\DB;
use Symfony\Component\HttpKernel\Exception\NotFoundHttpException;

class TopikController extends Controller
{
    /**
     * Display a listing of the resource.
     */
    protected $kategoriJudul = 'Topik-Kategori';
    protected $controller = '/suport/topik';
    protected $breadcums = ['Suport'];
    protected $columns = ['ID', 'Topik', 'Status'];
    protected $defColumns = [0,1,2];

    public function __construct()
    {
        $this->breadcums = array_merge($this->breadcums, [['link'=>$this->controller,'title'=>'Helpdesk']]);
        
    }
    public function index()
    {
        if(session('userData.current_role.ms_satker_id_keu')){
            $isdelete = true;
        }else{
            $isdelete = false;
        }
            return view('suport.topik.topikV', [
                'tableId' => 'dt-topik',
                'kategoriJudul' => $this->kategoriJudul,
                'breadcums' => $this->breadcums,
                //'columns' => $this->columns,
                //'defColumns' => $this->defColumns,
                'controller' => $this->controller
            ]);
    }
	protected function canCreatePermintaan()
    {
        // return true;
        return in_array(session('userData.current_role.ms_role_id'), config('constants.pelaksana_role'));
    }
    public function gridData(Request $request)
    {
        $model = new Topik();
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
        if(session('userData.current_role.ms_satker_id_keu')){
            $isupdateStatus = true;
        }else{
            $isupdateStatus = false;
        }
        
        $breadcum = 'Tambah';
        if ($id) {
            $breadcum = 'Ubah';
            $model = Topik::where('id', $id)->first();
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
            'isupdateStatus' => $isupdateStatus,
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
        return view('suport.topik.topikFormV', $data);
    }

    public function store(Request $request)
    {        
        $request->validate([
            'topik' => 'required'
        ]);
            $data['topik']      =$request->topik;
            if ($request->status) {
                $data['status'] = $request->status;
            } else {
                $data['status'] = 'Open';
            }
        
        $id = $request->input('id') ?? MyHelper::getPk(date('Ymd'), 'suport_topik_seq');
        Topik::updateOrCreate(['id' => $id],$data);
        return $this->resSuccess();
    }

    

    public function edit(string $id)
    {
        $model = [];
		$data = $this->getData($id);
		$model = Topik::where('id', $id)->first();
            if (!$model) {
                throw new NotFoundHttpException('Data Tidak Ditemukan');
            }
        $isNew = false;
		$status = ['Open', 'Close'];
        $data = [
			'model' => $model,
            'isNew' => $isNew,
			'breadcums' => $this->breadcums,
			'kategoriJudul' => $this->kategoriJudul,
            'statusOptions' => MyHelper::generateSelectOptions([
                'data' => $status,
                'text' => 'status',
                'value' => null,
                'selected' => $model['status'] ?? null,
            ]),    
        ];
        return view('suport.topik.topikFormV', $data);
    }
	
    public function update(Request $request, Topik $tanah)
    {
        //
    }
    public function show(string $id)
    {
        $model = [];
		$data = $this->getData($id);
		$model = Topik::where('id', $id)->first();
            if (!$model) {
                throw new NotFoundHttpException('Data Tidak Ditemukan');
            }
		
		$data = [
			'model' => $model,
			'breadcums' => $this->breadcums,
			'kategoriJudul' => $this->kategoriJudul
        ];
        return view('suport.topik.topikFormV', $data);
    }
    public function destroy(string $id)
    {
        try {
            DB::beginTransaction();
            Topik::destroy($id);
            DB::commit();
            return $this->resSuccess('Berhasil Dihapus!');
        } catch (\Throwable $th) {
            DB::rollBack();
        }
    }

}
