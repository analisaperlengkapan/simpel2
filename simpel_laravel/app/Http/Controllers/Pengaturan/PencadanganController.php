<?php

namespace App\Http\Controllers\Pengaturan;
use Illuminate\Routing\Controller;

use App\Models\Sistem\Backup;
use Illuminate\Http\Request;
use Symfony\Component\HttpKernel\Exception\NotFoundHttpException;

class PencadanganController extends Controller
{
    /**
     * Display a listing of the resource.
     */
    protected $breadcums = ['Pengaturan', 'Pencadangan dan Pemulihan Data'];
    private $controller = '/pengaturan/pencadangan';
    public function index()
    {
        $columns = [
            'Tanggal',
            'Filename',
            'Status',
        ];
        $defColumns = [0, 1, 2];
        $data = [
            'tableId' => 'dt-penetapan',
            'controller' => $this->controller,
            'breadcums' => $this->breadcums,
            'columns' => $columns,
            'defColumns' => $defColumns
        ];
        return view('pengaturan.pencadangan.formV', $data);
    }

    public function gridData(Request $request)
    {
        $model = new Backup();
        $pagingParams = $request->only(['start', 'length']);
        $searchParams = $request->only(['columns']);
        $data = $model->getDataGrid($pagingParams, $searchParams);
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

    }

    /**
     * Store a newly created resource in storage.
     */
    public function store(Request $request)
    {
        $fileName = date('YmdHis');
        $basePath = base_path();
        $model = Backup::create([
            'filename' => $fileName . '.sql',
            'path' => "{$basePath}/storage/bu/{$fileName}.sql",
            'created_by' => session('userData.username'),
            'is_complete' => 0,
        ]);

        // $scriptOutput = [];
        // $exitCode = 0;

        exec("sh {$basePath}/simanal_bu.sh {$fileName} {$model->id} &");
        // dd("sh {$basePath}/simanal_bu.sh {$fileName} {$model->id} &");
        return $this->resSuccess('ok', ['type' => 'redirect', 'url' => '/pengaturan/pencadangan']);
        // // Handle the script output and exit code as needed
        // if ($exitCode === 0) {
        //     // Script executed successfully
        //     return response()->json(['output' => $scriptOutput], 200);
        // } else {
        //     // Script execution failed
        //     return response()->json(['error' => 'Script execution failed'], 500);
        // }
    }

    public function restore(Request $request)
    {
        $id = $request->input('id');
        $bu = Backup::where('id', $id)->first();
        if (!$bu)
            throw new NotFoundHttpException();

        $isExists = file_exists($bu->path);
        if (!$isExists)
            throw new NotFoundHttpException('File Backup Tidak Ditermukan ');

        $scriptOutput = [];
        $errorOutput = [];
        $exitCode = 0;

        $basePath = base_path();
        exec("sh {$basePath}/simanal_restore.sh {$bu->path}", $scriptOutput, $exitCode);
        if ($exitCode !== 0) {
            return $this->resError('Script execution failed', null, $scriptOutput);
        }
        return $this->resSuccess();
    }

    /**
     * Display the specified resource.
     */
    public function show(string $id)
    {
        $bu = Backup::where('id', $id)->first();
        if (!$bu)
            throw new NotFoundHttpException();

        $isExists = file_exists($bu->path);
        if (!$isExists)
            throw new NotFoundHttpException('File Backup Tidak Ditermukan ');
        return response()->download($bu->path, $bu->filename);
    }

    /**
     * Show the form for editing the specified resource.
     */
    public function edit(string $id)
    {

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

    }
}
