<?php

namespace App\Http\Controllers\AnalisisKebutuhan\Bmn;

use App\Helpers\MyHelper;
use App\Http\Controllers\Controller;
use App\Models\AnalisisKebutuhan\Bmn;
use App\Models\AnalisisKebutuhan\Bmn as Model;
use App\Models\AnalisisKebutuhan\BmnAsset;
use App\Models\AnalisisKebutuhan\BmnSatker;
use App\Models\AnalisisKebutuhan\BmnSatkerAktifitas;
use App\Models\AnalisisKebutuhan\BmnSatkerBarang;
use App\Models\ApprovalUserSpseSirup;
use App\Models\Files;
use App\Models\Master;
use App\Models\Master\MsSatker as MasterMsSatker;
use App\Models\Notifikasi;
use Illuminate\Http\Request;
use Illuminate\Support\Arr;
use Illuminate\Support\Facades\DB;
use Illuminate\Support\Facades\File as FileManager;
use Symfony\Component\HttpKernel\Exception\NotFoundHttpException;

class PengajuanController extends Controller
{
    protected $breadcums = ['Analisis Kebutuhan', 'Kebutuhan BMN'];

    private $controller = '/analisis-kebutuhan/bmn/pengajuan';

    public function __construct(Request $request)
    {
        $this->breadcums = array_merge($this->breadcums, [['link' => $this->controller, 'title' => 'Pengajuan']]);
    }

    public function index()
    {
        $data = [
            'tableId' => 'dt-pengajuan',
            'breadcums' => $this->breadcums,
            'controller' => $this->controller,
            'operasi' => $this->userOperation(),
        ];

        return view('analisis_kebutuhan.bmn.pengajuan.pengajuanV', $data);
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
        $user = new Model;
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
        $user = new Model;
        $pagingParams = $request->only(['start', 'length']);
        $searchParams = $request->only(['columns', 'id']);
        $data = $user->getGridDataSatker($pagingParams, $searchParams);

        return response()->json([
            'data' => $data['data'],
            'recordsTotal' => $data['total'],
            'recordsFiltered' => $data['total'],
        ]);
    }

    public function gridDataBarang($id)
    {
        $data = BmnSatkerBarang::where('pengajuan_kebutuhan_bmn_satker_barang.pengajuan_kebutuhan_bmn_satker_id', $id)
            ->select('pengajuan_kebutuhan_bmn_satker_barang.id', 'pengajuan_kebutuhan_bmn_satker_barang.nama',
                'pengajuan_kebutuhan_bmn_satker_barang.jumlah', 'pengajuan_kebutuhan_bmn_satker_barang.alasan',
                'pengajuan_kebutuhan_bmn_satker_barang.file_pendukung', 'pengajuan_kebutuhan_bmn_satker_barang.pengajuan_kebutuhan_bmn_satker_id',
                'pengajuan_kebutuhan_bmn_satker_barang.prioritas', 'pengajuan_kebutuhan_bmn_satker_barang.keterangan', 'pengajuan_kebutuhan_bmn_satker_barang.jml_setuju', 'pengajuan_kebutuhan_bmn_satker_barang.kode_barang', DB::raw('COUNT(d.kode_barang) as jumlah_exist'))
            ->leftJoin('pengajuan_kebutuhan_bmn_satker as b', 'pengajuan_kebutuhan_bmn_satker_barang.pengajuan_kebutuhan_bmn_satker_id', '=', 'b.id')
            ->leftJoin('ms_satker as c', 'b.ms_satker_id', '=', 'c.inst_satkerkd')
            ->leftJoin('vw_asset_barang_kdsatker as d', function ($join) {
                $join->on('d.kdsatker_keu', '=', 'c.kdsatker_keu')
                    ->on('d.kode_barang', '=', 'pengajuan_kebutuhan_bmn_satker_barang.kode_barang');
            })
            ->groupBY('pengajuan_kebutuhan_bmn_satker_barang.id', 'pengajuan_kebutuhan_bmn_satker_barang.nama',
                'pengajuan_kebutuhan_bmn_satker_barang.jumlah', 'pengajuan_kebutuhan_bmn_satker_barang.alasan',
                'pengajuan_kebutuhan_bmn_satker_barang.file_pendukung', 'pengajuan_kebutuhan_bmn_satker_barang.pengajuan_kebutuhan_bmn_satker_id',
                'pengajuan_kebutuhan_bmn_satker_barang.prioritas', 'pengajuan_kebutuhan_bmn_satker_barang.keterangan', 'pengajuan_kebutuhan_bmn_satker_barang.jml_setuju', 'pengajuan_kebutuhan_bmn_satker_barang.kode_barang')
            ->orderBy('prioritas', 'ASC')->get()->toArray();

        return response()->json([
            'data' => $data,
        ]);
    }

