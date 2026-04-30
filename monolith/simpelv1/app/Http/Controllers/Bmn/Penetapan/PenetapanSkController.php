<?php

namespace App\Http\Controllers\Bmn\Penetapan;

use App\Helpers\MyHelper;
use App\Http\Controllers\Controller;
use App\Models\Approval;
use App\Models\Bmn\PenetapanSk;
use App\Models\Bmn\PenetapanSkAktifitas;
use App\Models\Bmn\PenetapanSkAsset;
use App\Models\Files;
use App\Models\Master;
use App\Models\Notifikasi;
use Illuminate\Http\Request;
use Illuminate\Support\Arr;
use Illuminate\Support\Facades\DB;
use Symfony\Component\HttpKernel\Exception\NotFoundHttpException;

class PenetapanSkController extends Controller
{
    /**
     * Display a listing of the resource.
     */
    protected $breadcums = ['BMN', 'Pengajuan SK Penetapan Status Penggunaan'];
    protected $controller = '/bmn/penetapan/penetapansk';
    public function index()
    {

        $columns = ['Nama Satker', 'No Surat Pernyataan', 'Tgl Surat Pernyataan', 'File Lampiran', 'Status', 'No SK PSP', 'Tgl SK PSP', 'File PSP'];
        $defColumns = [0, 1, 2, 3, 4, 5];
        $model = new PenetapanSk();
        $data = [
            'tableId' => 'dt-penetapan',
            'breadcums' => $this->breadcums,
            'controller' => $this->controller,
            'canChange' => $this->isPelaksanaSatker(),
            'columns' => $columns,
            'defColumns' => $defColumns
        ];
        return view('bmn.penetapansk.penetapanV', $data);
    }

    function isPelaksanaSatker()
    {
        $currentRole = session('userData.current_role');
        return $currentRole['ms_role_id'] == config('constants.pelaksana_satker_role_id') ? 1 : 0;
    }

    public function gridData(Request $request)
    {
        $model = new PenetapanSk();
        $pagingParams = $request->only(['start', 'length']);
        $searchParams = $request->only(['columns']);
        $data = $model->getDataGrid($pagingParams, $searchParams);
        return response()->json([
            'data' => $data['data'],
            'recordsTotal' => $data['total'],
            'recordsFiltered' => $data['total'],
        ]);
    }

    public function gridDataAset(Request $request)
    {
        $model = new PenetapanSk();
        $pagingParams = $request->only(['start', 'length']);
        $searchParams = $request->only(['columns', 'has_psp']);
        $data = $model->getDataGridAset($pagingParams, $searchParams);
        return response()->json([
            'data' => $data['data'],
            'recordsTotal' => $data['total'],
            'recordsFiltered' => $data['total'],
        ]);
    }

    function getData($id = null, $readOnly = false)
    {
        $model = [];
        $isNew = true;
        $aktifitasHistories = [];
        $breadcum = 'Tambah Pengajuan';
        if ($id) {
            $breadcum = 'Ubah Pengajuan';
            $model = PenetapanSk::where('id', $id)->first();
            if (!$model) {
                throw new NotFoundHttpException('Data Tidak Ditemukan');
            }

            $model = $model->toArray();
            $aktifitasHistories = PenetapanSkAktifitas::getDetail($model['id']);
            $savedAssets = PenetapanSk::getPengajuanAset($model['id']);
            $isNew = false;
        }
        if ($readOnly) {
            $breadcum = 'Detail';
        }

        $aktifitasOptions = Approval::getAktifitas(['ms_aktifitas_id' => $isNew ? 2000 : $model['ms_aktifitas_id']]);
        $currentAktifitas = Approval::getCurrentAktifitas($isNew ? 2000 : $model['ms_aktifitas_id']);
        $satkers = Master::getSatkersKeu();
        $assets = Master::getJenisAsset();
        $jenis = ['UAKPB', 'UAPPB-W', 'UAPPB-E1', 'UAPB'];
        $inputedAssetIds =  Arr::pluck($savedAssets, 'id');
        $data = [
            'model' => $model,
            'isNew' => $isNew,
            'assetOptions' => MyHelper::generateSelectOptions([
                'data' => $assets,
                'text' => 'name',
                'value' => 'nm_table',
            ]),
            'assets' => $savedAssets ?? [],
            'assetSelections' => PenetapanSk::getAset(['pspStatus' => 'BELUM', 'whereNotIn' => $inputedAssetIds]),
            'controller' => $this->controller,
            'breadcums' => array_merge($this->breadcums, [$breadcum]),
            'judul' => $breadcum,
            'readOnly' => $readOnly,
            'aktifitasOptions' => $aktifitasOptions,
            'aktifitas' => $currentAktifitas,
            'aktifitasHistories' => $aktifitasHistories,
            'isPelaksanaSatker' => $this->isPelaksanaSatker(),
            'satkerOptions' => MyHelper::generateSelectOptions([
                'data' => $satkers,
                'text' => 'inst_nama',
                'textkode' => 'kdsatker_keu',
                'value' => 'kdsatker_keu',
                'selected' => $model['kdsatker_keu'] ?? null,
            ]),
            'jenisOptions' => MyHelper::generateSelectOptions([
                'data' => $jenis,
                'text' => 'jenis',
                'value' => null,
                'selected' => $model['jenis_sk'] ?? null,
            ]),
        ];
        return $data;
    }

