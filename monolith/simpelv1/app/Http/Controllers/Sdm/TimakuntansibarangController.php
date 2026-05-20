<?php

namespace App\Http\Controllers\Sdm;

use App\Helpers\MyHelper;
use App\Http\Controllers\Controller;
use App\Models\Master;
use App\Models\Master\MsSatker;
use App\Models\Sdm\Timakuntansibarang;
use App\Models\Sdm\Timakuntansibarangpegawai;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\DB;
use Symfony\Component\HttpKernel\Exception\NotFoundHttpException;

class TimakuntansibarangController extends Controller
{
    /**
     * Display a listing of the resource.
     */
    protected $controller = '/sdm/timakuntansibarang';

    protected $breadcums = ['SDM'];

    protected $columns = ['Nama Satker', 'Jenis SK', 'No. Surat', 'Tgl. Surat'];

    protected $defColumns = [0, 1, 2, 3, 4, 5];

    protected function canCreate()
    {
        // return true;
        return in_array(session('userData.current_role.ms_role_id'), config('constants.pelaksana_role'));
    }

    public function index()
    {
        $data = ['tableId' => 'dt-timakuntansibarang', 'breadcums' => $this->breadcums, 'columns' => $this->columns, 'defColumns' => $this->defColumns, 'controller' => $this->controller, 'canCreate' => $this->canCreate()];

        return view('sdm.timakuntansibarang.timakuntansibarangV', $data);
    }

    public function gridData(Request $request)
    {
        $model = new Timakuntansibarang;
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
        $modelPegawai = [];
        if ($id) {
            $breadcum = 'Ubah';
            $model = Timakuntansibarang::where('id', $id)->first();
            $modelPegawai = Timakuntansibarangpegawai::where('pengajuan_id', $id)->get()->toArray();
            if (! $model) {
                throw new NotFoundHttpException('Data Tidak Ditemukan');
            }

            $model = $model->toArray();
            $isNew = false;

            $ms_satker_id = $model['id_satker'];
        } else {
            $currentRole = session('userData.current_role');
            $ms_satker_id = $currentRole['ms_satker_id'] ?? $model['id_satker'];
        }

        if ($readOnly) {
            $breadcum = 'Detail';
        }
        $satkers = Master::getSatkersKeu();
        $jenis = ['UAKPB', 'UAPPB-W', 'UAPPB-E1', 'UAPB'];
        $nama_satker = MsSatker::where('inst_satkerkd', $ms_satker_id)->first();
        $dikeluarkan_di = str_replace('KEJAKSAAN NEGERI ', '', $nama_satker['inst_nama']);
        $dikeluarkan_di = str_replace('KEJAKSAAN TINGGI ', '', $dikeluarkan_di);
        $dikeluarkan_di = str_replace('CABANG KEJAKSAAN NEGERI ', '', $dikeluarkan_di);
        $model['kdsatker_keu'] = $nama_satker['kdsatker_keu'];
        $model['inst_nama'] = MsSatker::where('inst_satkerkd', $ms_satker_id)->first()['inst_nama'];
        $model['dikeluarkan_di'] = $dikeluarkan_di;
        $data = [
            'model' => $model,
            'modelPegawai' => $modelPegawai,
            'isNew' => $isNew,
            'breadcums' => array_merge($this->breadcums, [$breadcum]),
            'judul' => $breadcum,
            'readOnly' => $readOnly,
            'controller' => $this->controller,
            'canCreate' => $this->canCreate(),
            'satkerOptions' => MyHelper::generateSelectOptions([
                'data' => $satkers,
                'text' => 'inst_nama',
                'textkode' => 'kdsatker_keu',
                'value' => 'kdsatker_keu',
                'selected' => $nama_satker['kdsatker_keu'] ?? null,
            ]),
            'jenisOptions' => MyHelper::generateSelectOptions([
                'data' => $jenis,
                'text' => 'jenis',
                'value' => null,
                'selected' => $model['jenis_sk'] ?? null,
            ]),
        ];

        return $data;
    }

    public function gridDataPegawai($pengajuan_id)
    {
        $model = new Timakuntansibarangpegawai;
        $data = $model->getDetail($pengajuan_id);

        return response()->json([
            'data' => $data,
        ]);
    }