    public function getData($id = null)
    {
        $model = [];
        $isNew = true;
        $breadcum = 'Tambah';
        $selectedSatker = [];
        $model['is_appv_daskrimti'] = '';
        $model['id_jenis_asset'] = '';
        if ($id) {
            $breadcum = 'Ubah';
            $model = Model::where('id', $id)->first();
            if (! $model) {
                throw new NotFoundHttpException('Data Tidak Ditemukan');
            }
            $satkerTerpilih = BmnSatker::where(['pengajuan_kebutuhan_bmn_id' => $id])->get()->toArray();
            if (! empty($satkerTerpilih)) {
                $selectedSatker = Arr::pluck($satkerTerpilih, 'ms_satker_id');
            }
            $model = $model->toArray();
            $isNew = false;
        }
        $current_year = date('Y');
        $range = range($current_year - 2, $current_year + 2);
        $years = array_combine($range, $range);
        $satkerTerpilih = Master::getSatkers();
        $id_jenis_asset = $model && $model['id_jenis_asset'] ? json_decode($model['id_jenis_asset']) : null;
        $asset = BmnAsset::where(['pengajuan_kebutuhan_bmn_id' => $id])->get()->toArray();
        $data = [
            'model' => $model,
            'isNew' => $isNew,
            'controller' => $this->controller,
            'breadcums' => array_merge($this->breadcums, [$breadcum]),
            'jenisAssetOptions' => MyHelper::generateSelectOptions([
                'data' => Master::getBarangAset(),
                'text' => 'nama_barang',
                'textkode' => 'kode_barang',
                'value' => 'kode_barang',
                'selected' => $id_jenis_asset,
            ]),
            'satkerOptions' => MyHelper::generateSelectOptions(['data' => $satkerTerpilih, 'text' => 'inst_nama', 'value' => 'inst_satkerkd', 'selected' => $selectedSatker]),
            'yearOptions' => MyHelper::generateSelectOptions(['data' => $years, 'selected' => ($model['tahun'] ?? date('Y'))]),
            'listBarang' => Master::getBarangAset(),
            'asset' => $asset,
        ];

        return $data;
    }

