<?php

namespace App\Http\Controllers\AnalisisKebutuhan\Bmn;
use Illuminate\Routing\Controller;

use App\Helpers\MyHelper;
use App\Models\AnalisisKebutuhan\Bmn as Model;
use App\Models\AnalisisKebutuhan\Bmn;
use App\Models\AnalisisKebutuhan\BmnSatker;
use App\Models\AnalisisKebutuhan\BmnSatkerAktivitas;
use App\Models\AnalisisKebutuhan\BmnSatkerBarang;
use App\Models\Sistem\Files;
use App\Models\Master\Master;
use App\Models\Master\MsSatker as MasterMsSatker;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\DB;
use Illuminate\Support\Facades\File as FileManager;

class DaftarKebutuhanBmnController extends Controller
{
    protected $breadcums = ['Analisis Kebutuhan', 'Kebutuhan BMN'];
    private $controller = '/analisis-kebutuhan/bmn/daftar-kebutuhan-bmn';

    public function __construct(Request $request)
    {
        $this->breadcums = array_merge($this->breadcums, [['link'=>$this->controller,'title'=>'Daftar Kebutuhan BMN']]);
    }

    public function index()
    {
        $data = [
            'tableId' => 'dt-pengajuan',
            'breadcums' => $this->breadcums,
            'controller' => $this->controller,
            'operasi' => $this->userOperation(),
        ];
        return view('analisis_kebutuhan.bmn.daftar_kebutuhan.gridV', $data);
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
        $searchParams = $request->only(['columns']);
        $data = $user->getGridDataSatker($pagingParams, $searchParams);
        return response()->json([
            'data' => $data['data'],
            'recordsTotal' => $data['total'],
            'recordsFiltered' => $data['total'],
        ]);
    }

    public function gridDataBarang($id)
    {
        $data = BmnSatkerBarang::where('pengajuan_kebutuhan_bmn_satker_id',$id)->get()->toArray();
        return response()->json([
            'data' => $data,
        ]);
    }

    /**
     * Show the form for creating a new resource.
     */
    public function create()
    {
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
        return view('analisis_kebutuhan.bmn.daftar_kebutuhan.gridSatkerV', $data);
    }

    /**
     * edit admin perlengkapan
     */
    public function edit(string $id)
    {
        $pengajuanSatker = BmnSatker::where('id',$id)->first();
        $pengajuan = Bmn::where('id',$pengajuanSatker['pengajuan_kebutuhan_bmn_id'])->first();
        $satker = MasterMsSatker::where('inst_satkerkd',$pengajuanSatker['ms_satker_id'])->first();

        //aktifitas
        $msAktivitasId = $pengajuanSatker->ms_aktivitas_id ?? 3000;
        $whereAct = ['ms_aktivitas_id' => $msAktivitasId,'group'=>'BMN'];
        $aktivitasHistories = BmnSatkerAktivitas::getDetail($pengajuanSatker['id']);
        $aktivitasOptions = [];
        $currentAktivitas = [];
        $id_jenis_asset = $pengajuan && $pengajuan['id_jenis_asset'] ? json_decode($pengajuan['id_jenis_asset']):null;
        $kodeBarang = Master::getBarangAset($id_jenis_asset);
        $data = [
            'model' => $pengajuan,
            'pengajuanSatker' => $pengajuanSatker,
            'controller' => $this->controller,
            'breadcums' => array_merge($this->breadcums, ['Approval']),
            'satker' => $satker['inst_nama'],
            'operasi' => $this->userOperation(),
            'aktivitasOptions' => $aktivitasOptions,
            'aktivitas' => $currentAktivitas,
            'aktivitasHistories' => $aktivitasHistories,
            'jenisAssetOptions' => MyHelper::generateSelectOptions([
                'data' => Master::getBarangAset(),
                'text' => 'nama_barang',
                'textkode' => 'kode_barang',
                'value' => 'kode_barang',
                'selected' => $id_jenis_asset
            ]),
            'kdBarangOptions' => $kodeBarang,
        ];
        return view('analisis_kebutuhan.bmn.daftar_kebutuhan.daftarKebutuhanBmnFormV', $data);
    }

