<?php

namespace App\Http\Controllers\Pengadaan;

use App\Helpers\MyHelper;
use App\Http\Controllers\Controller;
use App\Models\Master;
use App\Models\Master\MsDistribusiStatus;
use App\Models\Monsakti\KontrakHeader;
use App\Models\Pengadaan\Distribusi\Distribusi;
use App\Models\Pengadaan\Distribusi\DistribusiHist;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\DB;
use Illuminate\Support\Facades\Storage;
use Mccarlosen\LaravelMpdf\Facades\LaravelMpdf;
use Symfony\Component\HttpKernel\Exception\NotFoundHttpException;

class DistribusiController extends Controller
{
    /**
     * Display a listing of the resource.
     */
    protected $breadcums = ['Pengadaan', 'Penyimpanan dan Distribusi'];

    public function index()
    {
        $breadcum = array_merge($this->breadcums, ['Pengisian Data Pengadaan']);
        $judul = 'Daftar Data Pengadaan';

        return view('pengadaan.distribusi.distribusiV', ['judul' => $judul, 'tableId' => 'dt-distribusi', 'breadcums' => $breadcum]);
    }

    public function masukGudang()
    {
        $breadcum = array_merge($this->breadcums, ['Barang Masuk Gudang']);
        $judul = 'Daftar Barang Masuk Gudang';

        return view('pengadaan.distribusi.distribusiV', ['judul' => $judul, 'tableId' => 'dt-distribusi', 'breadcums' => $breadcum]);
    }

    public function keluarGudang()
    {
        $breadcum = array_merge($this->breadcums, ['Barang Keluar Gudang']);
        $judul = 'Daftar Barang Keluar Gudang';

        return view('pengadaan.distribusi.distribusiV', ['judul' => $judul, 'tableId' => 'dt-distribusi', 'breadcums' => $breadcum]);
    }

    public function konfirmasiPenerimaan()
    {
        $breadcum = array_merge($this->breadcums, ['Konfirmasi Penerimaan Barang']);
        $judul = 'Daftar Konfirmasi Penerimaan Barang';

        return view('pengadaan.distribusi.distribusiV', ['judul' => $judul, 'tableId' => 'dt-distribusi', 'breadcums' => $breadcum]);
    }

    public function gridData(Request $request, $jenis = '')
    {
        $model = new Distribusi;
        $pagingParams = $request->only(['start', 'length']);
        $searchParams = $request->only(['columns']);
        $data = $model->getDataGrid($pagingParams, $searchParams, $jenis);

        return response()->json([
            'data' => $data['data'],
            'recordsTotal' => $data['total'],
            'recordsFiltered' => $data['total'],
        ]);
    }

    public function gridDataAdd(Request $request, $jenis = '')
    {
        $model = new Distribusi;
        $pagingParams = $request->only(['start', 'length']);
        $searchParams = $request->only(['search',  'filterBy']);
        $data = $model->getDataGridAdd($pagingParams, $searchParams, $jenis);

        return response()->json([
            'data' => $data['data'],
            'recordsTotal' => $data['total'],
            'recordsFiltered' => $data['total'],
        ]);
    }

