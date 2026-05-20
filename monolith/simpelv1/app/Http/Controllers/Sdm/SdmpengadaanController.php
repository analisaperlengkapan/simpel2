<?php

namespace App\Http\Controllers\Sdm;

use App\Helpers\MyHelper;
use App\Http\Controllers\Controller;
use App\Models\Files;
use App\Models\Master;
use App\Models\Master\MsSatker;
use App\Models\Sdm\Sdmpengadaan;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\DB;
use Symfony\Component\HttpKernel\Exception\NotFoundHttpException;

class SdmpengadaanController extends Controller
{
    /**
     * Display a listing of the resource.
     */
    protected $controller = '/sdm/sdmpengadaan';

    protected $breadcums = ['SDM', 'Monitoring SDM Pengadaan Barang/Jasa'];

    protected $columns = ['Nama Satker', 'NIP', 'Nama Pegawai', 'Jabatan', 'NIK', 'NPWP', 'Telpon', 'Tgl. Sertifikat'];

    protected $defColumns = [0, 1, 2, 3, 4, 5, 6, 7, 8];

    protected function canCreate()
    {
        // return true;
        return in_array(session('userData.current_role.ms_role_id'), config('constants.pelaksana_role'));
    }

    public function index()
    {
        $data = ['tableId' => 'dt-sdmpengadaan', 'breadcums' => $this->breadcums, 'columns' => $this->columns, 'defColumns' => $this->defColumns, 'controller' => $this->controller, 'canCreate' => $this->canCreate()];

        return view('sdm.sdmpengadaan.sdmpengadaanV', $data);

        // echo "<pre>";
        // print_r(session('userData.current_role'));
    }

    public function gridData(Request $request)
    {
        $model = new Sdmpengadaan;
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
            $model = Sdmpengadaan::where('nip', $nip)->first();
            if (! $model) {
                throw new NotFoundHttpException('Data Tidak Ditemukan');
            }

            $model = $model->toArray();
            $isNew = false;

            $ms_satker_id = $model['kdsatker'];
        } else {
            $currentRole = session('userData.current_role');
            $ms_satker_id = $currentRole['ms_satker_id'] ?? $model['kdsatker'];
        }
        if ($readOnly) {
            $breadcum = 'Detail';
        }
        $satkers = Master::getSatkersKeu();
        $model['inst_nama'] = MsSatker::where('kdsatker_keu', $ms_satker_id)->first()['inst_nama'];

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

        return view('sdm.sdmpengadaan.sdmpengadaanFormV', $data);
    }

    /**
     * Store a newly created resource in storage.
     */
    public function store(Request $request)
    {
        $isNew = $request->input('isNew');
        $customMessages = [
            'tgl_sertifikat.required' => 'Tanggal Surat harus diisi',
        ];
        $validate = [
            'tgl_sertifikat' => 'required',
        ];
        $nip = $request->input('nip') ?? 0;
        if ($isNew) {
            $validate['file_sertifikat'] = 'required|mimes:jpeg,png,pdf|max:2048';
            $customMessages['file_sertifikat.required'] = 'File SK harus diupload';
        }
        $request->validate($validate, $customMessages);

        try {
            DB::beginTransaction();
            $data = [
                'tgl_sertifikat' => $request->input('tgl_sertifikat'),
                'updated_at' => date('Y-m-d H:i:s'),
                'updated_by' => session('userData.username'),
            ];

            if ($request->hasFile('file_sertifikat')) {
                $params = [
                    'kategori' => 'SDM - SDM Pengadaan Barang & Jasa',
                    'dir' => 'sdm/sdmpengadaan',
                    'fileKey' => 'file_sertifikat',
                    'pkey' => session('userData.username'),
                ];
                $fotos = Files::upload($request, $params);
                $data['file_sertifikat'] = $fotos['path'];
            }

            // if ($request->hasFile('file_sertifikat')) {
            //     $filepath = 'uploads/sdm/sdmpengadaan';
            //     $file = $request->file('file_sertifikat');
            //     $fileName = $id.'_sertifikat_pbj'.'.'.$file->getClientOriginalExtension();
            //     $filesave = $filepath.'/'.$fileName;
            //     $file->move($filepath, $fileName);
            //     $data['file_sertifikat'] = $filesave;
            // }
            // Sdmpengadaan::updateOrCreate(['nip' => $nip], $data);

            Sdmpengadaan::where('nip', $nip)->update($data);

            DB::commit();

            return $this->resSuccess('Berhasil Disimpan!', [
                'type' => 'redirect',
                'url' => \URL::to('/sdm/sdmpengadaan'),
            ]);
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
        $data = $this->getData($nip, true);

        return view('sdm.sdmpengadaan.sdmpengadaanFormV', $data);
    }

    /**
     * Show the form for editing the specified resource.
     */
    public function edit(string $nip)
    {
        $data = $this->getData($nip);

        return view('sdm.sdmpengadaan.sdmpengadaanFormV', $data);
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
            Sdmpengadaan::destroy($id);
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