    public function gridDataMsPegawai()
    {
        $model = new Timakuntansibarangpegawai;
        $currentRole = session('userData.current_role');
        $data = $model->getMsPegawai($currentRole['ms_satker_id']);

        return response()->json([
            'data' => $data,
        ]);
    }

    /**
     * Show the form for creating a new resource.
     */
    public function create()
    {
        if (! $this->canCreate()) {
            throw new UnauthorizedHttpException('Tidak Punya Akses');
        }

        $data = $this->getData();

        return view('sdm.timakuntansibarang.timakuntansibarangFormV', $data);
    }

    /**
     * Store a newly created resource in storage.
     */
    public function store(Request $request)
    {
        $isNew = $request->input('isNew');
        $customMessages = [
            'kdsatker_keu.required' => 'Satker harus diisi',
            'jenis_sk.required' => 'Jenis SK harus diisi',
            'no_surat.required' => 'Nomor Surat harus diisi',
            'tgl_surat.required' => 'Tanggal Surat harus diisi',
        ];
        $validate = [
            'kdsatker_keu' => 'required',
            'jenis_sk' => 'required',
            'no_surat' => 'required',
            'tgl_surat' => 'required',
        ];
        $id = $request->input('id') ?? MyHelper::getPk(date('Ymd'), 'sdm_timakuntansibarang_seq');
        if ($isNew) {
            $validate['file_sk'] = 'required|mimes:jpeg,png,pdf|max:2048';
            $customMessages['file_spk.required'] = 'File SK harus diupload';
        } else {

        }
        $request->validate($validate, $customMessages);
        try {
            DB::beginTransaction();
            $currentRole = session('userData.current_role');
            $ms_satker_id = $currentRole['ms_satker_id'] ?? '';
            $ms_satker_id_keu = $currentRole['ms_satker_id_keu'] ?? '';
            $data = [
                'id_satker' => $ms_satker_id,
                'kdsatker_keu' => $ms_satker_id_keu,
                'jenis_sk' => $request->input('jenis_sk'),
                'no_surat' => $request->input('no_surat'),
                'tgl_surat' => $request->input('tgl_surat'),
                'dikeluarkan_di' => $request->input('dikeluarkan_di'),
            ];
            if ($request->hasFile('file_sk')) {
                $filepath = 'uploads/sdm/timakuntansibarang';
                $file = $request->file('file_sk');
                $fileName = $id.'_sk'.'.'.$file->getClientOriginalExtension();
                $filesave = $filepath.'/'.$fileName;
                $file->move($filepath, $fileName);
                $data['file_sk'] = $filesave;
            }
            Timakuntansibarang::updateOrCreate(['id' => $id], $data);
            $pegawais = $request->input('jpnid');
            if (count($pegawais) > 0) {
                Timakuntansibarangpegawai::where('pengajuan_id', $id)->delete();
                foreach ($pegawais as $pegawai) {
                    $peg = explode('#', $pegawai);
                    $modelPegawai = new Timakuntansibarangpegawai;
                    $datapeg = [
                        'pengajuan_id' => $id,
                        'nip' => $peg[0],
                        'nama' => $peg[1],
                        'pangkat' => $peg[2],
                        'jabatan' => $peg[3],
                        'jabatan_tim' => $peg[4],
                    ];
                    Timakuntansibarangpegawai::insert($datapeg);
                }
            }
            DB::commit();

            return $this->resSuccess('Berhasil Disimpan!',
                [
                    'type' => 'redirect',
                    'url' => \URL::to('/sdm/timakuntansibarang'),
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
    public function show(string $id)
    {
        $data = $this->getData($id, true);

        return view('sdm.timakuntansibarang.timakuntansibarangFormV', $data);
    }

    /**
     * Show the form for editing the specified resource.
     */
    public function edit(string $id)
    {
        $data = $this->getData($id);

        return view('sdm.timakuntansibarang.timakuntansibarangFormV', $data);
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
            Timakuntansibarang::destroy($id);
            DB::commit();

            return $this->resSuccess('Berhasil Dihapus!');
        } catch (\Throwable $th) {
            DB::rollBack();
        }
    }
}
