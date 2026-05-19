<?php

namespace App\Http\Controllers\AnalisisKebutuhan\Monev;

use App\Exports\ExportExcel;
use App\Http\Controllers\Controller;
use App\Models\AnalisisKebutuhan\Bmn as Model;
use App\Models\AnalisisKebutuhan\BmnSatkerBarang;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\DB;
use Maatwebsite\Excel\Facades\Excel;

class KebutuhanBmnController extends Controller
{
    protected $breadcums = ['Analisis Kebutuhan', 'Monitoring dan Evaluasi', 'Kebutuhan BMN'];

    private $controller = '/analisis-kebutuhan/monev/kebutuhan-bmn';

    public function index()
    {
        $data = [
            'tableId' => 'dt-pengajuan',
            'breadcums' => $this->breadcums,
            'controller' => $this->controller,
        ];

        return view('analisis_kebutuhan.monev.daftar_kebutuhan.gridV', $data);
    }

    protected function canCreatePermintaan()
    {
        return true;
        // return session('userData.current_role.ms_role_id') == config('constants.admin_biro_lengkap_role_id') ? true : false;
    }

    public function gridData(Request $request)
    {
        $user = new Model;
        $pagingParams = $request->only(['start', 'length']);
        $searchParams = $request->only(['columns']);

        $data = $user->getGridData($pagingParams, $searchParams);

        return response()->json([
            'data' => $data['data'],
            'recordsTotal' => $data['total'],
            'recordsFiltered' => $data['total'],
        ]);
    }

    public function gridDataSatker(Request $request)
    {
        $user = new Model;
        $pagingParams = $request->only(['start', 'length']);
        $searchParams = $request->only(['columns', 'id']);
        $data = $user->getGridDataSatkerDetail($pagingParams, $searchParams);

        return response()->json([
            'data' => $data['data'],
            'recordsTotal' => $data['total'],
            'recordsFiltered' => $data['total'],
        ]);
    }

    public function gridDataBarang($id)
    {
        $data = BmnSatkerBarang::where('pengajuan_kebutuhan_bmn_satker_id', $id)->get()->toArray();

        return response()->json([
            'data' => $data,
        ]);
    }

    /**
     * Show the form for creating a new resource.
     */
    public function create()
    {
        // $data = $this->getData();
        // return view('analisis_kebutuhan.bmn.pengajuanFormV', $data);
    }

    /**
     * Store a newly created resource in storage.
     */
    public function store(Request $request) {}

    /**
     * edit admin perlengkapan
     */
    public function show(string $id)
    {
        $data = [
            'tableId' => 'dt-pengajuan',
            'breadcums' => array_merge($this->breadcums, ['Daftar Kebutuhan BMN Satker']),
            'controller' => $this->controller,
            'id' => $id,
        ];

        return view('analisis_kebutuhan.monev.daftar_kebutuhan.gridSatkerV', $data);
    }

    /**
     * pelaksana ngisi /  validator ngeliat
     */
    public function edit(string $id) {}

    /**
     * Update the specified resource in storage.
     */
    public function update(Request $request, string $id)
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
            Model::destroy($id);
            DB::commit();

            return $this->resSuccess('Berhasil Dihapus!');
        } catch (\Throwable $th) {
            DB::rollBack();

            return $this->resError('Gagal Menghapus data');
        }
    }

    public function cetakExcel(Request $request)
    {
        $user = new Model;
        $pagingParams['start'] = 0;
        $pagingParams['length'] = -1;
        $searchParams = $request->only(['columns', 'id']);
        $data = $user->getGridDataSatkerDetail($pagingParams, $searchParams);
        $columns = ['id', 'tahun', 'nama', 'satker', 'nm_barang', 'kode_barang', 'jumlah di satker', 'jumlah', 'jml_setuju', 'jml_tolak', 'keterangan', 'alasan', 'prioritas'];

        return Excel::download(new ExportExcel($data['data']->toArray(), $columns, 'kebutuhan bmn'), 'kebutuhan_bmn.xlsx', \Maatwebsite\Excel\Excel::XLSX);
    }
}
