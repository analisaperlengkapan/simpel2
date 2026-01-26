<?php

namespace App\Http\Controllers\AnalisisKebutuhan\PakaianDinas;
use Illuminate\Routing\Controller;

use App\Helpers\MyHelper;
use App\Models\AnalisisKebutuhan\PakaianDinas as Model;
use App\Models\AnalisisKebutuhan\PakaianDinasPakaian;
use App\Models\AnalisisKebutuhan\PakaianDinasSatker;
use App\Models\AnalisisKebutuhan\PakaianDinasSatkerAktivitas;
use App\Models\AnalisisKebutuhan\PakaianDinasSatkerPegawai;
use App\Models\AnalisisKebutuhan\PakaianDinasSatkerPegawaiUkuran;
use App\Models\AnalisisKebutuhan\PakaianDinasSatkerTerpilih;
use App\Models\Persetujuan\Approval;
use App\Models\Master\Master;
use App\Models\Master\PakaianDinas\JenisPakaianDinas;
use App\Models\Master\PakaianDinas\SpesifikasiPakaianDinas;
use App\Models\Master\PakaianDinas\SpesifikasiPakaianDinasFoto;
use App\Models\Master\MsSatker;
use App\Models\Komunikasi\Notifikasi;
use App\Models\AnalisisKebutuhan\PegawaiPakaianDinas;
use Illuminate\Http\Request;
use Illuminate\Support\Arr;
use Illuminate\Support\Facades\DB;
use Illuminate\Support\Facades\URL;
use Symfony\Component\HttpKernel\Exception\NotFoundHttpException;
use Symfony\Component\HttpKernel\Exception\UnauthorizedHttpException;

class PengajuanController extends Controller
{
    protected $breadcums = ['Analisis Kebutuhan', 'Pakaian Dinas', 'Pengajuan'];
    private $controller = '/analisis-kebutuhan/pakaian-dinas/pengajuan';
    public function index()
    {

        $columns = ['Nama', 'Deskripsi', 'Tgl Mulai', 'Tanggal Selesai'];
        $defColumns = [0, 1, 2, 3];
        $data = [
            'tableId' => 'dt-pengajuan',
            'breadcums' => $this->breadcums,
            'controller' => $this->controller,
            'operasi' => $this->userOperation(),
            'columns' => $columns,
            'defColumns' => $defColumns
        ];
        return view('analisis_kebutuhan.pakaian_dinas.pengajuanV', $data);
    }

    protected function userOperation()
    {
        if (MyHelper::isSuperAdmin() || MyHelper::isAdmin()) {
            return 'VIEW';
        }

        if (MyHelper::isValidatorPusat()) {
            return 'CREATE';
        }

        if (MyHelper::isValidatorWilayah()) {
            return 'VIEW';
        }

        if (MyHelper::isPelaksanaSatker()) {
            return 'INPUT';
        }
    }

    public function gridData(Request $request)
    {
        $user = new Model();
        $pagingParams = $request->only(['start', 'length']);
        $searchParams = $request->only(['columns']);
        $filter['is_reguler'] = 1;
        if (MyHelper::isValidatorPusat()) {
            $filter = [];
        }

        $data = $user->getGridData($pagingParams, $searchParams, $filter);
        return response()->json([
            'data' => $data['data'],
            'recordsTotal' => $data['total'],
            'recordsFiltered' => $data['total'],
        ]);
    }
    // http://localhost:8000/analisis-kebutuhan/pakaian-dinas/pengajuan/20240109116/list-satker?id_wilayah=00

