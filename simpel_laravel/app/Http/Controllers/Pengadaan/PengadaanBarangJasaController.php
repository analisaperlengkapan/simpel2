<?php

namespace App\Http\Controllers\Pengadaan;
use Illuminate\Routing\Controller;

use App\Helpers\MyHelper;
use App\Models\Master\Master;
use App\Models\Master\MsSatker;
use App\Models\Pengadaan\PengadaanBarangJasa;
use App\Models\Pengadaan\PengadaanBarangJasa\PengadaanBarangJasaBast;
use App\Models\Pengadaan\PengadaanBarangJasa\PengadaanBarangJasaHps;
use App\Models\Pengadaan\PengadaanBarangJasa\PengadaanBarangJasaKontrak;
use App\Models\Pengadaan\PengadaanBarangJasa\PengadaanBarangJasaNodis;
use App\Models\Pengadaan\PengadaanBarangJasa\PengadaanBarangJasaRingkasan;
use App\Models\Pengadaan\PengadaanBarangJasa\PengadaanBarangJasaSkppbj;
use App\Models\Pengadaan\PengadaanBarangJasa\PengadaanBarangJasaSpk;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\DB;
use Mccarlosen\LaravelMpdf\Facades\LaravelMpdf;
use Symfony\Component\HttpKernel\Exception\NotFoundHttpException;
use Symfony\Component\HttpKernel\Exception\UnauthorizedHttpException;

class PengadaanBarangJasaController extends Controller
{
    /**
     * Display a listing of the resource.
     */
    private $controller = '/pengadaan/barang-jasa';
    protected $breadcums = ['Pengadaan'];

    public function __construct(Request $request)
    {
        $this->breadcums = array_merge($this->breadcums, [['link'=>$this->controller,'title'=>'Data Pengadaan Barang dan Jasa']]);
    }

    protected function canCreate()
    {
        // return true;
        return session('userData.current_role.ms_satker_id') == config('constants.ms_satker_kejagung_id');
    }

    public function index()
    {
        $data = ['tableId' => 'dt-rencanapengadaanlangsung', 'breadcums' => $this->breadcums, 'canCreate' => $this->canCreate()];
        return view('pengadaan.barang_jasa.indexV', $data);
    }

    public function gridData(Request $request)
    {
        $model = new PengadaanBarangJasa();
        $pagingParams = $request->only(['start', 'length']);
        $searchParams = $request->only(['columns']);
        $data = $model->getDataGrid($pagingParams, $searchParams);
        return response()->json([
            'data' => $data['data'],
            'recordsTotal' => $data['total'],
            'recordsFiltered' => $data['total'],
        ]);
    }

    public function gridDataAnggaran(Request $request){
        $sql = "
            select a.*, concat( a.kode_kegiatan,'.',a.kode_output,'.',a.kode_suboutput,'.',a.kode_komponen,'.',a.kode_subkomponen,'.',a.kode_akun ) as kode_anggaran
            from monsakti_anggaran a
        ";
        $query = DB::select($sql);
        return response()->json([
            'data' => $query,
        ]);
    }

