<?php

namespace App\Http\Controllers\Bmn\Pemanfaatan;

use App\Helpers\MyHelper;
use App\Http\Controllers\Controller;
use App\Models\Bmn\PemanfaatanSk;
use App\Models\Bmn\PemanfaatanSkFile;
use App\Models\Master;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\DB;
use Symfony\Component\HttpKernel\Exception\NotFoundHttpException;

class PemanfaatanMonitorController extends Controller
{
    /**
     * Display a listing of the resource.
     */
    protected $breadcums = ['BMN', 'Monitoring Pemanfaatan'];
    protected $columns = ['Nama Satker', 'Nama Barang','Tgl. Mulai', 'Tgl. Berakhir'];
    protected $defColumns = [0,1,2,3];
    public function index()
    {
        //$data = ['tableId' => 'dt-pemanfaatan', 'breadcums' => $this->breadcums];
        //return view('bmn.pemanfaatansk.pemanfaatanmonitorV', $data);

        return view('bmn.pemanfaatansk.pemanfaatanmonitorV', [
            'tableId' => 'dt-pemanfaatan',
            'breadcums' => $this->breadcums,
            'columns' => $this->columns,
            'defColumns' => $this->defColumns
        ]);     

    }

    public function gridData(Request $request)
    {
        $model = new PemanfaatanSk();
        $pagingParams = $request->only(['start', 'length']);
        //$searchParams =  $request->only(['search',  'filterBy']);
        $searchParams =  $request->only(['columns']);
        //dd($searchParams);
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
        $modelSk = [];
        $isNew = true;
        $breadcum = 'Tambah';
        if ($id) {
            $breadcum = 'Ubah';
            $model = PemanfaatanSk::where('id', $id)->first();
            if (!$model) {
                throw new NotFoundHttpException('Data Tidak Ditemukan');
            }

            $model = $model->toArray();
            $modelSk = PemanfaatanSkFile::where('id_pemanfaatan', $id)->get();
            $modelSk = $modelSk->toArray();
            $isNew = false;
        }
        if($readOnly){
            $breadcum = 'Detail';
        }
        $satkers = Master::getSatkersKeu();
        $jenis = ['UAKPB', 'UAPPB-W', 'UAPPB-E1', 'UAPB'];
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
                'data' => Master::getBarangAset(),
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
        if($isNew){
            $validate['file_sk'] = 'required|mimes:jpeg,png,pdf|max:2048';
            $customMessages['file_spk.required'] = 'File SK harus diupload';
        }else{

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
                $filepath = 'uploads/bmn/pemanfaatansk';
                $file = $request->file('file_sk');
                $fileName = $id.'_sk'.'.'.$file->getClientOriginalExtension();
                $filesave = $filepath.'/'.$fileName;
                $file->move($filepath, $fileName);
                $data['file_sk'] = $filesave;
            }
            PemanfaatanMonitor::updateOrCreate(['id' => $id], $data);

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
        return view('bmn.pemanfaatansk.pemanfaatanmonitoringFormV', $data);
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
    public function update(Request $request, PemanfaatanMonitor $hakcipta)
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
            PemanfaatanMonitor::destroy($id);
            DB::commit();
            return $this->resSuccess('Berhasil Dihapus!');
        } catch (\Throwable $th) {
            DB::rollBack();
        }
    }



    public function cetakLabel($id)
    {
        $data = PemanfaatanMonitor::findOne($id);
        $data = (array) $data;
        $pdf = MyHelper::generateLabelBankAsset($data);
        return $pdf->stream('label-asset-tak-berwujud.pdf');
    }

}