    function getData($id = null)
    {
        $model = [];
        $isNew = true;
        $breadcum = 'Tambah';
        $selectedSatker = [];
        $specs = new SpesifikasiPakaianDinas();
        $selectedSpecs = [];

        if ($id) {
            $breadcum = 'Ubah';
            $model = Model::where('id', $id)->first();
            if (!$model) {
                throw new NotFoundHttpException('Data Tidak Ditemukan');
            }
            $satkerTerpilih = PakaianDinasSatkerTerpilih::where(['pengajuan_pakaian_dinas_id' => $id, 'is_show_in_form' => 1])->get()->toArray();
            $pakaianTerpilih = PakaianDinasPakaian::where(['pengajuan_pakaian_dinas_id' => $id])->get();
            $selectedSpecs = Arr::pluck($pakaianTerpilih, 'spesifikasi_id');
            if (!empty($satkerTerpilih)) {
                $selectedSatker = Arr::pluck($satkerTerpilih, 'ms_satker_id');
            }
            $model = $model->toArray();
            $isNew = false;
        }
        $satkerTerpilih = Master::getSatkers();
        $pakaianSpecs = $specs->getDataGrid(['length' => null, 'start' => 0])['data'];
        $specOptions = [];

        foreach ($pakaianSpecs as $spec) {
            $specOptions[] = (object) [
                'text' => "{$spec->nama} - {$spec->gender}",
                'value' => $spec->id,
                'ms_jenis_pakaian_id' => $spec->ms_jenis_pakaian_dinas_id,
                'checked' => in_array($spec->id, $selectedSpecs)
            ];
        }
        $tahuns = MyHelper::generateTahun();

        $data = [
            'model' => $model,
            'isNew' => $isNew,
            'controller' => $this->controller,
            'pakaians' => JenisPakaianDinas::all()->toArray(),
            'specOptions' => $specOptions,
            'satkerOptions' => MyHelper::generateSelectOptions(['data' => $satkerTerpilih, 'text' => 'inst_nama', 'value' => 'inst_satkerkd', 'selected' => $selectedSatker]),
            'breadcums' => array_merge($this->breadcums, [$breadcum]),
            'tahuns' => MyHelper::generateSelectOptions(['data' => $tahuns, 'selected' => $model['tahun'] ?? date('Y')]),
            'jenisPakaianOptions' => MyHelper::generateSelectOptions([
                'data' => JenisPakaianDinas::all(),
                'text' => 'nama',
                'value' => 'id',
                'selected' => $model['ms_jenis_pakaian_dinas_id'] ?? null
            ]),
        ];
        return $data;
    }

    /**
     * Show the form for creating a new resource.
     */
    public function create()
    {

        if (!$this->userOperation() == 'CREATE') {
            throw new UnauthorizedHttpException('Tidak Punya Akses');
        }
        $data = $this->getData();
        return view('analisis_kebutuhan.pakaian_dinas.pengajuanFormV', $data);
    }

