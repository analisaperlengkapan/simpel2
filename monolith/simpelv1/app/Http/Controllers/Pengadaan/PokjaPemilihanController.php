<?php

namespace App\Http\Controllers\Pengadaan;

use App\Helpers\MyHelper;
use App\Http\Controllers\Controller;
use App\Models\ApprovalUserSpseSirup as Approval;
use App\Models\Files;
use App\Models\Master\MsSatker;
use App\Models\Notifikasi;
use App\Models\Pengadaan\PokjaPemilihan\PokjaPemilihan as Model;
use App\Models\Pengadaan\PokjaPemilihan\PokjaPemilihanAktifitas as Aktifitas;
use App\Models\Pengadaan\PokjaPemilihan\PokjaPemilihanPegawai as Pegawai;
use App\Models\Pengadaan\PokjaPemilihan\PokjaPemilihanPegawaiFile as File;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\DB;
use Illuminate\Support\Facades\File as FileManager;
use Symfony\Component\HttpKernel\Exception\NotFoundHttpException;

class PokjaPemilihanController extends Controller
{
    protected $kategoriJudul = 'Layanan Penetapan Pokmil';

    protected $kategori = 'pokmil';

    protected $breadcums = ['Pengadaan'];

    private $controller = '/pengadaan/pokja-pemilihan';

    public function __construct(Request $request)
    {
        $this->breadcums = array_merge($this->breadcums, [['link' => $this->controller, 'title' => $this->kategoriJudul]]);
    }

