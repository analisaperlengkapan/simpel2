<?php

namespace App\Http\Controllers\Master\PakaianDinas;

use App\Helpers\MyHelper;
use App\Http\Controllers\Controller;
use App\Models\Master\PakaianDinas\SpesifikasiPakaianDinas;
use App\Models\Master\PakaianDinas\SubSpesifikasiPakaianDinas as Model;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\DB;
use Symfony\Component\HttpKernel\Exception\NotFoundHttpException;

class SubSpesifikasiPakaianDinasController extends Controller
{
    /**
     * Display a listing of the resource.
     */
    protected $breadcums = ['Master', 'Pakaian Dinas', 'Sub Spesifikasi Pakaian Dinas'];
    public function index()
    {
        return view('master.pakaian-dinas.subspesifikasi-pakaian-dinas.gridV', ['tableId' => 'dt-jenis', 'breadcums' => $this->breadcums]);
    }

    public function gridData(Request $request)
    {
        $model = new Model();
        $pagingParams = $request->only(['start', 'length']);
        $searchParams = $request->only(['search', 'filterBy']);
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
        if ($readOnly) {
            $breadcum = 'Detail';
        }
        $spesifikasis = SpesifikasiPakaianDinas::getSelections();
        $genders = ['SEMUA', 'L', 'P'];
        $data = [
            'model' => $model,
            'isNew' => $isNew,
            'breadcums' => array_merge($this->breadcums, [$breadcum]),
            'judul' => $breadcum,
            'readOnly' => $readOnly,
            'genderOptions' => MyHelper::generateSelectOptions(['data' => $genders, 'selected' => 'SEMUA']),
            'spesifikasiOptions' => MyHelper::generateSelectOptions([
                'data' => $spesifikasis,
                'text' => 'nama',
                'value' => 'id',
                'selected' => $model['id_spesifikasi'] ?? null,
            ]),
        ];
        //dd($data['satkerOptions']);
        return $data;
    }

    /**
     * Show the form for creating a new resource.
     */
    public function create()
    {
        $data = $this->getData();
        return view('master.pakaian-dinas.subspesifikasi-pakaian-dinas.formV', $data);
    }

    /**
     * Store a newly created resource in storage.
     */
    public function store(Request $request)
    {
        $customMessages = [
            'nama.required' => 'Nama harus diisi ',
            'id_spesifikasi.required' => 'Spesifikasi harus dipilih',
        ];
        $request->validate([
            'nama' => 'required',
            'id_spesifikasi' => 'required',
        ], $customMessages);
        $id = $request->input('id') ?? MyHelper::getPk(date('Ymd'), 'ms_subspesifikasi_pakaian_dinas_seq');
        Model::updateOrCreate(['id' => $id], $request->input());
        return $this->resSuccess();
    }

    /**
     * Display the specified resource.
     */
    public function show(string $id)
    {
        $data = $this->getData($id);
        return view('master.pakaian-dinas.subspesifikasi-pakaian-dinas.formV', $data);
    }

    /**
     * Show the form for editing the specified resource.
     */
    public function edit(string $id)
    {
    }

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