    /**
     * Store a newly created resource in storage.
     */
    public function store(Request $request)
    {
        $validasi = [
            'nama' => 'required',
            'pilihan_satker' => 'required',
            'spesifikasi_id' => 'required',
        ];
        $isReguler = $request->has('is_reguler') ? false : true;
        if ($isReguler) {
            $validasi['tgl_mulai'] = 'required';
            $validasi['tgl_selesai'] = 'required';
        }

        $request->validate($validasi);
        try {

            $inputan = $request->input();

            $pakaians = SpesifikasiPakaianDinas::getComplete($inputan['spesifikasi_id']);
            DB::beginTransaction();
            $id = $request->input('id');
            if ($request->has('id')) {
                $isNew = false;
                PakaianDinasSatkerTerpilih::where(['pengajuan_pakaian_dinas_id' => $id])->delete();
            } else {
                $isNew = true;
                $inputan['created_by'] = session('userData.username');
                $id = MyHelper::getPk(date('Ymd'), 'pengajuan_pakaian_dinas_seq');
                $inputan['ms_aktifitas_id'] = 1000;
            }

            $satkerQ = DB::table('ms_satker')->select('inst_satkerkd', 'is_pusat')->orderBy('inst_satkerkd');

            if ($request->input('pilihan_satker') == 'sebagian') {
                $satkerQ->where(function ($q) use ($inputan) {
                    foreach ($inputan['satkers'] as $kdSatker) {
                        if ($kdSatker == '00') {
                            $q->orWhere('inst_satkerkd', '00')->orWhere('is_pusat', 1);
                        } else {
                            $q->orWhere('inst_satkerkd', "like", "{$kdSatker}%");
                        }
                    }
                });
            }
            // if ($request->input('pilihan_satker') == 'all') {
            // }

            $satkers = $satkerQ->get();
            // dd($satkers);
            $targetSatker = [];
            $targetSatkerPusat = [];
            Model::updateOrCreate(['id' => $id], $inputan);
            foreach ($satkers as $value) {
                $currentSatker = [
                    'ms_satker_id' => $value->inst_satkerkd,
                    'pengajuan_pakaian_dinas_id' => $id,
                    'ms_satker_pusat_id' => null,
                    'is_show_in_form' => in_array($value->inst_satkerkd, $request->input('satkers', [])) ? 1 : 0
                ];
                if ($isNew) {
                    $satkerId = $value->inst_satkerkd;
                    $pusatId = null;
                    $targetSatker[] = $satkerId;
                    if ($value->is_pusat == 1) {
                        $pusatId = $satkerId;
                        $satkerId = '00';
                        $currentSatker['ms_satker_id'] = '00';
                        $currentSatker['ms_satker_pusat_id'] = $pusatId;
                        $targetSatkerPusat[] = $pusatId;
                    }

                    if (!$isReguler) {
                        $pakainDinasSatkerData[] = [
                            'pengajuan_pakaian_dinas_id' => $id,
                            'ms_aktifitas_id' => 1008,
                            //selesai
                            'ms_satker_id' => $value->inst_satkerkd,
                            'ms_satker_pusat_id' => $satkerId
                        ];
                    }
                }
                $satkerTerpilih[] = $currentSatker;
            }

            PakaianDinasSatkerTerpilih::insert($satkerTerpilih);

            foreach ($pakaians as $key => $pakaian) {
                $checkedPakaians[] = [
                    'pengajuan_pakaian_dinas_id' => $id,
                    'jenis_pakaian_id' => $pakaian->ms_jenis_pakaian_dinas_id,
                    'jenis_pakaian_nama' => $pakaian->jenis,
                    'spesifikasi_id' => $pakaian->id,
                    'spesifikasi_nama' => $pakaian->nama,
                    'spesifikasi_ukuran_group' => $pakaian->ms_ukuran_group,
                    'subspesifikasi_gender' => $pakaian->gender,
                ];
            }

            if ($isNew && $isReguler) {
                $notifParams = [
                    'url' => "analisis-kebutuhan/pakaian-dinas/pengajuan/{$id}/edit",
                    'judul' => 'Kebutuhan Pakaian Dinas Baru',
                    'isi' => '',
                    'target' => 'role',
                    'targetValue' => config('constants.pelaksana_satker_role_id'),
                    'targetSatker' => $targetSatker,
                    'targetSatkerPusat' => $targetSatkerPusat,
                ];
                Notifikasi::sendNotif($notifParams);
            }
            if ($isNew) {
                PakaianDinasPakaian::insert($checkedPakaians);
            }
            // PakaianDinasPakaian::where(['pengajuan_pakaian_dinas_id' => $id])->delete();

            if ($isNew && !$isReguler) {
                //create pakaian_dinas_satker
                // PakaianDinasSatker::insert($pakainDinasSatkerData);
                //create pakaian_dinas_satker_pegawai
                //ambil nip dari mw_curr sesuai satker join ke pegawai_pakain_dinas
                //looping pakaian, ambil ukuranya sesuai sama ukuran_group
                //create pakaian_dinas_satker_pegawai_ukuran
            }
            DB::commit();
            return $this->resSuccess('Berhasil Dismpan');
        } catch (\Throwable $th) {
            dd($th->getMessage());
            return $this->resError('Gagal menyimpan data');
        }
    }

    /**
     * edit admin perlengkapan
     */
    public function show(string $id)
    {
        if (!$this->userOperation() == 'CREATE') {
            throw new UnauthorizedHttpException('Tidak Punya Akses');
        }
        $data = $this->getData($id);
        return view('analisis_kebutuhan.pakaian_dinas.pengajuanFormV', $data);
    }