    public function index()
    {
        return view('pengadaan.pokja.pengajuanV', [
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

    public function gridDataPegawai($pengajuan_id)
    {
        $model = new Pegawai;
        $data = $model->getDetail($pengajuan_id);

        return response()->json([
            'data' => $data,
        ]);
    }

    public function gridDataMsPegawai($pengajuan_id)
    {
        $model = new Pegawai;
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
            if ($id == '' || $id == null) {
                $inputan['inst_satkerkd'] = session('userData.current_role.ms_satker_id');
                $id = MyHelper::getPk(date('Ymd'), 'pengajuan_pokja_pemilihan_seq');
                $inputan['ms_aktifitas_id'] = 1000;
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
            $customMessages[$file->jenis.'.required'] = $file->nama.' harus diupload';
            $validasi[$file->jenis] = 'required|mimes:jpeg,png,pdf|max:2048';
        }
        $request->validate($validasi, $customMessages);
        try {
            DB::beginTransaction();
            $id = $request->input('pengajuan_pegawai_id');
            $pegawai = Pegawai::updateOrCreate(['id' => $id], $request->only(['nip', 'nama', 'pangkat', 'jabatan', 'pengajuan_id']));
            File::where(['pengajuan_pegawai_id' => $id])->delete();
            $msFile = File::getMasterFile($this->kategori);
            foreach ($msFile as $file) {
                $jenis = $file->jenis;
                $pegawaiId = $pegawai->id;
                if ($request->hasFile($jenis)) {
                    $filepath = 'uploads/pengadaan/pokmil';
                    $file = $request->file($jenis);
                    $fileName = $pegawaiId.'_'.$jenis.'.'.$file->getClientOriginalExtension();
                    $filesave = $filepath.'/'.$fileName;
                    $file->move($filepath, $fileName);
                    $newFile = [
                        'filename' => $file->getClientOriginalName(),
                        'path' => $filesave,
                        'pkey' => $pegawaiId,
                        'kategori' => $jenis,
                        'kategori_slug' => MyHelper::generateSlug($jenis),
                        'filetype' => $file->getClientOriginalExtension(),
                        'created_by' => session('userData.username'),
                        'created_at' => date('Y-m-d H:i:s'),
                    ];
                    $insertedFiles[] = $newFile;
                    $file = new File;
                    $file->pengajuan_pegawai_id = $pegawaiId;
                    $file->jenis = $jenis;
                    $file->url = $filesave;
                    $file->save();
                }
            }
            Files::insert($insertedFiles);
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
            $pegawaiId = $request->input('id');
            if ($request->hasFile('sk_penetapan')) {
                $filepath = 'uploads/pengadaan/pokmil';
                $file = $request->file('sk_penetapan');
                $fileName = $pegawaiId.'_sk_penetapan.'.$file->getClientOriginalExtension();
                $filesave = $filepath.'/'.$fileName;
                $file->move($filepath, $fileName);

                $file = new File;
                $file->pengajuan_pegawai_id = $pegawaiId;
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
        $msAktifitasId = $model->ms_aktifitas_id == 1000 ? 1006 : $model->ms_aktifitas_id;
        $whereAct = ['ms_aktifitas_id' => $msAktifitasId];
        $aktifitasHistories = Aktifitas::getDetail($model->id);
        $aktifitasOptions = Approval::getAktifitas($whereAct);
        $aktifitasOptions = array_map(function ($row) {
            return (object) [
                'id' => $row->id,
                'nama' => ($row->id == 1008 ? 'Pengajuan Terbit SK Penetapan '.strtoupper($this->kategori) : $row->nama),
                'nama_di_pelaksana' => ($row->id == 1008 ? 'Pengajuan Terbit SK Penetapan '.strtoupper($this->kategori) : $row->nama_di_pelaksana),
                'nama_di_validator' => ($row->id == 1008 ? 'Pengajuan Terbit SK Penetapan '.strtoupper($this->kategori) : $row->nama_di_validator),
                'group' => $row->group ?? '',
                'jawaban_dari_aktifitas' => $row->jawaban_dari_aktifitas,
                'urutan' => $row->urutan,
                'keterangan' => $row->keterangan,
                'created_at' => $row->created_at,
                'updated_at' => $row->updated_at,
                'role_id' => $row->role_id,
                'tingkat' => $row->tingkat,
                'next_aktifitas' => $row->next_aktifitas,
                'can_change' => $row->can_change,
                'can_view' => $row->can_view,
            ];
        }, $aktifitasOptions);
        $currentAktifitas = Approval::getCurrentAktifitas($model->ms_aktifitas_id);

        // file
        $msFile = File::getMasterFile($this->kategori);

        $data = [
            'model' => $model,
            'controller' => $this->controller,
            'aktifitasOptions' => $aktifitasOptions,
            'aktifitas' => $currentAktifitas,
            'aktifitasHistories' => $aktifitasHistories,
            'breadcums' => array_merge($this->breadcums, ['Pengajuan']),
            'isNew' => $isNew,
            'kategoriJudul' => $this->kategoriJudul,
            'kategori' => strtoupper($this->kategori),
            'msFile' => $msFile,
            'canCreate' => $this->canCreatePermintaan(),
        ];
        if ($request->wantsJson()) {
            return response()->json($data);
        } else {
            return view('pengadaan.pokja.pengajuanFormSatkerIsiV', $data);
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
        $pegawai = Pegawai::where(['pengajuan_id' => $id])->get();
        if ($pegawai->isEmpty()) {
            return $this->resError('Pegawai harus diisi');
        }
        if ($ms_aktifitas_id == 1009) {
            $filesk = File::getDetail($id, 'sk_penetapan');
            if ($filesk->isEmpty()) {
                return $this->resError('SK Penetapan masing-masing pegawai harus diupload');
            }
            foreach ($filesk as $file) {
                if (empty($file->url)) {
                    return $this->resError('SK Penetapan masing-masing pegawai harus diupload');
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
                'to_satker_induk' => $ms_aktifitas_id == 1000 ? false : true,
            ];
            $acts = Approval::roleCheck($dataAktifitas);
            Aktifitas::insert($acts['act']);

            if (! empty($acts['nextAct'])) {
                $newAct = $acts['nextAct'];
                if (in_array($newAct, [1003, 1005, 1007])) { // revisi
                    $newAct = 1000;
                }
                Model::where(['id' => $id])->update(['ms_aktifitas_id' => $newAct]);
            }
            if ($ms_aktifitas_id == 1008) {
                $targetSatker[] = config('constants.ms_satker_kejagung_id');
                $notifParams = [
                    'url' => 'pengadaan/pokja-pemilihan/'.$id,
                    'judul' => 'Pengajuan Terbit SK Penetapan Pokmil',
                    'isi' => '',
                    'target' => 'role',
                    'targetValue' => config('constants.validator_pusat_role_id'),
                    'targetSatker' => $targetSatker,
                    'targetSatkerPusat' => $targetSatker,
                ];
                Notifikasi::sendNotif($notifParams);
            }
            DB::commit();

            return $this->resSuccess(
                'Berhasil Disimpan!',
                [
                    'type' => 'redirect',
                    'url' => \URL::to('/pengadaan/pokja-pemilihan'),
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
}
