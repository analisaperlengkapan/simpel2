<?php

namespace App\Http\Controllers\Bantuan;
use Illuminate\Routing\Controller;

use App\Helpers\MyHelper;
use App\Models\Bantuan\FaqPengguna;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\DB;
use Symfony\Component\HttpKernel\Exception\NotFoundHttpException;

class FaqPenggunaController extends Controller
{
    /**
     * Display a listing of the resource.
     */
    protected $breadcums = ['Support', 'FAQ'];
    public function index()
    {
        $faq = $this->indexData();
        return view('support.faq.faquserV', ['faq' => $faq, 'tableId' => 'dt-kritik', 'breadcums' => $this->breadcums]);
    }

    function indexData($platform = 'WEB')
    {
        return FaqPengguna::where(['status' => 'Aktif', 'platform' => strtoupper($platform)])->get()->toArray();
    }

    public function gridData(Request $request)
    {
        $model = new FaqPengguna();
        $pagingParams = $request->only(['start', 'length']);
        $searchParams = $request->only(['search', 'filterBy']);
        $data = $model->getDataGrid($pagingParams, $searchParams);

        return response()->json([
            'data' => $data['data'],
            'recordsTotal' => $data['total'],
            'recordsFiltered' => $data['total'],
        ]);
    }

    function getData($id = null, $readOnly = false)
    {
        $model = [];
        $isNew = true;
        $breadcum = 'Tambah';
        if ($id) {
            $breadcum = 'Ubah';
            $model = FaqPengguna::where('id', $id)->first();
            if (!$model) {
                throw new NotFoundHttpException('Data Tidak Ditemukan');
            }

            $model = $model->toArray();
            $isNew = false;
        }
        if ($readOnly) {
            $breadcum = 'Detail';
        }
        $kategori = ['Analisa Kebutuhan', 'Bank Aset', 'Approval Pengajuan', 'support'];
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
        return view('support.faq.faqFormV', $data);
    }

    public function store(Request $request)
    {
        $request->validate([
            'pertanyaan' => 'required',
            'jawaban' => 'required'
        ]);

        //dd($request->all());

        $data['pertanyaan'] = $request->pertanyaan;
        $data['jawaban'] = $request->jawaban;
        if ($request->status) {
            $data['status'] = $request->status;
        } else {
            $data['status'] = 'Aktif';
        }

        $id = $request->input('id') ?? MyHelper::getPk(date('Ymd'), 'support_kritik_seq');
        FaqPengguna::updateOrCreate(['id' => $id], $data);
        return $this->resSuccess();
    }

    public function show(string $id)
    {
        $data = $this->getData($id, true);
        return view('support.faq.faqFormV', $data);
    }

    public function edit(string $id)
    {
        $data = $this->getData($id);
        return view('support.faq.faqFormV', $data);
    }

    public function update(Request $request, Faq $tanah)
    {
        //
    }

    public function destroy(string $id)
    {
        try {
            DB::beginTransaction();
            FaqPengguna::destroy($id);
            DB::commit();
            return $this->resSuccess('Berhasil Dihapus!');
        } catch (\Throwable $th) {
            DB::rollBack();
        }
    }

}
