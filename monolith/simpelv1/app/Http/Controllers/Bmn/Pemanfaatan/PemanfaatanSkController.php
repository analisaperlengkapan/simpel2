<?php

namespace App\Http\Controllers\Bmn\Pemanfaatan;

use App\Helpers\MyHelper;
use App\Http\Controllers\Controller;
use App\Models\Bmn\PemanfaatanSk;
use App\Models\Bmn\PemanfaatanSkFile;
use App\Models\Master;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\DB;
use Illuminate\Support\Facades\File;
use Symfony\Component\HttpKernel\Exception\NotFoundHttpException;

class PemanfaatanSkController extends Controller
{
    /**
     * Display a listing of the resource.
     */
    // protected $breadcums = ['BMN', 'Pengajuan SK Pemanfaatan'];
    protected $breadcums = ['BMN', 'Pengajuan SK Pemanfaatan'];

    protected $columns = ['Nama Satker', 'Nama Barang', 'Tgl. Mulai', 'Tgl. Berakhir'];

    protected $defColumns = [0, 1, 2, 3];

    public function index()
    {
        // $data = ['tableId' => 'dt-pemanfaatan', 'breadcums' => $this->breadcums];
        // return view('bmn.pemanfaatansk.pemanfaatanV', $data);
        return view('bmn.pemanfaatansk.pemanfaatanmonitorV', [
            'tableId' => 'dt-pemanfaatan',
            'breadcums' => $this->breadcums,
            'columns' => $this->columns,
            'defColumns' => $this->defColumns,
        ]);
    }

    public function gridData(Request $request)
    {
        $model = new PemanfaatanSk;
        $pagingParams = $request->only(['start', 'length']);
        $searchParams = $request->only(['search',  'filterBy']);
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
        $modelSk = [];
        $isNew = true;
        $breadcum = 'Tambah';
        if ($id) {
            $breadcum = 'Ubah';
            $model = PemanfaatanSk::where('id', $id)->first();
            if (! $model) {
                throw new NotFoundHttpException('Data Tidak Ditemukan');
            }

            $model = $model->toArray();
            $isNew = false;
            $modelSk = PemanfaatanSkFile::where('id_pemanfaatan', $id)->get();
            $modelSk = $modelSk->toArray();
        }
        if ($readOnly) {
            $breadcum = 'Detail';
        }
        // $satker= session('userData.current_role.ms_satker_id_keu');
        $satkers = Master::getSatkersKeu();
        $jenis = ['Sewa Bangunan', 'Pinjam-Pakai Bangunan', 'Pinjam-Pakai Kendaraan'];
        $data = [
            'model' => $model,
            'modelSk' => $modelSk,
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
            'listBarang' => MyHelper::generateSelectOptions([
                'data' => Master::getBarangAsetBmn(),
                'text' => 'nama_barang',
                'textkode' => 'kode_barang',
                'value' => 'kode_barang',
                'selected' => $model['kode_barang'] ?? null,
            ]),
            // 'listBarang' => Master::getBarangAset(),
        ];

        return $data;
    }

    /**
     * Show the form for creating a new resource.
     */
    public function create()
    {
        $data = $this->getData();

        return view('bmn.pemanfaatansk.pemanfaatanFormV', $data);
    }

