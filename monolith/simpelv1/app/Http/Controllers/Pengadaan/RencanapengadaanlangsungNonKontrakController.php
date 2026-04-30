<?php

namespace App\Http\Controllers\Pengadaan;

use App\Helpers\MyHelper;
use App\Http\Controllers\Controller;
use App\Models\Sdm\Rencanapengadaanlangsung;
use App\Models\Master;
use App\Models\Master\MsSatker;
use App\Models\Monsakti\BastNonKontrakHeader;
use App\Models\Monsakti\KontrakHeader;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\DB;
use Symfony\Component\HttpKernel\Exception\NotFoundHttpException;
use Symfony\Component\HttpKernel\Exception\UnauthorizedHttpException;

class RencanapengadaanlangsungNonKontrakController extends Controller
{
    /**
     * Display a listing of the resource.
     */
    private $controller = '/pengadaan/pengadaanlangsung-nonkontrak';
    protected $breadcums = ['Pengadaan'];

    public function __construct(Request $request)
    {

        $this->breadcums = array_merge($this->breadcums, [['link'=>$this->controller,'title'=>'Data Pengadaan Langsung Non Kontrak']]);
    }

    protected function canCreate()
    {
        // return true;
        return in_array(session('userData.current_role.ms_role_id'), config('constants.pelaksana_role'));
    }

    public function index()
    {   
        $satkers = DB::table('ms_satker as a')
            ->where('a.is_pusat', '=', '0')
            ->where('a.inst_satkerkd', '!=', '99')
            ->where('a.inst_satkerkd', '!=', '97')
            ->where('a.kdsatker_keu', '!=', null)
            ->where('a.inst_level', '!=', '4')->get()->toArray();

        $data = [
            'tableId' => 'dt-rencanapengadaanlangsung', 
            'breadcums' => $this->breadcums, 
            'canCreate' => $this->canCreate(),
            'satkerAllOptions' => MyHelper::generateSelectOptions([
                'data' => $satkers,
                'text' => 'inst_akronim',
                'value' => 'kdsatker_keu',
                'selected' => null,
            ]),
        ];
        return view('pengadaan.rencanapengadaanlangsung-nonkontrak.rencanapengadaanlangsungV', $data);
    }

