<?php

namespace App\Http\Controllers\Aset;
use Illuminate\Routing\Controller;

use App\Exports\ExportExcel;
use App\Helpers\MyHelper;
use App\Models\Aset\Rumah;
use App\Models\Master\Master;
use Barryvdh\DomPDF\Facade\Pdf;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\DB;
use Maatwebsite\Excel\Facades\Excel;
use Mccarlosen\LaravelMpdf\Facades\LaravelMpdf;
use Symfony\Component\HttpKernel\Exception\NotFoundHttpException;

class RumahController extends Controller
{
    /**
     * Display a listing of the resource.
     */
    protected $breadcums = ['Aset'];
    protected $columns = ['Kode Satker', 'Nama Satker', 'Kode Barang', 'Nama Barang', 'NUP', 'Kondisi', 'Jenis Dokumen', 'Merk/Tipe', 'Tgl Rekam Pertama', 'Tgl Perolehan', 'Nilai Perolehan Pertama' ,'Nilai Mutasi' ,'Nilai Perolehan' ,'Nilai Penyusutan' ,'Nilai Buku' ,'Kuantitas' ,'Luas Bangunan' ,'Luas Dasar Bangunan','Jml Foto','Status Penggunaan','Status Pengelolaan','No. PSP','Tgl PSP','ALAMAT','Kota/Kabupaten','Kode Kab/Kota','Provinsi','Kode Provinsi','Kode Pos','Jumlah KIB','SBSK','OPTIMALISASI'];
    protected $defColumns = [0,1,2,3,4,5];
    protected $controller = '/asset/rumah';

    public function __construct()
    {
        $this->breadcums = array_merge($this->breadcums, [['link'=>$this->controller,'title'=>'Rumah Negara']]);
    }

    public function index()
    {
        return view('asset.rumah.rumahV', ['tableId' => 'dt-rumah', 'breadcums' => $this->breadcums, 'columns' => $this->columns, 'defColumns' => $this->defColumns, 'controller' => $this->controller]);
    }

    public function gridData(Request $request)
    {
        $model = new Rumah();
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
            $model = Rumah::where('id', $id)->first();
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
        return view('asset.rumah.rumahFormV', $data);
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
        ], $customMessages);
        $id = $request->input('id') ?? MyHelper::getPk(date('Ymd'), 'asset_rumah_negara_seq');
        Rumah::updateOrCreate(['id' => $id],$request->input());
        return $this->resSuccess();
    }

    /**
     * Display the specified resource.
     */
    public function show(string $id)
    {
        $data = $this->getData($id, true);
        return view('asset.rumah.rumahFormV', $data);
    }

    /**
     * Show the form for editing the specified resource.
     */
    public function edit(string $id)
    {
        $data = $this->getData($id);
        return view('asset.rumah.rumahFormV', $data);
    }

    /**
     * Update the specified resource in storage.
     */
    public function update(Request $request, Rumah $rumah)
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
            Rumah::destroy($id);
            DB::commit();
            return $this->resSuccess('Berhasil Dihapus!');
        } catch (\Throwable $th) {
            DB::rollBack();
        }
    }

    public function cetakLabel($id)
    {
        $data = Rumah::findOne($id);
        $data = (array) $data;
        $pdf = MyHelper::generateLabelBankAsset($data);
        return $pdf->stream('label-asset-rumah-negara.pdf');
    }

    public function cetakExcel(Request $request){
        $model = new Rumah();
        $searchParams =  $request->only(['columns']);
        $paging['length'] = -1;
        $paging['start'] = 1;
        $isKolom =  $request->input('isKolom');
        $select = array();
        $selectView = array();
        if($isKolom=='all'){
            $columns = \DB::getSchemaBuilder()->getColumnListing((new Rumah())->getTable());
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
        return Excel::download(new ExportExcel($data['data']->toArray(),$columns,'Daftar Aset Rumah Negara'), 'aset_rumah_negara.xlsx', \Maatwebsite\Excel\Excel::XLSX);
    }

    public function cetakPdf(Request $request){
        $model = new Rumah();
        $searchParams =  $request->only(['columns']);
        $data = $model->getDataExport($searchParams,$this->defColumns);
        $selectedColumns = array_intersect_key($this->columns, array_flip($this->defColumns));
        $pdf = MyHelper::generateAssetpdf('exports.asset',$selectedColumns,$data,$this->defColumns,'Daftar Aset Rumah Negara');
        return $pdf;
    }
}
