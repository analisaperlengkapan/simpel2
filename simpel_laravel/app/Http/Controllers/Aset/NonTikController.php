<?php

namespace App\Http\Controllers\Aset;
use Illuminate\Routing\Controller;

use App\Exports\ExportExcel;
use App\Helpers\MyHelper;
use App\Models\Aset\NonTik;
use App\Models\Master\Master;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\DB;
use Maatwebsite\Excel\Facades\Excel;
use Symfony\Component\HttpKernel\Exception\NotFoundHttpException;

class NonTikController extends Controller
{
    /**
     * Display a listing of the resource.
     */
    protected $breadcums = ['Aset'];
    protected $columns = ['Kode Satker', 'Nama Satker', 'Kode Barang', 'Nama Barang', 'NUP', 'Kondisi', 'Merk/Tipe', 'Tgl Rekam Pertama', 'Tgl Perolehan', 'Nilai Perolehan Pertama' ,'Nilai Mutasi' ,'Nilai Perolehan' ,'Nilai Penyusutan' ,'Nilai Buku' ,'Kuantitas','Jml Foto','Status Penggunaan','Status Pengelolaan','No. PSP','Tgl PSP','Jumlah KIB'];
    protected $defColumns = [0,1,2,3,4,5];
    protected $controller = '/asset/non_tik';

    public function __construct()
    {
        $this->breadcums = array_merge($this->breadcums, [['link'=>$this->controller,'title'=>'Peralatan dan Mesin Non TIK']]);
    }

    public function index()
    {
        $columns = [
            'Kode Satker',
            'Nama Satker',
            'Kode Barang',
            'Nama Barang',
            'NUP',
            'Kondisi',
            'Merk/Tipe',
            'Tgl Rekam Pertama',
            'Tgl Perolehan',
            'Nilai Perolehan Pertama',
            'Nilai Mutasi',
            'Nilai Perolehan',
            'Nilai Penyusutan',
            'Nilai Buku',
            'Kuantitas',
            'Jml Foto',
            'Status Penggunaan',
            'Status Pengelolaan',
            'No. PSP',
            'Tgl PSP',
            'Jumlah KIB',
        ];
        $defColumns = [0, 1, 2, 3, 4, 5];
        return view('asset.non_tik.non_tikV', [
            'tableId' => 'dt-non',
            'breadcums' => $this->breadcums,
            'columns' => $columns,
            'defColumns' => $defColumns,
            'controller' => $this->controller
        ]);
    }

    public function gridData(Request $request)
    {
        $model = new NonTik();
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
        $breadcum = 'Tambah';
        if ($id) {
            $breadcum = 'Ubah';
            $model = NonTik::where('id', $id)->first();

            if (!$model) {
                throw new NotFoundHttpException('Data Tidak Ditemukan');
            }

            $model = $model->toArray();
            $isNew = false;
        }
        if($readOnly){
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
            'satkerOptions' => MyHelper::generateSelectOptions([
                'data' => $satkers,
                'text' => 'inst_nama',
                'textkode' => 'kdsatker_keu',
                'value' => 'kdsatker_keu',
                'selected' => $model['kdsatker_keu'] ?? null,
            ]),
            'kondisiOptions' => MyHelper::generateSelectOptions([
                'data' => $kondisi,
                'text' => 'kondisi',
                'value' => null,
                'selected' => $model['kondisi'] ?? null,
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
        return view('asset.non_tik.non_tikFormV', $data);
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
            'nm_barang' => 'required'
        ],$customMessages);
        $id = $request->input('id') ?? MyHelper::getPk(date('Ymd'), 'asset_peralatan_mesin_non_tik_seq');
        NonTik::updateOrCreate(['id' => $id],$request->input());
        return $this->resSuccess();
    }

    /**
     * Display the specified resource.
     */
    public function show(string $id)
    {
        $data = $this->getData($id, true);
        return view('asset.non_tik.non_tikFormV', $data);
    }

    /**
     * Show the form for editing the specified resource.
     */
    public function edit(string $id)
    {
        $data = $this->getData($id);
        return view('asset.non_tik.non_tikFormV', $data);
    }

    /**
     * Update the specified resource in storage.
     */
    public function update(Request $request, NonTik $non)
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
            NonTik::destroy($id);
            DB::commit();
            return $this->resSuccess('Berhasil Dihapus!');
        } catch (\Throwable $th) {
            DB::rollBack();
        }
    }

    public function cetakLabel($id)
    {
        $data = NonTik::findOne($id);
        $data = (array) $data;
        $pdf = MyHelper::generateLabelBankAsset($data);
        return $pdf->stream('label-asset-non-tik.pdf');
    }

    public function cetakExcel(Request $request){
        $model = new NonTik();
        $searchParams =  $request->only(['columns']);
        $paging['length'] = -1;
        $paging['start'] = 1;
        $isKolom =  $request->input('isKolom');
        $select = array();
        $selectView = array();
        if($isKolom=='all'){
            $columns = \DB::getSchemaBuilder()->getColumnListing((new NonTik())->getTable());
        }else{
            $visible = explode(',', $request->input('visible'));
            foreach ($visible as $key) {
                if (isset($searchParams["columns"][$key]["data"])) {
                    $kolomSelect = 'a.'.$searchParams["columns"][$key]["data"];
                    $kolomView = $searchParams["columns"][$key]["data"];
                    array_push($select,$kolomSelect);
                    array_push($selectView,$kolomView);
                }
            }
            $columns = $selectView;
        }
        $data = $model->getDataGrid($paging, $searchParams, $select);
        return Excel::download(new ExportExcel($data['data']->toArray(),$columns,'Daftar Aset Peralatan dan Mesin Non TIK'), 'aset_peralatan_mesin_non_tik.xlsx', \Maatwebsite\Excel\Excel::XLSX);
    }

    public function cetakPdf(Request $request){
        $model = new NonTik();
        $searchParams =  $request->only(['columns']);
        $data = $model->getDataExport($searchParams,$this->defColumns);
        $selectedColumns = array_intersect_key($this->columns, array_flip($this->defColumns));
        $pdf = MyHelper::generateAssetpdf('exports.asset',$selectedColumns,$data,$this->defColumns,'Daftar Aset Peralatan dan Mesin Non TIK');
        return $pdf;
    }
}
