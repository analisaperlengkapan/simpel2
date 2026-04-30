<?php

namespace App\Http\Controllers\Pengguna;

use App\Http\Controllers\Controller;
use App\Models\Pengguna\Level;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\DB;
use Symfony\Component\HttpKernel\Exception\NotFoundHttpException;


class LevelController extends Controller
{
    protected $breadcums = ['Pengaturan', 'Pengaturan Level Pengguna'];
    protected $controller = '/pengguna/level';
    /**
     * Display a listing of the resource.
     */
    public function index()
    {
        $columns = ['Nama', 'Deskripsi'];
        $defColumns = [0, 1];
        return view('pengguna.level.levelV', [
            'tableId' => 'dt-role',
            'columns' => $columns,
            'defColumns' => $defColumns,
            'controller' => $this->controller,
            'breadcums' => $this->breadcums
        ]);
    }

    public function gridData(Request $request)
    {
        $model = new Level();
        $pagingParams = $request->only(['start', 'length']);
        $searchParams = $request->only(['columns']);
        $data = $model->getGridData($pagingParams, $searchParams);
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

        $role = new Level();

        if ($id) {
            $breadcum = 'Ubah';
            $model = Level::where('id', $id)->first();
            if (!$model) {
                throw new NotFoundHttpException('Data Tidak Ditemukan');
            }
            $model = $model->toArray();
            $isNew = false;
        }
        $menus = $role->getRoleMenu($id);
        $data = [
            'model' => $model,
            'controller' => $this->controller,
            'menus2' => $menus,
            'isNew' => $isNew,
            'breadcums' => array_merge($this->breadcums, [$breadcum]),
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
        return view('pengguna.level.levelFormV', $data);
    }

    /**
     * Store a newly created resource in storage.
     */
    public function store(Request $request)
    {
        //
        $request->validate([
            'name' => 'required',
            'description' => 'required',
            'menu_id' => 'array|min:1',
        ]);


        try {
            DB::beginTransaction();
            $role = Level::updateOrCreate(['id' => $request->input('id')], $request->only(['id', 'name', 'description']));
            DB::table('ms_role_menu')->where(['role_id' => $request->input('id')])->delete();
            foreach ($request->input('menu_id') as $key => $value) {
                $menus[] = [
                    'role_id' => $role->id,
                    'menu_id' => $value
                ];
            }
            DB::table('ms_role_menu')->insert($menus);
            DB::commit();
            return $this->resSuccess('Berhasil Dismpan');
        } catch (\Throwable $th) {
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
        return view('pengguna.level.levelFormV', $data);
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
            Level::destroy($id);
            DB::commit();
            return $this->resSuccess('Berhasil Dihapus!');
        } catch (\Throwable $th) {
            DB::rollBack();
        }
    }
}
