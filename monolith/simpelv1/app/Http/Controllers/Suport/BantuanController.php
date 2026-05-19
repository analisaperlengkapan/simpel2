<?php

namespace App\Http\Controllers\Suport;

use App\Helpers\MyHelper;
use App\Http\Controllers\Controller;
use App\Models\Suport\Bantuan;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\DB;
use Symfony\Component\HttpKernel\Exception\NotFoundHttpException;

class BantuanController extends Controller
{
    /**
     * Display a listing of the resource.
     */
    protected $breadcums = ['Suport', 'Panduan'];

    public function index()
    {
        return view('suport.bantuan.bantuanV', ['tableId' => 'dt-bantuan', 'breadcums' => $this->breadcums]);
    }

    public function gridData(Request $request)
    {
        $model = new bantuan;
        $pagingParams = $request->only(['start', 'length']);
        $searchParams = $request->only(['search', 'filterBy']);
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
            $model = bantuan::where('id', $id)->first();
            if (! $model) {
                throw new NotFoundHttpException('Data Tidak Ditemukan');
            }

            $model = $model->toArray();
            $isNew = false;
        }
        if ($readOnly) {
            $breadcum = 'Detail';
        }
        $data = [
            'model' => $model,
            'isNew' => $isNew,
            'breadcums' => array_merge($this->breadcums, [$breadcum]),
            'judul' => $breadcum,
            'readOnly' => $readOnly,
        ];

        return $data;
    }

    public function create()
    {
        $data = $this->getData();

        return view('suport.bantuan.bantuanFormV', $data);
    }

    public function store(Request $request)
    {
        $isNew = $request->input('isNew');
        $customMessages = [
            'judul.required' => 'Judul harus diisi',
        ];
        $validate = [
            'judul' => 'required',
        ];
        $id = $request->input('id') ?? MyHelper::getPk(date('Ymd'), 'suport_bantuan_seq');
        if ($isNew) {
            $validate['file_panduan'] = 'required|mimes:jpeg,png,pdf|max:2048';
            $customMessages['file_panduan.required'] = 'File SK harus diupload';
        } else {

        }
        $request->validate($validate, $customMessages);
        try {
            DB::beginTransaction();
            $data = [
                'judul' => $request->input('judul'),
                'status' => 'Aktif',

            ];

            if ($request->hasFile('file_panduan')) {
                $filepath = 'uploads/suport/panduan';
                $file = $request->file('file_panduan');
                $fileName = $id.'_panduan'.'.'.$file->getClientOriginalExtension();
                $filesave = $filepath.'/'.$fileName;
                $file->move($filepath, $fileName);
                $data['file_panduan'] = $filesave;
            }

            bantuan::updateOrCreate(['id' => $id], $data);

            DB::commit();

            return $this->resSuccess();
        } catch (\Throwable $th) {
            DB::rollBack();
            $errorMessage = $th->getMessage();

            return $this->resError($errorMessage);
        }

    }

    public function show(string $id)
    {
        $data = $this->getData($id, true);

        return view('suport.bantuan.bantuanFormV', $data);
    }

    public function edit(string $id)
    {
        $data = $this->getData($id);

        return view('suport.bantuan.bantuanFormV', $data);
    }

    public function update(Request $request, Faq $tanah)
    {
        //
    }

    public function destroy(string $id)
    {
        try {
            DB::beginTransaction();
            bantuan::destroy($id);
            DB::commit();

            return $this->resSuccess('Berhasil Dihapus!');
        } catch (\Throwable $th) {
            DB::rollBack();
        }
    }

    public function bukuPanduan()
    {
        $path = config('constants.buku_panduan_path');
        $exist = \File::exists($path);
        if (! $exist) {
            throw new NotFoundHttpException;
        }

        return response()->file($path);
    }
}
