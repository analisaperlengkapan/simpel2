<?php

namespace App\Http\Controllers\Bmn\Penghapusan;

use App\Helpers\MyHelper;
use App\Http\Controllers\Controller;
use App\Models\ApprovalUserSpseSirup as Approval;
use App\Models\Bmn\PengajuanPenghapusanBmn\PengajuanPenghapusanBmn as Model;
use App\Models\Bmn\PengajuanPenghapusanBmn\PengajuanPenghapusanBmnAktifitas as Aktifitas;
use App\Models\Bmn\PengajuanPenghapusanBmn\PengajuanPenghapusanBmnAsset as Asset;
use App\Models\Bmn\PengajuanPenghapusanBmn\PengajuanPenghapusanBmnAssetFile as File;
use App\Models\Master;
use App\Models\Master\MsSatker;
use App\Models\Notifikasi;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\DB;
use Illuminate\Support\Facades\File as FileManager;
use Symfony\Component\HttpKernel\Exception\NotFoundHttpException;

class PengajuanPenghapusanBmnController extends Controller
{
    protected $kategoriJudul = 'Pengajuan Penghapusan BMN';

    protected $kategori = 'penghapusan_bmn';

    protected $breadcums = ['Pengelolaan BMN', 'Penghapusan BMN'];

    private $controller = '/bmn/penghapusan/penghapusansk';

    public function __construct(Request $request)
    {
        $this->breadcums = array_merge($this->breadcums, [['link' => $this->controller, 'title' => $this->kategoriJudul]]);
    }

    public function index()
    {
        // dd(session('userData.current_role.ms_role_id'));
        return view('bmn.penghapusansk.pengajuan.pengajuanV', [
            'tableId' => 'dt-pengajuan',
            'breadcums' => $this->breadcums,
            'controller' => $this->controller,
            'kategori' => $this->kategori,
            'kategoriJudul' => $this->kategoriJudul,
            'canCreate' => $this->canCreatePermintaan(),
        ]);
    }

    protected function canCreatePermintaan()
    {
        // return true;
        return in_array(session('userData.current_role.ms_role_id'), config('constants.pelaksana_role'));
    }

    public function gridData(Request $request)
    {
        $user = new Model;
        $pagingParams = $request->only(['start', 'length']);
        $searchParams = $request->only(['columns']);
        $data = $user->getDataGrid($pagingParams, $searchParams, $this->kategori);

        return response()->json([
            'data' => $data['data'],
            'recordsTotal' => $data['total'],
            'recordsFiltered' => $data['total'],
        ]);
    }

    public function gridDataAsset($pengajuan_id)
    {
        $model = new Asset;
        $data = $model->getDetail($pengajuan_id);

        return response()->json([
            'data' => $data,
        ]);
    }

    public function gridDataMsPegawai($pengajuan_id)
    {
        $model = new Asset;
        $currentRole = session('userData.current_role');
        $data = $model->getMsPegawai($currentRole['ms_satker_id'], $pengajuan_id);

        return response()->json([
            'data' => $data,
        ]);
    }

