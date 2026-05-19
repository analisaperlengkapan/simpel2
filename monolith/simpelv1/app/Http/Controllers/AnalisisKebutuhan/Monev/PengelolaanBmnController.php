<?php

namespace App\Http\Controllers\AnalisisKebutuhan\Monev;

use App\Http\Controllers\Controller;
use App\Models\AnalisisKebutuhan\Bmn as Model;
use App\Models\MsSatker;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\DB;

class PengelolaanBmnController extends Controller
{
    protected $breadcums = ['Analisis Kebutuhan', 'Monitoring dan Evaluasi', 'Pengelolaan BMN'];

    private $controller = '/analisis-kebutuhan/monev/pengelolaan-bmn';

    public function index()
    {
        $data = [
            'tableId' => 'dt-pengajuan',
            'breadcums' => $this->breadcums,
            'controller' => $this->controller,
            'canCreate' => $this->canCreatePermintaan(),
        ];

        return view('analisis_kebutuhan.monev.pengelolaanBmnV', $data);
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
        $searchParams = $request->only(['columns', 'id']);
        $data = $user->getGridDataRusak($pagingParams, $searchParams);

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
        $searchParams = $request->only(['search', 'filterBy']);
        $msSatkerId = $_GET['satkerId'] ?? null;
        $data = $user->getGridData($pagingParams, $searchParams, $msSatkerId);

        return response()->json([
            'data' => $data['data'],
            'recordsTotal' => $data['total'],
            'recordsFiltered' => $data['total'],
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
    public function store(Request $request)
    {
        // $validasi = [
        //     'nama' => 'required',
        //     'jenis_asset' => 'required',
        //     'jumlah' => 'required',
        //     'satuan' => 'required',
        // ];

        // $request->validate($validasi);
        // try {

        //     $inputan = $request->input();
        //     DB::beginTransaction();
        //     $id = $request->input('id');
        //     if (!$request->has('id')) {
        //         $inputan['created_by'] = session('userData.username');
        //         $id = MyHelper::getPk(date('Ymd'), 'pengajuan_pakaian_dinas_seq');
        //         $inputan['ms_aktifitas_id'] = 1000;
        //     }
        //     Model::updateOrCreate(['id' => $id], $inputan);
        //     DB::commit();
        //     return $this->resSuccess('Berhasil Dismpan');
        // } catch (\Throwable $th) {
        //     dd($th->getMessage());
        //     return $this->resError('Gagal menyimpan data');
        // }
    }

    /**
     * edit admin perlengkapan
     */
    public function show(string $id)
    {
        $data = [
            'tableId' => 'dt-pengajuan',
            'breadcums' => array_merge($this->breadcums, ['Satuan Kerja']),
            'satker' => MsSatker::where(['inst_satkerkd' => $id])->first(),
            'controller' => $this->controller,
        ];

        return view('analisis_kebutuhan.monev.pengelolaanBmnSatkerV', $data);
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
}
