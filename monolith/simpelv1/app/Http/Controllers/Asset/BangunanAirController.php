<?php

namespace App\Http\Controllers\Asset;

use App\Exports\ExportExcel;
use App\Helpers\MyHelper;
use App\Http\Controllers\Controller;
use App\Models\Asset\BangunanAir;
use App\Models\Master;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\DB;
use Maatwebsite\Excel\Facades\Excel;
use Symfony\Component\HttpKernel\Exception\NotFoundHttpException;

class BangunanAirController extends Controller
{
    /**
     * Display a listing of the resource.
     */
    protected $breadcums = ['Aset'];

    protected $columns = ['Kode Satker', 'Nama Satker', 'Kode Barang', 'Nama Barang', 'NUP', 'Kondisi', 'Merk/Tipe', 'Tgl Rekam Pertama', 'Tgl Perolehan', 'Nilai Perolehan Pertama', 'Nilai Mutasi', 'Nilai Perolehan', 'Nilai Penyusutan', 'Nilai Buku', 'Kuantitas', 'Luas Bangunan', 'Luas Dasar', 'Jml Foto', 'Status Penggunaan', 'Status Pengelolaan', 'No. PSP', 'Tgl PSP', 'Jumlah KIB'];

    protected $defColumns = [0, 1, 2, 3, 4, 5];

    protected $controller = '/asset/bangunan_air';

    public function __construct()
    {
        $this->breadcums = array_merge($this->breadcums, [['link' => $this->controller, 'title' => 'Bangunan Air']]);
    }

    public function index()
    {
        return view('asset.air.airV', ['tableId' => 'dt-air', 'breadcums' => $this->breadcums, 'columns' => $this->columns, 'defColumns' => $this->defColumns, 'controller' => $this->controller]);
    }

    public function gridData(Request $request)
    {
        $model = new BangunanAir;
        $pagingParams = $request->only(['start', 'length']);
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
            $model = BangunanAir::where('id', $id)->first();
            if (! $model) {
                throw new NotFoundHttpException('Data Tidak Ditemukan');
            }

            $model = $model->toArray();
            $isNew = false;
        }
        if ($readOnly) {
            $breadcum = 'Detail';
        }
        $kondisi = ['Baik', 'Rusak Ringan', 'Rusak Berat'];
        $satkers = Master::getSatkersKeu();
        $data = [
            'model' => $model,
            'isNew' => $isNew,
            'breadcums' => array_merge($this->breadcums, [$breadcum]),
            'judul' => $breadcum,
            'readOnly' => $readOnly,
            'kondisiOptions' => MyHelper::generateSelectOptions([
                'data' => $kondisi,
                'text' => 'kondisi',
                'value' => null,
                'selected' => $model['kondisi'] ?? null,
            ]),
            'satkerOptions' => MyHelper::generateSelectOptions([
                'data' => $satkers,
                'text' => 'inst_nama',
                'textkode' => 'kdsatker_keu',
                'value' => 'kdsatker_keu',
                'selected' => $model['kdsatker_keu'] ?? null,
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

        return view('asset.air.airFormV', $data);
    }

    /**
     * Store a newly created resource in storage.
     */
    public function store(Request $request)
    {
        $customMessages = [
            'kdsatker_keu.required' => 'Kode Satker harus dipilih '.$request->input('kdsatker_keu'),
        ];
        $request->validate([
            'kdsatker_keu' => 'required',
            'kode_barang' => 'required',
            'nm_barang' => 'required',
        ], $customMessages);
        $id = $request->input('id') ?? MyHelper::getPk(date('Ymd'), 'asset_bangunan_air_seq');
        BangunanAir::updateOrCreate(['id' => $id], $request->input());

        return $this->resSuccess();
    }

    /**
     * Display the specified resource.
     */
    public function show(string $id)
    {
        $data = $this->getData($id, true);

        return view('asset.air.airFormV', $data);
    }

    /**
     * Show the form for editing the specified resource.
     */
    public function edit(string $id)
    {
        $data = $this->getData($id);

        return view('asset.air.airFormV', $data);
    }

    /**
     * Update the specified resource in storage.
     */
    public function update(Request $request, BangunanAir $air)
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
            BangunanAir::destroy($id);
            DB::commit();

            return $this->resSuccess('Berhasil Dihapus!');
        } catch (\Throwable $th) {
            DB::rollBack();
        }
    }

    public function cetakLabel($id)
    {
        $data = BangunanAir::findOne($id);
        $data = (array) $data;
        $pdf = MyHelper::generateLabelBankAsset($data);

        return $pdf->stream('label-asset-tanah.pdf');
    }

    public function cetakExcel(Request $request)
    {
        $model = new BangunanAir;
        $searchParams = $request->only(['columns']);
        $paging['length'] = -1;
        $paging['start'] = 1;
        $isKolom = $request->input('isKolom');
        $select = [];
        $selectView = [];
        if ($isKolom == 'all') {
            $columns = \DB::getSchemaBuilder()->getColumnListing((new BangunanAir)->getTable());
        } else {
            $visible = explode(',', $request->input('visible'));
            foreach ($visible as $key) {
                if (isset($searchParams['columns'][$key]['data'])) {
                    $kolomSelect = 'a.'.$searchParams['columns'][$key]['data'];
                    $kolomView = $searchParams['columns'][$key]['data'];
                    array_push($select, $kolomSelect);
                    array_push($selectView, $kolomView);
                }
            }
            $columns = $selectView;
        }
        $data = $model->getDataGrid($paging, $searchParams, $select);

        return Excel::download(new ExportExcel($data['data']->toArray(), $columns, 'Daftar Aset Bangunan Air'), 'aset_bangunan_air.xlsx', \Maatwebsite\Excel\Excel::XLSX);
    }

    public function cetakPdf(Request $request)
    {
        $model = new BangunanAir;
        $searchParams = $request->only(['columns']);
        $data = $model->getDataExport($searchParams, $this->defColumns);
        $selectedColumns = array_intersect_key($this->columns, array_flip($this->defColumns));
        $pdf = MyHelper::generateAssetpdf('exports.asset', $selectedColumns, $data, $this->defColumns, 'Daftar Aset Bangunan Air');

        return $pdf;
    }
}
