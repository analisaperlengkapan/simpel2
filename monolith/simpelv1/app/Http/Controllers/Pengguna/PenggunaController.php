<?php

namespace App\Http\Controllers\Pengguna;

use App\Helpers\MyHelper;
use App\Http\Controllers\Controller;
use App\Models\Master;
use App\Models\MsSatker;
use App\Models\Pengguna\Level;
use App\Models\Pengguna\Pengguna;
use Illuminate\Http\Request;
use Illuminate\Support\Arr;
use Illuminate\Support\Facades\DB;
use Symfony\Component\HttpKernel\Exception\NotFoundHttpException;

class PenggunaController extends Controller
{
    /**
     * Display a listing of the resource.
     */
    protected $breadcums = [
        'Pengguna',
        ['link' => '/pengguna/pengguna', 'title' => 'Pengaturan Pengguna']
    ];
    protected $controller = '/pengguna/pengguna';
    public function index()
    {
        $columns = [
            'NIP',
            'Nama',
            'Satker',
            'Role',
        ];
        $defColumns = [0, 1, 2, 3];
        return view('pengguna.pengguna.penggunaV', [
            'columns' => $columns,
            'defColumns' => $defColumns,
            'tableId' => 'dt-user',
            'breadcums' => $this->breadcums,
            'controller' => $this->controller
        ]);
    }

    public function gridData(Request $request)
    {
        $user = new Pengguna();
        $pagingParams = $request->only(['start', 'length']);
        $searchParams = $request->only(['columns']);
        $data = $user->getUserGrid($pagingParams, $searchParams);
        return response()->json([
            'data' => $data['data'],
            'recordsTotal' => $data['total'],
            'recordsFiltered' => $data['total'],
        ]);
    }

    function getData($id = null)
    {
        $model = [];
        $roles = [];
        $isNew = true;
        $breadcum = 'Tambah';
        $canResetPassword = false;

        if ($id) {
            $breadcum = 'Ubah';
            $model = Pengguna::where('id', $id)->first();
            if (!$model) {
                throw new NotFoundHttpException('Data Tidak Ditemukan');
            }
            if (session('userData.current_role.ms_role_id') == config('constants.superadmin_role_id')) {
                $canResetPassword = true;
            }
            $roles = Pengguna::getRoles(['user_id' => $id]);
            $selectedRoles = Arr::pluck($roles, 'ms_role_id');
            $model = $model->toArray();
            $model['nama'] = $model['name'];
            $isNew = false;
        }

        $satkers = Master::getSatkers();
        $data = [
            'model' => $model,
            'roles' => $roles,
            'isNew' => $isNew,
            'breadcums' => array_merge($this->breadcums, [$breadcum]),
            'canChangePassword' => $isNew,
            'canResetPassword' => $canResetPassword,
            'controller' => $this->controller,
            'satkers' => $satkers,
            'satkerOptions' => MyHelper::generateSelectOptions([
                'data' => $satkers,
                'text' => 'inst_nama',
                'value' => 'inst_satkerkd',
                'selected' => $model['ms_satker_id'] ?? null
            ]),
            'roleOptions' => MyHelper::generateSelectOptions([
                'data' => Master::getRoles(),
                'text' => 'name',
                'value' => 'id',
                'selected' => $selectedRoles ?? null
            ]),
        ];
        return $data;
    }
    /**
     * Show the form for creating a new resource.
     */
    public function create()
    {
        //
        $data = $this->getData();
        return view('pengguna.pengguna.penggunaFormV', $data);
    }

    /**
     * Store a newly created resource in storage.
     */
    public function store(Request $request)
    {
        //
        $validasi = [
            'username' => 'required',
            'roles' => 'required|array|min:1'
        ];
        if (!$request->has('id')) { //nginsert
            $validasi['username'] = 'required|unique:users';
            $validasi['password'] = 'required';
        }
        $request->validate($validasi);

        try {
            $roleModel = new Level();
            $pegawai = Master::getPegawaiByNip($request->input('username'));
            if (empty($pegawai)) {
                return $this->resError('Pegawai Tidak Ditemukan');
            }
            $pegawai = (array) $pegawai;
            $newUser = [
                'username' => $request->input('username'),
                'name' => $pegawai['nama'],
                'email' => $pegawai['pns_mail'],
                'pangkat' => $pegawai['pangkat'],
                'jabatan' => $pegawai['jabatan'],
                'ms_satker_id' => $pegawai['inst_satkerkd'],
                'satker' => $pegawai['satker'],
                'ms_satker_pusat_id' => $pegawai['mapped_unit_kerja'],
                'foto' => MyHelper::getFotoMysimkari($pegawai['foto']),
                'satker_pusat' => $pegawai['mapped_unit_kerja_nama'],
            ];
            if (!$request->has('id'))
                $newUser['password'] = $request->input('password');

            DB::beginTransaction();
            $user = Pengguna::updateOrCreate(['username' => $request->input('username')], $newUser);
            foreach ($request->input('roles') as $key => $role) {
                $msSatker = MsSatker::where(['inst_satkerkd' => $pegawai['inst_satkerkd']])->first();
                $inputanRoles[] = [
                    'user_id' => $user->id,
                    'ms_role_id' => $role,
                    'ms_satker_id' => $pegawai['inst_satkerkd'],
                    'ms_satker_id_keu' => $msSatker->kdsatker_keu ?? null,
                    'ms_satker_pusat_id' => $newUser['ms_satker_pusat_id'],
                ];
                // $arrKodeSatker = MyHelper::toIdSatkerCms($role['ms_satker_id']);
                // $inputanRoles[] = array_merge($inputan, $arrKodeSatker);
            }
            $roleModel->delInsertUserRole($user->id, $inputanRoles);
            DB::commit();
            return $this->resSuccess('Berhasil Dismpan');
        } catch (\Throwable $th) {
            dd($th);
            return $this->resError('Gagal menyimpan data');
            // DB::rollBack();
        }
    }

    /**
     * Display the specified resource.
     */
    public function show(string $id)
    {
        $data = $this->getData($id);
        return view('pengguna.pengguna.penggunaFormV', $data);
    }

    /**
     * Show the form for editing the specified resource.
     */
    public function edit(string $id)
    {
        //
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
            Pengguna::destroy($id);
            DB::commit();
            return $this->resSuccess('Berhasil Dihapus!');
        } catch (\Throwable $th) {
            DB::rollBack();
        }
    }
}
