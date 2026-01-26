<?php

namespace App\Http\Controllers\Bmn\Penghapusan;
use Illuminate\Routing\Controller;

use App\Helpers\MyHelper;
use App\Models\Bmn\PermohonanPenghapusan\PermohonanPenghapusanBmn;
use App\Models\Sistem\Files;
use App\Models\Master\MsSatker;
use App\Models\Komunikasi\Notifikasi;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\DB;
use Symfony\Component\HttpKernel\Exception\NotFoundHttpException;
use App\Models\Bmn\PermohonanMonitor\PermohonanPenghapusanBmnMonitor as Model;
use App\Models\Bmn\PermohonanMonitor\PermohonanPenghapusanBmnMonitorFile;
use App\Models\Bmn\PermohonanMonitor\PersetujuanPenghapusanBmnMonitorFile;
use App\Models\Bmn\PermohonanPenghapusan\PermohonanPenghapusanBmnAktivitas as Aktivitas;
use App\Models\Bmn\PermohonanPenghapusan\PermohonanPenghapusanBmnFile;
use App\Models\Bmn\PermohonanPenghapusan\PermohonanPenghapusanBmnFoto;
use App\Models\Bmn\PermohonanPenghapusan\PermohonanPenghapusanBmnFotocopy;
use App\Models\Bmn\PermohonanSk\PermohonanPenghapusanBmnSk;
use App\Models\Master\Master;
use Illuminate\Support\Facades\File as FileManager;
use Illuminate\Support\Str;

class PersetujuanBmnMonitorController extends Controller
{
    protected $kategoriJudul = 'Monitoring Persetujuan Penghapusan BMN';
    protected $kategori = [];
    protected $breadcums = ['Pengelolaan BMN','Persetujuan Penghapusan BMN'];
    private $controller = '/bmn/penghapusan/persetujuanmonitor';
    private $whereKategori = [];

    public function __construct(Request $request)
    {
        $segment = $request->segment(3);
        $this->whereKategori = [];
        $this->breadcums = array_merge($this->breadcums, [['link'=>$this->controller,'title'=>$this->kategoriJudul]]);
    }


    public function index()
    {
        return view('bmn.persetujuan-monitor.pengajuanV', [
            'tableId' => 'dt-pengajuan',
            'breadcums' => $this->breadcums,
            'controller' => $this->controller,
            'kategori' => $this->kategori,
            'kategoriJudul' => $this->kategoriJudul,
            'canCreate' => $this->canCreatePermintaan()
        ]);
    }

    protected function canCreatePermintaan()
    {
        // return true;
        return in_array(session('userData.current_role.ms_role_id'), config('constants.pelaksana_role'));
    }

    protected function isValidator()
    {
        if (session('userData.current_role.ms_role_id') == config('constants.superadmin_role_id')) {
            return true;
        }

        if (session('userData.current_role.ms_role_id') == config('constants.validator_pusat_role_id')) {
            return true;
        }

        if (session('userData.current_role.ms_role_id') == config('constants.validator_wilayah_role_id')) {
            return true;
        }

        if (session('userData.current_role.ms_role_id') == config('constants.pelaksana_satker_role_id')) {
            return false;
        }
    }

    public function gridData(Request $request)
    {
        $user = new PermohonanPenghapusanBmn();
        $pagingParams = $request->only(['start', 'length']);
        $searchParams = $request->only(['columns']);
        $data = $user->getDataGrid($pagingParams, $searchParams);
        return response()->json([
            'data' => $data['data'],
            'recordsTotal' => $data['total'],
            'recordsFiltered' => $data['total'],
        ]);
    }

    public function gridDataFotoCopy($pengajuan_id)
    {
        $model = new PermohonanPenghapusanBmnFotocopy();
        $data = $model->getDetail($pengajuan_id);
        return response()->json([
            'data' => $data,
        ]);
    }

    public function gridDataFilelain($pengajuan_id)
    {
        $model = new PersetujuanPenghapusanBmnMonitorFile();
        $data = $model->getDetail($pengajuan_id);
        return response()->json([
            'data' => $data,
        ]);
    }

