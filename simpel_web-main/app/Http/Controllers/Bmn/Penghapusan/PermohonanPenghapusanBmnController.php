<?php

namespace App\Http\Controllers\Bmn\Penghapusan;

use App\Helpers\MyHelper;
use App\Http\Controllers\Controller;
use App\Models\ApprovalUserSpseSirup as Approval;
use App\Models\Bmn\PermohonanMonitor\PermohonanPenghapusanBmnMonitorFile;
use App\Models\Files;
use App\Models\Master\MsSatker;
use App\Models\Notifikasi;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\DB;
use Symfony\Component\HttpKernel\Exception\NotFoundHttpException;
use App\Models\Bmn\PermohonanPenghapusan\PermohonanPenghapusanBmn as Model;
use App\Models\Bmn\PermohonanPenghapusan\PermohonanPenghapusanBmn;
use App\Models\Bmn\PermohonanPenghapusan\PermohonanPenghapusanBmnAktifitas as Aktifitas;
use App\Models\Bmn\PermohonanPenghapusan\PermohonanPenghapusanBmnFile;
use App\Models\Bmn\PermohonanPenghapusan\PermohonanPenghapusanBmnFoto;
use App\Models\Bmn\PermohonanPenghapusan\PermohonanPenghapusanBmnFotocopy;
use App\Models\Master;
use Illuminate\Support\Facades\File as FileManager;
use ZipArchive;

class PermohonanPenghapusanBmnController extends Controller
{
    protected $kategoriJudul = 'Permohonan Penerbitan Persetujuan Penghapusan BMN';
    protected $kategori = 'penghapusan_bmn';
    protected $breadcums = ['Pengelolaan BMN','Penghapusan BMN','Permohonan Penerbitan Persetujuan Penghapusan BMN'];
    private $controller = '/bmn/penghapusan/penghapusansk';
    private $url = 'bmn/penghapusan/penghapusansk';
    private $whereKategori = [];

    public function __construct(Request $request)
    {
        $segment = $request->segment(3);
        if($segment == 'pemindahtanganan'){
            $this->kategoriJudul = 'Pemindahtanganan';
            $this->kategori = ['penjualan'=>'Penjualan/Lelang','tukar'=>'Tukar Menukar','hibah'=>'Hibah'];
            $this->controller = '/bmn/penghapusan/pemindahtanganan';
            $this->url = 'bmn/penghapusan/pemindahtanganan';
            $this->whereKategori = ['penjualan','tukar','hibah'];
        }
        if($segment == 'pemusnahan'){
            $this->kategoriJudul = 'Pemusnahan';
            $this->kategori = ['pemusnahan'=>'pemusnahan'];
            $this->controller = '/bmn/penghapusan/pemusnahan';
            $this->url = 'bmn/penghapusan/pemusnahan';
            $this->whereKategori = ['pemusnahan'];
        }
        if($segment == 'sebablain'){
            $this->kategoriJudul = 'Sebab Lain';
            $this->kategori = ['sebablain'=>'Sebab Lain'];
            $this->controller = '/bmn/penghapusan/sebablain';
            $this->url = 'bmn/penghapusan/sebablain';
            $this->whereKategori = ['sebablain'];
        }
        $this->breadcums = array_merge($this->breadcums, [['link'=>$this->controller,'title'=>$this->kategoriJudul]]);
    }


