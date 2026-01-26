<?php

namespace App\Http\Controllers\Aset;
use Illuminate\Routing\Controller;

use App\Exports\ExportExcel;
use App\Helpers\MyHelper;
use App\Models\Aset\Konstruksi;
use App\Models\Master\Master;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\DB;
use Maatwebsite\Excel\Facades\Excel;
use Symfony\Component\HttpKernel\Exception\NotFoundHttpException;

class KonstruksiController extends Controller
{
    /**
     * Display a listing of the resource.
     */
    protected $breadcums = ['Aset'];
    protected $columns = ['Kode Satker', 'Nama Satker', 'Kode Barang', 'Nama Barang', 'NUP', 'Kondisi', 'Merk/Tipe', 'Tgl Perolehan', 'Nilai Perolehan Pertama' ,'Nilai Mutasi' ,'Nilai Perolehan' ,'Nilai Penyusutan' ,'Nilai Buku' ,'Kuantitas','Jml Foto','Status Penggunaan','Status Pengelolaan','No. PSP','Tgl PSP'];
    protected $defColumns = [0,1,2,3,4,5];
    protected $controller = '/asset/konstruksi';

    public function __construct()
    {
        $this->breadcums = array_merge($this->breadcums, [['link'=>$this->controller,'title'=>'Konstruksi Dalam Pengerjaan']]);
    }

    public function index()
    {
        return view('asset.konstruksi.konstruksiV', ['tableId' => 'dt-konstruksi', 'breadcums' => $this->breadcums, 'columns' => $this->columns, 'defColumns' => $this->defColumns, 'controller' => $this->controller]);
    }

    public function gridData(Request $request)
    {
        $model = new Konstruksi();
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
            $model = Konstruksi::where('id', $id)->first();
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
        return view('asset.konstruksi.konstruksiFormV', $data);
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
        $id = $request->input('id') ?? MyHelper::getPk(date('Ymd'), 'asset_konstruksi_dalam_pengerjaan_seq');
        Konstruksi::updateOrCreate(['id' => $id],$request->input());
        return $this->resSuccess();
    }

    /**
     * Display the specified resource.
     */
    public function show(string $id)
    {
        $data = $this->getData($id, true);
        return view('asset.konstruksi.konstruksiFormV', $data);
    }

    /**
     * Show the form for editing the specified resource.
     */
    public function edit(string $id)
    {
        $data = $this->getData($id);
        return view('asset.konstruksi.konstruksiFormV', $data);
    }

    /**
     * Update the specified resource in storage.
     */
    public function update(Request $request, Konstruksi $konstruksi)
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
            Konstruksi::destroy($id);
            DB::commit();
            return $this->resSuccess('Berhasil Dihapus!');
        } catch (\Throwable $th) {
            DB::rollBack();
        }
    }

    public function cetakLabel($id)
    {
        $data = Konstruksi::findOne($id);
        $data = (array) $data;
        $pdf = MyHelper::generateLabelBankAsset($data);
        return $pdf->stream('label-asset-tanah.pdf');
    }

    public function cetakExcel(Request $request){
        $model = new Konstruksi();
        $searchParams =  $request->only(['columns']);
        $paging['length'] = -1;
        $paging['start'] = 1;
        $isKolom =  $request->input('isKolom');
        $select = array();
        $selectView = array();
        if($isKolom=='all'){
            $columns = \DB::getSchemaBuilder()->getColumnListing((new Konstruksi())->getTable());
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
        return Excel::download(new ExportExcel($data['data']->toArray(),$columns,'Daftar Aset Konstruksi Dalam Pengerjaan'), 'aset_konstruksi_dalam_pengerjaan.xlsx', \Maatwebsite\Excel\Excel::XLSX);
    }

    public function cetakPdf(Request $request){
        $model = new Konstruksi();
        $searchParams =  $request->only(['columns']);
        $data = $model->getDataExport($searchParams,$this->defColumns);
        $selectedColumns = array_intersect_key($this->columns, array_flip($this->defColumns));
        $pdf = MyHelper::generateAssetpdf('exports.asset',$selectedColumns,$data,$this->defColumns,'Daftar Aset Konstruksi Dalam Pengerjaan');
        return $pdf;
    }
}
