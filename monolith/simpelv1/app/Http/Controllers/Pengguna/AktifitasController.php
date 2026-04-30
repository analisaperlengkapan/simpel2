<?php

namespace App\Http\Controllers\Pengguna;

use App\Http\Controllers\Controller;
use App\Models\Pengguna\Aktifitas;
use Illuminate\Http\Request;


class AktifitasController extends Controller
{
    protected $breadcums = ['Pengaturan', 'Aktifitas Pengguna'];
    protected $controller = '/pengguna/aktifitas';
    /**
     * Display a listing of the resource.
     */
    public function index()
    {
        $columns = ['Waktu', 'NIP', 'Operasi', 'Keterangan', 'IP', 'Peramban', 'Akses VIA'];
        $defColumns = [0, 1, 2, 3, 4, 5, 6];
        return view('pengguna.aktifitas.aktifitasV', [
            'tableId' => 'dt-role',
            'columns' => $columns,
            'defColumns' => $defColumns,
            'controller' => $this->controller,
            'breadcums' => $this->breadcums
        ]);
    }

    public function gridData(Request $request)
    {
        $model = new Aktifitas();
        $pagingParams = $request->only(['start', 'length']);
        $searchParams = $request->only(['columns']);
        $data = $model->getGridData($pagingParams, $searchParams);
        return response()->json([
            'data' => $data['data'],
            'recordsTotal' => $data['total'],
            'recordsFiltered' => $data['total'],
        ]);
    }

    function getData($id = null)
    {

    }
    /**
     * Show the form for creating a new resource.
     */
    public function create()
    {

    }

    /**
     * Store a newly created resource in storage.
     */
    public function store(Request $request)
    {
    }

    /**
     * Display the specified resource.
     */
    public function show(string $id)
    {
    }

    /**
     * Show the form for editing the specified resource.
     */
    public function edit(string $id)
    {
        //
    }

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
    }
}
