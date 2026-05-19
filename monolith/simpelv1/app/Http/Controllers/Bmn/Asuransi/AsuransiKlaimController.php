<?php

namespace App\Http\Controllers\Bmn\Asuransi;

use App\Exports\ExportExcel;
use App\Helpers\MyHelper;
use App\Http\Controllers\Controller;
use App\Models\Bmn\Asuransi;
use App\Models\Bmn\AsuransiImport;
use App\Models\Bmn\AsuransiTransaksi;
use App\Models\Bmn\AsuransiTransaksiKlaim as Model;
use App\Models\Files;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\DB;
use Maatwebsite\Excel\Facades\Excel;
use Symfony\Component\HttpKernel\Exception\NotFoundHttpException;

class AsuransiKlaimController extends Controller
{
    /**
     * Display a listing of the resource.
     */
    protected $breadcums = ['BMN', 'Klaim Asuransi'];

    protected $controller = '/bmn/asuransi/klaim';

    public function index()
    {
        $columns = ['Nama Satker', 'Kode Barang', 'NUP', 'Nama Aset', 'No Polis', 'Tgl Polis', 'Premi'];
        $defColumns = array_keys($columns);
        $data = [
            'tableId' => 'dt-obyekasuransi',
            'breadcums' => $this->breadcums,
            'columns' => $columns,
            'defColumns' => $defColumns,
            'controller' => $this->controller,
        ];

        return view('bmn.asuransi.claim.gridV', $data);
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

    public function getData($id = null)
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
        $polises = AsuransiTransaksi::where('ms_satker_id', session('userData.current_role.ms_satker_id'))->get();
        $data = [
            'model' => $model,
            'isNew' => $isNew,
            'breadcums' => array_merge($this->breadcums, [$breadcum]),
            'judul' => $breadcum,
            'controller' => $this->controller,
            'polisOptions' => MyHelper::generateSelectOptions([
                'data' => $polises,
                'text' => 'polis_no',
                'value' => 'id',
                'selected' => $model['asuransi_transaksi_id'] ?? null,
            ]),
        ];

        return $data;
    }

    /**
     * Show the form for creating a new resource.
     */
    public function create()
    {
        $data = $this->getData();

        return view('bmn.asuransi.claim.formV', $data);
    }

    /**
     * Store a newly created resource in storage.
     */
    public function store(Request $request)
    {
        $isNew = $request->has('id');

        $id = $request->input('id') ?? MyHelper::getPk(date('Ymd'), 'asuransi_transaksi_klaim_id_seq');
        $validate = [
            'asuransi_transaksi_id' => 'required',
            'surat_no' => 'required',
            'surat_tgl' => 'required',
        ];
        $request->validate($validate);
        try {
            DB::beginTransaction();
            $params = [
                'kategori' => 'Klaim Asuransi BMN',
                'dir' => 'bmn/asuransi-klaim',
                'fileKey' => 'filename',
                'required' => $isNew,
                'pkey' => $id,
            ];
            $file = Files::upload($request, $params);
            $inputan = $request->input();
            if ($file['path'] ?? null) {
                $inputan['filename'] = $file['path'];
            }
            $inputan['ms_satker_id'] = session('userData.current_role.ms_satker_id');
            $inputan = Model::updateOrCreate(['id' => $id], $inputan);
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
        $data = $this->getData($id);

        return view('bmn.asuransi.claim.formV', $data);
    }

    /**
     * Show the form for editing the specified resource.
     */
    public function edit(string $id)
    {
        $data = $this->getData($id);

        return view('bmn.asuransi.claim.formV', $data);
    }

    /**
     * Update the specified resource in storage.
     */
    public function update(Request $request, Asuransi $hakcipta) {}

    /**
     * Remove the specified resource from storage.
     */
    public function importExcel(Request $request)
    {
        $request->validate([
            'excel_file' => 'required|mimes:xlsx,xls',
        ]);
        $file = $request->file('excel_file');
        Excel::import(new AsuransiImport, $file);

        return $this->resSuccess('Berhasil Dismpan', ['type' => 'redirect', 'url' => $this->controller]);
    }

    public function exportExcel(Request $request)
    {
        $model = new AsuransiTransaksi;
        $params = [
            'length' => -1,
            'start' => 0,
        ];
        $searchParams = $request->only(['columns']);
        $grid = $model->getDataGrid($params, $searchParams);
        $data = $grid['data']->toArray();
        $columns = array_keys((array) $data[0]);
        $fileName = 'Objek Asuransi';

        return Excel::download(new ExportExcel($data, $columns, $fileName), "{$fileName}.xlsx", \Maatwebsite\Excel\Excel::XLSX);
    }
}
