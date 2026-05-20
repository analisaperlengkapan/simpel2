<?php

namespace App\Http\Controllers\AnalisisKebutuhan\PakaianDinas\Laporan;

use App\Http\Controllers\Controller;
use App\Models\AnalisisKebutuhan\PakaianDinas as Model;
use App\Models\Master;
use Barryvdh\DomPDF\Facade\Pdf;
use Illuminate\Http\Request;

class CetakRekap extends Controller
{
    protected $breadcums = ['Analisis Kebutuhan', 'Pakaian Dinas', 'Laporan', 'Cetak Rekap'];

    private $controller = '/analisis-kebutuhan/pakaian-dinas/laporan/cetak-rekap';

    public function index()
    {
        return view('analisis_kebutuhan.pakaian_dinas.laporan.cetakRekapV', [
            'tableId' => 'dt-user',
            'breadcums' => $this->breadcums,
            'controller' => $this->controller,
        ]);
    }

    public function gridData(Request $request)
    {
        $model = new Model;
        $pagingParams = $request->only(['start', 'length']);
        $searchParams = $request->only(['search', 'filterBy']);
        $data = $model->getGridData($pagingParams, $searchParams);

        return response()->json([
            'data' => $data['data'],
            'recordsTotal' => $data['total'],
            'recordsFiltered' => $data['total'],
        ]);
    }

    public function cetak(int $pengajuanId)
    {

        $model = new Model;
        $rekaps = $model->getDataRekap($pengajuanId);
        $rekaps = $this->mapUkuranObj($rekaps);
        $satkers = Master::getSatkers(true);
        $data = [
            'header' => Model::where(['id' => $pengajuanId])->first(),
            'satkers' => $satkers,
            'rekaps' => $rekaps,
        ];
        $content = view('analisis_kebutuhan.pakaian_dinas.laporan.cetakRekapTemplateV', $data)->render();
        // return view('analisis_kebutuhan.pakaian_dinas.laporan.cetakRekapTemplateV', $data);
        $pdf = Pdf::loadHTML($content);
        $pdf->setPaper('A4', 'landscape');

        return $pdf->stream("{$model->nama}.pdf");
    }

    public function mapUkuranObj($result)
    {
        $ukurans = [];
        foreach ($result as $key => $value) {
            $ukurans[$value->inst_satkerkd] = [
                'baju' => $this->parseUkuranObject($value->baju),
                'celana' => $this->parseUkuranObject($value->celana),
                'sepatu' => $this->parseUkuranObject($value->sepatu),
                'hijab' => json_decode($value->hijab)->hijab ?? null,
            ];
        }

        return $ukurans;
    }

    public function parseUkuranObject($obj)
    {
        $ukurans = json_decode($obj);
        $isian = '';
        foreach ($ukurans as $ukuran => $jumlah) {
            $isian .= "{$ukuran}: {$jumlah}<br>";
        }

        return $isian;
    }
}
