<?php

namespace App\Http\Controllers\Asset;

use App\Helpers\MyHelper;
use App\Http\Controllers\Controller;
use App\Models\Asset\LaporMasalah;
use App\Models\Master;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\DB;
use Symfony\Component\HttpKernel\Exception\NotFoundHttpException;

class LaporMasalahController extends Controller
{
    /**
     * Display a listing of the resource.
     */
    protected $breadcums = ['Asset', 'Tanah', 'Laporan Permasalahan'];

    public function index()
    {
        return view('asset.lapor.laporV', ['tableId' => 'dt-lapor', 'breadcums' => $this->breadcums]);
    }

    public function gridData(Request $request)
    {
        $model = new LaporMasalah;
        $pagingParams = $request->only(['start', 'length']);
        $searchParams = $request->only(['search',  'filterBy']);
        $data = $model->getDataGrid($pagingParams, $searchParams, $kategori, $id_asset);

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
}
