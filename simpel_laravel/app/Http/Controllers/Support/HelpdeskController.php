<?php

namespace App\Http\Controllers\Support;
use Illuminate\Routing\Controller;

use App\Helpers\MyHelper;
use App\Models\Master;
use App\Models\Support\Helpdesk;
use App\Models\Support\UserBantuanAktivitas as Aktivitas;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\DB;
use Symfony\Component\HttpKernel\Exception\NotFoundHttpException;

class HelpdeskController extends Controller
{
    /**
     * Display a listing of the resource.
     */
    protected $kategoriJudul = 'Bantuan';
    //protected $breadcums = ['Bantuan', 'Ajukan Tiket'];
    protected $controller = '/support/helpdesk';
    protected $breadcums = ['Support'];
    protected $columns = ['Tgl Tiket', 'ID Tiket', 'User', 'Topik', 'Judul',  'Status'];
    protected $defColumns = [0, 1, 2, 3, 4, 5, 6];
    //protected $breadcums = ['Support', 'Panduan'];

    public function __construct()
    {
        $this->breadcums = array_merge($this->breadcums, [['link' => $this->controller, 'title' => 'Helpdesk']]);

    }
    public function index()
    {
        if (session('userData.current_role.ms_satker_id_keu')) {
            $isdelete = true;
        } else {
            $isdelete = false;
        }
        $columns = [
            'Tgl Tiket',
            'ID Tiket',
            'User',
            'Topik',
            'Judul',
            'Status',
        ];
        $defColumns = [0, 1, 2, 3, 4, 5];
        return view('support.helpdesk.helpdeskV', [
            'tableId' => 'dt-helpdesk',
            'kategoriJudul' => $this->kategoriJudul,
            'breadcums' => $this->breadcums,
            'columns' => $columns,
            'defColumns' => $defColumns,
            'controller' => $this->controller,
            'candelete' => $isdelete,
            'canCreate' => $this->canCreatePermintaan()
        ]);
    }
    protected function canCreatePermintaan()
    {
        // return true;
        return in_array(session('userData.current_role.ms_role_id'), config('constants.pelaksana_role'));
    }
    public function gridData(Request $request)
    {
        $model = new Helpdesk();
        $pagingParams = $request->only(['start', 'length']);
        //$searchParams =  $request->only(['search',  'filterBy']);
        $searchParams = $request->only(['columns']);
        $data = $model->getDataGrid($pagingParams, $searchParams);
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
        if (session('userData.current_role.ms_satker_id_keu')) {
            $isupdateStatus = true;
        } else {
            $isupdateStatus = false;
        }

        $breadcum = 'Tambah';
        if ($id) {
            $breadcum = 'Ubah';
            $model = Helpdesk::where('id', $id)->first();
            if (!$model) {
                throw new NotFoundHttpException('Data Tidak Ditemukan');
            }

            $model = $model->toArray();
            $isNew = false;
        }
        if ($readOnly) {
            $breadcum = 'Detail';
        }
        $tipe = ['Low', 'Medium', 'High'];
        $status = ['Open', 'Close'];
        $topik = Master::gettopik();
        $tujuan = Master::getrole();
        $data = [
            'model' => $model,
            'isupdateStatus' => $isupdateStatus,
            'isNew' => $isNew,
            'breadcums' => array_merge($this->breadcums, [$breadcum]),
            'judul' => $breadcum,
            'readOnly' => $readOnly,
            'topik' => MyHelper::generateSelectOptions([
                'data' => $topik,
                'text' => 'topik',
                'textkode' => 'id',
                'value' => 'topik',
                'selected' => $model['topik'] ?? null,
            ]),
            'tujuan' => MyHelper::generateSelectOptions([
                'data' => $tujuan,
                'text' => 'name',
                'textkode' => 'id',
                'value' => 'id',
                'selected' => $model['id'] ?? null,
            ]),
            'tipeOptions' => MyHelper::generateSelectOptions([
                'data' => $tipe,
                'text' => 'tipe',
                'value' => null,
                'selected' => $model['tipe_tiket'] ?? null,
            ]),
            'statusOptions' => MyHelper::generateSelectOptions([
                'data' => $status,
                'text' => 'status',
                'value' => null,
                'selected' => $model['status'] ?? null,
            ]),
        ];
        return $data;
    }

    public function create()
    {
        $data = $this->getData();
        return view('support.helpdesk.helpdeskFormV', $data);
    }

    public function getTopik()
    {
        $data = Master::gettopik();
        return response()->json($data);
    }