    public function histData($id_penyimpanan)
    {
        $model = new Distribusi;
        $data = $model->getDataHist($id_penyimpanan);

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
            $model = Distribusi::where('id', $id)->first();
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
                'selected' => $model['kdsatker_tujuan'] ?? null,
            ]),
        ];

        // dd($data['satkerOptions']);
        return $data;
    }

    public function gridDataKontrak()
    {
        $model = new KontrakHeader;
        $data = $model->getAll();

        return response()->json([
            'data' => $data,
        ]);
    }

    /**
     * Show the form for creating a new resource.
     */
    public function create()
    {
        $data = $this->getData();

        return view('pengadaan.distribusi.distribusiFormV', $data);
    }

    /**
     * Store a newly created resource in storage.
     */
    public function store(Request $request)
    {
        $isNew = $request->input('isNew');
        $customMessages = [
            'no_kontrak.required' => 'Nomor Kontrak harus diisi',
            'no_kontrak.unique' => 'Nomor Kontrak sudah ada',
            'tgl_kontrak.required' => 'Tanggal Kontrak harus diisi',
            'nm_barang.required' => 'Nama Barang harus diisi',
            'jml_barang.required' => 'Jumlah Barang harus diisi',
            'nilai_barang.required' => 'Nilai Barang harus diisi',
            'kdsatker_tujuan.required' => 'Satker Tujuan harus dipilih',
        ];
        $validate = [
            'no_kontrak' => 'required|unique:penyimpanan_distribusi,no_kontrak',
            'tgl_kontrak' => 'required',
            'nm_barang' => 'required',
            'jml_barang' => 'required',
            'nilai_barang' => 'required',
            'kdsatker_tujuan' => 'required',
        ];
        $id = $request->input('id') ?? MyHelper::getPk(date('Ymd'), 'penyimpanan_distribusi_seq');
        if ($isNew) {
            $validate['file_spk'] = 'required|mimes:jpeg,png,pdf|max:2048';
            $customMessages['file_spk.required'] = 'File SPK harus diupload';
        } else {
            $validate['no_kontrak'] = 'required|unique:penyimpanan_distribusi,no_kontrak,'.$id;
        }
        $request->validate($validate, $customMessages);

        try {
            DB::beginTransaction();
            $data = [
                'no_kontrak' => $request->input('no_kontrak'),
                'tgl_kontrak' => $request->input('tgl_kontrak'),
                'nm_barang' => $request->input('nm_barang'),
                'jml_barang' => $request->input('jml_barang'),
                'nilai_barang' => $request->input('nilai_barang'),
                'kdsatker_tujuan' => $request->input('kdsatker_tujuan'),
                'id_status' => 1,
                'is_gudang' => $request->input('is_gudang'),
                'id_kontrak' => $request->input('id_kontrak'),
                'nilai_kontrak' => $request->input('nilai_kontrak'),
            ];
            if ($request->hasFile('file_spk')) {
                $filepath = 'uploads/pengadaan/distribusi';
                $file = $request->file('file_spk');
                $fileName = $id.'_spk'.'.'.$file->getClientOriginalExtension();
                $filesave = $filepath.'/'.$fileName;
                $file->move($filepath, $fileName);
                $data['file_spk'] = $filesave;
            }
            Distribusi::updateOrCreate(['id' => $id], $data);
            if ($isNew) {
                DistribusiHist::create([
                    'id_penyimpanan' => $id,
                    'id_status' => 1,
                ]);
            }
            DB::commit();

            return $this->resSuccess();
        } catch (\Throwable $th) {
            DB::rollBack();
            $errorMessage = $th->getMessage();

            return $this->resError($errorMessage);
        }
    }

    public function pilihData(Request $request)
    {
        $jenis = $request->input('jenis');
        $id = $request->input('id');
        try {
            DB::beginTransaction();
            $model = Distribusi::find($id);
            if ($jenis == 'masuk-gudang') {
                $model->id_status = 2;
                $hist = [
                    'id_penyimpanan' => $id,
                    'id_status' => 2,
                ];
            } elseif ($jenis == 'keluar-gudang') {
                $model->id_status = 5;
                $hist = [
                    'id_penyimpanan' => $id,
                    'id_status' => 5,
                ];
            } elseif ($jenis == 'konfirmasi-penerimaan') {
                $model->id_status = 8;
                $hist = [
                    'id_penyimpanan' => $id,
                    'id_status' => 8,
                ];
            }
            $model->save();
            DistribusiHist::create($hist);
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

        return view('pengadaan.distribusi.distribusiFormV', $data);
    }

    public function showKonfirmasi(string $id)
    {
        $data = $this->getData($id);

        return view('pengadaan.distribusi.konfirmasiFormV', $data);
    }

    /**
     * Remove the specified resource from storage.
     */
    public function destroy(string $id)
    {
        try {
            DB::beginTransaction();
            $record = Distribusi::find($id);
            if ($record) {
                if ($record->file_skp && Storage::exists($record->file_skp)) {
                    Storage::delete('public/'.$record->file_skp);
                }
                if ($record->file_bast && Storage::exists($record->file_bast)) {
                    Storage::delete('public/'.$record->file_bast);
                }
                if ($record->file_foto && Storage::exists($record->file_foto)) {
                    Storage::delete('public/'.$record->file_foto);
                }
                $record->delete();
            }
            DB::commit();

            return $this->resSuccess('Berhasil Dihapus!');
        } catch (\Throwable $th) {
            DB::rollBack();
            $errorMessage = $th->getMessage();

            return $this->resError($errorMessage);
        }
    }

    public function cetakLabel($id)
    {
        $data = Distribusi::findOne($id);
        $model = (array) $data;
        $data = 'Nomor Kontrak : '.$model['no_kontrak']."\n".'Nama Barang : '.$model['nm_barang']."\n".'Satker Tujuan : '.$model['inst_nama'];
        $qrCodeImage = MyHelper::generateQrCode($data);
        $pdfContent = '<html><style> table { margin-left: auto; margin-right: auto; } </style><body>';
        $pdfContent .= '<table><tbody>';
        $pdfContent .= '<tr><td>';
        $pdfContent .= '<center><img src="data:image/png;base64,'.base64_encode($qrCodeImage).'" /></center><br/><br/>';
        $pdfContent .= '</td>';
        $pdfContent .= '<td>';
        $pdfContent .= '<table><tbody>';
        $pdfContent .= '<tr><td>Nomor Kontrak</td><td>:</td><td>'.$model['no_kontrak'].'</td></tr>';
        $pdfContent .= '<tr><td>Nama Barang</td><td>:</td><td>'.$model['nm_barang'].'</td></tr>';
        $pdfContent .= '<tr><td>Satker Tujuan</td><td>:</td><td>'.$model['inst_nama'].'</td></tr>';
        $pdfContent .= '</tbody></table>';
        $pdfContent .= '</td></tr>';
        $pdfContent .= '</tbody></table>';
        $pdfContent .= '</body></html>';
        $pdf = LaravelMpdf::loadHTML($pdfContent, [
            'title' => 'Label',
            'format' => 'A5-L',
            'orientation' => 'L',
        ]);

        return $pdf->stream('label-barang-gudang.pdf');
    }

    public function getOptionStatus(Request $request)
    {
        $jenis = $request->input('jenis');
        $in = [];
        if ($jenis == 'masuk-gudang') {
            $in = [3, 4];
        } elseif ($jenis == 'keluar-gudang') {
            $in = [6, 7];
        } elseif ($jenis == 'konfirmasi-penerimaan') {
            $in = [9];
        }
        $options = MsDistribusiStatus::whereIn('id', $in)->get();

        return response()->json($options);
    }

    public function simpanUbahStatus(Request $request)
    {
        $id = $request->input('id');
        $id_status = $request->input('id_status');
        $ket = $request->input('ket');
        try {
            DB::beginTransaction();
            $model = Distribusi::find($id);
            $model->id_status = $id_status;
            $model->save();
            DistribusiHist::create([
                'id_penyimpanan' => $id,
                'id_status' => $id_status,
                'ket' => $ket,
            ]);
            DB::commit();

            return $this->resSuccess();
        } catch (\Throwable $th) {
            DB::rollBack();
            $errorMessage = $th->getMessage();

            return $this->resError($errorMessage);
        }
    }

    public function storeKonfirmasi(Request $request)
    {
        $isNew = $request->input('isNew');
        $customMessages = [
            'file_bast.required' => 'File BAST harus diupload',
            'file_foto.required' => 'File Foto harus diupload',
        ];
        $validate = [
            'file_bast' => 'required|mimes:jpeg,png,pdf|max:2048',
            'file_foto' => 'required|mimes:jpeg,png,jpg|max:2048',
        ];
        $request->validate($validate, $customMessages);
        $id = $request->input('id');
        try {
            DB::beginTransaction();
            $data = [];
            if ($request->hasFile('file_bast')) {
                $filepath = 'uploads/pengadaan/distribusi';
                $file = $request->file('file_bast');
                $fileName = $id.'_bast'.'.'.$file->getClientOriginalExtension();
                $filesave = $filepath.'/'.$fileName;
                $file->move($filepath, $fileName);
                $data['file_bast'] = $filesave;
            }
            if ($request->hasFile('file_foto')) {
                $filepath = 'uploads/pengadaan/distribusi';
                $file = $request->file('file_foto');
                $fileName = $id.'_foto'.'.'.$file->getClientOriginalExtension();
                $filesave = $filepath.'/'.$fileName;
                $file->move($filepath, $fileName);
                $data['file_foto'] = $filesave;
            }
            Distribusi::updateOrCreate(['id' => $id], $data);
            DB::commit();

            return $this->resSuccess();
        } catch (\Throwable $th) {
            DB::rollBack();
            $errorMessage = $th->getMessage();

            return $this->resError($errorMessage);
        }
    }
}