    public function getData($id = null)
    {
        $model = [];
        $isNew = true;
        $breadcum = 'Tambah';

        if ($id) {
            $breadcum = 'Ubah';
            $model = Model::where('id', $id)->first();
            if (! $model) {
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
    public function create() {}

    /**
     * Store a newly created resource in storage.
     */
    public function store(Request $request)
    {
        //
        $customMessages = [
            'tgl_pengajuan.required' => 'Tanggal Pengajuan harus diisi',
        ];
        $validasi = [
            'tgl_pengajuan' => 'required',
        ];

        $request->validate($validasi, $customMessages);
        try {
            DB::beginTransaction();
            $id = $request->input('id');
            $inputan['tgl_pengajuan'] = $request->input('tgl_pengajuan');
            $inputan['kategori'] = $request->input('kategori');
            $inputan['nama'] = $request->input('nama');
            if ($id == '' || $id == null) {
                $inputan['inst_satkerkd'] = session('userData.current_role.ms_satker_id');
                $id = MyHelper::getPk(date('Ymd'), 'pengajuan_penghapusan_bmn_seq');
                $inputan['ms_aktifitas_id'] = 3000;
            }
            Model::updateOrCreate(['id' => $id], $inputan);
            DB::commit();

            return $this->resSuccess(
                'Berhasil Disimpan!'
            );
        } catch (\Throwable $th) {
            dd($th->getMessage());

            return $this->resError('Gagal menyimpan data');
        }
    }

    public function saveAsset(Request $request)
    {
        $customMessages = [
            'kode_barang.required' => 'Barang harus dipilih',
        ];
        $validasi = [
            'kode_barang' => 'required',
        ];
        $request->validate($validasi, $customMessages);
        try {
            DB::beginTransaction();
            $id = $request->input('pengajuan_asset_id');
            $kode_barang = $request->input('kode_barang');
            $barang = DB::table('vw_asset_barang')->where('kode_barang', $kode_barang)->first();
            $data = [
                'pengajuan_id' => $request->input('pengajuan_id'),
                'kode_barang' => $kode_barang,
                'nm_barang' => $barang->nm_barang,
                'ms_jenis_asset_id' => $barang->ms_jenis_asset_id,
                'keterangan' => $request->input('keterangan'),
            ];
            Asset::updateOrCreate(['id' => $id], $data);
            DB::commit();

            return $this->resSuccess(
                'Berhasil Disimpan!'
            );
        } catch (\Throwable $th) {
            dd($th->getMessage());

            return $this->resError('Gagal menyimpan data');
        }
    }

    public function saveSkPenetapan(Request $request)
    {
        $customMessages = [
            'sk_penetapan.required' => 'SK Penetapan harus diupload',
        ];
        $validasi = [
            'sk_penetapan' => 'required',
        ];

        $request->validate($validasi, $customMessages);
        try {
            DB::beginTransaction();
            $pengajuan_asset_id = $request->input('id');
            $no_sk = $request->input('no_sk');
            $tgl_sk = $request->input('tgl_sk');
            File::where(['pengajuan_asset_id' => $pengajuan_asset_id])->delete();
            if ($request->hasFile('sk_penetapan')) {
                $filepath = 'uploads/bmn/penghapusansk';
                $file = $request->file('sk_penetapan');
                $fileName = $pengajuan_asset_id.'_sk_penetapan.'.$file->getClientOriginalExtension();
                $filesave = $filepath.'/'.$fileName;
                $file->move($filepath, $fileName);

                $file = new File;
                $file->pengajuan_asset_id = $pengajuan_asset_id;
                $file->no_sk = $no_sk;
                $file->tgl_sk = $tgl_sk;
                $file->jenis = 'sk_penetapan';
                $file->url = $filesave;
                $file->save();
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
    public function show(Request $request, string $id)
    {
        $data = $this->getData($id);
        $pengajuan = Model::where('id', $id)->first();
        if (! $pengajuan) {
            throw new NotFoundHttpException('Data Tidak Ditemukan');
        }
        $satker = MsSatker::where('inst_satkerkd', $pengajuan->inst_satkerkd)->first();
        $pengajuan->inst_nama = $satker->inst_nama;
        // $_GET['satker'] cuma ada klo diliat validator pusat /kejati
        $currentRole = session('userData.current_role');
        $whereData = ['ms_satker_id' => $_GET['satker'] ?? $currentRole['ms_satker_id'], 'pengajuan_id' => $pengajuan->id];
        $whereSatker = ['inst_satkerkd' => $_GET['satker'] ?? $currentRole['ms_satker_id']];

        if ($currentRole['ms_satker_id'] == '00' && ! isset($_GET['satker'])) {
            $whereData['ms_satker_pusat_id'] = $currentRole['ms_satker_pusat_id'];
            $whereSatker['unitkerja_idk'] = $currentRole['ms_satker_pusat_id'];
        }

        // model utama
        $model = $pengajuan ?? [];
        $isNew = empty($model) ? true : false;

        // aktifitas
        $msAktifitasId = $model->ms_aktifitas_id;
        $whereAct = ['ms_aktifitas_id' => $msAktifitasId, 'group' => 'BMN'];
        $aktifitasHistories = Aktifitas::getDetail($model->id);
        $aktifitasOptions = Approval::getAktifitas($whereAct);
        $currentAktifitas = Approval::getCurrentAktifitas($model->ms_aktifitas_id);

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
            'listBarang' => Master::getBarangAset(),
        ];
        if ($request->wantsJson()) {
            return response()->json($data);
        } else {
            return view('bmn.penghapusansk.pengajuan.pengajuanFormSatkerIsiV', $data);
        }
    }

    /**
     * pelaksana ngisi /  validator ngeliat
     */
    public function edit(string $id) {}

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

        // cek pegawai harus ada dan apabila 1009(selesai) cek SK
        $asset = Asset::where(['pengajuan_id' => $id])->get();
        if ($asset->isEmpty()) {
            return $this->resError('Barang harus diisi');
        }
        if ($ms_aktifitas_id == 3006) {
            $filesk = File::getDetail($id, 'sk_penetapan');
            if ($filesk->isEmpty()) {
                return $this->resError('SK Penetapan masing-masing barang harus diupload');
            }
            foreach ($filesk as $file) {
                if (empty($file->url)) {
                    return $this->resError('SK Penetapan masing-masing barang harus diupload');
                }
            }
        }

        try {
            DB::beginTransaction();
            $currentRole = session('userData.current_role');
            Model::where(['id' => $id])->update(['ms_aktifitas_id' => $ms_aktifitas_id]);

            $dataAktifitas = [
                'pengajuan_id' => $id,
                'ms_aktifitas_id' => $ms_aktifitas_id,
                'komentar' => $request->input('komentar'),
                'to_satker_induk' => $ms_aktifitas_id == 3000 ? false : true,
            ];
            $acts = Approval::roleCheck($dataAktifitas);
            Aktifitas::insert($acts['act']);

            if (! empty($acts['nextAct'])) {
                $newAct = $acts['nextAct'];
                if (in_array($newAct, [3003, 3005, 3007])) { // revisi
                    $newAct = 3000;
                }
                Model::where(['id' => $id])->update(['ms_aktifitas_id' => $newAct]);
            }
            // if($ms_aktifitas_id==1001){
            //     $targetSatker[] = config('constants.ms_satker_kejagung_id');
            //     $notifParams = [
            //         'url' => 'pengadaan/pokja-pemilihan/'.$id,
            //         'judul' => 'Approval Pengajuan '.$this->kategoriJudul,
            //         'isi' => '',
            //         'target' => 'role',
            //         'targetValue' => config('constants.admin_biro_lengkap_role_id'),
            //         'targetSatker' => $targetSatker,
            //         'targetSatkerPusat' => $targetSatker,
            //     ];
            //     Notifikasi::sendNotif($notifParams);
            // }
            DB::commit();

            return $this->resSuccess(
                'Berhasil Disimpan!',
                [
                    'type' => 'redirect',
                    'url' => \URL::to($this->controller),
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
            $asset = Asset::where('pengajuan_id', $id)->first();
            if (! empty($asset)) {
                $files = File::where(['pengajuan_asset_id' => $asset->id])->get();
                if (! empty($files)) {
                    foreach ($files as $file) {
                        $filePath = public_path($file->url);
                        // Delete the physical file from public directory
                        if (FileManager::exists($filePath)) {
                            FileManager::delete($filePath);
                        }
                    }
                }
            }
            Model::destroy($id);
            DB::commit();

            return $this->resSuccess('Berhasil Dihapus!');
        } catch (\Throwable $th) {
            DB::rollBack();

            return $this->resError('Gagal Menghapus data');
        }
    }

    public function deleteAsset(string $id)
    {
        try {
            DB::beginTransaction();
            $files = File::where(['pengajuan_asset_id' => $id])->get();
            if (! empty($files)) {
                foreach ($files as $file) {
                    $filePath = public_path($file->url);
                    // Delete the physical file from public directory
                    if (FileManager::exists($filePath)) {
                        FileManager::delete($filePath);
                    }
                }
            }
            Asset::destroy($id);
            DB::commit();

            return $this->resSuccess('Berhasil Dihapus!');
        } catch (\Throwable $th) {
            DB::rollBack();

            return $this->resError('Gagal Menghapus data');
        }
    }
}