    public function store(Request $request)
    {
        $request->validate([
            'judul' => 'required',
            'topik' => 'required',
            // 'tgl_pengajuan' => 'required',
            // 'deskripsi' => 'required',
            //'image' => 'image|mimes:png,jpg|max:1024'
        ]);

        //dd($request->all())
        $image = 'no image';
        if ($request->file('image')) {
            $image = $request->file('image')->store('helpdesk-image');

        }


        $data['judul'] = $request->judul;
        $data['topik'] = $request->topik;
        $data['tujuan'] = $request->tujuan;
        $data['tgl_pengajuan'] = date('Ymd');
        $data['deskripsi'] = $request->deskripsi ?? null;
        $data['catatan'] = $request->catatan;
        $data['image'] = $image;

        if (session('userData.current_role.ms_satker_id_keu')) {
            $data['kode_satker'] = session('userData.current_role.ms_satker_id_keu');
        }
        if ($request->status) {
            $data['status'] = $request->status;
        } else {
            $data['status'] = 'Open';
        }

        $id = $request->input('id') ?? MyHelper::getPk(date('Ymd'), 'support_helpdesk_seq');
        $data['kode_tiket'] = $id;
        Helpdesk::updateOrCreate(['id' => $id], $data);
        return $this->resSuccess();
    }

    public function show(string $id)
    {
        $model = [];
        $data = $this->getData($id);
        $model = Helpdesk::where('id', $id)->first();
        if (!$model) {
            throw new NotFoundHttpException('Data Tidak Ditemukan');
        }
        $aktivitasHistories = Aktivitas::getDetail($id);

        $data = [
            'model' => $model,
            'aktivitasHistories' => $aktivitasHistories,
            'breadcums' => $this->breadcums,
            'kategoriJudul' => $this->kategoriJudul
        ];
        return view('support.helpdesk.helpdeskFormKomentar_viewV', $data);
    }

    public function edit(string $id)
    {
        $model = [];
        $data = $this->getData($id);
        $model = Helpdesk::where('id', $id)->first();
        if (!$model) {
            throw new NotFoundHttpException('Data Tidak Ditemukan');
        }
        $aktivitasHistories = Aktivitas::getDetail($id);

        $data = [
            'model' => $model,
            'aktivitasHistories' => $aktivitasHistories,
            'breadcums' => $this->breadcums,
            'kategoriJudul' => $this->kategoriJudul
        ];
        return view('support.helpdesk.helpdeskFormKomentarV', $data);
    }
    public function saveKomentar(Request $request)
    {
        $validasi = [
            'ms_aktivitas_id' => 'required',
        ];
        $request->validate($validasi);

        if ($request->has('id')) {
            $isNew = false;
            $id = $request->input('id');
        } else {
            $isNew = true;
            $id = null;
        }

        $ms_aktivitas_id = $request->input('ms_aktivitas_id');

        if (session('userData.current_role.ms_satker_id_keu')) {
            $ms_satker_id = session('userData.current_role.ms_satker_id_keu');
        } else {
            $ms_satker_id = '';
        }
        $user_id = session('userData.current_role.user_id');
        $rote_id = session('userData.current_role.ms_role_id');
        $created_at = date('Ymd H:i:s');
        try {
            DB::beginTransaction();
            $currentRole = session('userData.current_role');
            Helpdesk::where(['id' => $id])->update(['status' => $ms_aktivitas_id]);
            $ids = MyHelper::getPk(date('Ymd'), 'support_helpdesk_aktivitas_id_seq');

            $dataAktivitas = [
                'id' => $ids,
                'pengajuan_id' => $id,
                'ms_aktivitas_id' => $ms_aktivitas_id,
                'ms_satker_id' => $ms_satker_id,
                'created_at' => $created_at,
                'nama' => $user_id,
                'role' => $rote_id,
                'komentar' => $request->input('komentar')
            ];
            //$acts = ["act" => [], "nextAct" => null];
            Aktivitas::insert($dataAktivitas);

            DB::commit();
            return $this->resSuccess(
                'Berhasil Disimpan!',
                [
                    'type' => 'redirect',
                    'url' => \URL::to('/support/helpdesk')
                ]
            );
        } catch (\Throwable $th) {
            DB::rollBack();
            dd($th->getMessage());
            return $this->resError('Gagal Menyimpan data');
        }
    }
    public function update(Request $request, Helpdesk $tanah)
    {
        //
    }

    public function destroy(string $id)
    {
        try {
            DB::beginTransaction();
            Helpdesk::destroy($id);
            DB::commit();
            return $this->resSuccess('Berhasil Dihapus!');
        } catch (\Throwable $th) {
            DB::rollBack();
        }
    }

}
