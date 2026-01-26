<?php

namespace App\Http\Controllers\Bmn\Penghapusan;
use Illuminate\Routing\Controller;

use App\Helpers\MyHelper;
use App\Models\Bmn\PengajuanPenghapusanBmn\PengajuanPenghapusanBmn;
use App\Models\Bmn\PenghapusanSk;
use App\Models\Master\Master;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\DB;
use Symfony\Component\HttpKernel\Exception\NotFoundHttpException;
use App\Models\Bmn\PengajuanPenghapusanBmn\PengajuanPenghapusanBmn as Model;
use App\Models\Bmn\PengajuanPenghapusanBmn\PengajuanPenghapusanBmnAktivitas as Aktivitas;
use App\Models\Bmn\PengajuanPenghapusanBmn\PengajuanPenghapusanBmnAsset as Asset;
use App\Models\Bmn\PengajuanPenghapusanBmn\PengajuanPenghapusanBmnAssetFile as File;
use App\Models\Master\MsSatker;

class PenghapusanMonitorController extends Controller
{
    /**
     * Display a listing of the resource.
     */
    protected $breadcums = ['BMN', 'Monitoring Penghapusan'];
    protected $controller = '/bmn/penghapusan/penghapusanmonitor';
    public function index()
    {
        $data = ['tableId' => 'dt-penghapusan', 'breadcums' => $this->breadcums,'controller'=>$this->controller];
        return view('bmn.penghapusansk.monitor.pengajuanV', $data);
    }

    public function gridData(Request $request)
    {
        $model = new PengajuanPenghapusanBmn();
        $pagingParams = $request->only(['start', 'length']);
        $searchParams = $request->only(['columns']);
        $data = $model->getDataGridDetail($pagingParams, $searchParams);
        return response()->json([
            'data' => $data['data'],
            'recordsTotal' => $data['total'],
            'recordsFiltered' => $data['total'],
        ]);
    }

    function getData($id = null)
    {
        $model = [];
        $isNew = true;
        $breadcum = 'Tambah';

        if ($id) {
            $breadcum = 'Ubah';
            $model = Model::where('id', $id)->first();
            if (!$model) {
                throw new NotFoundHttpException('Data Tidak Ditemukan');
            }
            $model = $model->toArray();
            $isNew = false;
        }

        $data = [
            'model' => $model,
            'isNew' => $isNew,
            'controller' => $this->controller,
            'breadcums' => array_merge($this->breadcums, [$breadcum]),
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
        $data = $this->getData($id);
        $pengajuan = Model::where('id', $id)->first();
        if (!$pengajuan) {
            throw new NotFoundHttpException('Data Tidak Ditemukan');
        }
        $satker = MsSatker::where('inst_satkerkd', $pengajuan->inst_satkerkd)->first();
        $pengajuan->inst_nama = $satker->inst_nama;
        //$_GET['satker'] cuma ada klo diliat validator pusat /kejati
        $currentRole = session('userData.current_role');
        $whereData = ['ms_satker_id' => $_GET['satker'] ?? $currentRole['ms_satker_id'], 'pengajuan_id' => $pengajuan->id];
        $whereSatker = ['inst_satkerkd' => $_GET['satker'] ?? $currentRole['ms_satker_id']];

        if ($currentRole['ms_satker_id'] == '00' && !isset($_GET['satker'])) {
            $whereData['ms_satker_pusat_id'] = $currentRole['ms_satker_pusat_id'];
            $whereSatker['unitkerja_idk'] = $currentRole['ms_satker_pusat_id'];
        }

        //model utama
        $model = $pengajuan ?? [];
        $isNew = empty($model) ? true : false;

        //aktifitas
        $msAktivitasId = $model->ms_aktifitas_id;
        $whereAct = ['ms_aktifitas_id' => $msAktivitasId,'group'=>'BMN'];
        $aktifitasHistories = Aktivitas::getDetail($model->id);
        $aktifitasOptions = [];
        $currentAktivitas = [];

        $data = [
            'model' => $model,
            'controller' => $this->controller,
            'aktivitasOptions' => $aktifitasOptions,
            'aktivitas' => $currentAktivitas,
            'aktivitasHistories' => $aktifitasHistories,
            'breadcums' => array_merge($this->breadcums, ['Pengajuan']),
            'isNew' => $isNew,
        ];
        return view('bmn.penghapusansk.monitor.pengajuanFormSatkerIsiV', $data);
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