    public function listSatker(string $pengajuanId)
    {
        $data = [
            'tableId' => 'dt-pengajuan',
            'breadcums' => array_merge($this->breadcums, ['Daftar Kebutuhan BMN Satker']),
            'controller' => $this->controller,
            'operasi' => $this->userOperation(),
            'id' => $pengajuanId,
        ];

        return view('analisis_kebutuhan.bmn.daftar_kebutuhan.daftarKebutuhanBmnV', $data);
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
        $isNew = false;
        $validasi = [
            'nama' => 'required',
            'tahun' => 'required',
        ];
        $customMessages = [
            'nama.required' => 'Nama harus diisi',
            'tahun.required' => 'Tahun harus diisi',
        ];
        $request->validate($validasi, $customMessages);
        try {
            $inputan = $request->input();
            DB::beginTransaction();
            $id = $request->input('id');
            if (! $request->has('id')) {
                $id = MyHelper::getPk(date('Ymd'), 'pengajuan_kebutuhan_bmn_seq');
                $isNew = true;
            }
            $datainsert = [
                'nama' => $request->input('nama'),
                'deskripsi' => $request->input('deskripsi'),
                'pilihan_satker' => $request->input('pilihan_satker'),
                'tahun' => $request->input('tahun'),
                'tgl_mulai' => $request->input('tgl_mulai'),
                'tgl_selesai' => $request->input('tgl_selesai'),
                'is_appv_daskrimti' => $request->input('is_appv_daskrimti'),
            ];
            Model::updateOrCreate(['id' => $id], $datainsert);
            $satkerQ = DB::table('ms_satker')->select('inst_satkerkd', 'is_pusat')->orderBy('inst_satkerkd');
            if ($request->input('pilihan_satker') == 'sebagian') {
                $satkerQ->where(function ($q) use ($inputan) {
                    foreach ($inputan['satkers'] as $kdSatker) {
                        if ($kdSatker == '00') {
                            $q->orWhere('inst_satkerkd', '00')->orWhere('is_pusat', 1);
                        } else {
                            $q->orWhere('inst_satkerkd', 'like', "{$kdSatker}%");
                        }
                    }
                });
            }
            $satkers = $satkerQ->get();
            $targetSatker = [];
            $targetSatkerPusat = [];
            foreach ($satkers as $value) {
                $satkerTerpilih[] = [
                    'id' => MyHelper::getPk(date('Ymd'), 'pengajuan_kebutuhan_bmn_satker_seq'),
                    'ms_satker_id' => $value->inst_satkerkd,
                    'pengajuan_kebutuhan_bmn_id' => $id,
                ];
                if ($isNew) {
                    $satkerId = $value->inst_satkerkd;
                    $pusatId = null;
                    $targetSatker[] = $satkerId;
                    if ($value->is_pusat == 1) {
                        $pusatId = $satkerId;
                        $satkerId = '00';
                        $targetSatkerPusat[] = $pusatId;
                    }
                }
            }
            BmnSatker::where(['pengajuan_kebutuhan_bmn_id' => $id])->delete();
            BmnSatker::insert($satkerTerpilih);

            $assets = $inputan['asset_kode_barang'];
            foreach ($assets as $index => $value) {
                $barang = DB::table('vw_asset_barang')->where('kode_barang', $value)->first();
                $dataAsset[] = [
                    'pengajuan_kebutuhan_bmn_id' => $id,
                    'kode_barang' => $value,
                    'nm_barang' => $barang->nm_barang,
                    'ms_jenis_asset_id' => $barang->ms_jenis_asset_id,
                    'keterangan' => $inputan['asset_keterangan'][$index],
                ];
            }
            BmnAsset::where(['pengajuan_kebutuhan_bmn_id' => $id])->delete();
            BmnAsset::insert($dataAsset);

            if ($isNew) {
                $notifParams = [
                    'url' => 'analisis-kebutuhan/bmn/pengajuan/'.$id.'/list-satker',
                    'judul' => 'Kebutuhan BMN Baru',
                    'isi' => '',
                    'target' => 'role',
                    'targetValue' => config('constants.pelaksana_satker_role_id'),
                    'targetSatker' => $targetSatker,
                    'targetSatkerPusat' => $targetSatkerPusat,
                ];
                Notifikasi::sendNotif($notifParams);
            }
            DB::commit();

            return $this->resSuccess('Berhasil Dismpan');
        } catch (\Throwable $th) {
            dd($th->getMessage());

            return $this->resError('Gagal menyimpan data');
        }
    }

