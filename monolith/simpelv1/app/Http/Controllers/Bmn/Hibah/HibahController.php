<?php

namespace App\Http\Controllers\Bmn\Hibah;

use App\Helpers\MyHelper;
use App\Http\Controllers\Controller;
use App\Models\Bmn\Hibah;
use App\Models\Bmn\HibahFile;
use App\Models\Master;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\DB;
use Illuminate\Support\Facades\File;
use Symfony\Component\HttpKernel\Exception\NotFoundHttpException;

class HibahController extends Controller
{
    /**
     * Display a listing of the resource.
     */
    // protected $breadcums = ['BMN', 'Pengajuan Persetujuan Penerimaan Hibah'];
    protected $kategoriJudul = 'Pengajuan Persetujuan Penerimaan Hibah';

    protected $controller = '/bmn/hibah/hibah';

    protected $breadcums = ['BMN'];

    protected $columns = ['Nama Satker', 'Jenis Hibah', 'Bentuk Hibah', 'Tanggal', 'Hibah Dari', 'nilai'];

    protected $defColumns = [0, 1, 2, 3, 4, 5];

    public function __construct()
    {
        $this->breadcums = array_merge($this->breadcums, [['link' => $this->controller, 'title' => 'Pengajuan Persetujuan Penerimaan Hibah']]);

    }

    public function index()
    {
        // $data = ['tableId' => 'dt-hibah', 'breadcums' => $this->breadcums];
        // return view('bmn.hibah.hibahV', $data);
        return view('bmn.hibah.hibahV', [
            'tableId' => 'dt-hibah',
            'kategoriJudul' => $this->kategoriJudul,
            'breadcums' => $this->breadcums,
            'columns' => $this->columns,
            'defColumns' => $this->defColumns,
            'controller' => $this->controller,
        ]);
    }

    public function gridData(Request $request)
    {
        $model = new hibah;
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
        $modelSk = [];
        $isNew = true;
        $breadcum = 'Tambah';
        if ($id) {
            $breadcum = 'Ubah';
            $model = hibah::where('id', $id)->first();
            if (! $model) {
                throw new NotFoundHttpException('Data Tidak Ditemukan');
            }

            $model = $model->toArray();
            $isNew = false;
            $modelSk = HibahFile::where('id_hibah', $id)->get();
            $modelSk = $modelSk->toArray();
        }
        if ($readOnly) {
            $breadcum = 'Detail';
        }
        $satkers = Master::getSatkersKeu();
        $jenis = ['Dalam Negeri', 'Luar Negeri'];
        $kategori = ['Barang', 'Uang'];
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
                'selected' => $model['jenis_hibah'] ?? null,
            ]),
            'kategoriOptions' => MyHelper::generateSelectOptions([
                'data' => $kategori,
                'text' => 'kategori',
                'value' => null,
                'selected' => $model['kategori'] ?? null,
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

        return view('bmn.hibah.hibahFormV', $data);
    }

    /**
     * Store a newly created resource in storage.
     */
    public function store(Request $request)
    {
        $isNew = $request->input('isNew');
        $customMessages = [
            'kategori.required' => 'kategori harus diisi',
            'jenis_hibah.required' => 'Jenis SK harus diisi',
            'nilai.required' => 'nilai hibah harus diisi',
            'tgl_register.required' => 'Tanggal Surat harus diisi',
        ];
        $validate = [
            'kategori' => 'required',
            'jenis_hibah' => 'required',
            'nilai' => 'required',
            'tgl_register' => 'required',
        ];
        $id = $request->input('id') ?? MyHelper::getPk(date('Ymd'), 'bmn_hibah_seq');
        // if($isNew){
        //     $validate['file_sk'] = 'required|mimes:jpeg,png,pdf|max:2048';
        //     $customMessages['file_spk.required'] = 'File SK harus diupload';
        // }else{

        // }
        $request->validate($validate, $customMessages);
        try {
            DB::beginTransaction();
            $data = [
                'kdsatker_keu' => session('userData.current_role.ms_satker_id_keu'),
                'jenis_hibah' => $request->input('jenis_hibah'),
                // 'no_register' => $request->input('no_register'),
                'tgl_register' => $request->input('tgl_register'),
                'kategori' => $request->input('kategori'),
                // 'nilai' => $request->input('nilai'),
                'nilai' => str_replace('.', '', $request->input('nilai')),
                'hibah_ke' => $request->input('hibah_ke'),
            ];
            if ($request->status) {
                $data['status'] = $request->status;
            } else {
                $data['status'] = 'On Proses';
            }
            hibah::updateOrCreate(['id' => $id], $data);

            $hibahFile = HibahFile::where('id_hibah', $id)->get();
            $fileIds = $hibahFile->map(function ($item) {
                return $item->id;
            })->toArray();
            $id_sk = $request->input('id_sk') ?? [];
            $idsToDelete = array_diff($fileIds, $id_sk);
            if (! empty($idsToDelete)) {
                foreach ($idsToDelete as $idDelete) {
                    $delHibah = HibahFile::where('id', $idDelete)->first();
                    if (! empty($delHibah)) {
                        $filePath = public_path($delHibah->file);
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
                        'id_hibah' => $id,
                        'no' => $request->input('no_sk')[$key],
                        'tgl' => $request->input('tgl_sk')[$key],
                    ];
                    if (isset($request->file('file_sk')[$key]) && $request->file('file_sk')[$key] !== null) {
                        $filepath = 'uploads/bmn/hibah';
                        $file = $request->file('file_sk')[$key];
                        $fileName = $key.'_'.$id.'_sk'.'.'.$file->getClientOriginalExtension();
                        $filesave = $filepath.'/'.$fileName;
                        $file->move($filepath, $fileName);
                        $dataSk['file'] = $filesave;
                    }
                    HibahFile::updateOrCreate(['id' => $idsk], $dataSk);
                }
            }
            DB::commit();

            return $this->resSuccess(
                'Berhasil Disimpan!',
                [
                    'type' => 'redirect',
                    'url' => \URL::to('/bmn/hibah/hibah'),
                ]
            );
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

        return view('bmn.hibah.hibahFormV', $data);
    }

    /**
     * Show the form for editing the specified resource.
     */
    public function edit(string $id)
    {
        $data = $this->getData($id);

        return view('bmn.hibah.hibahFormV', $data);
    }

    /**
     * Update the specified resource in storage.
     */
    public function update(Request $request, hibah $hakcipta)
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

            $files = HibahFile::where(['id_hibah' => $id])->get();
            foreach ($files as $file) {
                $filePath = public_path($file->file);
                if (File::exists($filePath)) {
                    File::delete($filePath);
                }
            }
            hibah::destroy($id);
            DB::commit();

            return $this->resSuccess('Berhasil Dihapus!');
        } catch (\Throwable $th) {
            DB::rollBack();
        }
    }
}
