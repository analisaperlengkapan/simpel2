<?php

namespace App\Http\Controllers\AnalisisKebutuhan\Bmn;

use App\Exports\ExportExcel;
use App\Helpers\MyHelper;
use App\Http\Controllers\Controller;
use App\Models\AnalisisKebutuhan\Bmn as Model;
use App\Models\AnalisisKebutuhan\BmnAsset;
use App\Models\AnalisisKebutuhan\BmnSatker;
use App\Models\AnalisisKebutuhan\BmnSatkerAktifitas;
use App\Models\AnalisisKebutuhan\BmnSatkerBarang;
use App\Models\AnalisisKebutuhan\PerioritasImport;
use App\Models\Master\MsSatker as MasterMsSatker;
use App\Models\ApprovalUserSpseSirup;
use App\Models\Master;
use App\Models\Master\MsJenisAset;
use Illuminate\Http\Request;
use Illuminate\Support\Arr;
use Illuminate\Support\Facades\DB;
use Maatwebsite\Excel\Facades\Excel;
use Mccarlosen\LaravelMpdf\Facades\LaravelMpdf;
use Symfony\Component\HttpKernel\Exception\NotFoundHttpException;

class PenyusunanPrioritasController extends Controller
{
    protected $breadcums = ['Analisis Kebutuhan', 'Kebutuhan BMN'];
    private $controller = '/analisis-kebutuhan/bmn/penyusunan-prioritas';
    protected $kategoriJudul = 'Penyusunan Prioritas';

    public function __construct(Request $request)
    {
        $segment = $request->segment(3);
        if($segment == 'cetak-dokumen'){
            $this->kategoriJudul = 'Cetak Dokumen';
            $this->controller = '/analisis-kebutuhan/bmn/cetak-dokumen';
        }
        $this->breadcums = array_merge($this->breadcums, [['link'=>$this->controller,'title'=>$this->kategoriJudul]]);
    }

    public function index()
    {
        $data = [
            'tableId' => 'dt-pengajuan',
            'breadcums' => $this->breadcums,
            'controller' => $this->controller,
            'operasi' => $this->userOperation(),
        ];
        return view('analisis_kebutuhan.bmn.prioritas.gridV', $data);
    }

    protected function userOperation()
    {
        if (session('userData.current_role.ms_role_id') == config('constants.superadmin_role_id')) {
            return 'VIEW';
        }

        if (session('userData.current_role.ms_role_id') == config('constants.validator_pusat_role_id')) {
            return 'CREATE';
        }

        if (session('userData.current_role.ms_role_id') == config('constants.validator_wilayah_role_id')) {
            return 'VIEW';
        }

        if (session('userData.current_role.ms_role_id') == config('constants.pelaksana_satker_role_id')) {
            return 'INPUT';
        }
    }

    public function gridData(Request $request)
    {
        $user = new Model();
        $pagingParams = $request->only(['start', 'length']);
        $searchParams = $request->only(['columns']);

        $data = $user->getGridData($pagingParams, $searchParams);
        return response()->json([
            'data' => $data['data'],
            'recordsTotal' => $data['total'],
            'recordsFiltered' => $data['total'],
        ]);
    }

    public function gridDataSatker(Request $request)
    {
        $user = new Model();
        $pagingParams = $request->only(['start', 'length']);
        $searchParams = $request->only(['columns','id']);
        $data = $user->getGridDataSatkerDetail($pagingParams, $searchParams);
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
        $breadcum = 'Tambah';
        $selectedSatker = [];
        if ($id) {
            $breadcum = 'Ubah';
            $model = Model::where('id', $id)->first();
            if (!$model) {
                throw new NotFoundHttpException('Data Tidak Ditemukan');
            }
            $satkerTerpilih = BmnSatker::where(['pengajuan_kebutuhan_bmn_id' => $id])->get()->toArray();
            if (!empty($satkerTerpilih)) {
                $selectedSatker = Arr::pluck($satkerTerpilih, 'ms_satker_id');
            }
            $model = $model->toArray();
            $isNew = false;
        }
        $satkerTerpilih = Master::getSatkers();
        $data = [
            'model' => $model,
            'isNew' => $isNew,
            'controller' => $this->controller,
            'breadcums' => array_merge($this->breadcums, [$breadcum]),
            'jenisAssetOptions' => MyHelper::generateSelectOptions([
                'data' => Master::getJenisAsset(),
                'text' => 'name',
                'value' => 'id',
                'selected' => $model['id_jenis_asset'] ?? null
            ]),
            'satkerOptions' => MyHelper::generateSelectOptions(['data' => $satkerTerpilih, 'text' => 'inst_nama', 'value' => 'inst_satkerkd', 'selected' => $selectedSatker]),
        ];
        return $data;
    }

    /**
     * Show the form for creating a new resource.
     */
    public function create()
    {
        $data = $this->getData();
        return view('analisis_kebutuhan.bmn.pengajuan.pengajuanFormV', $data);
    }

    /**
     * Store a newly created resource in storage.
     */
    public function store(Request $request)
    {
        $request->validate([
            'excel_file' => 'required|mimes:xlsx,xls',
        ]);
        $file = $request->file('excel_file');
        $import = Excel::import(new PerioritasImport, $file);
        return $this->resSuccess('Berhasil Dismpan');
    }

    public function show(string $id)
    {
        $data = [
            'tableId' => 'dt-pengajuan',
            'breadcums' => array_merge($this->breadcums, ['Daftar Kebutuhan BMN Satker']),
            'controller' => $this->controller,
            'operasi' => $this->userOperation(),
            'id' => $id,
        ];
        return view('analisis_kebutuhan.bmn.prioritas.gridSatkerV', $data);
    }