    /**
     * Store a newly created resource in storage.
     */
    public function store(Request $request)
    {
        $isNew = $request->input('isNew');
        $customMessages = [
            'jenis_sk.required' => 'Jenis SK harus diisi',
            // 'no_surat.required' => 'Nomor Surat harus diisi',
            // 'tgl_surat.required' => 'Tanggal Surat harus diisi',
            'kode_barang.required' => 'kode_barang harus diisi',
            'tgl_awal.required' => 'Tanggal Awal harus diisi',
            'tgl_akhir.required' => 'Tanggal Akhir harus diisi',
            'nama.required' => 'Nama harus diisi',
            'npwp.required' => 'NPWP harus diisi',
            'ktp.required' => 'KTP harus diisi',
        ];
        $validate = [
            'jenis_sk' => 'required',
            // 'no_surat' => 'required',
            // 'tgl_surat' => 'required',
            'kode_barang' => 'required',
            'tgl_awal' => 'required',
            'tgl_akhir' => 'required',
            'nama' => 'required',
            'npwp' => 'required',
            'ktp' => 'required',
        ];
        $id = $request->input('id') ?? MyHelper::getPk(date('Ymd'), 'sdm_timakuntansibarang_seq');
        // if($isNew){
        //     $validate['file_sk'] = 'required|mimes:jpeg,png,pdf|max:2048';
        //     $customMessages['file_spk.required'] = 'File SK harus diupload';
        // }else{

        // }
        if ($isNew) {
            $validate['file_mohon'] = 'required|mimes:jpeg,png,pdf|max:2048';
            $customMessages['file_mohon.required'] = 'File SK harus diupload';
        } else {

        }
        $request->validate($validate, $customMessages);
        try {
            DB::beginTransaction();
            $kode_barang = explode('-', $request->input('kode_barang'));
            $barang = DB::table('vw_asset_barang')->where('kode_barang', $kode_barang[0])->first();
            $data = [
                'kdsatker_keu' => session('userData.current_role.ms_satker_id_keu'),
                'jenis_sk' => $request->input('jenis_sk'),
                // 'no_surat' => $request->input('no_surat'),
                // 'tgl_surat' => $request->input('tgl_surat'),
                // 'dikeluarkan_di' => $request->input('dikeluarkan_di'),
                'kode_barang' => $request->input('kode_barang'),
                'nm_barang' => $barang->nm_barang,
                'ms_jenis_asset_id' => $barang->ms_jenis_asset_id,
                'tgl_awal' => $request->input('tgl_awal'),
                'tgl_akhir' => $request->input('tgl_akhir'),
                'nama' => $request->input('nama'),
                'npwp' => $request->input('npwp'),
                'ktp' => $request->input('ktp'),
                'nilai' => $request->input('nilai'),
            ];
            if ($request->hasFile('file_mohon')) {
                $filepath = 'uploads/bmn/pemanfaatansk';
                $file = $request->file('file_mohon');
                $fileName = $id.'_mohon'.'.'.$file->getClientOriginalExtension();
                $filesave = $filepath.'/'.$fileName;
                $file->move($filepath, $fileName);
                $data['file_mohon'] = $filesave;
            }
            PemanfaatanSk::updateOrCreate(['id' => $id], $data);

            $hibahFile = PemanfaatanSkFile::where('id_pemanfaatan', $id)->get();
            $fileIds = $hibahFile->map(function ($item) {
                return $item->id;
            })->toArray();
            $id_sk = $request->input('id_sk') ?? [];
            $idsToDelete = array_diff($fileIds, $id_sk);
            if (! empty($idsToDelete)) {
                foreach ($idsToDelete as $idDelete) {
                    $delHibah = PemanfaatanSkFile::where('id', $idDelete)->first();
                    if (! empty($delHibah)) {
                        $filePath = public_path($delHibah->file_sk);
                        if (File::exists($filePath)) {
                            File::delete($filePath);
                        }
                        $delHibah->delete();
                    }
                }
            }

            if (count($id_sk) > 0) {
                foreach ($id_sk as $key => $idsk) {
                    $dataSk = [
                        'id_pemanfaatan' => $id,
                        'no_sk' => $request->input('no_sk')[$key],
                        'tgl_sk' => $request->input('tgl_sk')[$key],
                    ];
                    if (isset($request->file('file_sk')[$key]) && $request->file('file_sk')[$key] !== null) {
                        $filepath = 'uploads/bmn/pemanfaatansk';
                        $file = $request->file('file_sk')[$key];
                        $fileName = $key.'_'.$id.'_sk'.'.'.$file->getClientOriginalExtension();
                        $filesave = $filepath.'/'.$fileName;
                        $file->move($filepath, $fileName);
                        $dataSk['file_sk'] = $filesave;
                    }
                    PemanfaatanSkFile::updateOrCreate(['id' => $idsk], $dataSk);
                }
            }
            DB::commit();

            return $this->resSuccess(
                'Berhasil Disimpan!',
                [
                    'type' => 'redirect',
                    'url' => \URL::to('/bmn/pemanfaatan/pemanfaatansk'),
                ]
            );
        } catch (\Throwable $th) {
            dd($th);
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

        return view('bmn.pemanfaatansk.pemanfaatanFormV', $data);
    }

    /**
     * Show the form for editing the specified resource.
     */
    public function edit(string $id)
    {
        $data = $this->getData($id);

        return view('bmn.pemanfaatansk.pemanfaatanFormV', $data);
    }

    /**
     * Update the specified resource in storage.
     */
    public function update(Request $request, PemanfaatanSk $hakcipta)
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
            $files = PemanfaatanSkFile::where(['id_pemanfaatan' => $id])->get();
            foreach ($files as $file) {
                $filePath = public_path($file->file_sk);
                if (File::exists($filePath)) {
                    File::delete($filePath);
                }
            }
            $model = PemanfaatanSk::where('id', $id)->first();
            $filePath = public_path($file->file_mohon);
            if (File::exists($filePath)) {
                File::delete($filePath);
            }
            $model->delete();
            DB::commit();

            return $this->resSuccess('Berhasil Dihapus!');
        } catch (\Throwable $th) {
            DB::rollBack();
        }
    }

    public function cetakLabel($id)
    {
        $data = PemanfaatanSk::findOne($id);
        $data = (array) $data;
        $pdf = MyHelper::generateLabelBankAsset($data);

        return $pdf->stream('label-asset-tak-berwujud.pdf');
    }
}
