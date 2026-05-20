<?php

namespace App\Http\Controllers\Master\PakaianDinas;

use App\Helpers\MyHelper;
use App\Http\Controllers\Controller;
use App\Models\Master\PakaianDinas\JenisPakaianDinas as Model;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\DB;
use Symfony\Component\HttpKernel\Exception\NotFoundHttpException;

class JenisPakaianDinasController extends Controller
{
    /**
     * Display a listing of the resource.
     */
    // protected $breadcums = ['Master', 'Pakaian Dinas', 'Jenis Pakaian Dinas'];
    protected $kategoriJudul = 'Jenis Pakaian Dinas';

    protected $controller = 'master/pakaian-dinas/jenis-pakaian-dinas';

    protected $breadcums = ['Master'];

    protected $columns = ['No', 'Nama'];

    protected $defColumns = [0, 1];

    public function __construct()
    {
        $this->breadcums = array_merge($this->breadcums, [['link' => $this->controller, 'title' => 'Jenis Pakaian Dinas']]);

    }

    public function index()
    {
        // return view('master.pakaian-dinas.jenis-pakaian-dinas.gridV', ['tableId' => 'dt-jenis', 'breadcums' => $this->breadcums]);
        return view('master.pakaian-dinas.jenis-pakaian-dinas.gridV', [
            'tableId' => 'dt-jenis',
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
        // $searchParams =  $request->only(['search',  'filterBy']);
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
        if ($id) {
            $breadcum = 'Ubah';
            $model = Model::where('id', $id)->first();
            if (! $model) {
                throw new NotFoundHttpException('Data Tidak Ditemukan');
            }

            $model = $model->toArray();
            $isNew = false;
        }
        if ($readOnly) {
            $breadcum = 'Detail';
        }
        $data = [
            'model' => $model,
            'isNew' => $isNew,
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

        return view('master.pakaian-dinas.jenis-pakaian-dinas.formV', $data);
    }

    /**
     * Store a newly created resource in storage.
     */
    public function store(Request $request)
    {
        $customMessages = [
            'nama.required' => 'Nama harus diisi ',
        ];
        $request->validate([
            'nama' => 'required',
        ], $customMessages);
        $id = $request->input('id') ?? MyHelper::getPk(date('Ymd'), 'ms_jenis_pakaian_dinas_seq');
        Model::updateOrCreate(['id' => $id], $request->input());

        return $this->resSuccess();
    }

    /**
     * Display the specified resource.
     */
    public function show(string $id)
    {
        $data = $this->getData($id);

        return view('master.pakaian-dinas.jenis-pakaian-dinas.formV', $data);
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
