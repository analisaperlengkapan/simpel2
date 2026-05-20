<?php

namespace App\Http\Controllers\Pengaturan\MenuTema;

use App\Helpers\MyHelper;
use App\Http\Controllers\Controller;
use App\Models\Master\MsMenu as Model;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\DB;
use Symfony\Component\HttpKernel\Exception\NotFoundHttpException;

class MenuController extends Controller
{
    /**
     * Display a listing of the resource.
     */
    protected $breadcums = ['Pengaturan', 'Menu dan Tema Aplikasi', 'Menu Aplikasi'];

    protected $controller = 'pengaturan/menu';

    public function index()
    {
        $columns = ['Nama', 'Level', 'Urutan', 'Aktif'];
        $defColumns = [0, 1, 2, 3];

        return view('pengaturan.menu.gridV', [
            'columns' => $columns,
            'defColumns' => $defColumns,
            'tableId' => 'dt-menu',
            'breadcums' => $this->breadcums,
            'controller' => $this->controller,
        ]);
    }

    public function gridData(Request $request)
    {
        $model = new Model;
        $pagingParams = $request->only(['start', 'length']);
        $searchParams = $request->only(['columns']);
        $data = $model->getDataGrid($pagingParams, $searchParams);

        return response()->json([
            'data' => $data['data'],
            'recordsTotal' => $data['total'],
            'recordsFiltered' => $data['total'],
        ]);
    }

    public function getData($id = null, $readOnly = false) {}

    /**
     * Show the form for creating a new resource.
     */
    public function create() {}

    /**
     * Store a newly created resource in storage.
     */
    public function store(Request $request)
    {
        try {
            DB::beginTransaction();
            $model = Model::where('id', $request->input('id'))->first();

            Model::where([
                'parent' => $request->input('parent'),
                'urutan' => $request->input('urutan'),
            ])->update(['urutan' => $model->urutan]);

            $inputan = $request->only(['name', 'urutan']);
            $inputan = array_merge($inputan, ['is_active' => $request->input('is_active') ?? 0]);
            Model::where('id', $request->input('id'))->update($inputan);
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
        $model = Model::where('id', $id)->first();
        if (! $model) {
            throw new NotFoundHttpException('Menu Tidak ditemukan');
        }

        $menus = Model::where('parent', $model->parent)->orderBy('urutan')->get();
        foreach ($menus as $key => $menu) {
            $urutans[] = (object) [
                'value' => $menu->urutan,
                'text' => "{$menu->urutan} - {$menu->name}",
            ];
        }

        $urutanOptions = MyHelper::generateSelectOptions([
            'data' => $urutans,
            'selected' => $model->urutan,
            'text' => 'text',
            'value' => 'value',
        ]);

        return view('pengaturan.menu.formV', [
            'controller' => $this->controller,
            'breadcums' => array_merge($this->breadcums, ['Atur Menu']),
            'urutanOptions' => $urutanOptions,
            'model' => $model,
        ]);
    }

    /**
     * Show the form for editing the specified resource.
     */
    public function edit(string $id) {}

    /**
     * Update the specified resource in storage.
     */
    public function update(Request $request, Model $model)
    {
        //
    }

    /**
     * Remove the specified resource from storage.
     */
    public function destroy(string $id) {}
}
