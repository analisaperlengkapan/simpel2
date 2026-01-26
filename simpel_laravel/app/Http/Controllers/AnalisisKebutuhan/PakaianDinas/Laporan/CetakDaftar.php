<?php

namespace App\Http\Controllers\AnalisisKebutuhan\PakaianDinas\Laporan;
use Illuminate\Routing\Controller;

use App\Models\AnalisisKebutuhan\PakaianDinas as Model;
use App\Models\AnalisisKebutuhan\PakaianDinasSatker;
use App\Models\AnalisisKebutuhan\PakaianDinasSatkerPegawai;
use App\Models\Master\MsSatker;
use Barryvdh\DomPDF\Facade\Pdf;
use Illuminate\Http\Request;

class CetakDaftar extends Controller
{
    protected $breadcums = ['Analisis Kebutuhan', 'Pakaian Dinas', 'Laporan', 'Cetak Daftar'];
    private $controller = '/analisis-kebutuhan/pakaian-dinas/laporan/cetak-daftar';
    public function index()
    {
        return view('analisis_kebutuhan.pakaian_dinas.laporan.cetakDaftarV', [
            'tableId' => 'dt-user',
            'breadcums' => $this->breadcums,
            'controller' => $this->controller
        ]);
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

    public function cetak(int $pengajuanId)
    {
        $pengajuanSatkerId = $_GET['pengajuanSatkerId'];
        $satker = PakaianDinasSatker::where(['id' => $pengajuanSatkerId])->first();
        $data = [
            'header' => Model::where(['id' => $pengajuanId])->first(),
            'satker' => MsSatker::where(['inst_satkerkd' => $satker['ms_satker_id']])->first(),
            'pegawais' => PakaianDinasSatkerPegawai::getDetail($pengajuanSatkerId),
        ];
        // return view('analisis_kebutuhan.pakaian_dinas.laporan.cetakDaftarTemplateV', $data);
        $content = view('analisis_kebutuhan.pakaian_dinas.laporan.cetakDaftarTemplateV', $data)->render();
        $pdf = Pdf::loadHTML($content);
        $pdf->setPaper('A4', 'potrait');
        return $pdf->stream("{$data['header']->nama} - {$data['satker']->inst_nama}.pdf");
    }

}