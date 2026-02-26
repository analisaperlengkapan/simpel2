<?php

namespace App\Http\Controllers\Bmn\Wasdal;

use App\Exports\ExportExcel;
use App\Helpers\MyHelper;
use App\Http\Controllers\Controller;
use App\Models\Bmn\Wasdal\Sewabmn;
use App\Models\Master;
use App\Models\Files;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\DB;
use Maatwebsite\Excel\Facades\Excel;
use Symfony\Component\HttpKernel\Exception\NotFoundHttpException;

class SewabmnController extends Controller
{
    /**
     * Display a listing of the resource.
     */
    protected $breadcums = ['BMN', 'Wasdal Sewa BMN'];
    protected $controller = '/bmn/wasdal/sewabmn';
    public function index()
    {
        $columns = ['Nama Satker', 'No. SK', 'Tgl. SK', 'Jenis Aset', 'Total BMN', 'Nilai Penetapan', 'Uraian', 'Tgl. Rekam'];
        $defColumns = array_keys($columns);
        $data = [
            'tableId' => 'dt-sewabmn',
            'breadcums' => $this->breadcums,
            'columns' => $columns,
            'canChange' => $this->canChange(),
            'defColumns' => $defColumns,
            'controller' => $this->controller
        ];
        return view('bmn.wasdal.sewabmnV', $data);
    }

    public function gridData(Request $request)
    {
        $model = new Sewabmn();
        $pagingParams = $request->only(['start', 'length']);
        $searchParams = $request->only(['columns']);
        $data = $model->getDataGrid($pagingParams, $searchParams);
        return response()->json([
            'data' => $data['data'],
            'recordsTotal' => $data['total'],
            'recordsFiltered' => $data['total'],
        ]);
    }

    function canChange()
    {
        return MyHelper::isSuperAdmin() || MyHelper::isValidatorPusat();
    }
    function getData($id = null)
    {
        $model = [];
        $isNew = true;
        $breadcum = 'Tambah';
        if ($id) {
            $breadcum = 'Detail Data Wasdal PSP BMN';
            $model = Sewabmn::where('id', $id)->first();
            if (!$model) {
                throw new NotFoundHttpException('Data Tidak Ditemukan');
            }
        }
        $satkers = Master::getSatkersKeu();
        $sewabmn = new Sewabmn();

        $detail = $sewabmn->getDataDetail($model['id_sk']);

        $data = [
            'canChange' => $this->canChange(),
            'model' => $model,
            'isNew' => $isNew,
            'breadcums' => array_merge($this->breadcums, [$breadcum]),
            'judul' => $breadcum,
            'controller' => $this->controller,
            'detail' => $detail,

            // 'satkerOptions' => MyHelper::generateSelectOptions([
            //     'data' => $satkers,
            //     'text' => 'inst_nama',
            //     'textkode' => 'kdsatker_keu',
            //     'value' => 'kdsatker_keu',
            //     'selected' => $model['kdsatker_keu'] ?? null,
            // ]),
            // 'listBarang' => MyHelper::generateSelectOptions([
            //     'data' => Master::getBarangAset(),
            //     'text' => 'nama_barang',
            //     'textkode' => 'kode_barang',
            //     'value' => 'kode_barang',
            //     'selected' => $model['kode_barang'] ?? null,
            // ]),
            // 'listBarang' => Master::getBarangAset(),
            
        ];
        return $data;
    }

    /**
     * Show the form for creating a new resource.
     */
    public function create()
    {
        $data = $this->getData();
        return view('bmn.asuransi.asuransiFormV', $data);
    }

    /**
     * Store a newly created resource in storage.
     */
    public function store(Request $request)
    {
        $isNew = $request->has('id');

        $id = $request->input('id') ?? MyHelper::getPk(date('Ymd'), 'asuransi_transaksi_id_seq');
        $validate = [
            //'jenis_sk' => 'required',
            'polis_no' => 'required',
            'polis_tgl' => 'required',
            'polis_premi' => 'required',
        ];
        $request->validate($validate);
        try {
            DB::beginTransaction();
            $params = [
                'kategori' => 'Asuransi BMN',
                'dir' => 'bmn/asuransi',
                'fileKey' => 'filename',
                'required' => $isNew,
                'pkey' => $id
            ];
            $file = Files::upload($request, $params);
            $inputan = $request->input();
            if ($file['path'] ?? null) {
                $inputan['filename'] = $file['path'];
            }
            $inputan['polis_premi'] = MyHelper::money2int($inputan['polis_premi']);
            $inputan['ms_satker_id'] = session('userData.current_role.ms_satker_id');
            $inputan = AsuransiTransaksi::updateOrCreate(['id_asset' => $inputan['id_asset']], $inputan);
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
        return view('bmn.wasdal.sewabmnViewV', $data);
    }

    /**
     * Show the form for editing the specified resource.
     */
    public function edit(string $id)
    {
        $data = $this->getData($id);
        return view('bmn.wasdal.penertibanViewV', $data);
    }

    /**
     * Update the specified resource in storage.
     */
    public function update(Request $request, Asuransi $hakcipta)
    {

    }

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
        $model = new AsuransiTransaksi();
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
