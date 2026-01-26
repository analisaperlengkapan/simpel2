<?php

namespace App\Http\Controllers\AnalisisKebutuhan\Bmn;
use Illuminate\Routing\Controller;

use App\Helpers\MyHelper;
use App\Models\AnalisisKebutuhan\Bmn as Model;
use App\Models\AnalisisKebutuhan\Bmn;
use App\Models\AnalisisKebutuhan\BmnAsset;
use App\Models\AnalisisKebutuhan\BmnSatker;
use App\Models\AnalisisKebutuhan\BmnSatkerAktivitas;
use App\Models\AnalisisKebutuhan\BmnSatkerBarang;
use App\Models\Master\MsSatker as MasterMsSatker;
use App\Models\Master\Master;
use App\Models\Master\MsJenisAset;
use Illuminate\Http\Request;
use Illuminate\Support\Arr;
use Illuminate\Support\Facades\DB;
use Symfony\Component\HttpKernel\Exception\NotFoundHttpException;

class AnalisisKelayakanController extends Controller
{
    protected $breadcums = ['Analisis Kebutuhan', 'Kebutuhan BMN'];
    private $controller = '/analisis-kebutuhan/bmn/analisis-kelayakan';

    public function __construct(Request $request)
    {
        $this->breadcums = array_merge($this->breadcums, [['link'=>$this->controller,'title'=>'Analisis Kelayakan dan Prediktif']]);
    }

    public function index()
    {
        $data = [
            'tableId' => 'dt-pengajuan',
            'breadcums' => $this->breadcums,
            'controller' => $this->controller,
            'operasi' => $this->userOperation(),
        ];
        return view('analisis_kebutuhan.bmn.analisis.gridV', $data);
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
        $data = $user->getGridDataSatker($pagingParams, $searchParams);
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
        return view('analisis_kebutuhan.bmn.analisis.gridSatkerV', $data);
    }

