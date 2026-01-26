<?php

namespace App\Http\Controllers\Bantuan;
use Illuminate\Routing\Controller;

use App\Helpers\MyHelper;
use App\Models\Master\Master;
use App\Models\Pengguna\Review;
use App\Models\Bantuan\NotifikasiManual;
use App\Models\Bantuan\NotifikasiManualTarget;
use Illuminate\Http\Request;
use Illuminate\Support\Arr;
use Illuminate\Support\Facades\DB;
use Symfony\Component\HttpKernel\Exception\NotFoundHttpException;

class NotifikasiManualController extends Controller
{
    /**
     * Display a listing of the resource.
     */
    protected $breadcums = ['Bantuan', 'Pengaturan Notifikasi', 'Pengiriman Notifikasi manual'];
    protected $controller = 'support/notifikasi-manual';
    public function index()
    {

        $columns = ['Username', 'Judul', 'Tanggal', 'Isi', 'Role', 'Aktif'];
        $defColumns = array_keys($columns);
        return view('support.pengaturan_notifikasi.notifikasiManualV', [
            'columns' => $columns,
            'defColumns' => $defColumns,
            'tableId' => 'dt-survey',
            'canCreate' => $this->canCreate(),
            'controller' => $this->controller,
            'breadcums' => $this->breadcums,
        ]);
    }

    function canCreate()
    {
        return in_array(session('userData.current_role.ms_role_id'), [
            config('constants.validator_pusat_role_id'),
            config('constants.admin_biro_lengkap_role_id'),
            config('constants.superadmin_role_id'),
        ]);
    }

    public function gridData(Request $request)
    {
        $model = new NotifikasiManual();
        $pagingParams = $request->only(['start', 'length']);
        $searchParams = $request->only(['columns']);
        $data = $model->getDataGrid($pagingParams, $searchParams);
        return response()->json([
            'data' => $data['data'],
            'recordsTotal' => $data['total'],
            'recordsFiltered' => $data['total'],
        ]);
    }

    function getData($id = null, $isRaw = false)
    {
        $model = [];
        $isNew = true;
        $breadcum = 'Tambah';
        if ($id) {
            $breadcum = 'Ubah';
            $model = NotifikasiManual::where('id', $id)->first();
            if (!$model) {
                throw new NotFoundHttpException('Data Tidak Ditemukan');
            }
            $model = $model->toArray();
            $isNew = false;
            $targets = NotifikasiManualTarget::where('notifikasi_manual_id', $model['id'])->get();
            $selectedRole = Arr::pluck($targets, 'role_id');
        }

        $data = [
            'isNew' => $isNew,
            'model' => $model,
            'controller' => $this->controller,
            'breadcums' => array_merge($this->breadcums, [$breadcum]),
            'roleOptions' => MyHelper::generateSelectOptions([
                'data' => Master::getRoles(),
                'value' => 'id',
                'text' => 'name',
                'type' => $isRaw ? 'raw' : null,
                'selected' => $selectedRole ?? null,
            ])
        ];
        return $data;

    }

    public function create()
    {
        return view('support.pengaturan_notifikasi.notifikasiManualFormV', $this->getData());

    }

    public function store(Request $request)
    {
        $request->validate(
            [
                'judul' => 'required',
                'isi' => 'required',
                'ms_role_id' => 'required|array|min:1'
            ],
            [
                'ms_role_id.required' => 'Role harus Pilih minimal 1'
            ]
        );
        $data = $request->input();
        $data['created_by'] = session('userData.username');
        $roles = DB::table('ms_role')->whereIn('id', $data['ms_role_id'])->get()->toArray();
        $data['target_role_id'] = implode(',', Arr::pluck($roles, 'name'));
        $data['is_active'] = $request->has('is_active') ? 1 : 0;

        $notif = NotifikasiManual::updateOrCreate(['id' => $data['id'] ?? null], $data);
        foreach ($roles as $key => $role) {
            $targets[] = [
                'notifikasi_manual_id' => $notif['id'],
                'role_id' => $role->id,
                'role_name' => $role->name,
            ];
        }

        $notif = NotifikasiManualTarget::insert($targets);
        return $this->resSuccess();
    }

    public function show(string $id)
    {

        return view('support.pengaturan_notifikasi.notifikasiManualFormV', $this->getData($id));

    }

    public function edit(string $id)
    {

    }
    public function update(Request $request)
    {
        //
    }

    public function destroy(string $id)
    {
        try {
            DB::beginTransaction();
            Review::destroy($id);
            DB::commit();
            return $this->resSuccess('Berhasil Dihapus!');
        } catch (\Throwable $th) {
            DB::rollBack();
        }
    }

    public function getByRole(int $msRoleId = null)
    {
        $notifikasiManual = NotifikasiManual::getRoleNotif($msRoleId);
        return response()->json($notifikasiManual);
    }

    public function getOne(string $msRoleId = null)
    {
        $notifikasiManual = NotifikasiManual::getRoleNotif($msRoleId);
        return response()->json($notifikasiManual);
    }

    public function createData()
    {
        return response()->json($this->getData(null, true));
    }

    public function showData(int $id)
    {
        return response()->json($this->getData($id, true));
    }

}