    public function gridDataFoto($pengajuan_id)
    {
        $model = new PermohonanPenghapusanBmnFoto();
        $data = $model->getDetail($pengajuan_id);
        return response()->json([
            'data' => $data,
        ]);
    }

    public function gridDataPermohonan()
    {
        $model = new Model();
        $data = $model->getDataPermohonan();
        return response()->json([
            'data' => $data,
        ]);
    }

    function getData($id = null)
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

        $data = [
            'model' => $model,
            'isNew' => $isNew,
            'controller' => $this->controller,
            'breadcums' => array_merge($this->breadcums, [$breadcum]),
        ];
        return $data;
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
        $customMessages = [
            'pengajuan_id.required' => 'Permohonan Peersetujuan Penghapusan BMN harus dipilih',
        ];
        $validasi = [
            'pengajuan_id' => 'required',
        ];

        $request->validate($validasi, $customMessages);
        try {
            DB::beginTransaction();
            $id = $request->input('id');
            $inputan['pengajuan_id'] = $request->input('pengajuan_id');
            if ($id == '' || $id == null) {
                $inputan['inst_satkerkd'] = session('userData.current_role.ms_satker_id');
                $id = MyHelper::getPk(date('Ymd'), 'permohonan_penghapusan_bmn_sk_seq');
                $inputan['ms_aktifitas_id'] = 3000;
            }
            Model::updateOrCreate(['id' => $id], $inputan);
            if ($request->hasFile('file_surat_permohonan')) {
                $filepath = 'uploads/bmn/permohonan_sk_penghapusan/'.$id;
                $file = $request->file('file_surat_permohonan');
                $fileName = $id.'_file_surat_permohonan.'.$file->getClientOriginalExtension();
                $filesave = $filepath.'/'.$fileName;
                $file->move($filepath, $fileName);
                $newFile = [
                    'filename' => $file->getClientOriginalName(),
                    'path' => $filesave,
                    'pkey' => $id,
                    'kategori' => 'file_surat_keputusan_permohonan',
                    'kategori_slug' => MyHelper::generateSlug('file_surat_keputusan_permohonan'),
                    'filetype' => $file->getClientOriginalExtension(),
                    'created_by' => session('userData.username'),
                    'created_at' => date('Y-m-d H:i:s'),
                ];
                $insertedFiles[] = $newFile;
                Files::insert($insertedFiles);
                Model::updateOrCreate(['id' => $id], ['file_surat_permohonan'=>$filesave]);

            }
            DB::commit();
            return $this->resSuccess(
                'Berhasil Disimpan!',
                [
                    'type' => 'redirect',
                    'url' => \URL::to($this->controller)
                ]
            );
        } catch (\Throwable $th) {
            dd($th->getMessage());
            return $this->resError('Gagal menyimpan data');
        }
    }

    public function saveLain(Request $request){
        $customMessages = [
            'ket.required' => 'Keterangan harus diisi',
        ];
        $validasi = [
            'ket' => 'required',
        ];

        $request->validate($validasi, $customMessages);
        try {
            DB::beginTransaction();
            $id = $request->input('id');
            $nomor = $request->input('nomor');
            $tanggal = $request->input('tanggal');
            $pengajuan_id = $request->input('pengajuan_id');
            $ket = $request->input('ket');

            $inputan = [
                'nomor'=>$nomor,
                'tanggal'=>$tanggal,
                'pengajuan_id'=>$pengajuan_id,
                'ket'=>$ket,
            ];
            $model = PersetujuanPenghapusanBmnMonitorFile::updateOrCreate(['id' => $id], $inputan);
            $insertedId = $model->id;
            if ($request->hasFile('file')) {
                $filepath = 'uploads/bmn/permohonan_penghapusan/'.$pengajuan_id;
                $file = $request->file('file');
                $namaFile = Str::slug($ket);
                $fileName = $insertedId.'_filelain_'.$namaFile.'.'.$file->getClientOriginalExtension();
                $filesave = $filepath.'/'.$fileName;
                $file->move($filepath, $fileName);
                $newFile = [
                    'filename' => $file->getClientOriginalName(),
                    'path' => $filesave,
                    'pkey' => $insertedId,
                    'kategori' => 'file_lain',
                    'kategori_slug' => MyHelper::generateSlug('file_lain'),
                    'filetype' => $file->getClientOriginalExtension(),
                    'created_by' => session('userData.username'),
                    'created_at' => date('Y-m-d H:i:s'),
                ];
                $insertedFiles[] = $newFile;
                Files::insert($insertedFiles);
                PersetujuanPenghapusanBmnMonitorFile::updateOrCreate(['id' => $insertedId], ['file'=>$filesave]);
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

    /**
     * edit admin perlengkapan
     */
    public function show(Request $request,string $id)
    {
        $pengajuan = PermohonanPenghapusanBmn::where('id', $id)->first();
        if (!$pengajuan) {
            throw new NotFoundHttpException('Data Tidak Ditemukan');
        }
        $satker = MsSatker::where('inst_satkerkd', $pengajuan->inst_satkerkd)->first();
        $pengajuan->inst_nama = $satker->inst_nama;
        //$_GET['satker'] cuma ada klo diliat validator pusat /kejati
        $currentRole = session('userData.current_role');
        $whereData = ['ms_satker_id' => $_GET['satker'] ?? $currentRole['ms_satker_id'], 'pengajuan_id' => $pengajuan->id];
        $whereSatker = ['inst_satkerkd' => $_GET['satker'] ?? $currentRole['ms_satker_id']];

        if ($currentRole['ms_satker_id'] == '00' && !isset($_GET['satker'])) {
            $whereData['ms_satker_pusat_id'] = $currentRole['ms_satker_pusat_id'];
            $whereSatker['unitkerja_idk'] = $currentRole['ms_satker_pusat_id'];
        }

        //model utama
        $model = $pengajuan ?? [];
        $isNew = empty($model) ? true : false;
        $permohonan = $pengajuan;
        //ms file
        $ms_file = DB::table('ms_penghapusan_file as a')
                ->select('a.id as ms_penghapusan_file_id','a.nm_file','a.is_nomor','b.id','b.nomor','b.tanggal','b.file','a.jenis_file','a.is_validator','a.kategori','a.sub_kategori')
                ->leftJoin('permohonan_penghapusan_bmn_file as b','a.id','b.ms_penghapusan_file_id')
                ->where('sub_kategori',$permohonan->kategori)
                ->orderBy('a.id')->get()->toArray();

        //aktifitas
        $aktifitasHistories = Aktivitas::getDetail($model->id);
        $currentAktivitas = [];
        $aktifitasOptions = DB::table('ms_aktifitas_user')->where(['group' => 'BMN'])->whereIn('id',[3000,3002,3004,3006,3007])->get()->toArray();

        $data = [
            'model' => $model,
            'controller' => $this->controller,
            'aktivitasOptions' => $aktifitasOptions,
            'aktivitas' => $currentAktivitas,
            'aktivitasHistories' => $aktifitasHistories,
            'breadcums' => array_merge($this->breadcums, ['Pengajuan']),
            'isNew' => $isNew,
            'kategoriJudul' => $this->kategoriJudul,
            'canCreate' => $this->canCreatePermintaan(),
            'isValidator' => $this->isValidator(),
            'listBarang' => Master::getBarangAset(),
            'msFile' => $ms_file,
            'permohonan' => $permohonan,
            'permohonansk' => $pengajuan,
        ];
        if ($request->wantsJson()) {
            return response()->json($data);
        } else {
            return view('bmn.persetujuan-monitor.pengajuanFormV', $data);
        }
    }

    /**
     * pelaksana ngisi /  validator ngeliat
     */
    public function edit(string $id)
    {

    }

    public function savePengajuan(Request $request)
    {
        if ($request->has('id')) {
            $isNew = false;
            $id = $request->input('id');
        } else {
            $isNew = true;
            $id = null;
        }

        try {
            DB::beginTransaction();
            $currentRole = session('userData.current_role');
            //$model =  Model::where(['id' => $id])->first();
            $ms_aktifitas_id = $request->input('ms_aktifitas_id');
            if($ms_aktifitas_id){
                PermohonanPenghapusanBmn::where(['id' => $id])->update(['ms_aktifitas_id' => $ms_aktifitas_id]);
                $dataAktivitas = [
                    'pengajuan_id' => $id,
                    'ms_aktifitas_id' => $ms_aktifitas_id,
                    'komentar' => $request->input('komentar'),
                    'to_satker_induk' => $ms_aktifitas_id == 3000 ? false : true
                ];
                $acts = ["act" => [], "nextAct" => null];
                Aktivitas::insert($acts['act']);
            }

            $permohonansk = PermohonanPenghapusanBmn::where('id',$id)->first();
            if(1 == 1){
                foreach($request->input('ms_penghapusan_file_id') as $key => $ms_penghapusan_file_id){
                    $data = [
                        'pengajuan_id'=>$permohonansk->pengajuan_id,
                        'ms_penghapusan_file_id'=>$ms_penghapusan_file_id,
                        'nomor'=>$request->input('file_nomor')[$key],
                        'tanggal'=>$request->input('file_tanggal')[$key],
                    ];
                    $modelFile = PermohonanPenghapusanBmnFile::updateOrCreate(['id' => $request->input('file_id')[$key]], $data);
                    $insertedId = $modelFile->id;
                    if ($request->hasFile('file_file.'.$key)) {
                        $filepath = 'uploads/bmn/permohonan_penghapusan/'.$permohonansk->pengajuan_id;
                        $file = $request->file('file_file')[$key];
                        $fileName = $insertedId.'_'.$request->input('file_jenis_file')[$key].'.'.$file->getClientOriginalExtension();
                        $filesave = $filepath.'/'.$fileName;
                        $file->move($filepath, $fileName);
                        $newFile = [
                            'filename' => $file->getClientOriginalName(),
                            'path' => $filesave,
                            'pkey' => $insertedId,
                            'kategori' => $request->input('file_jenis_file')[$key],
                            'kategori_slug' => MyHelper::generateSlug($request->input('file_jenis_file')[$key]),
                            'filetype' => $file->getClientOriginalExtension(),
                            'created_by' => session('userData.username'),
                            'created_at' => date('Y-m-d H:i:s'),
                        ];
                        $insertedFiles[] = $newFile;
                        Files::insert($insertedFiles);
                        PermohonanPenghapusanBmnFile::updateOrCreate(['id' => $insertedId], ['file'=>$filesave]);
                    }
                }
            }

            DB::commit();
            return $this->resSuccess(
                'Berhasil Disimpan!',
                [
                    'type' => 'redirect',
                    'url' => \URL::to($this->controller)
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
            $model = Model::where('id',$id)->first();
            $model->delete();
            DB::commit();
            return $this->resSuccess('Berhasil Dihapus!');
        } catch (\Throwable $th) {
            dd($th->getMessage());
            DB::rollBack();
            return $this->resError('Gagal Menghapus data');
        }
    }

    public function deleteFilelain(string $id)
    {
        try {
            DB::beginTransaction();
            $fc = PersetujuanPenghapusanBmnMonitorFile::where('id', $id)->first();
            if(!empty($fc)){
                if($fc->file){
                    $filePath = public_path($fc->file);
                    // Delete the physical file from public directory
                    if (FileManager::exists($filePath)) {
                        FileManager::delete($filePath);
                    }
                }
                $fc->delete();
            }
            DB::commit();
            return $this->resSuccess('Berhasil Dihapus!');
        } catch (\Throwable $th) {
            DB::rollBack();
            return $this->resError('Gagal Menghapus data');
        }
    }
}
