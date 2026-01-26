<?php

namespace App\Http\Controllers\Bmn;
use Illuminate\Routing\Controller;

use App\Helpers\MyHelper;
use App\Models\Bmn\ApprovalBmn as Approval;
use App\Models\Master\MsSatker;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\DB;
use Symfony\Component\HttpKernel\Exception\NotFoundHttpException;
use App\Models\Bmn\BmnIjin as Model;
use App\Models\Bmn\BmnIjinAktivitas as Aktivitas;
use App\Models\Bmn\BmnIjinPegawai as Pegawai;
use App\Models\Bmn\BmnIjinAset as Aset;
use App\Models\Bmn\BmnIjinFile as File;
use Illuminate\Support\Facades\File as FileManager;
use Mccarlosen\LaravelMpdf\Facades\LaravelMpdf;


class IjinController extends Controller
{
    protected $kategoriJudul = 'Ijin Pemakaian BMN';
    protected $kategori = 'ijin';
    protected $breadcums = ['Bmn'];
    private $controller = '/bmn/ijin';

    public function __construct(Request $request)
    {
        $segment = $request->segment(2);
        if ($segment == 'bmnijin') {
            $this->kategoriJudul = 'Ijin BMN';
            $this->kategori = 'ijin';
            $this->controller = '/bmn/ijin';
        }
        $this->breadcums = array_merge($this->breadcums, [['link' => $this->controller, 'title' => $this->kategoriJudul]]);
    }

    public function index()
    {
        $columns = [
            'Periode',
            'Satker',
            'Aktivitas',
            'Aksi',
        ];
        $defColumns = [0, 1, 2, 3];
        return view('bmn.ijinV', [
            'tableId' => 'dt-pengajuan',
            'breadcums' => $this->breadcums,
            'controller' => $this->controller,
            'kategori' => $this->kategori,
            'kategoriJudul' => $this->kategoriJudul,
            'canCreate' => $this->canCreatePermintaan(),
            'columns' => $columns,
            'defColumns' => $defColumns
        ]);
    }

    protected function canCreatePermintaan()
    {
        // return true;
        return in_array(session('userData.current_role.ms_role_id'), config('constants.pelaksana_role'));
    }

    public function gridData(Request $request)
    {
        $user = new Model();
        $pagingParams = $request->only(['start', 'length']);
        $searchParams = $request->only(['search', 'filterBy']);
        $data = $user->getDataGrid($pagingParams, $searchParams, $this->kategori);
        return response()->json([
            'data' => $data['data'],
            'recordsTotal' => $data['total'],
            'recordsFiltered' => $data['total'],
        ]);
    }

    public function gridDataPegawai($pengajuan_id, $json = false)
    {
        $model = new Pegawai();
        $data = $model->getDetail($pengajuan_id);
        if ($json)
            return $data;
        return response()->json([
            'data' => $data,
        ]);
    }

    public function gridDataMsPegawai($pengajuan_id)
    {
        $model = new Pegawai();
        $currentRole = session('userData.current_role');
        $data = $model->getMsPegawai($currentRole['ms_satker_id'], $pengajuan_id);
        return response()->json([
            'data' => $data,
        ]);
    }

    public function gridDataAset($pengajuan_id)
    {
        $model = new Aset();
        $currentRole = session('userData.current_role');
        $data = $model->getAset($currentRole['ms_satker_id'], $pengajuan_id);
        //$data = $model->getAset( $pengajuan_id);
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
            'breadcums' => array_merge($this->breadcums, [$breadcum])
        ];
        if (request()->wantsJson()) {
            return response()->json(
                [
                    'model' => $model,
                    'pegawai' => $this->gridDataPegawai($model['id'], true),
                    'aktivitas' => Aktivitas::getDetail($model['id']),
                ]
            );
        }
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
        //
        $customMessages = [
            'tgl_pengajuan1.required' => 'Periode 1 harus diisi',
            'tgl_pengajuan2.required' => 'Periode 2 harus diisi',
        ];
        $validasi = [
            'tgl_pengajuan1' => 'required',
            'tgl_pengajuan2' => 'required',
        ];

