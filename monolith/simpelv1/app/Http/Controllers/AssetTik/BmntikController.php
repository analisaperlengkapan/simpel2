<?php

namespace App\Http\Controllers\AssetTik;

use App\Helpers\MyHelper;
use App\Http\Controllers\Controller;
use App\Models\AssetTik\Bmntik;
use App\Models\Master;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\DB;
use Symfony\Component\HttpKernel\Exception\NotFoundHttpException;

class BmntikController extends Controller
{
    /**
     * Display a listing of the resource.
     */
    protected $breadcums = ['Asset TIK'];

    protected $columns = ['Nama Satker', 'Kode Barang', 'Nama Barang', 'NUP', 'Kondisi', 'Merk/Tipe', 'Tgl Rekam Pertama', 'Tgl Perolehan', 'Nilai Perolehan Pertama', 'Nilai Mutasi', 'Nilai Perolehan', 'Nilai Penyusutan', 'Nilai Buku', 'Kuantitas', 'Jml Foto', 'Status Penggunaan', 'Status Pengelolaan', 'No. PSP', 'Tgl PSP'];

    protected $defColumns = [0, 1, 2, 3, 4, 5, 6];

    protected $controller = '/asset-tik/bmntik';

    public function __construct()
    {
        $this->breadcums = array_merge($this->breadcums, [['link' => $this->controller, 'title' => 'Daftar Aset BMN Khusus TIK']]);
    }

    public function index()
    {
        $data = ['tableId' => 'dt-bmntik', 'breadcums' => $this->breadcums, 'columns' => $this->columns, 'defColumns' => $this->defColumns, 'controller' => $this->controller];

        return view('asset_tik.bmntik.bmntikV', $data);
    }

    public function gridData(Request $request)
    {
        $model = new Bmntik;
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
            $model = Bmntik::where('id', $id)->first();
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

        return view('asset_tik.hakcipta.hakciptaFormV', $data);
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
            'nm_lisensi' => 'required',
            'nm_lisensi' => 'required',
        ], $customMessages);
        $id = $request->input('id') ?? MyHelper::getPk(date('Ymd'), 'asset_tik_hakcipta_seq');
        Bmntik::updateOrCreate(['id' => $id], $request->input());

        return $this->resSuccess();
    }

    /**
     * Display the specified resource.
     */
    public function show(string $id)
    {
        $data = $this->getData($id, true);

        return view('asset_tik.bmntik.bmntikFormV', $data);
    }

    /**
     * Show the form for editing the specified resource.
     */
    public function edit(string $id)
    {
        $data = $this->getData($id);

        return view('asset_tik.bmntik.bmntikFormV', $data);
    }

    /**
     * Update the specified resource in storage.
     */
    public function update(Request $request, Hakcipta $hakcipta)
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
            Hakcipta::destroy($id);
            DB::commit();

            return $this->resSuccess('Berhasil Dihapus!');
        } catch (\Throwable $th) {
            DB::rollBack();
        }
    }

    public function cetakLabel($id)
    {
        $data = Hakcipta::findOne($id);
        $data = (array) $data;
        $pdf = MyHelper::generateLabelBankAsset($data);

        return $pdf->stream('label-asset-tak-berwujud.pdf');
    }
}
