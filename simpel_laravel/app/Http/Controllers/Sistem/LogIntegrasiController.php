<?php

namespace App\Http\Controllers\Sistem;
use Illuminate\Routing\Controller;

use App\Models\Sistem\LogIntegrasi as Model;
use Illuminate\Http\Request;


class LogIntegrasiController extends Controller
{
    protected $breadcums = ['Log Integrasi'];
    protected $controller = '/log-integrasi';
    /**
     * Display a listing of the resource.
     */
    public function index()
    {
        $columns = [
            'Waktu',
            'Aplikasi',
            'Endpoint',
            'Deskripsi',
            'Total Data',
        ];
        $defColumns = [0, 1, 2, 3, 4];
        return view('log-integrasiV', [
            'tableId' => 'dt-role',
            'columns' => $columns,
            'defColumns' => $defColumns,
            'title' => 'Log Integrasi',
            'controller' => $this->controller,
            'breadcums' => $this->breadcums
        ]);
    }

    public function gridData(Request $request)
    {
        $model = new Model();
        $pagingParams = $request->only(['start', 'length']);
        $searchParams = $request->only(['columns']);
        $data = $model->getDataGrid($pagingParams, $searchParams);
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
