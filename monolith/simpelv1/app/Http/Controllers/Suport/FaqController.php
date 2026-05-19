<?php

namespace App\Http\Controllers\Suport;

use App\Helpers\MyHelper;
use App\Http\Controllers\Controller;
use App\Models\Suport\Faq;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\DB;
use Symfony\Component\HttpKernel\Exception\NotFoundHttpException;

class FaqController extends Controller
{
    /**
     * Display a listing of the resource.
     */
    // protected $breadcums = ['Suport', 'FAQ'];
    protected $kategoriJudul = 'FAQ';

    protected $controller = '/suport/faq';

    protected $breadcums = ['FAQ'];

    protected $columns = ['Create By', 'Pertanyaan', 'Jawaban', 'Platform', 'Status'];

    protected $defColumns = [0, 1, 2, 3, 4];

    public function __construct()
    {
        $this->breadcums = array_merge($this->breadcums, [['link' => $this->controller, 'title' => 'FAQ']]);

    }

    public function index()
    {
        $columns = [
            'Create By',
            'Pertanyaan',
            'Jawaban',
            'Platform',
            'Status',
        ];
        $defColumns = [0, 1, 2, 3, 4];

        return view('suport.faq.faqV', [
            'tableId' => 'dt-kritik',
            'kategoriJudul' => $this->kategoriJudul,
            'breadcums' => $this->breadcums,
            'columns' => $columns,
            'defColumns' => $defColumns,
            'controller' => $this->controller,
        ]);
    }

    public function gridData(Request $request)
    {
        $model = new Faq;
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
            $model = Faq::where('id', $id)->first();
            if (! $model) {
                throw new NotFoundHttpException('Data Tidak Ditemukan');
            }

            $model = $model->toArray();
            $isNew = false;
        }
        if ($readOnly) {
            $breadcum = 'Detail';
        }
        $kategori = ['Analisa Kebutuhan', 'Bank Aset', 'Approval Pengajuan', 'suport'];
        $status = ['Aktif', 'Non Aktif'];
        $data = [
            'model' => $model,
            'isNew' => $isNew,
            'breadcums' => array_merge($this->breadcums, [$breadcum]),
            'judul' => $breadcum,
            'readOnly' => $readOnly,
            'kategoriOptions' => MyHelper::generateSelectOptions([
                'data' => $kategori,
                'text' => 'kategori',
                'value' => null,
                'selected' => $model['kategori'] ?? null,
            ]),
            'platformOptions' => MyHelper::generateSelectOptions(['data' => config('constants.platforms'), 'selected' => $model['platform'] ?? null]),
            'statusOptions' => MyHelper::generateSelectOptions([
                'data' => $status,
                'text' => 'status',
                'value' => null,
                'selected' => $model['status'] ?? null,
            ]),
        ];

        return $data;
    }

    public function create()
    {
        $data = $this->getData();

        return view('suport.faq.faqFormV', $data);
    }

    public function store(Request $request)
    {
        $request->validate([
            'pertanyaan' => 'required',
            'jawaban' => 'required',
        ]);

        // dd($request->all());

        $data['pertanyaan'] = $request->pertanyaan;
        $data['jawaban'] = $request->jawaban;
        $data['platform'] = $request->platform;
        if ($request->status) {
            $data['status'] = $request->status;
        } else {
            $data['status'] = 'Aktif';
        }

        $id = $request->input('id') ?? MyHelper::getPk(date('Ymd'), 'suport_kritik_seq');
        Faq::updateOrCreate(['id' => $id], $data);

        return $this->resSuccess();
    }

    public function show(string $id)
    {
        $data = $this->getData($id, true);

        return view('suport.faq.faqFormV', $data);
    }

    public function edit(string $id)
    {
        $data = $this->getData($id);

        return view('suport.faq.faqFormV', $data);
    }

    public function update(Request $request, Faq $tanah)
    {
        //
    }

    public function destroy(string $id)
    {
        try {
            DB::beginTransaction();
            Faq::destroy($id);
            DB::commit();

            return $this->resSuccess('Berhasil Dihapus!');
        } catch (\Throwable $th) {
            DB::rollBack();
        }
    }
}
