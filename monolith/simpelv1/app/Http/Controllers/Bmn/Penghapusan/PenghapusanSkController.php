<?php

namespace App\Http\Controllers\Bmn\Penghapusan;

use App\Helpers\MyHelper;
use App\Http\Controllers\Controller;
use App\Models\Bmn\PenghapusanSk;
use App\Models\Master;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\DB;
use Symfony\Component\HttpKernel\Exception\NotFoundHttpException;

class PenghapusanSkController extends Controller
{
    /**
     * Display a listing of the resource.
     */
    protected $breadcums = ['BMN', 'Pengajuan SK Penghapusan'];

    protected $controller = '/bmn/penghapusan/penghapusansk';

    public function index()
    {
        $data = ['tableId' => 'dt-penghapusan', 'breadcums' => $this->breadcums, 'controller' => $this->controller];

        return view('bmn.penghapusansk.penghapusanV', $data);
    }

    public function gridData(Request $request)
    {
        $model = new penghapusanSk;
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
        if ($id) {
            $breadcum = 'Ubah';
            $model = PenghapusanSk::where('id', $id)->first();
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
        $jenis = ['UAKPB', 'UAPPB-W', 'UAPPB-E1', 'UAPB'];
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
            'controller' => $this->controller,
        ];

        return $data;
    }

    /**
     * Show the form for creating a new resource.
     */
    public function create()
    {
        $data = $this->getData();

        return view('bmn.penghapusansk.penghapusanFormV', $data);
    }

    /**
     * Store a newly created resource in storage.
     */
    public function store(Request $request)
    {
        $isNew = $request->input('isNew');
        $customMessages = [
            'kdsatker_keu.required' => 'Satker harus diisi',
            'tgl_surat.required' => 'Tanggal Surat harus diisi',
        ];
        $validate = [
            'kdsatker_keu' => 'required',
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
            $data = [
                'kdsatker_keu' => $request->input('kdsatker_keu'),
                'jenis_sk' => $request->input('jenis_sk'),
                'no_surat' => $request->input('no_surat'),
                'tgl_surat' => $request->input('tgl_surat'),
                'dikeluarkan_di' => $request->input('dikeluarkan_di'),
            ];
            if ($request->hasFile('file_sk')) {
                $filepath = 'uploads/bmn/penghapusansk';
                $file = $request->file('file_sk');
                $fileName = $id.'_sk'.'.'.$file->getClientOriginalExtension();
                $filesave = $filepath.'/'.$fileName;
                $file->move($filepath, $fileName);
                $data['file_sk'] = $filesave;
            }
            PenghapusanSk::updateOrCreate(['id' => $id], $data);

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

        return view('bmn.penghapusansk.penghapusanFormV', $data);
    }

    /**
     * Show the form for editing the specified resource.
     */
    public function edit(string $id)
    {
        $data = $this->getData($id);

        return view('bmn.penghapusansk.penghapusanFormV', $data);
    }

    /**
     * Update the specified resource in storage.
     */
    public function update(Request $request, PenghapusanSk $hakcipta)
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
            PenghapusanSk::destroy($id);
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
