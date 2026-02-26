<?php

namespace App\Http\Controllers\Suport;

use App\Helpers\MyHelper;
use App\Http\Controllers\Controller;
use App\Models\Suport\Survey;
use App\Models\Suport\UserSurveyAktifitas as Aktifitas;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\DB;
use Symfony\Component\HttpKernel\Exception\NotFoundHttpException;

class SurveyController extends Controller
{
    /**
     * Display a listing of the resource.
     */
    protected $kategoriJudul = 'Survey';
    protected $controller = '/suport/survey';
    protected $breadcums = ['Survey'];
    protected $columns = ['Tgl Survey', 'Pembuat', 'Pertanyaan', 'Deskripsi', 'Status'];
    protected $defColumns = [0,1,2,3,4];
   
    public function __construct()
    {
        $this->breadcums = array_merge($this->breadcums, [['link'=>$this->controller,'title'=>'Survey']]);
        
    }

    public function index()
    {
        if(session('userData.current_role.ms_satker_id_keu')){
            $isdelete = true;
        }else{
            $isdelete = false;
        }
        $columns = [
            'Tgl Survey',
            'Pembuat',
            'Pertanyaan',
            'Deskripsi',
            'Status',
        ];
        $defColumns = [0, 1, 2, 3, 4];
        return view('suport.survey.surveyV', [
            'tableId' => 'dt-survey',
            'kategoriJudul' => $this->kategoriJudul,
            'breadcums' => $this->breadcums,
            'columns' => $columns,
            'defColumns' => $defColumns,
            'controller' => $this->controller,
            'candelete' => $isdelete,
            'canCreate' => $this->canCreateSurvey()
        ]);
    }
	protected function canCreateSurvey()
    {
        // return true;
        return in_array(session('userData.current_role.ms_role_id'), config('constants.buat_survey'));
    }
    public function gridData(Request $request)
    {
        $model = new Survey();
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
            $model = Survey::where('id', $id)->first();
            if (!$model) {
                throw new NotFoundHttpException('Data Tidak Ditemukan');
            }

            $model = $model->toArray();
            $isNew = false;
        }
        if($readOnly){
            $breadcum = 'Detail';
        }
        $tipe = ['Low', 'Medium', 'High'];
        $status = ['Open', 'Close'];
        $data = [
            'model' => $model,
            'isupdateStatus' => $isupdateStatus,
            'isNew' => $isNew,
            'breadcums' => array_merge($this->breadcums, [$breadcum]),
            'judul' => $breadcum,
            'readOnly' => $readOnly,
            'tipeOptions' => MyHelper::generateSelectOptions([
                'data' => $tipe,
                'text' => 'tipe',
                'value' => null,
                'selected' => $model['tipe_tiket'] ?? null,
            ]),
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
        return view('suport.survey.surveyFormV', $data);
    }

    public function store(Request $request)
    {        
        $request->validate([
            'pertanyaan' => 'required',
            'tgl_survey' => 'required',
            'deskripsi' => 'required',
        ]);
        
        //dd($request->all())
        $image ='no image';
        //if($request->file('image')){
        //    $image = $request->file('image')->store('helpdesk-image');

        //}

            
            $data['pertanyaan']      =$request->pertanyaan;
            $data['tgl_survey'] =$request->tgl_survey;
            $data['deskripsi'] =$request->deskripsi;
            
            if(session('userData.current_role.ms_satker_id_keu')){
                $data['kode_satker'] = session('userData.current_role.ms_satker_id_keu');
            }
        // if($request->status){
        //     $data['status'] =$request->status;
        // }else{
            $data['status'] ='Open';
        // }
        
        $id = $request->input('id') ?? MyHelper::getPk(date('Ymd'), 'suport_survey_seq');
        $data['kode_tiket'] =$id;
        Survey::updateOrCreate(['id' => $id],$data);
        return $this->resSuccess();
    }

    public function show(string $id)
    {
        $model = [];
		$data = $this->getData($id);
		$model = Survey::where('id', $id)->first();
            if (!$model) {
                throw new NotFoundHttpException('Data Tidak Ditemukan');
            }
		$aktifitasHistories = Aktifitas::getDetail($id);
		
		$data = [
			'model' => $model,
			'aktifitasHistories' => $aktifitasHistories,
			'breadcums' => $this->breadcums,
			'kategoriJudul' => $this->kategoriJudul
        ];
        return view('suport.survey.surveyFormKomentar_viewV', $data);
    }

    public function edit(string $id)
    {
        $model = [];
		$data = $this->getData($id);
		$model = Survey::where('id', $id)->first();
            if (!$model) {
                throw new NotFoundHttpException('Data Tidak Ditemukan');
            }
		$aktifitasHistories = Aktifitas::getDetail($id);
		
		$data = [
			'model' => $model,
			'aktifitasHistories' => $aktifitasHistories,
			'breadcums' => $this->breadcums,
			'kategoriJudul' => $this->kategoriJudul
        ];
        return view('suport.survey.surveyFormKomentarV', $data);
    }
	public function saveKomentar(Request $request)
    {
        $validasi = [
            'ms_aktifitas_id' => 'required',
        ];
        $request->validate($validasi);

        if ($request->has('id')) {
            $isNew = false;
            $id = $request->input('id');
        } else {
            $isNew = true;
            $id = null;
        }

        $ms_aktifitas_id = $request->input('ms_aktifitas_id');

        if(session('userData.current_role.ms_satker_id_keu')){
                $ms_satker_id = session('userData.current_role.ms_satker_id_keu');
            }else{
				$ms_satker_id = '';
			}
			$user_id=session('userData.current_role.user_id');
			$rote_id=session('userData.current_role.ms_role_id');
	    $created_at=date('Ymd H:i:s');
        try {
            DB::beginTransaction();
            $currentRole = session('userData.current_role');
            Survey::where(['id' => $id])->update(['status' => $ms_aktifitas_id]);
			$ids = MyHelper::getPk(date('Ymd'), 'suport_helpdesk_aktifitas_id_seq');
			
            $dataAktifitas = [
                'id' => $ids,
				'pengajuan_id' => $id,
                'ms_aktifitas_id' => $ms_aktifitas_id,
				'ms_satker_id' => $ms_satker_id,
				'created_at'=>$created_at,
				'nama'=>$user_id,
				'role'=>$rote_id,
                'komentar' => $request->input('komentar')
            ];
            //$acts = Approval::roleCheck($dataAktifitas);
            Aktifitas::insert($dataAktifitas);

            DB::commit();
            return $this->resSuccess(
                'Berhasil Disimpan!',
                [
                    'type' => 'redirect',
                    'url' => \URL::to('/suport/helpdesk')
                ]
            );
        } catch (\Throwable $th) {
            DB::rollBack();
            dd($th->getMessage());
            return $this->resError('Gagal Menyimpan data');
        }
    }
    public function update(Request $request, Helpdesk $tanah)
    {
        //
    }

    public function destroy(string $id)
    {
        try {
            DB::beginTransaction();
            Survey::destroy($id);
            DB::commit();
            return $this->resSuccess('Berhasil Dihapus!');
        } catch (\Throwable $th) {
            DB::rollBack();
        }
    }

}
