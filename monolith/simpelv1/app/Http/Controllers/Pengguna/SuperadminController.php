<?php

namespace App\Http\Controllers\Pengguna;

use App\Http\Controllers\Controller;
use App\Models\Pengguna\Level;
use App\Models\Pengguna\Pengguna;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\DB;
use Symfony\Component\HttpKernel\Exception\NotFoundHttpException;

class SuperadminController extends Controller
{
    /**
     * Display a listing of the resource.
     */
    protected $breadcums = ['Pengguna', 'Superadmin'];
    private $controller = '/pengguna/superadmin';
    public function index()
    {
        $columns = [
            'Username',
            'Nama',
            'Email',
        ];
        $defColumns = [0, 1, 2];
        return view('pengguna.superadmin.superadminV', [
            'tableId' => 'dt-user',
            'columns' => $columns,
            'defColumns' => $defColumns,
            'breadcums' => $this->breadcums,
            'controller' => $this->controller
        ]);
    }

    public function gridData(Request $request)
    {
        $user = new Pengguna();
        $pagingParams = $request->only(['start', 'length']);
        $searchParams = $request->only(['columns']);
        $data = $user->getUserGrid($pagingParams, $searchParams, true);
        return response()->json([
            'data' => $data['data'],
            'recordsTotal' => $data['total'],
            'recordsFiltered' => $data['total'],
        ]);
    }

    function getData($id = null)
    {
        $model = [];
        $isNew = true;
        $breadcum = 'Tambah';

        if ($id) {
            $breadcum = 'Ubah';
            $model = Pengguna::where('id', $id)->first();
            if (!$model) {
                throw new NotFoundHttpException('Data Tidak Ditemukan');
            }

            $model = $model->toArray();
            $isNew = false;
        }

        $data = [
            'model' => $model,
            'tableId' => 'dt-user',
            'isNew' => $isNew,
            'canChangePassword' => !$isNew && session('id') == $model['id'] ? true : false,
            'breadcums' => array_merge($this->breadcums, [$breadcum]),
            'controller' => $this->controller,
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
        return view('pengguna.superadmin.superadminFormV', $data);
    }

    /**
     * Store a newly created resource in storage.
     */
    public function store(Request $request)
    {
        //
        $validasi = [
            'name' => 'required',
            'email' => 'required|email',
            'username' => 'required',
        ];

        $inputan = $request->input();

        if ($request->has('id')) {
            $inputan = $request->except('password');
        } else {
            $validasi['password'] = 'required';
        }

        $request->validate($validasi);
        $inputan['is_superadmin'] = 1;
        try {
            DB::beginTransaction();
            $user = Pengguna::updateOrCreate(['id' => $request->input('id')], $inputan);

            $roleModel = new Level();
            $roleModel->delInsertUserRole($user->id, ['user_id' => $user->id, 'ms_role_id' => config('constants.superadmin_role_id')]);

            DB::commit();
            return $this->resSuccess('Berhasil Dismpan');
        } catch (\Throwable $th) {
            dd($th);
            DB::rollBack();
            return $this->resError('Gagal menyimpan data');
        }
    }

    /**
     * Display the specified resource.
     */
    public function show(string $id)
    {
        $data = $this->getData($id);
        return view('pengguna.superadmin.superadminFormV', $data);
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
