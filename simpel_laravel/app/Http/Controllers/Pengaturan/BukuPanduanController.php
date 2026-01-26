<?php

namespace App\Http\Controllers\Pengaturan;
use Illuminate\Routing\Controller;

use App\Helpers\MyHelper;
use App\Models\BukuPanduan as Model;
use App\Models\Sistem\Files;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\DB;
use Symfony\Component\HttpKernel\Exception\NotFoundHttpException;

class BukuPanduanController extends Controller
{
    /**
     * Display a listing of the resource.
     */
    protected $breadcums = ['Pengaturan', 'Buku Panduan'];
    private $controller = '/pengaturan/buku-panduan';
    public function index()
    {

        $columns = ['Judul', 'File', 'Platform'];
        $defColumns = [0, 1, 2];
        $data = [
            'tableId' => 'dt-penetapan',
            'breadcums' => $this->breadcums,
            'controller' => $this->controller,
            'title' => 'Buku Panduan',
            'columns' => $columns,
            'defColumns' => $defColumns
        ];
        return view('pengaturan.buku-panduan.gridV', $data);
    }

    function isPelaksanaSatker()
    {
        $currentRole = session('userData.current_role');
        return $currentRole['ms_role_id'] == config('constants.pelaksana_satker_role_id') ? 1 : 0;
    }

    public function gridData(Request $request)
    {
        $model = new Model();
        $pagingParams = $request->only(['start', 'length']);
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
        $aktifitasHistories = [];
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


        $kategories = Model::getKategori();
        $data = [
            'model' => $model,
            'isNew' => $isNew,
            'controller' => $this->controller,
            'breadcums' => array_merge($this->breadcums, [$breadcum]),
            'title' => $breadcum . ' Buku Panduan',
            'readOnly' => $readOnly,
            'kategoriOptions' => MyHelper::generateSelectOptions(['data' => $kategories, 'selected' => $model['kategori'] ?? null]),
            'platformOptions' => MyHelper::generateSelectOptions(['data' => config('constants.platforms'), 'selected' => $model['platform'] ?? null])

        ];
        return $data;
    }

    /**
     * Show the form for creating a new resource.
     */
    public function create()
    {
        $data = $this->getData();
        return view('pengaturan.buku-panduan.formV', $data);
    }

    /**
     * Store a newly created resource in storage.
     */

    public function store(Request $request)
    {
        $validate = [
            'judul' => 'required',
            'kategori' => 'required',
        ];

        $isNew = !$request->has('id') ? true : false;
        $id = $request->input('id') ?? MyHelper::getPk(date('Ymd'), 'sdm_timakuntansibarang_seq');
        $request->validate($validate);

        try {
            DB::beginTransaction();
            // $currentRole = session('userData.current_role');
            $inputan = $request->only(['judul', 'kategori', 'platform']);
            if ($request->hasFile('file')) {
                $params = [
                    'kategori' => 'Buku Panduan',
                    'dir' => 'buku-panduan',
                    'isRequired' => $isNew,
                    'fileKey' => 'file',
                    'pkey' => $id
                ];
                $file = Files::upload($request, $params);
                $inputan['path'] = $file['path'];
            }
            Model::updateOrCreate(['id' => $id], $inputan);

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
        return view('pengaturan.buku-panduan.formV', $data);
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
            Model::destroy($id);
            DB::commit();
            return $this->resSuccess('Berhasil Dihapus!');
        } catch (\Throwable $th) {
            DB::rollBack();
        }
    }
}
