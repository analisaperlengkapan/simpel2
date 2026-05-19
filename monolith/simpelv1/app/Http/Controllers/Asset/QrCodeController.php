<?php

namespace App\Http\Controllers\Asset;

use App\Helpers\MyHelper;
use App\Http\Controllers\Controller;
use App\Models\Asset\QrCode;
use App\Models\Files;
use App\Models\Master;
use App\Models\Master\MsJenisAset;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\DB;
use Mccarlosen\LaravelMpdf\Facades\LaravelMpdf;

class QrCodeController extends Controller
{
    protected $breadcums = ['Aset'];

    protected $controller = '/asset/qr-code';

    protected $columns = ['Kode Satker', 'Nama Satker', 'Kode Barang', 'Nama Barang', 'NUP', 'Kondisi', 'Merk/Tipe'];

    public function __construct()
    {
        $this->breadcums = array_merge($this->breadcums, [['link' => $this->controller, 'title' => 'Qr Code']]);
    }

    protected function userOperation()
    {
        if (session('userData.current_role.ms_role_id') == config('constants.superadmin_role_id')) {
            return 'PUSAT';
        }

        if (session('userData.current_role.ms_role_id') == config('constants.validator_pusat_role_id')) {
            return 'PUSAT';
        }

        if (session('userData.current_role.ms_role_id') == config('constants.validator_wilayah_role_id')) {
            return 'WILAYAH';
        }

        if (session('userData.current_role.ms_role_id') == config('constants.pelaksana_satker_role_id')) {
            return 'SATKER';
        }
    }

    public function index()
    {
        $satkerTerpilih = Master::getSatkersSakti();
        $data = [
            'tableId' => 'dt-qrcode',
            'breadcums' => $this->breadcums,
            'controller' => $this->controller,
            'columns' => $this->columns,
            'userOperation' => $this->userOperation(),
            'jenisAssetOptions' => MyHelper::generateSelectOptions([
                'data' => Master::getJenisAsset(),
                'text' => 'name',
                'value' => 'id',
                'selected' => $model['id_jenis_asset'] ?? null,
            ]),
            'qrcode' => QrCode::find(1),
            'satkerOptions' => MyHelper::generateSelectOptions(['data' => $satkerTerpilih, 'text' => 'deskripsi', 'value' => 'kdsatker', 'selected' => session('userData.current_role.ms_satker_id_keu')]),
        ];

        return view('asset.qr_code.indexV', $data);
    }

    public function gridData(Request $request)
    {
        $jenisAset = MsJenisAset::where('id', $request['id_jenis_asset'])->first();
        $tableAsset = $jenisAset['nm_table'] ?? '';
        $inst_satkerkd = $request['inst_satkerkd'] ?? '';
        $model = new QrCode;
        $pagingParams = $request->only(['start', 'length']);
        $searchParams = $request->only(['columns']);
        $data = $model->getDataGrid($tableAsset, $inst_satkerkd, $pagingParams, $searchParams);

        return response()->json([
            'data' => $data['data'],
            'recordsTotal' => $data['total'],
            'recordsFiltered' => $data['total'],
        ]);
    }