    public function index()
    {
        //dd(session('userData.current_role.ms_role_id'));
        return view('bmn.permohonan-penghapusan.pengajuanV', [
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
        $user = new Model();
        $pagingParams = $request->only(['start', 'length']);
        $searchParams = $request->only(['columns']);
        $data = $user->getDataGrid($pagingParams, $searchParams,  $this->whereKategori);
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

    public function gridDataFoto($pengajuan_id)
    {
        $model = new PermohonanPenghapusanBmnFoto();
        $data = $model->getDetail($pengajuan_id);
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
            'kategori.required' => 'Kategori harus dipilih',
            'no_surat_permohonan.required' => 'Nomor Surat Permohonan harus diisi',
            'tgl_surat_permohonan.required' => 'Tanggal Surat Permohonan harus diisi',
            'file_surat_permohonan.required' => 'File Surat Permohonan harus diupload',
        ];
        $validasi = [
            'kategori' => 'required',
            'no_surat_permohonan' => 'required',
            'tgl_surat_permohonan' => 'required',
            'file_surat_permohonan' => 'required'
        ];

        $request->validate($validasi, $customMessages);
        try {
            DB::beginTransaction();
            $id = $request->input('id');
            $inputan['kategori'] = $request->input('kategori');
            $inputan['jenis_barang'] = $request->input('jenis_barang');
            $inputan['no_surat_permohonan'] = $request->input('no_surat_permohonan');
            $inputan['tgl_surat_permohonan'] = $request->input('tgl_surat_permohonan');
            if ($id == '' || $id == null) {
                $inputan['inst_satkerkd'] = session('userData.current_role.ms_satker_id');
                $id = MyHelper::getPk(date('Ymd'), 'permohonan_penghapusan_bmn_seq');
                $inputan['ms_aktifitas_id'] = 3000;
                $inputan['created_by'] = session('userData.name');
            }
            $inputan['updated_by'] = session('userData.name');
            Model::updateOrCreate(['id' => $id], $inputan);
            if ($request->hasFile('file_surat_permohonan')) {
                $filepath = 'uploads/bmn/permohonan_penghapusan/'.$id;
                $file = $request->file('file_surat_permohonan');
                $fileName = $id.'_file_surat_permohonan.'.$file->getClientOriginalExtension();
                $filesave = $filepath.'/'.$fileName;
                $file->move($filepath, $fileName);
                $newFile = [
                    'filename' => $file->getClientOriginalName(),
                    'path' => $filesave,
                    'pkey' => $id,
                    'kategori' => 'file_surat_permohonan',
                    'kategori_slug' => MyHelper::generateSlug('file_surat_permohonan'),
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

    public function saveFotocopy(Request $request){
        $customMessages = [
            'nomor.required' => 'Nomor harus diisi',
            'tanggal.required' => 'Tanggal harus diisi',
            'file.required' => 'Tanggal harus diisi',
        ];
        $validasi = [
            'nomor' => 'required',
            'tanggal' => 'required',
            'file' => 'required',
        ];

        $request->validate($validasi, $customMessages);
        try {
            DB::beginTransaction();
            $id = $request->input('id');
            $pengajuan_id = $request->input('pengajuan_id');
            $nomor = $request->input('nomor');
            $tanggal = $request->input('tanggal');
            $inputan = [
                'pengajuan_id'=>$pengajuan_id,
                'nomor'=>$nomor,
                'tanggal'=>$tanggal,
            ];
            $model = PermohonanPenghapusanBmnFotocopy::updateOrCreate(['id' => $id], $inputan);
            $insertedId = $model->id;
            if ($request->hasFile('file')) {
                $filepath = 'uploads/bmn/permohonan_penghapusan/'.$pengajuan_id;
                $file = $request->file('file');
                $fileName = $insertedId.'_file_fotocopy.'.$file->getClientOriginalExtension();
                $filesave = $filepath.'/'.$fileName;
                $file->move($filepath, $fileName);
                $newFile = [
                    'filename' => $file->getClientOriginalName(),
                    'path' => $filesave,
                    'pkey' => $insertedId,
                    'kategori' => 'file_fotocopy',
                    'kategori_slug' => MyHelper::generateSlug('file_fotocopy'),
                    'filetype' => $file->getClientOriginalExtension(),
                    'created_by' => session('userData.username'),
                    'created_at' => date('Y-m-d H:i:s'),
                ];
                $insertedFiles[] = $newFile;
                Files::insert($insertedFiles);
                PermohonanPenghapusanBmnFotocopy::updateOrCreate(['id' => $insertedId], ['file'=>$filesave]);
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

    public function saveFoto(Request $request){
        $customMessages = [
            'ket.required' => 'Keterangan harus diisi',
            'file.required' => 'Tanggal harus diisi',
        ];
        $validasi = [
            'ket' => 'required',
            'file' => 'required',
        ];

        $request->validate($validasi, $customMessages);
        try {
            DB::beginTransaction();
            $id = $request->input('id');
            $pengajuan_id = $request->input('pengajuan_id');
            $ket = $request->input('ket');
            $inputan = [
                'pengajuan_id'=>$pengajuan_id,
                'ket'=>$ket,
            ];
            $model = PermohonanPenghapusanBmnFoto::updateOrCreate(['id' => $id], $inputan);
            $insertedId = $model->id;
            if ($request->hasFile('file')) {
                $filepath = 'uploads/bmn/permohonan_penghapusan/'.$pengajuan_id;
                $file = $request->file('file');
                $fileName = $insertedId.'_file_foto.'.$file->getClientOriginalExtension();
                $filesave = $filepath.'/'.$fileName;
                $file->move($filepath, $fileName);
                $newFile = [
                    'filename' => $file->getClientOriginalName(),
                    'path' => $filesave,
                    'pkey' => $insertedId,
                    'kategori' => 'file_foto',
                    'kategori_slug' => MyHelper::generateSlug('file_foto'),
                    'filetype' => $file->getClientOriginalExtension(),
                    'created_by' => session('userData.username'),
                    'created_at' => date('Y-m-d H:i:s'),
                ];
                $insertedFiles[] = $newFile;
                Files::insert($insertedFiles);
                PermohonanPenghapusanBmnFoto::updateOrCreate(['id' => $insertedId], ['file'=>$filesave]);
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
        $data = $this->getData($id);
        $pengajuan = Model::where('id', $id)->first();
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

        //ms file
        $ms_file = DB::table('ms_penghapusan_file as a')
                ->select('a.id as ms_penghapusan_file_id','a.nm_file','a.is_nomor','b.id','b.nomor','b.tanggal','b.file','a.jenis_file','a.is_validator','a.ms_role_id')
                ->leftJoin('permohonan_penghapusan_bmn_file as b', function($join) use ($id)  {
                    $join->on('a.id', '=', 'b.ms_penghapusan_file_id')
                         ->where('b.pengajuan_id', '=', $id);
                })
                ->where('kategori', $request->segment(3) )
                ->where('sub_kategori',$pengajuan->kategori)->orderBy('a.id')->get()->toArray();

        //aktifitas
        $msAktifitasId = $model->ms_aktifitas_id;
        $whereAct = ['ms_aktifitas_id' => $msAktifitasId,'group'=>'BMN'];
        $aktifitasHistories = Aktifitas::getDetail($model->id);
        $aktifitasOptions = Approval::getAktifitas($whereAct);
        $currentAktifitas = Approval::getCurrentAktifitas($model->ms_aktifitas_id);
        if($msAktifitasId==3000){
            $aktifitasOptions = DB::table('ms_aktifitas_user')->where(['group' => 'BMN'])->whereIn('id',[3000,3002])->get()->toArray();
        }
        if (MyHelper::isPelaksanaSatker() && session('userData.current_role.ms_satker_id') == '00') {
            $aktifitasOptions = DB::table('ms_aktifitas_user')->where(['group' => 'BMN'])->whereIn('id',[3000,3004])->get()->toArray();
        }
        $data = [
            'model' => $model,
            'controller' => $this->controller,
            'aktifitasOptions' => $aktifitasOptions,
            'aktifitas' => $currentAktifitas,
            'aktifitasHistories' => $aktifitasHistories,
            'breadcums' => array_merge($this->breadcums, ['Pengajuan']),
            'isNew' => $isNew,
            'kategoriJudul' => $this->kategoriJudul,
            'canCreate' => $this->canCreatePermintaan(),
            'isValidator' => $this->isValidator(),
            'listBarang' => Master::getBarangAset(),
            'msFile' => $ms_file,
            'ms_role_id' => session('userData.current_role.ms_role_id'),
        ];
        if ($request->wantsJson()) {
            return response()->json($data);
        } else {
            return view('bmn.permohonan-penghapusan.pengajuanFormV', $data);
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

        $countFc = PermohonanPenghapusanBmnFotocopy::where('pengajuan_id', $id)->count();
        if($countFc<1){
            return $this->resError('Harap Isi Fotocopy Keputusan Penetapan Status Penggunaannya');
        }
        $countF = PermohonanPenghapusanBmnFoto::where('pengajuan_id', $id)->count();
        if($countF<4){
            return $this->resError('Foto terkini BMN yang akan dihapus minimal 4');
        }
        try {
            DB::beginTransaction();
            $currentRole = session('userData.current_role');
            Model::where(['id' => $id])->update(['ms_aktifitas_id' => $ms_aktifitas_id]);

            $dataAktifitas = [
                'pengajuan_id' => $id,
                'ms_aktifitas_id' => $ms_aktifitas_id,
                'komentar' => $request->input('komentar'),
                'to_satker_induk' => in_array($ms_aktifitas_id, [3000, 3003, 3005, 3007]) ? false : true
            ];
            $acts = Approval::roleCheck($dataAktifitas);
            Aktifitas::insert($acts['act']);

            if (!empty($acts['nextAct'])) {
                $newAct = $acts['nextAct'];
                if (in_array($newAct, [3003, 3005, 3007])) { //revisi
                    $newAct = 3000;
                }
                Model::where(['id' => $id])->update(['ms_aktifitas_id' => $newAct]);
            }
            $aktifitasOptions = DB::table('ms_aktifitas_user')->where(['id'=>$ms_aktifitas_id])->first();
            $bmnSatker = Model::where(['id' => $id])->first();
            $notifParams = [
                'url' => $this->url."/".$id,
                'judul' => 'Permohonan Penghapusan BMN ('.($aktifitasOptions->nama=='Draft'?'Revisi':$aktifitasOptions->nama).')',
                'isi' => $dataAktifitas['komentar'],
                'target' => 'role',
                'targetValue' => $aktifitasOptions->can_change ?? config('constants.pelaksana_satker_role_id'),
                'targetSatker' => [$acts['act']['ms_satker_id'] ?? $bmnSatker->inst_satkerkd],
                'targetSatkerPusat' => [$acts['act']['ms_satker_id'] ?? $bmnSatker->inst_satkerkd],
            ];
            Notifikasi::sendNotif($notifParams);

            if(1 == 1){
                foreach($request->input('ms_penghapusan_file_id') as $key => $ms_penghapusan_file_id){
                    $data = [
                        'pengajuan_id'=>$id,
                        'ms_penghapusan_file_id'=>$ms_penghapusan_file_id,
                        'nomor'=>$request->input('file_nomor')[$key],
                        'tanggal'=>$request->input('file_tanggal')[$key],
                    ];
                    $modelFile = PermohonanPenghapusanBmnFile::updateOrCreate(['id' => $request->input('file_id')[$key]], $data);
                    $insertedId = $modelFile->id;
                    if ($request->hasFile('file_file.'.$key)) {
                        $filepath = 'uploads/bmn/permohonan_penghapusan/'.$id;
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
            $model = PermohonanPenghapusanBmn::where('id',$id)->first();
            if(!empty($model)){
                $filePath = public_path($model->file_surat_permohonan);
                // Delete the physical file from public directory
                if (FileManager::exists($filePath)) {
                    FileManager::delete($filePath);
                }

                $fc = PermohonanPenghapusanBmnFotocopy::where('pengajuan_id', $id)->get();
                if(!empty($fc)){
                    foreach ($fc as $file) {
                        $filePath = public_path($file->file);
                        // Delete the physical file from public directory
                        if (FileManager::exists($filePath)) {
                            FileManager::delete($filePath);
                        }
                    }
                }
                $foto = PermohonanPenghapusanBmnFoto::where('pengajuan_id', $id)->get();
                if(!empty($foto)){
                    foreach ($foto as $file) {
                        $filePath = public_path($file->file);
                        // Delete the physical file from public directory
                        if (FileManager::exists($filePath)) {
                            FileManager::delete($filePath);
                        }
                    }
                }
                $files = PermohonanPenghapusanBmnFile::where('pengajuan_id', $id)->get();
                if(!empty($files)){
                    foreach ($files as $file) {
                        $filePath = public_path($file->file);
                        // Delete the physical file from public directory
                        if (FileManager::exists($filePath)) {
                            FileManager::delete($filePath);
                        }
                    }
                }
            }
            $model->delete();
            DB::commit();
            return $this->resSuccess('Berhasil Dihapus!');
        } catch (\Throwable $th) {
            dd($th->getMessage());
            DB::rollBack();
            return $this->resError('Gagal Menghapus data');
        }
    }

    public function deleteFotocopy(string $id)
    {
        try {
            DB::beginTransaction();
            $fc = PermohonanPenghapusanBmnFotocopy::where('id', $id)->first();
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

    public function deleteFoto(string $id)
    {
        try {
            DB::beginTransaction();
            $fc = PermohonanPenghapusanBmnFoto::where('id', $id)->first();
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

    public function downloadZip(string $id)
    {
        try{
            $zip = new ZipArchive;
            $fileName = $id.'.zip';
            $folderPath = public_path('uploads/bmn/permohonan_penghapusan/'.$id);
            if ($zip->open(public_path($fileName), ZipArchive::CREATE) === true) {
                $files = glob("$folderPath/*");
                foreach ($files as $file) {
                    $zip->addFile($file, basename($file));
                }
                $zip->close();
            }
            return response()->download(public_path($fileName))->deleteFileAfterSend(true);
        }catch (\Throwable $th) {
            return redirect()->back()->with('error', 'Gagal Download File');
        }
    }

    public function deleteFilelain(string $id)
    {
        try {
            DB::beginTransaction();
            $fc = PermohonanPenghapusanBmnMonitorFile::where('id', $id)->first();
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