    public function gridData(Request $request)
    {
        $model = new BastNonKontrakHeader();
        $pagingParams = $request->only(['start', 'length']);
        $searchParams = $request->only(['columns']);
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
            $model = BastNonKontrakHeader::where('id_bast', $id)->first();
            if (!$model) {
                throw new NotFoundHttpException('Data Tidak Ditemukan');
            }

            $model = $model->toArray();
            $isNew = false;
            $ms_satker_id = $model['kode_satker'];
        }else{
            $currentRole = session('userData.current_role');
            $ms_satker_id = $currentRole['ms_satker_id'] ?? $model['inst_satkerkd'];
        }
        $breadcum = 'Detail';
        $model['inst_nama'] = MsSatker::where('kdsatker_keu', $ms_satker_id)->first()['inst_nama'];
        $kontrak = new BastNonKontrakHeader();
        $dataDetail = $kontrak->getDetail($model['id_bast']);
        $data = [
            'model' => $model,
            'isNew' => $isNew,
            'controller' => $this->controller,
            'breadcums' => array_merge($this->breadcums, [$breadcum]),
            'judul' => $breadcum,
            'readOnly' => true,
            'dataDetail' => $dataDetail,
        ];
        return $data;
    }

    /**
     * Show the form for creating a new resource.
     */
    public function create()
    {
        if (!$this->canCreate()) {
            throw new UnauthorizedHttpException('Tidak Punya Akses');
        }
        $data = $this->getData();
        return view('pengadaan.rencanapengadaanlangsung.rencanapengadaanlangsungFormV', $data);
    }

    /**
     * Store a newly created resource in storage.
     */
    public function store(Request $request)
    {
        //echo $request->input('nilai_kontrak');exit;

        $isNew = $request->input('isNew');
        $customMessages = [
            'jenis_pengadaan.required' => 'Jenis Pengadaan harus dipilih',
            'jenis_kontrak.required' => 'Jenis Kontrak harus diisi',
            'no_kontrak.required' => 'Nomor Kontrak harus diisi',
            'nilai_kontrak.required' => 'Nilai Kontrak harus diisi',
            'tgl_kontrak.required' => 'Tanggal Kontrak harus diisi',
        ];
        $validate = [
            'jenis_pengadaan' => 'required',
            'jenis_kontrak' => 'required',
            'no_kontrak' => 'required',
            'nilai_kontrak' => 'required',
            'tgl_kontrak' => 'required',
        ];
        $id = $request->input('id') ?? MyHelper::getPk(date('Ymd'), 'pengadaan_rencana_langsung_seq');
        if($isNew){
            $validate['file_kontrak'] = 'required|mimes:jpeg,png,pdf|max:2048';
            $customMessages['file_kontrak.required'] = 'File Kontrak harus diupload';
        }else{

        }
        $request->validate($validate, $customMessages);
        try {
            DB::beginTransaction();
            $currentRole = session('userData.current_role');
            $ms_satker_id = $currentRole['ms_satker_id'] ?? '';
            $ms_satker_id_keu = $currentRole['ms_satker_id_keu'] ?? '';
            $data = [
                'inst_satkerkd' => $ms_satker_id,
                'kdsatker_keu' => $ms_satker_id_keu,
                'jenis_kontrak' => $request->input('jenis_kontrak'),
                'no_kontrak' => $request->input('no_kontrak'),
                'nilai_kontrak' => str_replace('.', '', $request->input('nilai_kontrak')), //,
                'tgl_kontrak' => $request->input('tgl_kontrak'),
                'jenis_pengadaan' => $request->input('jenis_pengadaan'),
            ];
            if($request->input('no_spk')) $data['no_spk'] = $request->input('no_spk');
            if($request->input('tgl_spk')) $data['tgl_spk'] = $request->input('tgl_spk');
            if($request->input('jangka_waktu_pelaksanaan')) $data['jangka_waktu_pelaksanaan'] = $request->input('jangka_waktu_pelaksanaan');
            if($request->input('jangka_waktu_pelaksanaan')) $data['jangka_waktu_pelaksanaan'] = $request->input('jangka_waktu_pelaksanaan');
            $filepath = 'uploads/pengadaan/rencanapengadaanlangsung';
            if ($request->hasFile('file_kontrak')) {
                $file = $request->file('file_kontrak');
                $fileName = $id.'_file_kontrak'.'.'.$file->getClientOriginalExtension();
                $filesave = $filepath.'/'.$fileName;
                $file->move($filepath, $fileName);
                $data['file_kontrak'] = $filesave;
            }
            if ($request->hasFile('konsep_hps')) {
                $file = $request->file('konsep_hps');
                $fileName = $id.'_konsep_hps'.'.'.$file->getClientOriginalExtension();
                $filesave = $filepath.'/'.$fileName;
                $file->move($filepath, $fileName);
                $data['konsep_hps'] = $filesave;
            }
            if ($request->hasFile('surat_keputusan_penyedia')) {
                $file = $request->file('surat_keputusan_penyedia');
                $fileName = $id.'_surat_keputusan_penyedia'.'.'.$file->getClientOriginalExtension();
                $filesave = $filepath.'/'.$fileName;
                $file->move($filepath, $fileName);
                $data['surat_keputusan_penyedia'] = $filesave;
            }
            if ($request->hasFile('bast')) {
                $file = $request->file('bast');
                $fileName = $id.'_bast'.'.'.$file->getClientOriginalExtension();
                $filesave = $filepath.'/'.$fileName;
                $file->move($filepath, $fileName);
                $data['bast'] = $filesave;
            }
            if ($request->hasFile('ba_pembayaran')) {
                $file = $request->file('ba_pembayaran');
                $fileName = $id.'_ba_pembayaran'.'.'.$file->getClientOriginalExtension();
                $filesave = $filepath.'/'.$fileName;
                $file->move($filepath, $fileName);
                $data['ba_pembayaran'] = $filesave;
            }
            if ($request->hasFile('nodis_pengantar_kuitansi')) {
                $file = $request->file('nodis_pengantar_kuitansi');
                $fileName = $id.'_nodis_pengantar_kuitansi'.'.'.$file->getClientOriginalExtension();
                $filesave = $filepath.'/'.$fileName;
                $file->move($filepath, $fileName);
                $data['nodis_pengantar_kuitansi'] = $filesave;
            }
            Rencanapengadaanlangsung::updateOrCreate(['id' => $id], $data);
            DB::commit();
            return $this->resSuccess(
                'Berhasil Disimpan!',
                [
                    'type' => 'redirect',
                    'url' => \URL::to('/pengadaan/rencanapengadaanlangsung')
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
        return view('pengadaan.rencanapengadaanlangsung-nonkontrak.rencanapengadaanlangsungViewV', $data);
    }

    /**
     * Show the form for editing the specified resource.
     */
    public function edit(string $id)
    {
        if (!$this->canCreate()) {
            throw new UnauthorizedHttpException('Tidak Punya Akses');
        }
        $data = $this->getData($id);
        return view('pengadaan.rencanapengadaanlangsung.rencanapengadaanlangsungFormV', $data);
    }

    /**
     * Update the specified resource in storage.
     */
    public function update(Request $request)
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
            Rencanapengadaanlangsung::destroy($id);
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