    public function cetakLabel(Request $request)
    {
        $model = new QrCode;
        $jenisAset = MsJenisAset::where('id', $request['id_jenis_asset'])->first();
        $tableAsset = $jenisAset['nm_table'] ?? '';
        $id = $request['id'];
        if (is_string($id)) {
            $id = explode(',', $id);
        }
        $data = $model->getDataCetak($tableAsset, $id);
        // dd($data);exit;
        $qr = QrCode::find(1);
        $logoPath = public_path($qr['logo']);
        $pdfContent = '<html><style> table.myFormat tr td { font-size: 8px; } table { margin-left: auto; margin-right: auto;}, table, th, td {border: 1px solid black;border-collapse: collapse;} </style><body>';
        $totalRows = count($data);
        foreach ($data as $index => $row) {
            // $dataqr = $request['id_jenis_asset'] . '*' . $row->kdsatker_keu . "*" . $row->kode_barang . "*" . $row->nup;
            $dataqr = $row->kdsatker_keu.'*'.$row->kode_barang.'*'.$row->nup;
            $dateString = $row->tgl_perolehan;
            $date = \Carbon\Carbon::parse($dateString);
            $year = $date->year;
            $jmlKata = str_word_count($row->deskripsi);
            $qrCodeImage = MyHelper::generateQrCode($dataqr);
            $pdfContent .= '<table class="myFormat" style="border: 1px solid black;border-collapse: collapse;" width="100%"><tbody>';
            $pdfContent .= '<tr>';
            $pdfContent .= '<td width="20%"><center><img src="'.$logoPath.'" width="15"/></center></td>';
            $pdfContent .= '<td colspan="2"><center><b>'.$row->deskripsi.'<br/>'.$row->kdsatker_keu.'.'.$year.'</center></b></td>';
            $pdfContent .= '</tr>';
            $pdfContent .= '<tr><td colspan="2">';
            $pdfContent .= $row->kode_barang.'<br/>'.$row->nm_barang.'<br/>'.($row->nup ? 'NUP : '.$row->nup : '').'<br/><br/>'.($row->merk ? 'Merk : '.$row->merk : '');
            $pdfContent .= '</td>';
            $pdfContent .= '<td width="40%">';
            $pdfContent .= '<center><img src="data:image/png;base64,'.base64_encode($qrCodeImage).'" '.($jmlKata > 4 ? 'width="69"' : 'width="79"').'/></center>';
            $pdfContent .= '</td></tr>';
            $pdfContent .= '</tbody></table>';
            if ($index < $totalRows - 1) {
                $pdfContent .= '<pagebreak />';
            }
        }
        $pdfContent .= '</body></html>';
        $pdf = LaravelMpdf::loadHTML($pdfContent, [
            'title' => 'Label',
            'format' => [30, 60],
            'orientation' => 'L',
            'margin_left' => 1,
            'margin_right' => 1,
            'margin_top' => 1,
            'margin_bottom' => 1,
        ]);

        return $pdf->stream('label_aset.pdf');
    }

    public function store(Request $request)
    {
        $customMessages['logo.required'] = 'logo harus diupload';
        $validasi['logo'] = 'required|mimes:jpeg,png,jpg|max:2048';
        $request->validate($validasi, $customMessages);
        try {
            DB::beginTransaction();
            $id = 1;
            $params = [
                'kategori' => 'Logo Qr Code',
                'dir' => 'setting/qr-code',
                'fileKey' => 'logo',
                'pkey' => $id,
            ];
            $fotos = Files::upload($request, $params);
            if (! empty($fotos)) {
                $qr = QrCode::find($id);
                $qr->logo = $fotos['path'];
                $qr->save();
            }
            DB::commit();

            return $this->resSuccess(
                'Berhasil Disimpan!'
            );
        } catch (\Throwable $th) {
            dd($th->getMessage());

            return $this->resError('Gagal menyimpan data');
        }
    }

    public function readQr(Request $request)
    {
        $qrString = $request->input('content');

        $validasi['content'] = 'required';
        $request->validate($validasi);

        [$idAsset, $satkerKeu, $kdBarang, $nup] = explode('*', $qrString);
        $jenisAset = MsJenisAset::where('id', $idAsset)->first();
        if (! $jenisAset) {
            return response()->json(null, 404);
        }
        // dd([$idAsset, $satkerKeu, $kdBarang, $nup]);
        $data = DB::table($jenisAset->nm_table)
            ->select(
                'nm_satker',
                'kode_barang',
                'nm_barang',
                'nup',
                'kondisi',
                'merk',
                'tgl_rekam_pertama',
                'tgl_perolehan',
                'nilai_perolehan_pertama',
                'nilai_mutasi',
                'nilai_perolehan',
                'nilai_penyusutan',
                'nilai_buku',
                'kuantitas',
                'jml_foto',
                'status_penggunaan',
                'status_pengelolaan',
                'no_psp',
                'tgl_psp',
                'jml_kib',
            )
            ->where(['kode_barang' => $kdBarang, 'kdsatker_keu' => $satkerKeu, 'nup' => $nup])->first();

        return response()->json($data, $data ? 200 : 404);
    }
}