    function getData($id = null, $readOnly = false)
    {
        $model = [];
        $isNew = true;
        $breadcum = 'Tambah';
        if ($id) {
            $breadcum = 'Ubah';
            $model = PengadaanBarangJasa::where('id', $id)->first();
            if (!$model) {
                throw new NotFoundHttpException('Data Tidak Ditemukan');
            }

            $model = $model->toArray();
            $isNew = false;
            $ms_satker_id = ""; //$model['inst_satkerkd'];
        }else{
            $currentRole = session('userData.current_role');
            $ms_satker_id = ""; //$currentRole['ms_satker_id'] ?? $model['inst_satkerkd'];
        }
        if($readOnly){
            $breadcum = 'Detail';
        }
        $jenis = ['Diumumkan di SIRUP', 'Tidak Diumumkan di SIRUP'];
        $jenisPengadaan = (object) array(
            (object) array('id'=>1,'text'=>'Pengadaan lebih dari 200jt'),
            (object) array('id'=>2,'text'=>'Pengadaan s.d 200jt'),
        );
        //$model['inst_nama'] = MsSatker::where('inst_satkerkd', $ms_satker_id)->first()['inst_nama'];
        $satkerTerpilih = Master::getSatkers();
        $hps = PengadaanBarangJasaHps::where('id',$id)->first();
        $skppbj = PengadaanBarangJasaSkppbj::where('id',$id)->first();
        $spk = PengadaanBarangJasaSpk::where('id',$id)->first();
        $ringkasan = PengadaanBarangJasaRingkasan::where('id',$id)->first();
        $kontrak = PengadaanBarangJasaKontrak::where('id',$id)->first();
        $bast = PengadaanBarangJasaBast::where('id',$id)->first();
        $nodis = PengadaanBarangJasaNodis::where('id',$id)->first();
        $data = [
            'model' => $model,
            'isNew' => $isNew,
            'controller' => $this->controller,
            'breadcums' => array_merge($this->breadcums, [$breadcum]),
            'judul' => $breadcum,
            'readOnly' => $readOnly,
            'hps' => $hps,
            'skppbj' => $skppbj,
            'spk' => $spk,
            'ringkasan' => $ringkasan,
            'kontrak' => $kontrak,
            'bast' => $bast,
            'nodis' => $nodis,
            'jenisPengadaanOptions' => MyHelper::generateSelectOptions([
                'data' => $jenisPengadaan,
                'value' => 'id',
                'text' => 'text',
                'type' => 'raw',
                'selected' => $model['jenisPengadaan'] ?? null,
            ]),
            'jenisOptions' => MyHelper::generateSelectOptions([
                'data' => $jenis,
                'text' => 'jenis',
                'value' => null,
                'selected' => $model['jenis_kontrak'] ?? null,
            ]),
            'satkerOptions' => MyHelper::generateSelectOptions(['data' => $satkerTerpilih, 'text' => 'inst_nama', 'value' => 'inst_satkerkd', 'selected' => $ms_satker_id]),
        ];

        return $data;
    }

    /**
     * Show the form for creating a new resource.
     */
    public function create()
    {
        // if (!$this->canCreate()) {
        //     throw new UnauthorizedHttpException('Tidak Punya Akses');
        // }
        $data = $this->getData();
        return view('pengadaan.barang_jasa.formV', $data);
    }

