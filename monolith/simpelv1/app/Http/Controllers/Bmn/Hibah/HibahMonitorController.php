<?php

namespace App\Http\Controllers\Bmn\Hibah;

use App\Helpers\MyHelper;
use App\Http\Controllers\Controller;
use App\Models\Bmn\Hibah;
use App\Models\Master;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\DB;
use Symfony\Component\HttpKernel\Exception\NotFoundHttpException;

class HibahMonitorController extends Controller
{
    /**
     * Display a listing of the resource.
     */
    // protected $breadcums = ['BMN', 'Monitoring Penerimaan Hibah'];
    protected $kategoriJudul = 'Monitoring Penerimaan Hibah';

    protected $controller = '/bmn/hibah/hibah';

    protected $breadcums = ['BMN'];

    protected $columns = ['Nama Satker', 'Jenis Hibah', 'Bentuk Hibah', 'Tanggal', 'Hibah Dari', 'nilai'];

    protected $defColumns = [0, 1, 2, 3, 4, 5];

    public function __construct()
    {
        $this->breadcums = array_merge($this->breadcums, [['link' => $this->controller, 'title' => 'Monitoring Penerimaan Hibah']]);

    }

    public function index()
    {
        // $data = ['tableId' => 'dt-hibah', 'breadcums' => $this->breadcums];
        // return view('bmn.hibah.hibahMonitoringV', $data);
        return view('bmn.hibah.hibahMonitoringV', [
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
        }
        if ($readOnly) {
            $breadcum = 'Detail';
        }
        $satkers = Master::getSatkersKeu();
        $jenis = ['Dalam Negeri', 'Luar Negeri'];
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
            'jenisOptions' => MyHelper::generateSelectOptions([
                'data' => $jenis,
                'text' => 'jenis',
                'value' => null,
                'selected' => $model['jenis_sk'] ?? null,
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
            'kdsatker_keu.required' => 'Satker harus diisi',
            'jenis_hibah.required' => 'Jenis SK harus diisi',
            'no_register.required' => 'Nomor Surat harus diisi',
            'tgl_register.required' => 'Tanggal Surat harus diisi',
        ];
        $validate = [
            'kdsatker_keu' => 'required',
            'jenis_hibah' => 'required',
            'no_register' => 'required',
            'tgl_register' => 'required',
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
            $data = [
                'kdsatker_keu' => $request->input('kdsatker_keu'),
                'jenis_hibah' => $request->input('jenis_hibah'),
                'no_register' => $request->input('no_register'),
                'tgl_register' => $request->input('tgl_register'),
                'hibah_ke' => $request->input('hibah_ke'),
            ];
            if ($request->hasFile('file_sk')) {
                $filepath = 'uploads/bmn/hibah';
                $file = $request->file('file_sk');
                $fileName = $id.'_sk'.'.'.$file->getClientOriginalExtension();
                $filesave = $filepath.'/'.$fileName;
                $file->move($filepath, $fileName);
                $data['file_sk'] = $filesave;
            }
            hibah::updateOrCreate(['id' => $id], $data);

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
}
