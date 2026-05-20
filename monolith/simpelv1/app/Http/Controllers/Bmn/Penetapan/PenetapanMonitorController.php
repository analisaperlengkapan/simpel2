<?php

namespace App\Http\Controllers\Bmn\Penetapan;

use App\Exports\ExportExcel;
use App\Helpers\MyHelper;
use App\Http\Controllers\Controller;
use App\Models\Bmn\PenetapanSk;
use App\Models\Master;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\DB;
use Maatwebsite\Excel\Facades\Excel;
use Symfony\Component\HttpKernel\Exception\NotFoundHttpException;

class PenetapanMonitorController extends Controller
{
    /**
     * Display a listing of the resource.
     */
    protected $breadcums = ['BMN', 'Monitoring PSP'];

    public function index()
    {
        $columns = ['Nama Asset', 'Nama Barang', 'NUP', 'Nilai Perolehan', 'No PSP', 'Tgl PSP', 'SIMAN', 'Satker'];
        $defColumns = array_keys($columns);
        $data = [
            'tableId' => 'dt-penetapan',
            'breadcums' => $this->breadcums,
            'columns' => $columns,
            'defColumns' => $defColumns,
        ];

        return view('bmn.penetapansk.penetapanmonitorV', $data);
    }

    public function gridData(Request $request)
    {
        $model = new PenetapanSk;
        $pagingParams = $request->only(['start', 'length']);
        $searchParams = $request->only(['columns']);
        $data = $model->getDataGridMonitoring($pagingParams, $searchParams);

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
            $model = PenetapanSk::where('id', $id)->first();
            if (! $model) {
                throw new NotFoundHttpException('Data Tidak Ditemukan');
            }

            $model = $model->toArray();
            $isNew = false;
        }
        if ($readOnly) {
            $breadcum = 'Detail';
        }
        $satkers = Master::getSatkersKeu();
        $jenis = ['UAKPB', 'UAPPB-W', 'UAPPB-E1', 'UAPB'];
        $data = [
            'model' => $model,
            'isNew' => $isNew,
            'breadcums' => array_merge($this->breadcums, [$breadcum]),
            'judul' => $breadcum,
            'readOnly' => $readOnly,
            'satkerOptions' => MyHelper::generateSelectOptions([
                'data' => $satkers,
                'text' => 'inst_nama',
                'textkode' => 'kdsatker_keu',
                'value' => 'kdsatker_keu',
                'selected' => $model['kdsatker_keu'] ?? null,
            ]),
            'jenisOptions' => MyHelper::generateSelectOptions([
                'data' => $jenis,
                'text' => 'jenis',
                'value' => null,
                'selected' => $model['jenis_sk'] ?? null,
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

        return view('bmn.penetapansk.penetapanFormV', $data);
    }

    /**
     * Store a newly created resource in storage.
     */
    public function store(Request $request)
    {
        $isNew = $request->input('isNew');
        $customMessages = [
            'kdsatker_keu.required' => 'Satker harus diisi',
            'tgl_surat.required' => 'Tanggal Surat harus diisi',
        ];
        $validate = [
            'kdsatker_keu' => 'required',
            'tgl_surat' => 'required',
        ];
        $id = $request->input('id') ?? MyHelper::getPk(date('Ymd'), 'sdm_timakuntansibarang_seq');
        if ($isNew) {
            $validate['file_sk'] = 'required|mimes:jpeg,png,pdf|max:2048';
            $customMessages['file_spk.required'] = 'File SK harus diupload';
        } else {

        }
        $request->validate($validate, $customMessages);
        try {
            DB::beginTransaction();
            $data = [
                'kdsatker_keu' => $request->input('kdsatker_keu'),
                'tgl_surat' => $request->input('tgl_surat'),
                'dikeluarkan_di' => $request->input('dikeluarkan_di'),
            ];
            if ($request->hasFile('file_sk')) {
                $filepath = 'uploads/bmn/penetapansk';
                $file = $request->file('file_sk');
                $fileName = $id.'_sk'.'.'.$file->getClientOriginalExtension();
                $filesave = $filepath.'/'.$fileName;
                $file->move($filepath, $fileName);
                $data['file_sk'] = $filesave;
            }
            PenetapanSk::updateOrCreate(['id' => $id], $data);

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

        return view('bmn.penetapansk.penetapanFormV', $data);
    }

    /**
     * Show the form for editing the specified resource.
     */
    public function edit(string $id)
    {
        $data = $this->getData($id);

        return view('bmn.penetapansk.penetapanFormV', $data);
    }

    /**
     * Update the specified resource in storage.
     */
    public function cetakExcel(Request $request)
    {
        $model = new PenetapanSk;
        // $searchParams = $request->only(['columns']);
        $paging['length'] = -1;
        $paging['start'] = 1;
        // $isKolom = $request->input('isKolom');
        // $select = array();
        $data = $model->getDataGridMonitoring($paging, []);
        $data = $data['data']->toArray();
        $columns = array_keys((array) $data[0]);

        return Excel::download(new ExportExcel($data, $columns, 'Monitoring PSP'), 'Monitoring PSP.xlsx', \Maatwebsite\Excel\Excel::XLSX);
    }

    /**
     * Remove the specified resource from storage.
     */
    public function destroy(string $id)
    {
        try {
            DB::beginTransaction();
            PenetapanSk::destroy($id);
            DB::commit();

            return $this->resSuccess('Berhasil Dihapus!');
        } catch (\Throwable $th) {
            DB::rollBack();
        }
    }

    public function exportExcel(Request $request)
    {
        $model = new PenetapanSk;
        $params = [
            'length' => -1,
            'start' => 0,
        ];
        $searchParams = $request->only(['columns']);
        $grid = $model->getDataGridMonitoring($params, $searchParams);
        $data = $grid['data']->toArray();
        $columns = array_keys((array) $data[0]);
        $fileName = 'Monitoring PSP';

        return Excel::download(new ExportExcel($data, $columns, $fileName), "{$fileName}.xlsx", \Maatwebsite\Excel\Excel::XLSX);
    }
}