    /**
     * Show the form for creating a new resource.
     */
    public function create()
    {
        $data = $this->getData();
        return view('bmn.penetapansk.penetapanFormV', $data);
    }

    /**
     * Store a newly created resource in storage.
     */

    public function validatorAction(Request $request)
    {
        $customMessages = [
            'sk_no.required' => ' Nomor Surat Harus Diisi',
            'sk_tgl.required' => 'Tanggal Surat Harus Diisi',
            'ms_aktifitas_id.required' => 'Aksi Harus Diisi'
        ];

        $validate = [
            'ms_aktifitas_id' => 'required',
        ];

        if ($request->input('ms_aktifitas_id') == 2002) { //KLO DITERIMA
            $validate['sk_no'] = 'required';
            $validate['sk_tgl'] = 'required';
        }

        $request->validate($validate, $customMessages);
        $id = $request->input('id');
        $params = [
            'kategori' => 'SK PSP',
            'dir' => 'bmn/pengajuan-sk-penetapan',
            'isRequired' => $request->input('ms_aktifitas_id') == 2002 ? true : false,
            'fileKey' => 'sk_file',
            'pkey' => $id,
        ];
        try {
            DB::beginTransaction();
            // $currentRole = session('userData.current_role');
            $inputan = $request->only(['sk_no', 'sk_tgl', 'ms_aktifitas_id']);
            $file = Files::upload($request, $params);
            $inputan['sk_file'] = $file['path'] ?? null;

            PenetapanSk::updateOrCreate(['id' => $id], $inputan);

            $dataAktifitas = [
                'idKey' => 'bmn_penetapan_id',
                'idValue' => $id,
                'ms_aktifitas_id' => $request->input('ms_aktifitas_id'),
                'komentar' => $request->input('komentar'),
                'to_satker_induk' => false
            ];
            $acts = Approval::roleCheck($dataAktifitas);
            PenetapanSkAktifitas::insert($acts['act']);

            if (!empty($acts['nextAct'])) {
                $newAct = $acts['nextAct'];
                if ($newAct == 2002) { //selesa57Gi

                }
                PenetapanSk::where(['id' => $id])->update(['ms_aktifitas_id' => $newAct]);
                PenetapanSkAktifitas::where(['id' => $id])->update(['ms_aktifitas_id' => $newAct]);
            }

            $data = PenetapanSk::where(['id' => $id])->first();

            $notifParams = [
                'url' => "bmn/penetapan/penetapansk/{$id}",
                'judul' => 'SK PSP' . ' ' . $acts['msAct']->nama,
                'isi' => '',
                'target' => 'username',
                'targetValue' => $data->created_by,
                // 'targetSatker' => '10.12',
            ];
            Notifikasi::sendNotif($notifParams);
            DB::commit();
            return $this->resSuccess();
        } catch (\Throwable $th) {
            DB::rollBack();
            $errorMessage = $th->getMessage();
            return $this->resError($errorMessage);
        }
    }
    public function store(Request $request)
    {
        $isNew = $request->input('isNew');
        if (in_array($request->input('ms_aktifitas_id'), [2002, 2003])) {
            return $this->validatorAction($request);
        }
        $customMessages = [
            // 'surat_pernyataan.required' => 'Tanggal Surat harus diisi',
            'assets.required' => 'Minimal Input 1 Asset ',
            'sp_no.required' => ' Nomor Surat Harus Diisi',
            'sp_tgl.required' => 'Tanggal Surat Harus Diisi'
        ];

        $validate = [
            'vw_aset_psp_ids' => 'required|array|min:1',
            'sp_no' => 'required',
            'sp_tgl' => 'required',
        ];

        $id = $request->input('id') ?? MyHelper::getPk(date('Ymd'), 'sdm_timakuntansibarang_seq');
        $request->validate($validate, $customMessages);

        try {
            DB::beginTransaction();
            $params = [
                'kategori' => 'Surat Pernyataan Tanggung Jawab Mutlak',
                'dir' => 'bmn/pengajuan-sk-penetapan',
                'isRequired' => $request->input('ms_aktifitas_id') == 2001 ? true : false,
                'fileKey' => 'sp_file',
                'pkey' => $id
            ];
            $file = Files::upload($request, $params);
            // $currentRole = session('userData.current_role');
            $inputan = $request->only(['sp_no', 'sp_tgl', 'ms_aktifitas_id']);
            if (!$isNew && $file) {
                $inputan['sp_file'] = $file['path'];
            } else {
                $inputan['sp_file'] = $file['path'] ?? null;
            }

            PenetapanSk::updateOrCreate(['id' => $id], $inputan);
            $inputanAssets = [];

            foreach ($request->input('vw_aset_psp_ids') as $value) {
                $inputanAssets[] = [
                    'bmn_penetapan_id' => $id,
                    'vw_aset_psp_id' => $value
                ];
            }

            if (!empty($inputanAssets)) {
                PenetapanSkAsset::where('bmn_penetapan_id', $id)->delete();
                PenetapanSkAsset::insert($inputanAssets);
            }

            $dataAktifitas = [
                'idKey' => 'bmn_penetapan_id',
                'idValue' => $id,
                'ms_aktifitas_id' => $request->input('ms_aktifitas_id'),
                'komentar' => $request->input('komentar'),
                'to_satker_induk' => false
            ];
            $acts = Approval::roleCheck($dataAktifitas);
            PenetapanSkAktifitas::insert($acts['act']);

            if (!empty($acts['nextAct'])) {
                $newAct = $acts['nextAct'];
                PenetapanSk::where(['id' => $id])->update(['ms_aktifitas_id' => $newAct]);
                PenetapanSkAktifitas::where(['id' => $id])->update(['ms_aktifitas_id' => $newAct]);
            }

            if ($dataAktifitas['ms_aktifitas_id'] == 2001) {
                $params = [
                    'url' => "bmn/penetapan/penetapansk/{$id}",
                    'judul' => 'Mengajukan SK PSP',
                    'isi' => '',
                    'target' => 'role',
                    'targetValue' => config('constants.validator_pusat_role_id'),
                    // 'targetSatker' => '10.12',
                ];
                Notifikasi::sendNotif($params);
            }
            DB::commit();
            return $this->resSuccess();
        } catch (\Throwable $th) {
            DB::rollBack();
            $errorMessage = $th->getMessage();
            return $this->resError($errorMessage);
        }
    }