    /**
     * Store a newly created resource in storage.
     */
    public function store(Request $request)
    {
        //return $request->input('nama_pengadaan');exit;
        //print_r($request);exit;

        $isNew = $request->input('isNew');
        $customMessages = [
            'nama_pengadaan.required' => 'Nama Pengadaan harus diisi',
            'jenis_pengadaan.required' => 'Jenis Pengadaan harus dipilih',
            'kode_barang.required' => 'Kode Barang harus diisi',
            'nilai.required' => 'Nilai Pengadaan harus diisi',
        ];
        $validate = [
            'nama_pengadaan' => 'required',
            'jenis_pengadaan' => 'required',
            'kode_barang' => 'required',
            'nilai' => 'required',
        ];
        $id = $request->input('id') ?? MyHelper::getPk(date('Ymd'), 'pengadaan_barang_jasa_seq');
        if($isNew){
            //$validate['file_kontrak'] = 'required|mimes:jpeg,png,pdf|max:2048';
            //$customMessages['file_kontrak.required'] = 'File Kontrak harus diupload';
        }else{

        }
        $request->validate($validate, $customMessages);
        try {
            DB::beginTransaction();
            // $currentRole = session('userData.current_role');
            // $ms_satker_id = $currentRole['ms_satker_id'] ?? '';
            // $ms_satker_id_keu = $currentRole['ms_satker_id_keu'] ?? '';
            $data = [
                // 'inst_satkerkd' => $ms_satker_id,
                // 'kdsatker_keu' => $ms_satker_id_keu,

                'nama_pengadaan' => $request->input('nama_pengadaan'),
                'jenis_pengadaan' => $request->input('jenis_pengadaan'),
                'kode_barang' => $request->input('kode_barang'),
                'kode_anggaran' => $request->input('anggaran_text'),
                'nilai' => str_replace('.', '', $request->input('nilai')), //,
            ];
            //return print_r($data);exit;
            PengadaanBarangJasa::updateOrCreate(['id' => $id], $data);
            DB::commit();
            return $this->resSuccess(
                'Berhasil Disimpan!',
                [
                    'type' => 'redirect',
                    'url' => \URL::to('/pengadaan/barang-jasa')
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
        return view('pengadaan.barang_jasa.viewV', $data);
    }

    /**
     * Show the form for editing the specified resource.
     */
    public function edit(string $id)
    {
        // if (!$this->canCreate()) {
        //     throw new UnauthorizedHttpException('Tidak Punya Akses');
        // }
        $data = $this->getData($id);
        return view('pengadaan.barang_jasa.formV', $data);
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
            PengadaanBarangJasa::destroy($id);
            DB::commit();
            return $this->resSuccess('Berhasil Dihapus!');
        } catch (\Throwable $th) {
            DB::rollBack();
        }
    }

    public function saveHps(Request $request) {
        $customMessages = [
            'no_hps.required' => 'No HPS harus diisi',
            'tgl_hps.required' => 'Tanggal HPS harus dipilih',
            'nip_penandatangan.required' => 'NIP Penandatangan harus diisi',
            'nama_penandatangan.required' => 'Nama Penandatangan harus diisi',
            'pangkat_penandatangan.required' => 'Pangkat Penandatangan harus diisi',
            'barang.required' => 'Barang harus diisi',
        ];
        $validate = [
            'no_hps' => 'required',
            'tgl_hps' => 'required',
            'nip_penandatangan' => 'required',
            'nama_penandatangan' => 'required',
            'pangkat_penandatangan' => 'required',
            'barang' => 'required',
        ];
        $request->validate($validate, $customMessages);
        try {
            DB::beginTransaction();
            $data = [
                'no_hps' => $request->input('no_hps'),
                'tgl_hps' => $request->input('tgl_hps'),
                'nip_penandatangan' => $request->input('nip_penandatangan'),
                'nama_penandatangan' => $request->input('nama_penandatangan'),
                'pangkat_penandatangan' => $request->input('pangkat_penandatangan'),
                'keterangan' => $request->input('keterangan'),
                'barang' => json_encode($request->input('barang')),
            ];
            PengadaanBarangJasaHps::updateOrCreate(['id' => $request->input('id')], $data);
            DB::commit();
            return $this->resSuccess(
                'Berhasil Disimpan!',
                [
                    'type' => 'redirect',
                    'url' => \URL::to('/pengadaan/barang-jasa/'.$request->input('id').'/edit')
                ]);
        } catch (\Throwable $th) {
            DB::rollBack();
            $errorMessage = $th->getMessage();
            return $this->resError($errorMessage);
        }
    }

    public function saveSkppbj(Request $request) {
        $customMessages = [
            'nama_penandatangan.required' => 'Nama harus diisi',
            'nip_penandatangan.required' => 'Nip harus diisi',
            'pangkat_penandatangan.required' => 'Pangkat harus diisi',
            'jabatan_penandatangan.required' => 'Jabatan harus diisi',
            'alamat.required' => 'Alamat harus diisi',
            'tgl_skppbj.required' => 'Tanggal SKPPBJ diisi',
            'penyedia.required' => 'Penyedia harus diisi',
        ];
        $validate = [
            'nama_penandatangan' => 'required',
            'nip_penandatangan' => 'required',
            'pangkat_penandatangan' => 'required',
            'jabatan_penandatangan' => 'required',
            'alamat' => 'required',
            'tgl_skppbj' => 'required',
            'penyedia' => 'required',
        ];
        $request->validate($validate, $customMessages);
        try {
            DB::beginTransaction();
            $data = [
                'nama_penandatangan' => $request->input('nama_penandatangan'),
                'nip_penandatangan' => $request->input('nip_penandatangan'),
                'pangkat_penandatangan' => $request->input('pangkat_penandatangan'),
                'jabatan_penandatangan' => $request->input('jabatan_penandatangan'),
                'alamat' => $request->input('alamat'),
                'tgl_skppbj' => $request->input('tgl_skppbj'),
                'penyedia' => json_encode($request->input('penyedia')),
            ];
            PengadaanBarangJasaSkppbj::updateOrCreate(['id' => $request->input('id')], $data);
            DB::commit();
            return $this->resSuccess(
                'Berhasil Disimpan!',
                [
                    'type' => 'redirect',
                    'url' => \URL::to('/pengadaan/barang-jasa/'.$request->input('id').'/edit')
                ]);
        } catch (\Throwable $th) {
            DB::rollBack();
            $errorMessage = $th->getMessage();
            return $this->resError($errorMessage);
        }
    }

    public function saveSpk(Request $request) {
        $customMessages = [
            'no_spk.required' => 'Nomor SPK harus diisi',
            'no_permintaan.required' => 'Nomor Surat Permintaan Penawaran harus diisi',
            'tgl_permintaan.required' => 'Tanggal Surat Permintaan Penawaran harus diisi',
            'no_ba.required' => 'Nomor Berita Acara Negoisasi harus diisi',
            'tgl_ba.required' => 'Tanggal Berita Acara Negoisasi harus diisi',
            'tgl_mulai.required' => 'Tanggal Mulai Pelaksanaan harus diisi',
            'tgl_spk.required' => 'Tanggal SPK harus diisi',
            'tgl_selesai.required' => 'Tanggal Selesai Pelaksanaan harus diisi',
            'nama_penyedia.required' => 'Nama Penandatangan Penyedia harus diisi',
            'keterangan.required' => 'Keterangan SPK harus diisi',
            'instruksi.required' => 'Instruksi Kepada Penyedia harus diisi',
        ];
        $validate = [
            'no_spk' => 'required',
            'no_permintaan' => 'required',
            'tgl_permintaan' => 'required',
            'no_ba' => 'required',
            'tgl_ba' => 'required',
            'tgl_mulai' => 'required',
            'tgl_spk' => 'required',
            'tgl_selesai' => 'required',
            'nama_penyedia' => 'required',
            'keterangan' => 'required',
            'instruksi' => 'required',
        ];
        $request->validate($validate, $customMessages);
        try {
            DB::beginTransaction();
            PengadaanBarangJasaSpk::updateOrCreate(['id' => $request->input('id')], $request->input());
            DB::commit();
            return $this->resSuccess(
                'Berhasil Disimpan!',
                [
                    'type' => 'redirect',
                    'url' => \URL::to('/pengadaan/barang-jasa/'.$request->input('id').'/edit')
                ]);
        } catch (\Throwable $th) {
            DB::rollBack();
            $errorMessage = $th->getMessage();
            return $this->resError($errorMessage);
        }
    }

    public function saveRingkasan(Request $request) {
        $customMessages = [
            'no_dipa.required' => 'Nomor DIPA harus diisi',
            'tgl_dipa.required' => 'Tanggal DIPA harus diisi',
            'cara_pembayaran.required' => 'Cara Pembayaran harus diisi',
            'alamat_penyedia.required' => 'Alamat Kantor Penyedia harus diisi',
            'nama_bank.required' => 'Nama Bank Rek. Penyedia harus diisi',
            'kantor_bank.required' => 'Kantor Cabang Bank harus diisi',
            'no_rek.required' => 'No Rekening harus diisi',
            'npwp.required' => 'NPWP harus diisi',
            'sanksi.required' => 'Ketentuan Sanksi harus diisi'
        ];
        $validate = [
            'no_dipa' => 'required',
            'tgl_dipa' => 'required',
            'cara_pembayaran' => 'required',
            'alamat_penyedia' => 'required',
            'nama_bank' => 'required',
            'kantor_bank' => 'required',
            'no_rek' => 'required',
            'npwp' => 'required',
            'sanksi' => 'required'
        ];
        $request->validate($validate, $customMessages);
        try {
            DB::beginTransaction();
            PengadaanBarangJasaRingkasan::updateOrCreate(['id' => $request->input('id')], $request->input());
            DB::commit();
            return $this->resSuccess(
                'Berhasil Disimpan!',
                [
                    'type' => 'redirect',
                    'url' => \URL::to('/pengadaan/barang-jasa/'.$request->input('id').'/edit')
                ]);
        } catch (\Throwable $th) {
            DB::rollBack();
            $errorMessage = $th->getMessage();
            return $this->resError($errorMessage);
        }
    }

    public function saveKontrak(Request $request) {
        $customMessages = [
            'no_kontrak.required' => 'Nomor Kontrak harus diisi',
            'tgl_kontrak.required' => 'Tanggal Kontrak harus diisi'
        ];
        $validate = [
            'no_kontrak' => 'required',
            'tgl_kontrak' => 'required',
        ];
        $request->validate($validate, $customMessages);
        try {
            DB::beginTransaction();
            PengadaanBarangJasaKontrak::updateOrCreate(['id' => $request->input('id')], $request->input());
            DB::commit();
            return $this->resSuccess(
                'Berhasil Disimpan!',
                [
                    'type' => 'redirect',
                    'url' => \URL::to('/pengadaan/barang-jasa/'.$request->input('id').'/edit')
                ]);
        } catch (\Throwable $th) {
            DB::rollBack();
            $errorMessage = $th->getMessage();
            return $this->resError($errorMessage);
        }
    }

    public function saveBast(Request $request) {
        $customMessages = [
            'no_bast.required' => 'Nomor BAST harus diisi',
            'tgl_bast.required' => 'Tanggal BAST harus diisi',
            'nama_pejabat.required' => 'Nama Pejabatan harus diisi',
            'nip_pejabat.required' => 'Nip Pejabatan harus diisi',
            'pangkat_pejabat.required' => 'Pangkat Pejabatan harus diisi',
            'jabatan_pejabat.required' => 'Jabatan Pejabatan harus diisi',
        ];
        $validate = [
            'no_bast' => 'required',
            'tgl_bast' => 'required',
            'nama_pejabat' => 'required',
            'nip_pejabat' => 'required',
            'pangkat_pejabat' => 'required',
            'jabatan_pejabat' => 'required',
        ];
        $request->validate($validate, $customMessages);
        try {
            DB::beginTransaction();
            PengadaanBarangJasaBast::updateOrCreate(['id' => $request->input('id')], $request->input());
            DB::commit();
            return $this->resSuccess(
                'Berhasil Disimpan!',
                [
                    'type' => 'redirect',
                    'url' => \URL::to('/pengadaan/barang-jasa/'.$request->input('id').'/edit')
                ]);
        } catch (\Throwable $th) {
            DB::rollBack();
            $errorMessage = $th->getMessage();
            return $this->resError($errorMessage);
        }
    }

    public function saveBapp(Request $request) {
        $customMessages = [
            'no_bapp.required' => 'Nomor BAPP harus diisi',
            'tgl_bapp.required' => 'Tanggal BAPP harus diisi',
            'no_kepja.required' => 'Nomor Kepja harus diisi',
            'tgl_kepja.required' => 'Tanggal Kepja harus diisi'
        ];
        $validate = [
            'no_bapp' => 'required',
            'tgl_bapp' => 'required',
            'no_kepja' => 'required',
            'tgl_kepja' => 'required'
        ];
        $request->validate($validate, $customMessages);
        try {
            DB::beginTransaction();
            PengadaanBarangJasaBast::updateOrCreate(['id' => $request->input('id')], $request->input());
            DB::commit();
            return $this->resSuccess(
                'Berhasil Disimpan!',
                [
                    'type' => 'redirect',
                    'url' => \URL::to('/pengadaan/barang-jasa/'.$request->input('id').'/edit')
                ]);
        } catch (\Throwable $th) {
            DB::rollBack();
            $errorMessage = $th->getMessage();
            return $this->resError($errorMessage);
        }
    }

    public function saveNodis(Request $request) {
        $customMessages = [
            'no_nodis.required' => 'Nomor Nodis harus diisi',
            'tgl_nodis.required' => 'Tanggal Nodis harus diisi',
        ];
        $validate = [
            'no_nodis' => 'required',
            'tgl_nodis' => 'required',
        ];
        $request->validate($validate, $customMessages);
        try {
            DB::beginTransaction();
            PengadaanBarangJasaNodis::updateOrCreate(['id' => $request->input('id')], $request->input());
            DB::commit();
            return $this->resSuccess(
                'Berhasil Disimpan!',
                [
                    'type' => 'redirect',
                    'url' => \URL::to('/pengadaan/barang-jasa/'.$request->input('id').'/edit')
                ]);
        } catch (\Throwable $th) {
            DB::rollBack();
            $errorMessage = $th->getMessage();
            return $this->resError($errorMessage);
        }
    }

    public function cetakHps(string $id)
    {
        $hps = PengadaanBarangJasaHps::where('id',$id)->first()->toArray();
        $model = PengadaanBarangJasa::where('id',$id)->first()->toArray();
        $data = [
            'hps'=>$hps,
            'model'=>$model,
        ];
        $pdf = LaravelMpdf::loadView('pengadaan.barang_jasa.cetakHps',$data);
        return $pdf->stream('hps.pdf');
    }

    public function cetakSkppbj(string $id)
    {
        $skppbj = PengadaanBarangJasaSkppbj::where('id',$id)->first()->toArray();
        $model = PengadaanBarangJasa::where('id',$id)->first()->toArray();
        $data = [
            'skppbj'=>$skppbj,
            'model'=>$model,
        ];
        $pdf = LaravelMpdf::loadView('pengadaan.barang_jasa.cetakSkppbj',$data);
        return $pdf->stream('skppbj.pdf');
    }

    public function cetakSpk(string $id)
    {
        $hps = PengadaanBarangJasaHps::where('id',$id)->first()->toArray();
        $skppbj = PengadaanBarangJasaSkppbj::where('id',$id)->first()->toArray();
        $spk = PengadaanBarangJasaSpk::where('id',$id)->first()->toArray();
        $model = PengadaanBarangJasa::where('id',$id)->first()->toArray();
        $data = [
            'spk'=>$spk,
            'hps'=>$hps,
            'model'=>$model,
            'skppbj'=>$skppbj,
        ];
        $pdf = LaravelMpdf::loadView('pengadaan.barang_jasa.cetakSpk',$data);
        return $pdf->stream('spk.pdf');
    }

    public function cetakRingkasan(string $id)
    {
        $ringkasan = PengadaanBarangJasaRingkasan::where('id',$id)->first()->toArray();
        $hps = PengadaanBarangJasaHps::where('id',$id)->first()->toArray();
        $skppbj = PengadaanBarangJasaSkppbj::where('id',$id)->first()->toArray();
        $spk = PengadaanBarangJasaSpk::where('id',$id)->first()->toArray();
        $model = PengadaanBarangJasa::where('id',$id)->first()->toArray();
        $pemenang = '';
        $subtotal = 0;
        $nilai_spk = 0;
        $decodedData = json_decode($skppbj['penyedia'], true);
        if(!empty($decodedData)){
            foreach($decodedData as $key => $value){
                if($value['pemenang_skppbj'] == 1){
                    $pemenang = $value['penyedia_skppbj'];
                }
            }
        }
        $decodedDataHps = json_decode($hps['barang'], true);
        if(!empty($decodedDataHps)){
            foreach($decodedDataHps as $key => $value){
                $subtotal += $value['harga_total'];
            }
            $nilai_spk = ($subtotal*0.11)+$subtotal;
        }
        $data = [
            'spk'=>$spk,
            'hps'=>$hps,
            'model'=>$model,
            'skppbj'=>$skppbj,
            'ringkasan'=>$ringkasan,
            'pemenang'=>$pemenang,
            'nilai_spk'=>$nilai_spk,
        ];
        $pdf = LaravelMpdf::loadView('pengadaan.barang_jasa.cetakRingkasan',$data);
        return $pdf->stream('ringkasan.pdf');
    }

    public function cetakKontrak(string $id)
    {
        $kontrak = PengadaanBarangJasaKontrak::where('id',$id)->first()->toArray();
        $ringkasan = PengadaanBarangJasaRingkasan::where('id',$id)->first()->toArray();
        $hps = PengadaanBarangJasaHps::where('id',$id)->first()->toArray();
        $skppbj = PengadaanBarangJasaSkppbj::where('id',$id)->first()->toArray();
        $spk = PengadaanBarangJasaSpk::where('id',$id)->first()->toArray();
        $model = PengadaanBarangJasa::where('id',$id)->first()->toArray();
        $pemenang = '';
        $subtotal = 0;
        $nilai_spk = 0;
        $decodedData = json_decode($skppbj['penyedia'], true);
        if(!empty($decodedData)){
            foreach($decodedData as $key => $value){
                if($value['pemenang_skppbj'] == 1){
                    $pemenang = $value['penyedia_skppbj'];
                }
            }
        }
        $decodedDataHps = json_decode($hps['barang'], true);
        if(!empty($decodedDataHps)){
            foreach($decodedDataHps as $key => $value){
                $subtotal += $value['harga_total'];
            }
            $nilai_spk = ($subtotal*0.11)+$subtotal;
        }
        $carbonDate = \Carbon\Carbon::parse($ringkasan['tgl_dipa']);
        $year = $carbonDate->format('Y');
        $data = [
            'spk'=>$spk,
            'hps'=>$hps,
            'model'=>$model,
            'skppbj'=>$skppbj,
            'ringkasan'=>$ringkasan,
            'pemenang'=>$pemenang,
            'nilai_spk'=>$nilai_spk,
            'kontrak'=>$kontrak,
            'year'=>$year,
        ];
        $pdf = LaravelMpdf::loadView('pengadaan.barang_jasa.cetakKontrak',$data);
        return $pdf->stream('ringkasan.pdf');
    }

    public function cetakBast(string $id)
    {
        $bast = PengadaanBarangJasaBast::where('id',$id)->first()->toArray();
        $kontrak = PengadaanBarangJasaKontrak::where('id',$id)->first()->toArray();
        $ringkasan = PengadaanBarangJasaRingkasan::where('id',$id)->first()->toArray();
        $hps = PengadaanBarangJasaHps::where('id',$id)->first()->toArray();
        $skppbj = PengadaanBarangJasaSkppbj::where('id',$id)->first()->toArray();
        $spk = PengadaanBarangJasaSpk::where('id',$id)->first()->toArray();
        $model = PengadaanBarangJasa::where('id',$id)->first()->toArray();
        $pemenang = '';
        $subtotal = 0;
        $nilai_spk = 0;
        $decodedData = json_decode($skppbj['penyedia'], true);
        if(!empty($decodedData)){
            foreach($decodedData as $key => $value){
                if($value['pemenang_skppbj'] == 1){
                    $pemenang = $value['penyedia_skppbj'];
                }
            }
        }
        $decodedDataHps = json_decode($hps['barang'], true);
        if(!empty($decodedDataHps)){
            foreach($decodedDataHps as $key => $value){
                $subtotal += $value['harga_total'];
            }
            $nilai_spk = ($subtotal*0.11)+$subtotal;
        }
        $carbonDate = \Carbon\Carbon::parse($ringkasan['tgl_dipa']);
        $year = $carbonDate->format('Y');
        $data = [
            'bast'=>$bast,
            'spk'=>$spk,
            'hps'=>$hps,
            'model'=>$model,
            'skppbj'=>$skppbj,
            'ringkasan'=>$ringkasan,
            'pemenang'=>$pemenang,
            'nilai_spk'=>$nilai_spk,
            'kontrak'=>$kontrak,
            'year'=>$year,
        ];
        $pdf = LaravelMpdf::loadView('pengadaan.barang_jasa.cetakBast',$data);
        return $pdf->stream('bast.pdf');
    }

    public function cetakBapp(string $id)
    {
        $bast = PengadaanBarangJasaBast::where('id',$id)->first()->toArray();
        $kontrak = PengadaanBarangJasaKontrak::where('id',$id)->first()->toArray();
        $ringkasan = PengadaanBarangJasaRingkasan::where('id',$id)->first()->toArray();
        $hps = PengadaanBarangJasaHps::where('id',$id)->first()->toArray();
        $skppbj = PengadaanBarangJasaSkppbj::where('id',$id)->first()->toArray();
        $spk = PengadaanBarangJasaSpk::where('id',$id)->first()->toArray();
        $model = PengadaanBarangJasa::where('id',$id)->first()->toArray();
        $pemenang = '';
        $subtotal = 0;
        $nilai_spk = 0;
        $decodedData = json_decode($skppbj['penyedia'], true);
        if(!empty($decodedData)){
            foreach($decodedData as $key => $value){
                if($value['pemenang_skppbj'] == 1){
                    $pemenang = $value['penyedia_skppbj'];
                }
            }
        }
        $decodedDataHps = json_decode($hps['barang'], true);
        if(!empty($decodedDataHps)){
            foreach($decodedDataHps as $key => $value){
                $subtotal += $value['harga_total'];
            }
            $nilai_spk = ($subtotal*0.11)+$subtotal;
        }
        $carbonDate = \Carbon\Carbon::parse($ringkasan['tgl_dipa']);
        $year = $carbonDate->format('Y');
        $data = [
            'bast'=>$bast,
            'spk'=>$spk,
            'hps'=>$hps,
            'model'=>$model,
            'skppbj'=>$skppbj,
            'ringkasan'=>$ringkasan,
            'pemenang'=>$pemenang,
            'nilai_spk'=>$nilai_spk,
            'kontrak'=>$kontrak,
            'year'=>$year,
        ];
        $pdf = LaravelMpdf::loadView('pengadaan.barang_jasa.cetakBapp',$data);
        return $pdf->stream('bapp.pdf');
    }

    public function cetakNodis(string $id)
    {
        $nodis = PengadaanBarangJasaNodis::where('id',$id)->first()->toArray();
        $bast = PengadaanBarangJasaBast::where('id',$id)->first();
        $kontrak = PengadaanBarangJasaKontrak::where('id',$id)->first()->toArray();
        $ringkasan = PengadaanBarangJasaRingkasan::where('id',$id)->first()->toArray();
        $hps = PengadaanBarangJasaHps::where('id',$id)->first()->toArray();
        $skppbj = PengadaanBarangJasaSkppbj::where('id',$id)->first()->toArray();
        $spk = PengadaanBarangJasaSpk::where('id',$id)->first()->toArray();
        $model = PengadaanBarangJasa::where('id',$id)->first()->toArray();
        $pemenang = '';
        $subtotal = 0;
        $nilai_spk = 0;
        $decodedData = json_decode($skppbj['penyedia'], true);
        if(!empty($decodedData)){
            foreach($decodedData as $key => $value){
                if($value['pemenang_skppbj'] == 1){
                    $pemenang = $value['penyedia_skppbj'];
                }
            }
        }
        $decodedDataHps = json_decode($hps['barang'], true);
        if(!empty($decodedDataHps)){
            foreach($decodedDataHps as $key => $value){
                $subtotal += $value['harga_total'];
            }
            $nilai_spk = ($subtotal*0.11)+$subtotal;
        }
        $carbonDate = \Carbon\Carbon::parse($ringkasan['tgl_dipa']);
        $year = $carbonDate->format('Y');
        $data = [
            'nodis'=>$nodis,
            'bast'=>$bast,
            'spk'=>$spk,
            'hps'=>$hps,
            'model'=>$model,
            'skppbj'=>$skppbj,
            'ringkasan'=>$ringkasan,
            'pemenang'=>$pemenang,
            'nilai_spk'=>$nilai_spk,
            'kontrak'=>$kontrak,
            'year'=>$year,
        ];
        $pdf = LaravelMpdf::loadView('pengadaan.barang_jasa.cetakNodis',$data);
        return $pdf->stream('nodis.pdf');
    }

    /*
            if($request->input('no_spk')) $data['no_spk'] = $request->input('no_spk');
            if($request->input('tgl_spk')) $data['tgl_spk'] = $request->input('tgl_spk');
            if($request->input('jangka_waktu_pelaksanaan')) $data['jangka_waktu_pelaksanaan'] = $request->input('jangka_waktu_pelaksanaan');
            if($request->input('jangka_waktu_pelaksanaan')) $data['jangka_waktu_pelaksanaan'] = $request->input('jangka_waktu_pelaksanaan');
            $filepath = 'uploads/pengadaan/barang_jasa';
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

    */


}
