<?php

namespace App\Http\Controllers\Bmn\Perawatan;

use App\Helpers\MyHelper;
use App\Http\Controllers\Controller;
use App\Models\Bmn\Perawatan;
use App\Models\Master;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\DB;
use Symfony\Component\HttpKernel\Exception\NotFoundHttpException;

class PerawatanController extends Controller
{
    /**
     * Display a listing of the resource.
     */
    // protected $breadcums = ['BMN', 'Pengajuan Pemeliharaan BMN'];
    protected $kategoriJudul = 'Pengajuan Pemeliharaan BMN';

    protected $controller = 'bmn/perawatan/perawatan';

    protected $breadcums = ['BMN'];

    protected $columns = ['Nama Satker', 'Nomor', 'Tgl Perawatan', 'Kode Barang', 'Jenis', 'Pelaksana', 'Biaya', 'Status'];

    protected $defColumns = [0, 1, 2, 3, 4, 5, 6, 7];

    public function __construct()
    {
        $this->breadcums = array_merge($this->breadcums, [['link' => $this->controller, 'title' => 'Pengajuan Pemeliharaan BMN']]);

    }

    public function index()
    {
        // $data = ['tableId' => 'dt-pemeliharaan', 'breadcums' => $this->breadcums];
        // return view('bmn.perawatan.perawatanV', $data);
        return view('bmn.perawatan.perawatanV', [
            'tableId' => 'dt-pemeliharaan',
            'kategoriJudul' => $this->kategoriJudul,
            'breadcums' => $this->breadcums,
            'columns' => $this->columns,
            'defColumns' => $this->defColumns,
            'controller' => $this->controller,
        ]);
    }

    public function gridData(Request $request)
    {
        $model = new Perawatan;
        $pagingParams = $request->only(['start', 'length']);
        // $searchParams =  $request->only(['search',  'filterBy']);
        $searchParams = $request->only(['columns']);
        $data = $model->getDataGrid($pagingParams, $searchParams);

        // dd($data);
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
            $model = Perawatan::where('id', $id)->first();
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
        $jns_perawatan = ['Ringan', 'Sedang', 'Berat'];
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
            'jns_perawatan' => MyHelper::generateSelectOptions([
                'data' => $jns_perawatan,
                'text' => 'tahun',
                'value' => null,
                'selected' => $model['jns_perawatan'] ?? null,
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

        return view('bmn.perawatan.perawatanFormV', $data);
    }

    /**
     * Store a newly created resource in storage.
     */
    public function store(Request $request)
    {
        $isNew = $request->input('isNew');
        $customMessages = [
            'kdsatker_keu.required' => 'Satker harus diisi',
            'jns_perawatan.required' => 'Jenis Perawatan harus diisi',
            'kode_barang.required' => 'Nomor Surat harus diisi',
            'tgl_perawatan.required' => 'Tanggal Surat harus diisi',
            'pelaksana.required' => 'Tanggal Surat harus diisi',
            'biaya.required' => 'Tanggal Surat harus diisi',
            'spesifikasi.required' => 'Tanggal Surat harus diisi',
        ];
        $validate = [
            'kdsatker_keu' => 'required',
            'jns_perawatan' => 'required',
            'kode_barang' => 'required',
            'tgl_perawatan' => 'required',
            'pelaksana' => 'required',
            'biaya' => 'required',
            'spesifikasi' => 'required',
        ];
        $id = $request->input('id') ?? MyHelper::getPk(date('Ymd'), 'bmn_pemeliharaan_seq');

        $request->validate($validate, $customMessages);
        try {
            DB::beginTransaction();
            $data = [
                'kdsatker_keu' => $request->input('kdsatker_keu'),
                'jns_perawatan' => $request->input('jns_perawatan'),
                'kode_barang' => $request->input('kode_barang'),
                'tgl_perawatan' => $request->input('tgl_perawatan'),
                'pelaksana' => $request->input('pelaksana'),
                'biaya' => $request->input('biaya'),
                'spesifikasi' => $request->input('spesifikasi'),
                'no_perawatan' => $id,
            ];
            if ($request->status) {
                $data['status'] = $request->status;
            } else {
                $data['status'] = 'On Proses';
            }

            Perawatan::updateOrCreate(['id' => $id], $data);

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

        return view('bmn.perawatan.perawatanFormV', $data);
    }

    /**
     * Show the form for editing the specified resource.
     */
    public function edit(string $id)
    {
        $data = $this->getData($id);

        return view('bmn.perawatan.perawatanFormV', $data);
    }

    /**
     * Update the specified resource in storage.
     */
    public function update(Request $request, Perawatan $hakcipta)
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
            Perawatan::destroy($id);
            DB::commit();

            return $this->resSuccess('Berhasil Dihapus!');
        } catch (\Throwable $th) {
            DB::rollBack();
        }
    }
}
