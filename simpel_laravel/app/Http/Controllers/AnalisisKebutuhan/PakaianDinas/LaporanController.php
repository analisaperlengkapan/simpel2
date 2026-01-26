<?php

namespace App\Http\Controllers\AnalisisKebutuhan\PakaianDinas;
use Illuminate\Routing\Controller;

use App\Exports\ExportExcelFromView;
use App\Helpers\MyHelper;
use App\Models\AnalisisKebutuhan\PakaianDinas as Model;
use App\Models\AnalisisKebutuhan\PakaianDinas;
use App\Models\Master\Master;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\DB;
use Maatwebsite\Excel\Facades\Excel;
use Mccarlosen\LaravelMpdf\Facades\LaravelMpdf;
use Symfony\Component\HttpKernel\Exception\NotFoundHttpException;

use function Psy\debug;

class LaporanController extends Controller
{
    protected $breadcums = ['Analisis Kebutuhan', 'Pakaian Dinas', 'Laporan'];
    private $controller = '/analisis-kebutuhan/pakaian-dinas/laporan';

    public function getPengadaanByTahun(string $tahun)
    {
        $pengadaans = PakaianDinas::where(['tahun' => $tahun])->get();
        $pengadaans = MyHelper::generateSelectOptions(
            [
                'data' => $pengadaans,
                'text' => 'nama',
                'value' => 'id',
                'type' => 'raw'
            ]
        );
        return response()->json($pengadaans);
    }
    function mapFilter($filter)
    {
        $newFilter = [];
        if ($filter['jenis'] != '') {
            array_push($newFilter, ['label' => 'Status Pegawai', 'value' => $filter['jenis'] == '0' ? 'TU' : 'Jaksa']);
        }

        if ($filter['eselon'] == 'non') {
        } else if ($filter['eselon'] != '') {
            array_push($newFilter, ['label' => 'Eselon', 'value' => 'Non Eselon']);
        }

        if ($filter['jenis_kelamin'] != '') {
            array_push($newFilter, ['label' => 'Jenis Kelamin', 'value' => $filter['jenis_kelamin']]);
        }

        return $newFilter;
    }
    public function getData()
    {
        $pengadaans = PakaianDinas::all();
        $msSatker = Master::getSatkerWilayah();
        $isPusat = session('userData.current_role.ms_satker_id') == '00' ? 1 : 0;
        $tahuns = PakaianDinas::distinct()->select('tahun')->get();
        $data = [
            'breadcums' => $this->breadcums,
            'controller' => $this->controller,
            'isPusat' => $isPusat,
            'tahuns' => MyHelper::generateSelectOptions(['data' => $tahuns, 'value' => 'tahun', 'text' => 'tahun', 'type' => 'raw']),
            'pengadaans' => MyHelper::generateSelectOptions(
                [
                    'data' => $pengadaans,
                    'text' => 'nama',
                    'value' => 'id',
                    'type' => 'raw'
                ]
            ),
            'wilayahs' => MyHelper::generateSelectOptions(
                [
                    'data' => $msSatker['wilayahs'],
                    'text' => 'inst_nama',
                    'value' => 'inst_satkerkd',
                    'type' => 'raw'
                ]
            ),
            'satkers' => MyHelper::generateSelectOptions(
                [
                    'data' => $msSatker['satkers'],
                    'text' => 'inst_nama',
                    'value' => 'inst_satkerkd',
                    'type' => 'raw'
                ]
            ),
            'pusats' => MyHelper::generateSelectOptions(
                [
                    'data' => $msSatker['pusats'],
                    'text' => 'inst_nama',
                    'value' => 'inst_satkerkd',
                    'type' => 'raw'
                ]
            )
        ];
        return $data;
    }

    public function mobile()
    {
        $data = $this->getData();
        return response()->json($data);
    }
    public function index()
    {
        $data = $this->getData();
        return view('analisis_kebutuhan.pakaian_dinas.laporanV', $data);
    }