    public function edit(string $id,Request $request)
    {
        $pengajuanSatker = BmnSatker::where('id',$id)->first();
        $pengajuan = Model::where('id',$pengajuanSatker['pengajuan_kebutuhan_bmn_id'])->first();
        $satker = MasterMsSatker::where('inst_satkerkd',$pengajuanSatker['ms_satker_id'])->first();
        $asset = BmnAsset::where(['pengajuan_kebutuhan_bmn_id' => $pengajuanSatker['pengajuan_kebutuhan_bmn_id']])->get()->toArray();
        //aktifitas
        $msAktivitasId = $pengajuanSatker->ms_aktifitas_id ?? 3000;
        $whereAct = ['ms_aktifitas_id' => $msAktivitasId,'group'=>'BMN'];
        $aktifitasHistories = BmnSatkerAktivitas::getDetail($pengajuanSatker['id']);
        $aktifitasOptions = [];
        $currentAktivitas = [];

        if ($request->wantsJson()) {
            $daftarBarang = BmnSatkerBarang::where('pengajuan_kebutuhan_bmn_satker_barang.pengajuan_kebutuhan_bmn_satker_id',$id)
                    ->select('pengajuan_kebutuhan_bmn_satker_barang.id','pengajuan_kebutuhan_bmn_satker_barang.nama',
                    'pengajuan_kebutuhan_bmn_satker_barang.jumlah','pengajuan_kebutuhan_bmn_satker_barang.alasan',
                    'pengajuan_kebutuhan_bmn_satker_barang.file_pendukung','pengajuan_kebutuhan_bmn_satker_barang.pengajuan_kebutuhan_bmn_satker_id',
                    'pengajuan_kebutuhan_bmn_satker_barang.prioritas','pengajuan_kebutuhan_bmn_satker_barang.keterangan','pengajuan_kebutuhan_bmn_satker_barang.jml_setuju','pengajuan_kebutuhan_bmn_satker_barang.kode_barang',DB::raw('COUNT(d.kode_barang) as jumlah_exist'))
                    ->leftJoin('pengajuan_kebutuhan_bmn_satker as b', 'pengajuan_kebutuhan_bmn_satker_barang.pengajuan_kebutuhan_bmn_satker_id', '=', 'b.id')
                    ->leftJoin('ms_satker as c', 'b.ms_satker_id', '=', 'c.inst_satkerkd')
                    ->leftJoin('vw_asset_barang_kdsatker as d',function($join){
                        $join->on('d.kdsatker_keu', '=', 'c.kdsatker_keu')
                        ->on('d.kode_barang', '=', 'pengajuan_kebutuhan_bmn_satker_barang.kode_barang');
                    })
                    ->groupBY('pengajuan_kebutuhan_bmn_satker_barang.id','pengajuan_kebutuhan_bmn_satker_barang.nama',
                    'pengajuan_kebutuhan_bmn_satker_barang.jumlah','pengajuan_kebutuhan_bmn_satker_barang.alasan',
                    'pengajuan_kebutuhan_bmn_satker_barang.file_pendukung','pengajuan_kebutuhan_bmn_satker_barang.pengajuan_kebutuhan_bmn_satker_id',
                    'pengajuan_kebutuhan_bmn_satker_barang.prioritas','pengajuan_kebutuhan_bmn_satker_barang.keterangan','pengajuan_kebutuhan_bmn_satker_barang.jml_setuju','pengajuan_kebutuhan_bmn_satker_barang.kode_barang')
                    ->orderBy('prioritas','ASC')->get()->toArray();
            $kode_barang = BmnAsset::where(['pengajuan_kebutuhan_bmn_id' => $pengajuanSatker['pengajuan_kebutuhan_bmn_id']])->select('kode_barang')->get()->toArray();
            $kodeBarangArray = array_map(function ($item) {
                return $item['kode_barang'];
            }, $kode_barang);
            $jenisAset = Master::getBarangAset($kodeBarangArray);
            $bmn = new Bmn();
            $daftarAsetSatker = $bmn->gridDataSatkerAset(['length'=>-1,'start'=>0],$jenisAset,$id);
            $data = [
                'model' => $pengajuan,
                'pengajuanAset' => $asset,
                'daftarBarang' => $daftarBarang,
                'daftarAsetSatker' => $daftarAsetSatker['data'],
                'aktivitasHistories' => $aktifitasHistories,
            ];
            return response()->json($data);
        } else {
            $data = [
                'model' => $pengajuan,
                'pengajuanSatker' => $pengajuanSatker,
                'controller' => $this->controller,
                'breadcums' => array_merge($this->breadcums, [['link'=>'analisis-kebutuhan/bmn/analisis-kelayakan/'.$pengajuanSatker['pengajuan_kebutuhan_bmn_id'],'title'=>'Daftar Kebutuhan BMN Satker'],'Detail']),
                'satker' => $satker['inst_nama'],
                'operasi' => $this->userOperation(),
                'aktivitasOptions' => $aktifitasOptions,
                'aktivitas' => $currentAktivitas,
                'aktivitasHistories' => $aktifitasHistories,
                'asset' => $asset,
                'listBarang' => Master::getBarangAset(),
            ];
            return view('analisis_kebutuhan.bmn.analisis.formV', $data);
        }
    }

    public function gridDataBarang($id)
    {
        $data = BmnSatkerBarang::where('pengajuan_kebutuhan_bmn_satker_id',$id)->orderBy('prioritas','ASC')->get()->toArray();
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
        $jenisAset = MsJenisAset::where('id',$pengajuan['id_jenis_asset'])->first();
        $data = $user->gridDataSatkerAset($pagingParams, $jenisAset['nm_table'], $id);
        return response()->json([
            'data' => $data['data'],
            'recordsTotal' => $data['total'],
            'recordsFiltered' => $data['total'],
        ]);
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
}
