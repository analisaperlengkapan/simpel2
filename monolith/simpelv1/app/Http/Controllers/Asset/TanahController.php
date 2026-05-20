<?php

namespace App\Http\Controllers\Asset;

use App\Exports\ExportExcel;
use App\Helpers\MyHelper;
use App\Http\Controllers\Controller;
use App\Models\Asset\Tanah;
use App\Models\Master;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\DB;
use Maatwebsite\Excel\Facades\Excel;
use Symfony\Component\HttpKernel\Exception\NotFoundHttpException;

class TanahController extends Controller
{
    /**
     * Display a listing of the resource.
     */
    protected $controller = '/asset/tanah';

    protected $breadcums = ['Aset'];

    protected $columns = ['Kode Satker', 'Nama Satker', 'Kode Barang', 'Nama Barang', 'NUP', 'Kondisi', 'Jenis Dokumen', 'Kepemilikan', 'Jenis Sertifikat', 'Merk/Tipe', 'Tgl Rekam Pertama', 'Tgl Perolehan', 'Nilai Perolehan Pertama', 'Nilai Mutasi', 'Nilai Perolehan', 'Nilai Penyusutan', 'Nilai Buku', 'Kuantitas(m2)', 'Luas Tanah Seluruhnya', 'Luas Tanah Untuk Bangunan', 'Luas tanah Untuk Sarana Lingkungan', 'Luas Lahan Kosong', 'Jml Foto', 'Status Penggunaan', 'Status Pengelolaan', 'No. PSP', 'Tgl PSP', 'ALAMAT', 'RT/RW', 'Kelurahan/Desa', 'Kecamatan', 'Kota/Kabupaten', 'Kode Kab/Kota', 'Provinsi', 'Kode Provinsi', 'Kode Pos', 'Jumlah KIB', 'SBSK', 'OPTIMALISASI', 'Status SBSN'];

    protected $defColumns = [0, 1, 2, 3, 4, 5];

    public function __construct()
    {
        $this->breadcums = array_merge($this->breadcums, [['link' => $this->controller, 'title' => 'Tanah']]);
    }

    public function index()
    {
        // echo "<pre>"; print_r(session('userData.current_role'));
        // echo phpinfo();
        return view('asset.tanah.tanahV', ['tableId' => 'dt-tanah', 'breadcums' => $this->breadcums, 'columns' => $this->columns, 'defColumns' => $this->defColumns, 'controller' => $this->controller]);
    }

    public function gridData(Request $request)
    {
        $model = new Tanah;
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
            $model = Tanah::where('id', $id)->first();
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
        $jenis_dokumen = ['Sertifikat', 'Tidak Ada Dokumen Kepemilikan', 'Tidak Bersertifikat'];
        $jenis_sertifikat = ['HPL', 'SHGB', 'SHGU', 'SHM', 'SHP'];
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
            'jnsDokumenOptions' => MyHelper::generateSelectOptions([
                'data' => $jenis_dokumen,
                'text' => 'jenis_dokumen',
                'value' => null,
                'selected' => $model['jenis_dokumen'] ?? null,
            ]),
            'jnsSertifikatOptions' => MyHelper::generateSelectOptions([
                'data' => $jenis_sertifikat,
                'text' => 'jenis_sertifikat',
                'value' => null,
                'selected' => $model['jenis_sertifikat'] ?? null,
            ]),
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

        return view('asset.tanah.tanahFormV', $data);
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
        $id = $request->input('id') ?? MyHelper::getPk(date('Ymd'), 'asset_tanah_seq');
        Tanah::updateOrCreate(['id' => $id], $request->input());

        return $this->resSuccess();
    }

    /**
     * Display the specified resource.
     */
    public function show(string $id)
    {
        $data = $this->getData($id, true);

        return view('asset.tanah.tanahFormV', $data);
    }

    /**
     * Show the form for editing the specified resource.
     */
    public function edit(string $id)
    {
        $data = $this->getData($id);

        return view('asset.tanah.tanahFormV', $data);
    }

    /**
     * Update the specified resource in storage.
     */
    public function update(Request $request, Tanah $tanah)
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
            Tanah::destroy($id);
            DB::commit();

            return $this->resSuccess('Berhasil Dihapus!');
        } catch (\Throwable $th) {
            DB::rollBack();
        }
    }

    public function cetakLabel($id)
    {
        $data = Tanah::findOne($id);
        $data = (array) $data;
        $pdf = MyHelper::generateLabelBankAsset($data);

        return $pdf->stream('label-asset-tanah.pdf');
    }

    public function cetakExcel(Request $request)
    {
        $model = new Tanah;
        $searchParams = $request->only(['columns']);
        $paging['length'] = -1;
        $paging['start'] = 1;
        $isKolom = $request->input('isKolom');
        $select = [];
        $selectView = [];
        if ($isKolom == 'all') {
            $columns = \DB::getSchemaBuilder()->getColumnListing((new Tanah)->getTable());
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

        return Excel::download(new ExportExcel($data['data']->toArray(), $columns, 'Daftar Aset Tanah'), 'aset_tanah.xlsx', \Maatwebsite\Excel\Excel::XLSX);
    }

    public function cetakPdf(Request $request)
    {
        $model = new Tanah;
        $searchParams = $request->only(['columns']);
        $data = $model->getDataExport($searchParams, $this->defColumns);
        $selectedColumns = array_intersect_key($this->columns, array_flip($this->defColumns));
        $pdf = MyHelper::generateAssetpdf('exports.asset', $selectedColumns, $data, $this->defColumns, 'Daftar Aset Tanah');

        return $pdf;
    }

    // Untuk Pointing Maps Asset
    public function mapsSimpan(Request $request)
    {
        $customMessages = [
            'gps_longitude.required' => 'Longitude harus diisi',
            'gps_latitude.required' => 'Latitude harus diisi',
        ];
        $validasi = [
            'gps_longitude' => 'required',
            'gps_latitude' => 'required',
        ];
        $request->validate($validasi, $customMessages);

        try {
            DB::beginTransaction();
            $id = $request->input('id');
            $table = $request->input('jenis_aset');

            $data = [
                'gps_longitude' => $request->input('gps_longitude'),
                'gps_latitude' => $request->input('gps_latitude'),
            ];
            // $datagps = BmnSatkerBarang::updateOrCreate(['id' => $id], $data);

            DB::table($table)->where('id', $id)->update($data);

            DB::commit();

            return $this->resSuccess(
                'Berhasil Disimpan!'
            );

        } catch (\Throwable $th) {
            dd($th->getMessage());

            return $this->resError('Gagal menyimpan data');
        }
    }

    public function mapsDetail(Request $request)
    {
        $data = [
            'gps_longitude' => $request->input('gps_longitude'),
            'gps_latitude' => $request->input('gps_latitude'),
            'jenis_aset' => $request->input('jenis_aset'),
            'id' => $request->input('id_aset'),
        ];

        return view('monsakti.transaksi-aset.mapsdetailV', $data);
    }
    // End Untuk Pointing Maps Asset
}