    public function gridData(Request $request)
    {
        $model = new Model();
        $pagingParams = $request->only(['start', 'length']);
        $searchParams = $request->only(['search', 'filterBy']);
        $data = $model->getGridData($pagingParams, $searchParams);
        return response()->json([
            'data' => $data['data'],
            'recordsTotal' => $data['total'],
            'recordsFiltered' => $data['total'],
        ]);
    }

    public function show(int $pengajuanId)
    {
        return view('analisis_kebutuhan.pakaian_dinas.laporan.cetakDaftarSatkerV', [
            'tableId' => 'dt-user',
            'pengajuanId' => $pengajuanId,
            'breadcums' => array_merge($this->breadcums, ['List Satker']),
            'controller' => $this->controller
        ]);
    }

    public function gridDataSatker(Request $request)
    {
        $model = new Model();
        $pengajuanId = $_GET['pengajuanId'];
        $pagingParams = $request->only(['start', 'length']);
        $searchParams = $request->only(['search', 'filterBy']);
        $data = $model->getGridDataSatker($pagingParams, $searchParams, $pengajuanId);
        return response()->json([
            'data' => $data['data'],
            'recordsTotal' => $data['total'],
            'recordsFiltered' => $data['total'],
        ]);
    }

    function cetakRekap(Request $req, $jenisFile)
    {
        $model = new Model();
        try {
            [$header, $pakaians, $dataUkuran, $listSatker, $dataSummaryPakaian] = $model->dataRekap(
                [
                    'pengajuan_id' => $req->query('pengajuan_id'),
                    'kejati_id' => $req->query('kejati_id'),
                    'ms_satker_id' => $req->query('ms_satker_id'),
                    'filter' => $req->query('filter')
                ]
            );

            $data = [
                'header' => $header,
                'filter' => $this->mapFilter($req->query('filter')),
                'pakaians' => $pakaians,
                'listSatker' => $listSatker,
                'dataUkuran' => $dataUkuran,
                'dataSummaryPakaian' => $dataSummaryPakaian,
            ];
            // dd($data['dataSummaryPakaian']);
            // dd($data);
        } catch (\Throwable $th) {
            $msg = $th->getMessage();
            if (!$msg)
                dd($th);
            // dd($th->getMessage());
            // dd($data);
            echo "<script>alert('Satker Belum Input');window.close();</script>";
            exit;
        }

        // return view('analisis_kebutuhan.pakaian_dinas.laporan.cetakRekapTemplateV', $data);

        $title = "Laporan {$req->query('jenis_laporan')} - {$header['nama']} - {$header->inst_nama}";
        $view =
            'analisis_kebutuhan.pakaian_dinas.laporan.cetakRekapTemplateV';
        if ($jenisFile == 'pdf') {
            $pdf = LaravelMpdf::chunkLoadView(
                '<html-spearator/>',
                $view,
                $data,
                [],
                [
                    'default_font_size' => 7,
                    'orientation' => 'L',
                    'title' => $title
                ]
            );
            return $pdf->stream('Laporan.pdf');
        } elseif ($jenisFile == 'excel') {
            return $this->cetakExcel(['data' => $data, 'title' => $title, 'view' => $view]);
        } else {
            throw new NotFoundHttpException('Jenis Cetakan Laporan Tidak Ditemukan');
        }
    }

