<?php

namespace App\Http\Controllers\Bmn\Pnbp;

use App\Helpers\MyHelper;
use App\Http\Controllers\Controller;
use App\Models\Bmn\Pnbp;
use App\Models\Master;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\DB;
use Symfony\Component\HttpKernel\Exception\NotFoundHttpException;

class PnbpController extends Controller
{
    /**
     * Display a listing of the resource.
     */
    // protected $breadcums = ['BMN', 'Monitoring BMN yang Menghasilkan PNBP'];
    protected $kategoriJudul = 'Monitoring BMN yang Menghasilkan PNBP';

    protected $controller = 'bmn/pnbp/pnbp';

    protected $breadcums = ['BMN'];

    protected $columns = ['Nama Satker', 'Kode barang', 'Tahun Anggaran', 'Potensi PNBP', 'Realisasi PNBP'];

    protected $defColumns = [0, 1, 2, 3, 4];

    public function __construct()
    {
        $this->breadcums = array_merge($this->breadcums, [['link' => $this->controller, 'title' => 'Monitoring BMN yang Menghasilkan PNBP']]);

    }

    public function index()
    {
        $columns = [
            'Nama Satker',
            'Kode Barang',
            'Tahun Anggaran',
            'Potensi PNBP',
            'Realisasi PNBP',
        ];
        $defColumns = [0, 1, 2, 3, 4];

        return view('bmn.pnbp.pnbpV', [
            'tableId' => 'dt-pnbp',
            'kategoriJudul' => $this->kategoriJudul,
            'breadcums' => $this->breadcums,
            'columns' => $columns,
            'defColumns' => $defColumns,
            'controller' => $this->controller,
        ]);
    }

    public function gridData(Request $request)
    {
        $model = new pnbp;
        $pagingParams = $request->only(['start', 'length']);
        // $searchParams =  $request->only(['search',  'filterBy']);
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
            $model = pnbp::where('id', $id)->first();
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
        $thn_ang = ['2018', '2019', '2020', '2021', '2022', '2023', '2024'];
        $kode_barang = ['0001', '0002', '0003', '0004', '0005', '0006', '0007'];
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
            'thn_ang' => MyHelper::generateSelectOptions([
                'data' => $thn_ang,
                'text' => 'tahun',
                'value' => null,
                'selected' => $model['thn_anggaran'] ?? null,
            ]),
            'kode_barang' => MyHelper::generateSelectOptions([
                'data' => $kode_barang,
                'text' => 'barang',
                'value' => null,
                'selected' => $model['kode_barang'] ?? null,
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

        return view('bmn.pnbp.pnbpFormV', $data);
    }

    /**
     * Store a newly created resource in storage.
     */
    public function store(Request $request)
    {
        $isNew = $request->input('isNew');
        $customMessages = [
            'kdsatker_keu.required' => 'Satker harus diisi',
            'thn_anggaran.required' => 'Tahun harus diisi',
            'kode_barang.required' => 'Jenis SK harus diisi',
        ];
        $validate = [
            'kdsatker_keu' => 'required',
            'kode_barang' => 'required',
        ];
        $id = $request->input('id') ?? MyHelper::getPk(date('Ymd'), 'bmn_pnbp_seq');

        $request->validate($validate, $customMessages);
        try {
            DB::beginTransaction();
            $data = [
                'kdsatker_keu' => $request->input('kdsatker_keu'),
                'thn_anggaran' => $request->input('thn_anggaran'),
                'kode_barang' => $request->input('kode_barang'),
                'potensi' => $request->input('potensi'),
                'realisasi' => $request->input('realisasi'),
            ];

            pnbp::updateOrCreate(['id' => $id], $data);

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

        return view('bmn.pnbp.pnbpFormV', $data);
    }

    /**
     * Show the form for editing the specified resource.
     */
    public function edit(string $id)
    {
        $data = $this->getData($id);

        return view('bmn.pnbp.pnbpFormV', $data);
    }

    /**
     * Update the specified resource in storage.
     */
    public function update(Request $request, pnbp $hakcipta)
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
            pnbp::destroy($id);
            DB::commit();

            return $this->resSuccess('Berhasil Dihapus!');
        } catch (\Throwable $th) {
            DB::rollBack();
        }
    }

    public function cetakLabel($id)
    {
        $data = PenghapusanSk::findOne($id);
        $data = (array) $data;
        $pdf = MyHelper::generateLabelBankAsset($data);

        return $pdf->stream('label-asset-tak-berwujud.pdf');
    }
}
