<?php

namespace App\Http\Controllers\AsetIntelektual;
use Illuminate\Routing\Controller;

use App\Helpers\MyHelper;
use App\Models\Sistem\Files;
use App\Models\AsetIntelektual\Langganan;
use App\Models\Master\Master;
use App\Models\Master\MsSatker;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\DB;
use Symfony\Component\HttpKernel\Exception\NotFoundHttpException;

class LanggananController extends Controller
{
    /**
     * Display a listing of the resource.
     */
    protected $breadcums = ['Asset TIK', 'Daftar Langganan Jasa Khusus TIK'];
    protected $controller = '/sdm/langganan';
    protected $columns = ['Nama Satker', 'Nama Layanan', 'Penyedia', 'Tgl Mulai', 'Nilai'];
    protected $defColumns = [0,1,2,3,4,5];

    protected function canCreate(){
        return in_array(session('userData.current_role.ms_role_id'), config('constants.pelaksana_role'));
    }

    public function index()
    {
        //print_r(session('userData.current_role'));exit;

        $data = ['tableId' => 'dt-langganan', 'breadcums' => $this->breadcums, 'columns' => $this->columns, 'defColumns' => $this->defColumns, 'controller' => $this->controller, 'canCreate' => $this->canCreate()];
        return view('asset_tik.langganan.langgananV', $data);
    }

    public function gridData(Request $request)
    {
        $model = new Langganan();
        $pagingParams = $request->only(['start', 'length']);
        $searchParams =  $request->only(['columns']);
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
            $model = Langganan::where('id', $id)->first();
            if (!$model) {
                throw new NotFoundHttpException('Data Tidak Ditemukan');
            }

            $model = $model->toArray();
            $isNew = false;

            $ms_satker_id = $model['id_satker'];
        }else{
            $currentRole = session('userData.current_role');
            $ms_satker_id = $currentRole['ms_satker_id'] ?? $model['id_satker'];
        }

        if($readOnly){
            $breadcum = 'Detail';
        }

        $satkers = Master::getSatkersKeu();
        $nama_satker = MsSatker::where('inst_satkerkd', $ms_satker_id)->first();
        $tipe_beli = ['BELI BARU', 'PERPANJANGAN'];

        $model['kdsatker_keu']=$nama_satker['kdsatker_keu'];
        $model['inst_nama'] = MsSatker::where('inst_satkerkd', $ms_satker_id)->first()['inst_nama'];

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
            'tipebeliOptions' => MyHelper::generateSelectOptions([
                'data' => $tipe_beli,
                'text' => 'tipe_beli',
                'value' => null,
                'selected' => $model['tipe_beli'] ?? null,
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
        return view('asset_tik.langganan.langgananFormV', $data);
    }

    /**
     * Store a newly created resource in storage.
     */
    public function store(Request $request)
    {
        $isNew = $request->input('isNew');
        $customMessages = [
            'kdsatker_keu.required' => 'Satker harus diisi',
            'nm_layanan.required' => 'No Lisensi harus diisi',
            'nm_perusahaan.required' => 'Merk harus diisi',
            'tgl_mulai.required' => 'Lifetime harus diisi',
            'tgl_selesai.required' => 'PNBP harus diisi',
            'tipe_beli.required' => 'Tipe Beli harus diisi',
            'nilai.required' => 'PNBP harus diisi',
        ];
        $validate = [
            'kdsatker_keu' => 'required',
            'nm_layanan' => 'required',
            'nm_perusahaan' => 'required',
            'tgl_mulai' => 'required',
            'tgl_selesai' => 'required',
            'tipe_beli' => 'required',
            'nilai' => 'required',
        ];
        $id = $request->input('id') ?? MyHelper::getPk(date('Ymd'), 'asset_tik_langganan_seq');
        if($isNew){
            $validate['file_invoice'] = 'required|mimes:jpeg,png,pdf|max:2048';
            $customMessages['file_invoice.required'] = 'File Invoice harus diupload';
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
                'nm_layanan' => $request->input('nm_layanan'),
                'nm_perusahaan' => $request->input('nm_perusahaan'),
                'tgl_mulai' => $request->input('tgl_mulai'),
                'tgl_selesai' => $request->input('tgl_selesai'),
                'tipe_beli' => $request->input('tipe_beli'),
                'nilai' => str_replace('.','',$request->input('nilai')),
                'deskripsi' => $request->input('deskripsi'),
            ];
            
            if ($request->hasFile('file_invoice')) {
                $params = [
                    'kategori' => 'Asset TIK - Langganan Jasa TIK',
                    'dir' => 'asset-tik/langganan',
                    'fileKey' => 'file_invoice',
                    'pkey' => session('userData.username'),
                ];
                $fotos = Files::upload($request, $params);
                $data['file_invoice'] = $fotos['path'];
            }

            // echo "<pre>"; print_r($data);exit;

            Langganan::updateOrCreate(['id' => $id], $data);
            
            DB::commit();
            return $this->resSuccess('Berhasil Disimpan!',[
                'type' => 'redirect',
                'url' => \URL::to('/asset-tik/langganan')
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
        return view('asset_tik.langganan.langgananFormV', $data);
    }

    /**
     * Show the form for editing the specified resource.
     */
    public function edit(string $id)
    {
        $data = $this->getData($id);
        return view('asset_tik.langganan.langgananFormV', $data);
    }

    /**
     * Update the specified resource in storage.
     */
    public function update(Request $request, Langganan $langganan)
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
            Langganan::destroy($id);
            DB::commit();
            return $this->resSuccess('Berhasil Dihapus!');
        } catch (\Throwable $th) {
            DB::rollBack();
        }
    }

    public function cetakLabel($id)
    {
        $data = Langganan::findOne($id);
        $data = (array) $data;
        $pdf = MyHelper::generateLabelBankAsset($data);
        return $pdf->stream('label-asset-tak-berwujud.pdf');
    }

}