    function cetakDaftar(Request $req, $jenisFile)
    {
	ini_set('pcre.backtrack_limit', '100000000000');
        $model = new Model();
        try {
            [$header, $pakaians, $ukuranPegawais, $listSatker, $dataPerSatker, $isPusat] = $model->getDataDaftar(
                [
                    'pengajuan_id' => $req->query('pengajuan_id'),
                    'kejati_id' => $req->query('kejati_id'),
                    'ms_satker_id' => $req->query('ms_satker_id'),
                    'filter' => $req->query('filter')
                ]
            );
        } catch (\Throwable $th) {
            $msg = $th->getMessage();
            if (!$msg)
                dd($th);
            echo "<script>alert('{$msg}');window.close();</script>";
            exit;
        }
        $mappedUkurans = [];
        foreach ($ukuranPegawais as $ukuran) {
            $mappedUkurans[$ukuran->pengajuan_pakaian_dinas_satker_pegawai_id][$ukuran->pengajuan_pakaian_dinas_pakaian_id] = $ukuran->ukuran;
        }

        $data = [
            'filter' => $this->mapFilter($req->query('filter')),
            'header' => $header,
            'isPusat' => $isPusat,
            'pakaians' => $pakaians,
            'listSatker' => $listSatker,
            'dataPerSatker' => $dataPerSatker,
            'mappedUkurans' => $mappedUkurans,
            'isExcel' => $jenisFile == 'excel' ? true : false
        ];
        $view =
            'analisis_kebutuhan.pakaian_dinas.laporan.cetakDaftarTemplateV';
        $title = "Laporan {$req->query('jenis_laporan')} - {$header['nama']} - {$header->inst_nama}";
        if ($jenisFile == 'pdf') {
            $pdf = LaravelMpdf::chunkLoadView(
                '<html-spearator/>',
                $view,
                $data,
                [],
                [
                    'default_font_size' => 7,
                    'orientation' => 'L',
                    'title' => $title,
                ]
            );
        } elseif ($jenisFile == 'excel') {
            return $this->cetakExcel(['data' => $data, 'title' => $title, 'view' => $view]);
        } else {
            throw new NotFoundHttpException('Jenis Cetakan Laporan Tidak Ditemukan');
        }
        return $pdf->stream('Laporan.pdf');
    }
    public function cetak(Request $req)
    {
        switch ($req->query('jenis_laporan')) {
            case 'daftar':
                return $this->cetakDaftar($req, $req->query('jenis_file'));
            case 'rekap':
                return $this->cetakRekap($req, $req->query('jenis_file'));
            case 'update':
                return $this->eselon();
            default:
                throw new NotFoundHttpException('Jenis Cetakan Tidak Ditemukan');
        }
    }

    function eselon()
    {
        $unitkerjas = DB::table('unit_kerja')->get();
        foreach ($unitkerjas as $key => $unit) {
            $eselon = explode('.', $unit->id);
            $eselonya = count($eselon);
            $test[$unit->id] = $eselonya;
            switch ($eselonya) {
                case 5:
                    $eselon3nya = $this->removeLastElement($eselon);
                    $eselon2nya = $this->removeLastElement($eselon3nya);
                    $eselon1nya = $this->removeLastElement($eselon2nya);
                    $updated = [
                        'eselon1' => implode('.', $eselon1nya),
                        'eselon2' => implode('.', $eselon2nya),
                        'eselon3' => implode('.', $eselon3nya),
                    ];
                    break;
                case 4:
                    $eselon2nya = $this->removeLastElement($eselon);
                    $eselon1nya = $this->removeLastElement($eselon2nya);
                    $updated = [
                        'eselon1' => implode('.', $eselon1nya),
                        'eselon2' => implode('.', $eselon2nya),
                    ];
                    break;
                case 3:
                    $eselon1nya = $this->removeLastElement($eselon);
                    $updated = [
                        'eselon1' => implode('.', $eselon1nya),
                    ];
                    break;
                default:
                    $updated = [];
                    break;
            }
            if (!empty($updated)) {
                DB::table('unit_kerja')->where(['id' => $unit->id])->update($updated);
            }
        }
        dd($test);
    }
    function removeLastElement($array)
    {
        array_pop($array);
        return $array;
    }

    function cetakExcel($params)
    {
        return Excel::download(new ExportExcelFromView($params['data'], $params['view']), "{$params['title']}.xlsx", \Maatwebsite\Excel\Excel::XLSX);
    }
}