    public function edit(string $id)
    {
        $pengajuanSatker = BmnSatker::where('id',$id)->first();
        $pengajuan = Model::where('id',$pengajuanSatker['pengajuan_kebutuhan_bmn_id'])->first();
        $satker = MasterMsSatker::where('inst_satkerkd',$pengajuanSatker['ms_satker_id'])->first();

        //aktifitas
        $msAktifitasId = $pengajuanSatker->ms_aktifitas_id ?? 3000;
        $whereAct = ['ms_aktifitas_id' => $msAktifitasId,'group'=>'BMN'];
        $aktifitasHistories = BmnSatkerAktifitas::getDetail($pengajuanSatker['id']);
        $aktifitasOptions = ApprovalUserSpseSirup::getAktifitas($whereAct);
        $currentAktifitas = ApprovalUserSpseSirup::getCurrentAktifitas($msAktifitasId);

        $data = [
            'model' => $pengajuan,
            'pengajuanSatker' => $pengajuanSatker,
            'controller' => $this->controller,
            'breadcums' => array_merge($this->breadcums, [['link'=>'analisis-kebutuhan/bmn/penyusunan-prioritas/'.$pengajuanSatker['pengajuan_kebutuhan_bmn_id'],'title'=>'Daftar Kebutuhan BMN Satker'],'Detail']),
            'satker' => $satker['inst_nama'],
            'operasi' => $this->userOperation(),
            'aktifitasOptions' => $aktifitasOptions,
            'aktifitas' => $currentAktifitas,
            'aktifitasHistories' => $aktifitasHistories,
        ];
        return view('analisis_kebutuhan.bmn.prioritas.formV', $data);
    }

    public function gridDataBarang($id)
    {
        $data = BmnSatkerBarang::where('pengajuan_kebutuhan_bmn_satker_id',$id)
        ->select('pengajuan_kebutuhan_bmn_satker_barang.*',DB::raw('SUM(d.kode_barang) as jumlah_exist'))
        ->leftJoin('pengajuan_kebutuhan_bmn_satker as b', 'pengajuan_kebutuhan_bmn_satker_barang.pengajuan_kebutuhan_bmn_satker_id', '=', 'b.id')
        ->leftJoin('ms_satker as c', 'b.ms_satker_id', '=', 'c.inst_satkerkd')
        ->leftJoin('vw_asset_barang_kdsatker as d', 'pengajuan_kebutuhan_bmn_satker_barang.pengajuan_kebutuhan_bmn_satker_id', '=', 'b.id')
        ->groupBY('pengajuan_kebutuhan_bmn_satker_barang.*')
        ->orderBy('prioritas','ASC')->get()->toArray();
        return response()->json([
            'data' => $data,
        ]);
    }

    public function gridDataSatkerAset(Request $request){
        $user = new Model();
        $pagingParams = $request->only(['start', 'length']);
        $id = $request->only('id');
        $satker = BmnSatker::where('id',$id)->first();
        $pengajuan = Model::where('id',$satker['pengajuan_kebutuhan_bmn_id'])->first();
        $kode_barang = BmnAsset::where(['pengajuan_kebutuhan_bmn_id' => $satker['pengajuan_kebutuhan_bmn_id']])->select('kode_barang')->get()->toArray();
        $kodeBarangArray = array_map(function ($item) {
            return $item['kode_barang'];
        }, $kode_barang);
        $jenisAset = Master::getBarangAset($kodeBarangArray);
        $data = $user->gridDataSatkerAset($pagingParams, $jenisAset, $id);
        return response()->json([
            'data' => $data['data'],
            'recordsTotal' => $data['total'],
            'recordsFiltered' => $data['total'],
        ]);
    }

    public function savePrioritas(Request $request){
        $id = $request->input('id');
        $prioritas = $request->input('prioritas');
        BmnSatkerBarang::where(['id' => $id])->update(['prioritas' => $prioritas]);
        return $this->resSuccess();
    }

    /**
     * Update the specified resource in storage.
     */
    public function update(Request $request, string $id)
    {
        //
    }

    /**
     * Remove the specified resource from storage.
     */
    public function destroy(string $id)
    {
    }

    public function cetak(string $id)
    {
        $model = new Model();
        $data = $model->getDataCetak($id);
        $pengajuan = Model::where('id',$id)->first();
        $currentOrder = [];

        $user = new Model();
        $pagingParams['start'] = 0;
        $pagingParams['length'] = -1;
        $searchParams['id'] = $id;
        $data = $user->getGridDataSatkerDetail($pagingParams,$searchParams);
        $pdf = LaravelMpdf::loadView('analisis_kebutuhan.bmn.cetak', [
            'data' => $data['data'],
            'model' => $pengajuan
        ], [], [
            'title' => 'Cetak',
            'orientation' => 'L',
        ]);
        return $pdf->stream('cetak.pdf');
    }

    public function cetakExcel(Request $request){
        $user = new Model();
        $pagingParams['start'] = 0;
        $pagingParams['length'] = -1;
        $searchParams = $request->only(['columns','id']);
        $data = $user->getGridDataSatkerDetail($pagingParams, $searchParams);
        $columns = ['id','tahun','nama','satker','nm_barang','kode_barang','jumlah_exist','jumlah_pengajuan','jml_setuju','jml_tolak','keterangan','alasan','prioritas'];
        return Excel::download(new ExportExcel($data['data']->toArray(),$columns,'Daftar Aset Alat Besar'), 'aset_alat_besar.xlsx', \Maatwebsite\Excel\Excel::XLSX);
    }
}