    /**
     * Display the specified resource.
     */
    public function show(string $id)
    {

        $data = $this->getData($id, true);
        // return view('bmn.penetapansk.penetapanValidatorFormV', $data);
        return view('bmn.penetapansk.penetapanFormV', $data);
    }

    /**
     * Show the form for editing the specified resource.
     */
    public function edit(string $id)
    {
        $data = $this->getData($id);
        return view('bmn.penetapansk.penetapanFormV', $data);
    }

    /**
     * Update the specified resource in storage.
     */
    public function update(Request $request)
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
            PenetapanSk::destroy($id);
            DB::commit();
            return $this->resSuccess('Berhasil Dihapus!');
        } catch (\Throwable $th) {
            DB::rollBack();
        }
    }

    public function getBarang(string $jenisAsset)
    {
        $currentRole = session('userData.current_role');
        $idKeu = $currentRole['ms_satker_id_keu'];
        $idSatker = $currentRole['ms_satker_id'];
        $queryRes = DB::table($jenisAsset)->where('id_satker', $idKeu)->select(['kode_barang', 'nm_barang'])->distinct()->orderBy('nm_barang')->get();
        $udahPSP = DB::table('bmn_penetapan_asset as a')
            ->select('a.barang_kode')->distinct()
            ->join('bmn_penetapan as b', 'b.id', '=', 'a.bmn_penetapan_id')
            ->where(
                ['a.asset_kode' => $jenisAsset, 'b.ms_satker_id' => $idSatker]
            )->get();
        $disabled = Arr::pluck($udahPSP, 'barang_kode');
        $data = MyHelper::generateSelectOptions(
            [
                'data' => $queryRes,
                'text' => 'nm_barang',
                'value' => 'kode_barang',
                'type' => 'raw',
                'disableds' => $disabled,
            ]
        );
        return response()->json($data ?? []);
    }


    function getSelectedAsset(Request $request)
    {
        $data = PenetapanSk::getAset($request->input());
        return response()->json($data);
    }
}
