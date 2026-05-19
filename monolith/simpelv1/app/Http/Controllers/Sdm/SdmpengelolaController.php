<?php

namespace App\Http\Controllers\Sdm;

use App\Helpers\MyHelper;
use App\Http\Controllers\Controller;
use App\Models\Master;
use App\Models\Master\MsSatker;
use App\Models\Sdm\Sdmpengelola;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\DB;
use Symfony\Component\HttpKernel\Exception\NotFoundHttpException;

class SdmpengelolaController extends Controller
{
    /**
     * Display a listing of the resource.
     */
    protected $controller = '/sdm/sdmpengelola';

    protected $breadcums = ['SDM', 'Monitoring SDM Pengelolaan BMN'];

    protected $columns = ['Nama Satker', 'NIP', 'Nama Pegawai', 'Jabatan', 'Golongan Pangkat'];

    protected $defColumns = [0, 1, 2, 3, 4, 5];

    public function index()
    {
        // echo "<pre>";
        // print_r(session('userData.current_role'));exit;

        $data = ['tableId' => 'dt-sdmpengelola', 'breadcums' => $this->breadcums, 'columns' => $this->columns, 'defColumns' => $this->defColumns, 'controller' => $this->controller];

        return view('sdm.sdmpengelola.sdmpengelolaV', $data);
    }

    public function gridData(Request $request)
    {
        $model = new Sdmpengelola;
        $pagingParams = $request->only(['start', 'length']);
        $searchParams = $request->only(['columns']);
        $data = $model->getDataGrid($pagingParams, $searchParams);

        return response()->json([
            'data' => $data['data'],
            'recordsTotal' => $data['total'],
            'recordsFiltered' => $data['total'],
        ]);
    }

    public function getData($nip = null, $readOnly = false)
    {
        $model = [];
        $isNew = true;
        $breadcum = 'Tambah';
        if ($nip) {
            $breadcum = 'Ubah';
            $model = Sdmpengelola::where('peg_nip_baru', $nip)->first();
            if (! $model) {
                throw new NotFoundHttpException('Data Tidak Ditemukan');
            }

            $model = $model->toArray();
            $isNew = false;

            $ms_satker_id = $model['inst_satkerkd'];
        } else {
            $currentRole = session('userData.current_role');
            $ms_satker_id = $currentRole['ms_satker_id'] ?? $model['kdsatker'];
        }

        if ($readOnly) {
            $breadcum = 'Detail';
        }

        $satkers = Master::getSatkersKeu();
        $model['inst_nama'] = MsSatker::where('inst_satkerkd', $ms_satker_id)->first()['inst_nama'];

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
        ];

        return $data;
    }

    /**
     * Show the form for creating a new resource.
     */
    public function create()
    {
        $data = $this->getData();

        return view('sdm.sdmpengelola.sdmpengelolaFormV', $data);
    }

    /**
     * Store a newly created resource in storage.
     */
    public function store(Request $request)
    {
        $isNew = $request->input('isNew');
        $customMessages = [
            'kdsatker_keu.required' => 'Satker harus diisi',
            'nip.required' => 'Jenis SK harus diisi',
            'nama.required' => 'Nomor Surat harus diisi',
            'tgl_sertifikat.required' => 'Tanggal Surat harus diisi',
        ];
        $validate = [
            'kdsatker_keu' => 'required',
            'nip' => 'required',
            'nama' => 'required',
            'tgl_sertifikat' => 'required',
        ];
        $id = $request->input('id') ?? MyHelper::getPk(date('Ymd'), 'sdm_pengelola_seq');
        if ($isNew) {
            $validate['file_sertifikat'] = 'required|mimes:jpeg,png,pdf|max:2048';
            $customMessages['file_sertifikat.required'] = 'File SK harus diupload';
        }
        $request->validate($validate, $customMessages);

        try {
            DB::beginTransaction();
            $data = [
                'kdsatker_keu' => $request->input('kdsatker_keu'),
                'nip' => $request->input('nip'),
                'nama' => $request->input('nama'),
                'tgl_sertifikat' => $request->input('tgl_sertifikat'),
            ];
            if ($request->hasFile('file_sertifikat')) {
                $filepath = 'uploads/sdm/sdmpengelola';
                $file = $request->file('file_sertifikat');
                $fileName = $id.'_sertifikat_pbj'.'.'.$file->getClientOriginalExtension();
                $filesave = $filepath.'/'.$fileName;
                $file->move($filepath, $fileName);
                $data['file_sertifikat'] = $filesave;
            }
            Sdmpengelola::updateOrCreate(['id' => $id], $data);

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
    public function show(string $nip)
    {
        $model = new Sdmpengelola;

        $data = $this->getData($nip, true);

        $datapengadaan = $model->getdatapengadaan($nip);
        // echo "<pre>"; print_r($datapengadaan); exit;
        if ($datapengadaan) {
            $data['tgl_sertifikat_pengadaan'] = $datapengadaan[0]['tgl_sertifikat'];
            $data['file_sertifikat_pengadaan'] = $datapengadaan[0]['file_sertifikat'];
        }

        return view('sdm.sdmpengelola.sdmpengelolaFormV', $data);
    }

    /**
     * Show the form for editing the specified resource.
     */
    public function edit(string $nip)
    {
        $data = $this->getData($nip);

        return view('sdm.sdmpengelola.sdmpengelolaFormV', $data);
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
            Sdmpengelola::destroy($id);
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
