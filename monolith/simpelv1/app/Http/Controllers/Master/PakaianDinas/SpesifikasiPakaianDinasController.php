<?php

namespace App\Http\Controllers\Master\PakaianDinas;

use App\Helpers\MyHelper;
use App\Http\Controllers\Controller;
use App\Models\Files;
use App\Models\Master;
use App\Models\Master\PakaianDinas\JenisPakaianDinas;
use App\Models\Master\PakaianDinas\SpesifikasiPakaianDinas as Model;
use App\Models\Master\PakaianDinas\SpesifikasiPakaianDinasFoto;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\DB;
use Symfony\Component\HttpKernel\Exception\NotFoundHttpException;

class SpesifikasiPakaianDinasController extends Controller
{
    /**
     * Display a listing of the resource.
     */
    // protected $breadcums = ['Master', 'Pakaian Dinas', 'Spesifikasi Pakaian Dinas'];
    protected $kategoriJudul = 'Spesifikasi Pakaian Dinas';

    protected $controller = 'master/pakaian-dinas/spesifikasi-pakaian-dinas';

    protected $breadcums = ['Master'];

    protected $columns = ['No', 'Pakaian Dinas', 'Spesifikasi', 'Subspesifikasi'];

    protected $defColumns = [0, 1, 2, 3];

    public function __construct()
    {
        $this->breadcums = array_merge($this->breadcums, [['link' => $this->controller, 'title' => 'Spesifikasi Pakaian Dinas']]);

    }

    public function index()
    {
        // return view('master.pakaian-dinas.spesifikasi-pakaian-dinas.gridV', ['tableId' => 'dt-jenis', 'breadcums' => $this->breadcums]);
        return view('master.pakaian-dinas.spesifikasi-pakaian-dinas.gridV', [
            'tableId' => 'dt-spesifikasi',
            'kategoriJudul' => $this->kategoriJudul,
            'breadcums' => $this->breadcums,
            'columns' => $this->columns,
            'defColumns' => $this->defColumns,
            'controller' => $this->controller,
        ]);
    }

    public function gridData(Request $request)
    {
        $model = new Model;
        $pagingParams = $request->only(['start', 'length']);
        // $searchParams = $request->only(['search', 'filterBy']);
        $searchParams = $request->only(['columns']);
        $data = $model->getDataGrid($pagingParams, $searchParams);

        return response()->json([
            'data' => $data['data'],
            'recordsTotal' => $data['total'],
            'recordsFiltered' => $data['total'],
        ]);
    }

    public function getData($id = null, $readOnly = false)
    {
        $model = [];
        $isNew = true;
        $breadcum = 'Tambah';
        $fotos = [];
        if ($id) {
            $breadcum = 'Ubah';
            $model = Model::where('id', $id)->first();
            if (! $model) {
                throw new NotFoundHttpException('Data Tidak Ditemukan');
            }
            $fotos = SpesifikasiPakaianDinasFoto::where(['ms_spesifikasi_pakaian_dinas_id' => $model->id])->get();
            $model = $model->toArray();
            $isNew = false;
        }
        if ($readOnly) {
            $breadcum = 'Detail';
        }
        $jenis = JenisPakaianDinas::all();
        $genders = ['SEMUA', 'L', 'P'];
        $data = [
            'model' => $model,
            'isNew' => $isNew,
            'jenisPakaianOptions' => MyHelper::generateSelectOptions(['data' => $jenis, 'text' => 'nama', 'value' => 'id', 'selected' => $model['ms_jenis_pakaian_dinas_id'] ?? null]),
            'jenisUkuranOptions' => MyHelper::generateSelectOptions([
                'data' => Master::getMsUkuranGroup(true),
                'text' => 'group',
                'value' => 'group',
                'selected' => $model['ms_ukuran_group'] ?? null,
            ]),
            'fotos' => $fotos,
            'genderOptions' => MyHelper::generateSelectOptions(['data' => $genders, 'selected' => 'SEMUA']),
            'breadcums' => array_merge($this->breadcums, [$breadcum]),
            'judul' => $breadcum,
            'readOnly' => $readOnly,
        ];

        // dd($data['satkerOptions']);
        return $data;
    }

    /**
     * Show the form for creating a new resource.
     */
    public function create()
    {
        $data = $this->getData();

        return view('master.pakaian-dinas.spesifikasi-pakaian-dinas.formV', $data);
    }

    /**
     * Store a newly created resource in storage.
     */
    public function store(Request $request)
    {
        $customMessages = [
            'nama.required' => 'Nama harus diisi ',
            'ms_jenis_pakaian_dinas_id.required' => 'Jenis Pakaian harus diisi ',
        ];
        $rules = [
            'nama' => 'required',
            'gender' => 'required',
            'ms_jenis_pakaian_dinas_id' => 'required',
        ];

        $request->validate($rules, $customMessages);

        try {
            DB::beginTransaction();
            $id = $request->input('id') ?? MyHelper::getPk(date('Ymd'), 'ms_spesifikasi_pakaian_dinas_seq');
            Model::updateOrCreate(['id' => $id], $request->input());
            $params = [
                'kategori' => 'Spesifikasi Pakaian Dinas',
                'dir' => 'pakaian-dinas/spesifikasi',
                'fileKey' => 'foto',
                'pkey' => $id,
            ];
            $fotos = Files::upload($request, $params);

            // if ($request->hasFile('foto')) {
            //     foreach ($request->file('foto') as $foto) {
            //         $fileName = $id . '-' . uniqid() . '.' . $foto->getClientOriginalExtension();
            //         $storedPath = $foto->move('uploads/spek-pakaian-dinas', $fileName);
            //         $fotos[] = [
            //             'ms_spesifikasi_pakaian_dinas_id' => $id,
            //             'path' => $storedPath,
            //             'filename' => $foto->getClientOriginalName()
            //         ];
            //     }
            // }

            $deleted = $request->input('deleted');
            if (! empty($deleted)) {
                SpesifikasiPakaianDinasFoto::whereIn('id', $deleted)->delete();
            }

            if (! empty($fotos)) {
                foreach ($fotos as $foto) {
                    $newFotos[] = [
                        'ms_spesifikasi_pakaian_dinas_id' => $params['pkey'],
                        'path' => $foto['path'],
                        'filename' => $foto['filename'],
                    ];
                }
                SpesifikasiPakaianDinasFoto::insert($newFotos);
            }
            DB::commit();

            return $this->resSuccess();
        } catch (\Throwable $th) {
            dd($th);
            DB::rollBack();

            return $this->resError($th->getMessage());
        }
    }

    /**
     * Display the specified resource.
     */
    public function show(string $id)
    {
        $data = $this->getData($id);

        return view('master.pakaian-dinas.spesifikasi-pakaian-dinas.formV', $data);
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