        $request->validate($validasi, $customMessages);
        try {
            DB::beginTransaction();
            $id = $request->input('id');
            $inputan['tgl_pengajuan1'] = $request->input('tgl_pengajuan1');
            $inputan['tgl_pengajuan2'] = $request->input('tgl_pengajuan2');
            $inputan['kategori'] = $request->input('kategori');
            if ($id == '' || $id == null) {
                $inputan['inst_satkerkd'] = session('userData.current_role.ms_satker_id');
                $id = MyHelper::getPk(date('Ymd'), 'bmn_ijin_seq');
                $inputan['ms_aktifitas_id'] = 2000;
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

    public function savePegawai(Request $request)
    {
        $customMessages = [
            'nip.required' => 'Pegawai harus dipilih',
        ];
        $validasi = [
            'nip' => 'required',
        ];
        $msFile = File::getMasterFile($this->kategori);
        foreach ($msFile as $file) {
            $customMessages[$file->jenis . 'required'] = $file->nama . ' harus diupload';
            $validasi[$file->jenis] = 'required|mimes:jpeg,png,pdf|max:2048';
        }
        $request->validate($validasi, $customMessages);
        try {
            DB::beginTransaction();
            $id = $request->input('pengajuan_pegawai_id');
            $pegawai = Pegawai::updateOrCreate(['id' => $id], $request->only(['nip', 'nama', 'pangkat', 'jabatan', 'pengajuan_id']));
            $aset = Aset::updateOrCreate(['id' => $id], $request->only(['pengajuan_id', 'kode_barang', 'nama_barang', 'keterangan', 'nup', 'nip']));
            File::where(['pengajuan_pegawai_id' => $id])->delete();
            $msFile = File::getMasterFile($this->kategori);
            foreach ($msFile as $file) {
                $jenis = $file->jenis;
                $pegawaiId = $pegawai->id;
                if ($request->hasFile($jenis)) {
                    $filepath = 'uploads/bmn/pemakaian';
                    $file = $request->file($jenis);
                    $fileName = $pegawaiId . '_' . $jenis . '.' . $file->getClientOriginalExtension();
                    $filesave = $filepath . '/' . $fileName;
                    $file->move($filepath, $fileName);

                    $file = new File();
                    $file->pengajuan_pegawai_id = $pegawaiId;
                    $file->jenis = $jenis;
                    $file->url = $filesave;
                    $file->save();
                }
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

    public function saveUserPegawai(Request $request)
    {
        $customMessages = [
            'username.required' => 'Username harus diisi',
            'password.required' => 'Password harus diisi',
        ];
        $validasi = [
            'username' => 'required',
            'password' => 'required',
        ];

        $request->validate($validasi, $customMessages);
        try {
            DB::beginTransaction();
            $id = $request->input('id');
            Pegawai::where(['id' => $id])->update($request->only(['username', 'password']));
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
     * pelaksana ngisi /  validator ngeliat
     */
    public function show(string $id)
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

        //aktifitas
        $msAktivitasId = $model->ms_aktifitas_id;
        $whereAct = ['ms_aktifitas_id' => $msAktivitasId];
        $aktifitasHistories = Aktivitas::getDetail($model->id);
        $aktifitasOptions = [];
        $currentAktivitas = [];

        //file
        $msFile = File::getMasterFile($this->kategori);

        $data = [
            'pengajuan' => $pengajuan,
            'model' => $model,
            'controller' => $this->controller,
            'aktivitasOptions' => $aktifitasOptions,
            'aktivitas' => $currentAktivitas,
            'aktivitasHistories' => $aktifitasHistories,
            'breadcums' => array_merge($this->breadcums, ['Pengajuan']),
            'isNew' => $isNew,
            'kategoriJudul' => $this->kategoriJudul,
            'msFile' => $msFile,
            'canCreate' => $this->canCreatePermintaan()
        ];
        return view('bmn.ijinFormSatkerIsiV', $data);
    }

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

        //cek pegawai harus ada dan apabila 2009(selesai) cek username password
        $pegawai = Pegawai::where(['pengajuan_id' => $id])->get();
        if ($pegawai->isEmpty())
            return $this->resError('Pegawai harus diisi');
        if ($ms_aktifitas_id == 2009) {
            foreach ($pegawai as $data) {
                if (empty($data->username) || empty($data->password)) {
                    //return $this->resError('Username dan Password masing-masing pegawai harus diisi');
                }
            }
        }

        try {
            DB::beginTransaction();
            $currentRole = session('userData.current_role');
            Model::where(['id' => $id])->update(['ms_aktifitas_id' => $ms_aktifitas_id]);

            $dataAktivitas = [
                'pengajuan_id' => $id,
                'ms_aktifitas_id' => $ms_aktifitas_id,
                'komentar' => $request->input('komentar'),
                'to_satker_induk' => $ms_aktifitas_id == 2000 ? false : true
            ];
            $acts = ["act" => [], "nextAct" => null];
            Aktivitas::insert($acts['act']);

            if (!empty($acts['nextAct'])) {
                $newAct = $acts['nextAct'];
                if (in_array($newAct, [2003])) { //revisi
                    $newAct = 2000;
                }
                Model::where(['id' => $id])->update(['ms_aktifitas_id' => $newAct]);
            }

            DB::commit();
            return $this->resSuccess(
                'Berhasil Disimpan!',
                [
                    'type' => 'redirect',
                    'url' => \URL::to('/bmn/ijin')
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
            $files = File::where(['pengajuan_pegawai_id' => $id])->get();
            foreach ($files as $file) {
                $filePath = public_path($file->url);
                // Delete the physical file from public directory
                if (FileManager::exists($filePath)) {
                    FileManager::delete($filePath);
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

    public function deletePegawai(string $id)
    {
        try {
            DB::beginTransaction();
            $files = File::where(['pengajuan_pegawai_id' => $id])->get();
            foreach ($files as $file) {
                $filePath = public_path($file->url);
                // Delete the physical file from public directory
                if (FileManager::exists($filePath)) {
                    FileManager::delete($filePath);
                }
            }
            Pegawai::destroy($id);
            DB::commit();
            return $this->resSuccess('Berhasil Dihapus!');
        } catch (\Throwable $th) {
            DB::rollBack();
            return $this->resError('Gagal Menghapus data');
        }
    }
    public function cetak(string $id)
    {
        $model = new Model();
        $data = $model->getDataCetak($id);
        $pengajuan = Model::where('id', $id)->first();
        $currentOrder = [];
        foreach ($data as $row) {
            $orderId = $row->id;
            if (!array_key_exists($orderId, $currentOrder)) {
                $currentOrder[$orderId] = [
                    'satker' => $row->inst_satkerkd,
                    'nama_satker' => $row->inst_nama,
                ];
            }
            $currentOrder[$orderId]['items'][] = [
                'nip' => $row->nip,
                'nama' => $row->nama,
                'kode_barang' => $row->kode_barang,
                'nama_barang' => $row->nama_barang,
                'keterangan' => $row->keterangan,
                'nup' => $row->nup,

            ];
        }
        $judul = '';
        $pdf = LaravelMpdf::loadView('bmn.cetak', [
            'data' => $currentOrder,
            'judul' => $judul,
            'model' => $pengajuan
        ], [], [
            'title' => $judul,
        ]);
        return $pdf->stream('cetak.pdf');
    }
}