    public function saveBarang(Request $request)
    {
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
                'nama' => $request->input('nama'),
                'kode_barang' => $request->input('kode'),
                'jumlah' => $request->input('jumlah'),
                'alasan' => $request->input('alasan'),
                'keterangan' => $request->input('keterangan'),
                'jml_setuju' => $request->input('jml_setuju'),
                'pengajuan_kebutuhan_bmn_satker_id' => $request->input('pengajuan_kebutuhan_bmn_satker_id'),
            ];
            $idbarang = BmnSatkerBarang::updateOrCreate(['id' => $id], $data);
            if ($request->hasFile('file_pendukung')) {
                $filepath = 'uploads/kebutuhan_bmn';
                $file = $request->file('file_pendukung');
                $fileName = $idbarang['id'].'_file_pendukung.'.$file->getClientOriginalExtension();
                $filesave = $filepath.'/'.$fileName;
                $file->move($filepath, $fileName);
                BmnSatkerBarang::updateOrCreate(['id' => $idbarang['id']], ['file_pendukung' => $filesave]);
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
            if ($barang->file_pendukung) {
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
            'ms_aktifitas_id' => 'required',
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
        $ms_aktifitas_id = $request->input('ms_aktifitas_id');
        $type = $request->input('type') ?? '';
        // cek barang harus ada
        $barang = BmnSatkerBarang::where(['pengajuan_kebutuhan_bmn_satker_id' => $pengajuan_kebutuhan_bmn_satker_id])->get();
        if ($barang->isEmpty()) {
            return $this->resError('Barang harus diisi');
        }
        if ($ms_aktifitas_id == 3006 || $type == 'selesai') {
            foreach ($barang as $data) {
                if ($data->jml_setuju == '' || is_null($data->jml_setuju)) {
                    return $this->resError('Jumlah disetujui masing-masing barang harus diisi');
                }
            }
        }

        try {
            DB::beginTransaction();
            BmnSatker::where(['id' => $pengajuan_kebutuhan_bmn_satker_id])->update(['ms_aktifitas_id' => $ms_aktifitas_id]);
            $bmnSatker = BmnSatker::where(['id' => $pengajuan_kebutuhan_bmn_satker_id])->first();

            $dataAktifitas = [
                'pengajuan_id' => $pengajuan_kebutuhan_bmn_satker_id,
                'ms_aktifitas_id' => $ms_aktifitas_id,
                'komentar' => $request->input('komentar'),
                'to_satker_induk' => (in_array($ms_aktifitas_id, [3000, 3001, 3003, 3005, 3007, 3014])) ? false : true,
                'group' => 'BMN',
            ];
            $acts = ApprovalUserSpseSirup::roleCheck($dataAktifitas);
            BmnSatkerAktifitas::insert($acts['act']);
            if (! empty($acts['nextAct'])) {
                $newAct = $acts['nextAct'];
                if (in_array($newAct, [3003, 3005, 3007, 3014])) { // revisi
                    $newAct = 3000;
                }
                // BmnSatker::where(['id' => $id])->update(['ms_aktifitas_id' => $newAct]);
            }

            $aktifitasOptions = DB::table('ms_aktifitas_user')->where(['group' => 'BMN', 'id' => $ms_aktifitas_id])->first();
            $notifParams = [
                'url' => 'analisis-kebutuhan/bmn/pengajuan/'.$pengajuan_kebutuhan_bmn_satker_id.'/edit',
                'judul' => 'Kebutuhan BMN ('.($aktifitasOptions->nama == 'Draft' ? 'Revisi' : $aktifitasOptions->nama).')',
                'isi' => $dataAktifitas['komentar'],
                'target' => 'role',
                'targetValue' => $aktifitasOptions->can_change ?? config('constants.pelaksana_satker_role_id'),
                'targetSatker' => [$acts['act']['ms_satker_id'] ?? $bmnSatker->ms_satker_id],
                'targetSatkerPusat' => [$acts['act']['ms_satker_id'] ?? $bmnSatker->ms_satker_id],
            ];
            Notifikasi::sendNotif($notifParams);

            DB::commit();

            return $this->resSuccess(
                'Berhasil Disimpan!',
                [
                    'type' => 'redirect',
                    'url' => \URL::to('/analisis-kebutuhan/bmn/pengajuan/'.$id.'/list-satker'),
                ]
            );
        } catch (\Throwable $th) {
            DB::rollBack();
            dd($th->getMessage());

            return $this->resError('Gagal Menyimpan data');
        }
    }

    /**
     * edit admin perlengkapan
     */
    public function show(string $id)
    {
        $data = $this->getData($id);

        return view('analisis_kebutuhan.bmn.pengajuan.pengajuanFormV', $data);
    }

    /**
     * pelaksana ngisi /  validator ngeliat
     */
    public function edit(string $id)
    {
        $pengajuanSatker = BmnSatker::where('id', $id)->first();
        $pengajuan = Bmn::where('id', $pengajuanSatker['pengajuan_kebutuhan_bmn_id'])->first();
        $satker = MasterMsSatker::where('inst_satkerkd', $pengajuanSatker['ms_satker_id'])->first();
        $asset = BmnAsset::where(['pengajuan_kebutuhan_bmn_id' => $pengajuanSatker['pengajuan_kebutuhan_bmn_id']])->get()->toArray();
        $kodeBarangArray = array_map(function ($item) {
            return $item['kode_barang'];
        }, $asset);
        // aktifitas
        $msAktifitasId = $pengajuanSatker->ms_aktifitas_id ?? 3000;
        if (in_array($msAktifitasId, [3003, 3005, 3007, 3014])) { // revisi
            $msAktifitasId = 3000;
        }
        $whereAct = ['ms_aktifitas_id' => ($msAktifitasId == 3008 ? 3002 : $msAktifitasId), 'group' => 'BMN'];
        $aktifitasHistories = BmnSatkerAktifitas::getDetail($pengajuanSatker['id']);
        $aktifitasOptions = ApprovalUserSpseSirup::getAktifitas($whereAct);

        if (empty($msAktifitasId) || $msAktifitasId == 3000) {
            $aktifitasOptions = DB::table('ms_aktifitas_user')->where(['group' => 'BMN'])->whereIn('id', [3000, 3002])->get()->toArray();
        }

        if ($pengajuan['is_appv_daskrimti'] == 1 && $msAktifitasId == 3002) {
            $aktifitasOptions = DB::table('ms_aktifitas_user')->where(['group' => 'BMN'])->whereIn('id', [3008, 3009])->get()->toArray();
        }

        if (session('userData.current_role.ms_satker_id') == '00' && session('userData.current_role.ms_role_id') == config('constants.pelaksana_satker_role_id')) {
            $aktifitasOptions = DB::table('ms_aktifitas_user')->where(['group' => 'BMN'])->whereIn('id', [3004, 3005])->get()->toArray();
        }

        $currentAktifitas = ApprovalUserSpseSirup::getCurrentAktifitas($msAktifitasId);
        $kodeBarang = Master::getBarangAset($kodeBarangArray);
        $data = [
            'model' => $pengajuan,
            'pengajuanSatker' => $pengajuanSatker,
            'controller' => $this->controller,
            'breadcums' => array_merge($this->breadcums, ['Approval']),
            'satker' => $satker['inst_nama'],
            'operasi' => $this->userOperation(),
            'aktifitasOptions' => $aktifitasOptions,
            'aktifitas' => $currentAktifitas,
            'aktifitasHistories' => $aktifitasHistories,
            'asset' => $asset,
            'jenisAssetOptions' => MyHelper::generateSelectOptions([
                'data' => Master::getBarangAset(),
                'text' => 'nama_barang',
                'textkode' => 'kode_barang',
                'value' => 'kode_barang',
                'selected' => $kodeBarangArray,
            ]),
            'kdBarangOptions' => $kodeBarang,
            'listBarang' => Master::getBarangAset(),
        ];

        return view('analisis_kebutuhan.bmn.daftar_kebutuhan.daftarKebutuhanBmnFormV', $data);
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