    /**
     * pelaksana ngisi /  validator ngeliat
     */
    public function edit(string $id)
    {
        $data = $this->getData($id);
        $pengajuan = Model::where('id', $id)->first();
        if (!$pengajuan) {
            throw new NotFoundHttpException('Data Tidak Ditemukan');
        }

        $currentRole = session('userData.current_role');

        $selectedSatker = $_GET['satker'] ?? $currentRole['ms_satker_id'];
        if (MyHelper::isPelaksanaSatker() && $currentRole['ms_satker_id'] == '00') {
            $isPusatSelected = $_GET['isPusat'] ?? 0;
            //$_GET['satker'] cuma ada klo diliat validator pusat /kejati
            $whereData = ['ms_satker_id' => '00', 'ms_satker_pusat_id' => $currentRole['ms_satker_pusat_id'], 'pengajuan_pakaian_dinas_id' => $pengajuan->id];
            $whereSatker = ['inst_satkerkd' => $selectedSatker];
        } else {
            $selectedSatker = isset($_GET['satker']) ? $_GET['satker'] : $currentRole['ms_satker_id'];
            $isPusatSelected = $_GET['isPusat'] ?? 0;
            //$_GET['satker'] cuma ada klo diliat validator pusat /kejati
            $whereData = ['ms_satker_id' => $selectedSatker, 'pengajuan_pakaian_dinas_id' => $pengajuan->id];
            $whereSatker = ['inst_satkerkd' => $selectedSatker];
        }
        if ($isPusatSelected == 1) {
            $whereData['ms_satker_id'] = '00';
            $whereData['ms_satker_pusat_id'] = $selectedSatker;
        }
        $satkerInfo = MsSatker::where(['inst_satkerkd' => $whereData['ms_satker_id'], 'is_pusat' => $isPusatSelected])->first();
        $model = PakaianDinasSatker::where($whereData)->first() ?? [];
        $pakaians = PakaianDinasPakaian::where(['pengajuan_pakaian_dinas_id' => $pengajuan->id])->get();
        $msPakaianIds = Arr::pluck($pakaians, 'spesifikasi_id');
        $fotoPakaians = SpesifikasiPakaianDinasFoto::whereIn('ms_spesifikasi_pakaian_dinas_id', $msPakaianIds)->get();
        // $fotos = [];
        // foreach ($fotoPakaians as $key => $foto) {
        //     $fotos[$foto->ms_spesifikasi_pakaian_dinas_id][] = $foto;
        // }
        $ukurans = Master::getMsUkuranGroupMapped(Arr::pluck($pakaians, 'spesifikasi_ukuran_group'));

        $mappedUkurans = [];
        $isNew = empty($model) ? true : false;
        if ($isNew) {
            if ($selectedSatker == '00') {
                $whereSatker['mapped_unit_kerja'] = $currentRole['ms_satker_pusat_id'];
            }
            $pegawais = PakaianDinasSatkerPegawai::getExistingPakaianDinas($whereSatker);

            $tingkatSatker = MyHelper::getSatkerLevel($currentRole['ms_satker_id']);
            if ($tingkatSatker == 'KEJAGUNG') {
                $starterAct = 1009;
                // } elseif ($tingkatSatker == 'KEJATI') {
                //     $starterAct = 1011;
            } else {
                $starterAct = 1011;
            }
            $whereAct = ['ms_aktifitas_id' => $starterAct];
            $aktifitasHistories = [];
        } else {
            $msAktivitasId = $model->ms_aktifitas_id;
            $whereAct = ['ms_aktifitas_id' => $msAktivitasId];
            $pegawais = PakaianDinasSatkerPegawai::getDetail($model->id);
            $ukuranPegawais = PakaianDinasSatkerPegawaiUkuran::where(['pengajuan_pakaian_dinas_satker_id' => $model->id])->get();
            foreach ($ukuranPegawais as $ukuran) {
                $mappedUkurans[$ukuran->pengajuan_pakaian_dinas_satker_pegawai_id][$ukuran->pengajuan_pakaian_dinas_pakaian_id] = $ukuran->ukuran;
            }

            $aktifitasHistories = PakaianDinasSatkerAktivitas::getDetail($model->id);
        }
        $aktifitasOptions = [];
        $currentAktivitas = [];
        $pengajuanExpired = false;
        if (strtotime(date('Y-m-d')) > strtotime($pengajuan->tgl_selesai)) {
            $currentAktivitas->canChange = false;
            $pengajuanExpired = true;
        }
        $data = [
            'whereData' => $whereData,
            'isPengajuanExpired' => $pengajuanExpired,
            'satkerInfo' => $satkerInfo,
            'pengajuan' => $pengajuan,
            'pegawais' => $pegawais,
            'fotos' => $fotoPakaians,
            'mappedUkurans' => $mappedUkurans,
            'model' => $model,
            'controller' => $this->controller,
            'pakaians' => $pakaians,
            'ukurans' => $ukurans,
            'aktivitasOptions' => $aktifitasOptions,
            'aktivitas' => $currentAktivitas,
            'aktivitasHistories' => $aktifitasHistories,
            'breadcums' => array_merge($this->breadcums, ['Pengisian']),
            'isNew' => $isNew
        ];
        return view('analisis_kebutuhan.pakaian_dinas.pengajuanFormSatkerIsiV', $data);
    }