    public function saveBarang(Request $request){
        $customMessages = [
            'nama.required' => 'Nama Barang harus diisi',
            'jumlah.required' => 'Jumlah Barang harus diisi',
        ];
        $validasi = [
            'nama' => 'required',
            'jumlah' => 'required',
        ];
        $request->validate($validasi, $customMessages);
        try {
            DB::beginTransaction();
            $id = $request->input('id');
            $data = [
                'nama'=>$request->input('nama'),
                'kode_barang'=>$request->input('kode'),
                'jumlah'=>$request->input('jumlah'),
                'alasan'=>$request->input('alasan'),
                'keterangan'=>$request->input('keterangan'),
                'jml_setuju'=>$request->input('jml_setuju'),
                'pengajuan_kebutuhan_bmn_satker_id'=>$request->input('pengajuan_kebutuhan_bmn_satker_id'),
            ];
            $idbarang = BmnSatkerBarang::updateOrCreate(['id' => $id], $data);
            if ($request->hasFile('file_pendukung')) {
                $filepath = 'uploads/kebutuhan_bmn';
                $file = $request->file('file_pendukung');
                $fileName = $idbarang['id'].'_file_pendukung.'.$file->getClientOriginalExtension();
                $filesave = $filepath.'/'.$fileName;
                $file->move($filepath, $fileName);
                BmnSatkerBarang::updateOrCreate(['id' => $idbarang['id']], ['file_pendukung'=>$filesave]);
                $newFile = [
                    'filename' => $file->getClientOriginalName(),
                    'path' => $filesave,
                    'pkey' => $idbarang['id'],
                    'kategori' => 'File Pendukung Kebutuhan Bmn',
                    'kategori_slug' => MyHelper::generateSlug('File Pendukung Kebutuhan Bmn'),
                    'filetype' => $file->getClientOriginalExtension(),
                    'created_by' => session('userData.username'),
                    'created_at' => date('Y-m-d H:i:s'),
                ];
                $insertedFiles[] = $newFile;
                Files::insert($insertedFiles);
            }
            DB::commit();
            return $this->resSuccess(
                'Berhasil Disimpan!'
            );
        } catch (\Throwable $th) {
            dd($th->getMessage());
            return $this->resError('Gagal menyimpan data');
        }
    }

    public function deleteBarang(string $id)
    {
        try {
            DB::beginTransaction();
            $barang = BmnSatkerBarang::find($id);
            if($barang->file_pendukung){
                $filePath = public_path($barang->file_pendukung);
                if (FileManager::exists($filePath)) {
                    FileManager::delete($filePath);
                }
            }
            $barang->delete();
            DB::commit();
            return $this->resSuccess('Berhasil Dihapus!');
        } catch (\Throwable $th) {
            DB::rollBack();
            return $this->resError('Gagal Menghapus data');
        }
    }

    public function savePengajuan(Request $request)
    {
        $validasi = [
            'ms_aktivitas_id' => 'required',
        ];
        $request->validate($validasi);

        if ($request->has('id_pengajuan')) {
            $isNew = false;
            $id = $request->input('id_pengajuan');
        } else {
            $isNew = true;
            $id = null;
        }
        $pengajuan_kebutuhan_bmn_satker_id = $request->input('pengajuan_kebutuhan_bmn_satker_id');
        $ms_aktivitas_id = $request->input('ms_aktifitas_id');
        $type = $request->input('type')??'';
        //cek barang harus ada
        $barang = BmnSatkerBarang::where(['pengajuan_kebutuhan_bmn_satker_id' => $pengajuan_kebutuhan_bmn_satker_id])->get();
        if ($barang->isEmpty()) return $this->resError('Barang harus diisi');
        if($ms_aktivitas_id == 3006 || $type=='selesai'){
            foreach ($barang as $data) {
                if($data->jml_setuju == '' || is_null($data->jml_setuju)){
                    return $this->resError('Jumlah disetujui masing-masing barang harus diisi');
                }
            }
        }

        try {
            DB::beginTransaction();
            BmnSatker::where(['id' => $pengajuan_kebutuhan_bmn_satker_id])->update(['ms_aktifitas_id' => $ms_aktivitas_id]);

            $dataAktivitas = [
                'pengajuan_id' => $pengajuan_kebutuhan_bmn_satker_id,
                'ms_aktifitas_id' => $ms_aktivitas_id,
                'komentar' => $request->input('komentar'),
                'to_satker_induk' => $ms_aktivitas_id == 3000 ? false : true,
                'group'=>'BMN'
            ];
            $acts = ['act' => [], 'nextAct' => null];
            // BmnSatkerAktivitas::insert($acts['act']);

            if (!empty($acts['nextAct'])) {
                $newAct = $acts['nextAct'];
                if (in_array($newAct, [3003, 3005, 3007, 3014])) { //revisi
                    $newAct = 3000;
                }
                BmnSatker::where(['id' => $id])->update(['ms_aktifitas_id' => $newAct]);
            }

            DB::commit();
            return $this->resSuccess(
                'Berhasil Disimpan!',
                [
                    'type' => 'redirect',
                    'url' => \URL::to('/analisis-kebutuhan/bmn/daftar-kebutuhan-bmn')
                ]
            );
        } catch (\Throwable $th) {
            DB::rollBack();
            dd($th->getMessage());
            return $this->resError('Gagal Menyimpan data');
        }
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
        try {
            DB::beginTransaction();
            Model::destroy($id);
            DB::commit();
            return $this->resSuccess('Berhasil Dihapus!');
        } catch (\Throwable $th) {
            DB::rollBack();
            return $this->resError('Gagal Menghapus data');
        }
    }
}