    public function savePengajuan(Request $request)
    {
        $validasi = [
            'pegawais' => 'required|array|min:1',
            'ms_aktifitas_id' => 'required',
            'pengajuanId' => 'required',
        ];
        $request->validate($validasi);
        $pengajuanId = $request->input('pengajuanId');
        $ada = PakaianDinasSatker::where(['ms_satker_id' => $request->input('ms_satker_id'), 'pengajuan_pakaian_dinas_id' => $request->input('pengajuanId')])->first();
        if ($request->has('id')) {
            $isNew = false;
            $id = $request->input('id');
        } else {
            $isNew = true;
            $id = MyHelper::getPk(date('Ymd'), 'pengajuan_pakaian_dinas_satker_seq');
        }

        if ($isNew && $ada) {
            $isNew = false;
            $id = $ada->id;
        }

        try {
            DB::beginTransaction();
            $currentRole = session('userData.current_role');
            if ($isNew) {
                PakaianDinasSatker::create([
                    'id' => $id,
                    'pengajuan_pakaian_dinas_id' => $pengajuanId,
                    'ms_satker_id' => $currentRole['ms_satker_id'],
                    'ms_satker_pusat_id' => $currentRole['ms_satker_pusat_id'],
                    'ms_aktifitas_id' => $request->input('ms_aktifitas_id'),
                ]);
            } else {
                PakaianDinasSatker::where(['id' => $id])->update(['ms_aktifitas_id' => $request->input('ms_aktifitas_id')]);
            }


            PakaianDinasSatkerPegawai::where(['pengajuan_pakaian_dinas_satker_id' => $id])->delete();
            PakaianDinasSatkerPegawaiUkuran::where(['pengajuan_pakaian_dinas_satker_id' => $id])->delete();
            foreach ($request->input('pegawais') as $value) {
                $value['pengajuan_pakaian_dinas_satker_id'] = $id;
                $value['id'] = MyHelper::getPk(date('Ymd'), 'pengajuan_pakaian_dinas_satker_pegawai_id_seq');

                foreach ($value['details'] as $pakaianId => $ukuran) {
                    $pegawaiDetails[] = [
                        'pengajuan_pakaian_dinas_pakaian_id' => $pakaianId,
                        'pengajuan_pakaian_dinas_satker_id' => $id,
                        'pengajuan_pakaian_dinas_satker_pegawai_id' => $value['id'],
                        'ukuran' => $ukuran,
                    ];
                }
                unset($value['details']);
                $pegawais[] = $value;
            }
            PakaianDinasSatkerPegawai::insert($pegawais);
            PakaianDinasSatkerPegawaiUkuran::insert($pegawaiDetails);


            $dataAktivitas = [
                'idKey' => 'pengajuan_pakaian_dinas_satker_id',
                'idValue' => $id,
                'ms_aktifitas_id' => $request->input('ms_aktifitas_id'),
                'komentar' => $request->input('komentar'),
                'to_satker_induk' => $request->input('ms_aktifitas_id') == 1000 ? false : true
            ];
            $acts = ["act" => [], "nextAct" => null];
            PakaianDinasSatkerAktivitas::insert($acts['act']);
            // dd($acts);
            $this->sendNotif($pengajuanId, $acts['msAct'], $id);
            if (!empty($acts['nextAct'])) {
                $newAct = $acts['nextAct'];
                if (in_array($acts['msAct']->id, [1003, 1005, 1007])) { //revisi
                    $model = PakaianDinasSatker::where(['id' => $id])->first();
                    $satkerLv = MyHelper::getSatkerLevel($model->ms_satker_id);
                    //balikin ke awal approval
                    if ($satkerLv == 'KEJAGUNG') {
                        $newAct = 1009;
                    } else {
                        $newAct = 1011;
                    }
                }

                if ($newAct == 1008) { //selesai
                    $pegawais = PakaianDinasSatkerPegawai::where('pengajuan_pakaian_dinas_satker_id', $id)->get();
                    $ukuranPegawais = DB::select("SELECT a.*
                    , c.spesifikasi_ukuran_group
                    from pengajuan_pakaian_dinas_satker_pegawai_ukuran a
                    join pengajuan_pakaian_dinas_pakaian c on a.pengajuan_pakaian_dinas_pakaian_id = c.id
                    where pengajuan_pakaian_dinas_satker_id = ? ;
                    ", [$id]);
                    $mappedUkurans = [];
                    foreach ($ukuranPegawais as $ukuran) {
                        $mappedUkurans[$ukuran->pengajuan_pakaian_dinas_satker_pegawai_id][$ukuran->spesifikasi_ukuran_group] = $ukuran->ukuran;
                    }
                    // dd($mappedUkurans);

                    foreach ($pegawais as $key => $pegawai) {
                        $nips[] = $pegawai->nip;
                        $basic = [
                            'nip' => $pegawai->nip,
                            'with_hijab' => $pegawai->with_hijab,
                            'pangkat' => $pegawai->pangkat,
                            'jabatan' => $pegawai->jabatan,
                            'last_pengajuan_pakaian_dinas_satker_pegawai_id' => $pegawai->id
                        ];

                        if (isset($mappedUkurans[$pegawai->id]['BAJU'])) {
                            $basic['ukuran_baju'] = $mappedUkurans[$pegawai->id]['BAJU'];
                        } else {
                            $basic['ukuran_baju'] = null;
                        }
                        if (isset($mappedUkurans[$pegawai->id]['CELANA'])) {
                            $basic['ukuran_celana'] = $mappedUkurans[$pegawai->id]['CELANA'] ?? null;
                        } else {
                            $basic['ukuran_celana'] = null;
                        }
                        if (isset($mappedUkurans[$pegawai->id]['SEPATU'])) {
                            $basic['ukuran_sepatu'] = $mappedUkurans[$pegawai->id]['SEPATU'] ?? null;
                        } else {
                            $basic['ukuran_sepatu'] = null;
                        }

                        $data[] = $basic;
                    }
                    PegawaiPakaianDinas::whereIn('nip', $nips)->delete();
                    DB::table('pegawai_pakaian_dinas')->insert($data);
                }
                PakaianDinasSatker::where(['id' => $id])->update(['ms_aktifitas_id' => $newAct]);
            }

            DB::commit();
            return $this->resSuccess(
                'Berhasil Disimpan!',
                [
                    'type' => 'redirect',
                    'url' => URL::to('/analisis-kebutuhan/pakaian-dinas/pengajuan')
                ]
            );
        } catch (\Throwable $th) {
            DB::rollBack();
            dd($th);
            return $this->resError('Gagal Menyimpan data');
        }
    }
    public function listSatker(int $pengajuanId)
    {
        $pengajuan = Model::where('id', $pengajuanId)->first();
        if (!$pengajuan) {
            throw new NotFoundHttpException('Data Tidak Ditemukan');
        }

        $idWilayah = request('id_wilayah', null);
        $idSatker = request('id_satker', null);
        $data = [
            'pengajuan' => $pengajuan,
            'tableId' => 'dt-satker',
            'tableIdPusat' => 'dt-satker-pusat',
            'controller' => $this->controller,
            'breadcums' => array_merge($this->breadcums, ['Lihat Satker']),

        ];

        if (MyHelper::isSuperAdmin()) {
            return $this->listSatkerWilayah($data);
        }
        if (MyHelper::isValidatorPusat() && empty($idWilayah)) {
            return $this->lisSatkerPusat($data);
        } elseif (MyHelper::isValidatorWilayah() || $idWilayah) {
            return $this->listSatkerWilayah($data);
        } else {
            throw new UnauthorizedHttpException('', 'Tidak ada Akses');
        }
    }

    function lisSatkerPusat($data)
    {

        return view('analisis_kebutuhan.pakaian_dinas.pengajuanSatkerListPusatV', $data);
    }

    function listSatkerWilayah($data)
    {

        $enableCheckOn = [];
        if (MyHelper::isValidatorPusat()) {
            $data['approveAct'] = 1008;
            $data['rejectAct'] = 1007;
            $enableCheckOn = [1004, 1010];
        }

        if (MyHelper::isValidatorWilayah() || MyHelper::isSuperAdmin()) {
            $data['approveAct'] = 1010;
            $data['rejectAct'] = 1005;
            $enableCheckOn = [1001, 1012];
        }
        $data['enableCheckOn'] = $enableCheckOn;
        return view('analisis_kebutuhan.pakaian_dinas.pengajuanSatkerListV', $data);
    }

    public function gridDataSatker(Request $request)
    {
        $user = new Model();
        $pagingParams = $request->only(['start', 'length']);
        $searchParams = $request->only(['search', 'filterBy']);

        $idWilayah = request('id_wilayah', null);
        $idSatker = request('id_satker', null);

        $filters = [];
        $isPusat = false;
        if (MyHelper::isValidatorPusat() || MyHelper::isSuperAdmin()) {
            $newF = ['ms_satker.inst_satkerinduk', '=', '00'];
            if ($idWilayah) {
                if ($idWilayah == '00') {
                    $isPusat = true;
                    $newF = ['ms_satker.is_pusat', '=', '1'];
                } else {
                    $newF = ['ms_satker.inst_satkerkd', 'like', "{$idWilayah}%"];
                }
            }
        }

        if (MyHelper::isValidatorWilayah()) {
            $roleSatker = session('userData.current_role.ms_satker_id');
            $newF = ['ms_satker.inst_satkerkd', 'like', "{$roleSatker}%"];
        }

        array_push($filters, $newF);

        $data = $user->getGridDataSatker($pagingParams, $searchParams, $request->input('pengajuanId'), $filters, $isPusat);
        return response()->json([
            'data' => $data['data'],
            'recordsTotal' => $data['total'],
            'recordsFiltered' => $data['total'],
        ]);
    }

    public function gridDataSatkerPusat(Request $request)
    {
        $user = new Model();
        $pagingParams = $request->only(['start', 'length']);
        $searchParams = $request->only(['search', 'filterBy', 'unitKerja']);
        $data = $user->getGridDataSatkerPusat($pagingParams, $searchParams, $request->input('pengajuanId'));
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

    public function validatorAction(Request $request)
    {
        $ms_aktifitas_id = $request->input('ms_aktifitas_id');

        if ($ms_aktifitas_id == 1008) {
            return $this->setSelesai($request);
        } else if (in_array($ms_aktifitas_id, [1005, 1007])) {
            return $this->setRevisi($request);
        } else if ($ms_aktifitas_id == 1010) {
            return $this->setNext($request);
        }
    }

    function setNext(Request $request)
    {
        $pengajuanId = $request->input('pengajuanId');
        $ms_aktifitas_id = $request->input('ms_aktifitas_id');
        $pengajuanSatkerIds = $request->input('pengajuanSatkerId');
        try {
            DB::beginTransaction();
            foreach ($pengajuanSatkerIds as $key => $id) {
                $dataAktivitas = [
                    'idKey' => 'pengajuan_pakaian_dinas_satker_id',
                    'idValue' => $id,
                    'ms_aktifitas_id' => $ms_aktifitas_id,
                    'komentar' => "OK",
                    'to_satker_induk' => false,
                ];
                $acts = ["act" => [], "nextAct" => null];
                PakaianDinasSatkerAktivitas::insert($acts['act']);
                PakaianDinasSatker::where(['id' => $id])->update(['ms_aktifitas_id' => $ms_aktifitas_id]);
            }

            DB::commit();
            return $this->resSuccess();
        } catch (\Throwable $th) {
            DB::rollBack();
            dd($th);
            return $this->resError();
        }
    }

    function setSelesai(Request $request)
    {
        $ms_aktifitas_id = $request->input('ms_aktifitas_id');
        $pengajuanId = $request->input('pengajuanId');
        $pengajuanSatkerIds = $request->input('pengajuanSatkerId');
        $pakaians = PakaianDinasPakaian::where('pengajuan_pakaian_dinas_id', $pengajuanId)->get();
        try {
            DB::beginTransaction();
            foreach ($pengajuanSatkerIds as $key => $id) {
                $dataAktivitas = [
                    'idKey' => 'pengajuan_pakaian_dinas_satker_id',
                    'idValue' => $id,
                    'ms_aktifitas_id' => $ms_aktifitas_id,
                    'komentar' => "Selesai",
                    'to_satker_induk' => false,
                ];
                $acts = ["act" => [], "nextAct" => null];
                PakaianDinasSatkerAktivitas::insert($acts['act']);
                PakaianDinasSatker::where(['id' => $id])->update(['ms_aktifitas_id' => $ms_aktifitas_id]);
                //selesai


            }
            $pegawais = PakaianDinasSatkerPegawai::whereIn('pengajuan_pakaian_dinas_satker_id', $pengajuanSatkerIds)->get();
            $ukuranPegawais = DB::table('pengajuan_pakaian_dinas_satker_pegawai_ukuran as a')
                ->select(['a.*', 'c.spesifikasi_ukuran_group'])
                ->join('pengajuan_pakaian_dinas_pakaian as c', 'a.pengajuan_pakaian_dinas_pakaian_id', '=', 'c.id')
                ->whereIn('pengajuan_pakaian_dinas_satker_id', $pengajuanSatkerIds)->get();
            $mappedUkurans = [];
            foreach ($ukuranPegawais as $ukuran) {
                $mappedUkurans[$ukuran->pengajuan_pakaian_dinas_satker_pegawai_id][$ukuran->spesifikasi_ukuran_group] = $ukuran->ukuran;
            }
            // dd($mappedUkurans);
            foreach ($pegawais as $key => $pegawai) {
                $nips[] = $pegawai->nip;
                $basic = [
                    'nip' => $pegawai->nip,
                    'with_hijab' => $pegawai->with_hijab,
                    'pangkat' => $pegawai->pangkat,
                    'jabatan' => $pegawai->jabatan,
                    'gol_kd' => $pegawai->gol_kd,
                    'last_pengajuan_pakaian_dinas_satker_pegawai_id' => $pegawai->id
                ];
                foreach ($pakaians as $key => $pakaian) {
                    $ukuranGroup = $pakaian->spesifikasi_ukuran_group;
                    $groupName = strtolower($ukuranGroup);
                    $key = "ukuran_{$groupName}";
                    $basic[$key] = $mappedUkurans[$pegawai->id][$ukuranGroup];
                }
                $data[] = $basic;
            }
            PegawaiPakaianDinas::whereIn('nip', $nips)->delete();
            DB::table('pegawai_pakaian_dinas')->insert($data);
            DB::commit();
            return $this->resSuccess();
        } catch (\Throwable $th) {
            DB::rollBack();
            dd($th);
            return $this->resError();
        }
    }

    function setRevisi(Request $request)
    {
        $ms_aktifitas_id = $request->input('ms_aktifitas_id');
        $pengajuanId = $request->input('pengajuanId');
        $pengajuanSatkerIds = $request->input('pengajuanSatkerId');

        $satkers = PakaianDinasSatker::whereIn('id', $pengajuanSatkerIds)->get();
        try {
            DB::beginTransaction();
            foreach ($satkers as $key => $model) {
                # code...
                $satkerLv = MyHelper::getSatkerLevel($model->ms_satker_id);
                //balikin ke awal approval
                if ($satkerLv == 'KEJAGUNG') {
                    $newAct = 1009;
                } else if ($satkerLv == 'KEJAGUNG') {
                    $newAct = 1011;
                } else {
                    $newAct = 1000;
                }

                $dataAktivitas = [
                    'idKey' => 'pengajuan_pakaian_dinas_satker_id',
                    'idValue' => $model->id,
                    'ms_aktifitas_id' => $ms_aktifitas_id,
                    'komentar' => "Revisi",
                    'to_satker_induk' => false,
                ];
                $acts = ["act" => [], "nextAct" => null];
                PakaianDinasSatkerAktivitas::insert($acts['act']);
                PakaianDinasSatker::where(['id' => $model->id])->update(['ms_aktifitas_id' => $newAct]);
            }
            DB::commit();
            return $this->resSuccess();
        } catch (\Throwable $th) {
            DB::rollBack();
            dd($th);
            return $this->resError();
        }
    }
    function sendNotif($pengajuanId, $msAct, $pengajuanSatkerId)
    {
        $satkerId = session('userData.current_role.ms_satker_id');
        $satkerParent = explode('.', $satkerId)[0];
        $baseUrl = "analisis-kebutuhan/pakaian-dinas/pengajuan/{$pengajuanId}/list-satker";
        $urlKePusat = "{$baseUrl}?id_wilayah={$satkerParent}";
        $notifParams = [
            'url' => $baseUrl,
            'target' => 'role',
            'isi' => '',
        ];
        $sendNotif = false;
        if (MyHelper::isPelaksanaSatker()) {
            $sendNotif = true;
            if ($msAct->tingkat == 'KEJAGUNG') {
                $notifParams['judul'] = 'Verifikasi Pakaian Dinas Ke Validator Pusat';
                $notifParams['url'] = $urlKePusat;
                $notifParams['targetValue'] = config('constants.validator_pusat_role_id');
            } else {
                $targetSatker = $msAct->tingkat == 'KEJATI' ? $satkerId : $msAct->nextSatker;
                $notifParams['judul'] = 'Verifikasi Pakaian Dinas Ke Validator Wilayah';
                $notifParams['targetValue'] = config('constants.validator_wilayah_role_id');
                $notifParams['targetSatker'] = [$targetSatker];
            }
        }

        if (MyHelper::isValidatorWilayah()) {
            $sendNotif = true;
            $notifParams['url'] = $urlKePusat;
            $notifParams['judul'] = 'Verifikasi Pakaian Dinas Ke Validator Pusat';
            $notifParams['targetValue'] = config('constants.validator_pusat_role_id');

            if ($msAct->id == 1003) { //kirim ke pelaksana satker saat revisi
                $pengajuanSatker = PakaianDinasSatker::where(['id' => $pengajuanSatkerId])->first();
                $targetSatker = $pengajuanSatker['ms_satker_id'] == '00' ? $pengajuanSatker['ms_satker_pusat_id'] : $pengajuanSatker['ms_satker_id'];
                $notifParams['url'] = "analisis-kebutuhan/pakaian-dinas/pengajuan/{$pengajuanId}/edit";
                $notifParams['judul'] = 'Revisi Pengajuan Pakaian Dinas';
                $notifParams['targetValue'] = config('constants.pelaksana_satker_role_id');
                $notifParams['targetSatker'] = [$targetSatker];
            }
        }
        if ($sendNotif) {
            Notifikasi::sendNotif($notifParams);
        }
    }
}
