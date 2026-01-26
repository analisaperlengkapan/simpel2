<?php
namespace App\Http\Controllers\Monsakti;
use Illuminate\Routing\Controller;

use Illuminate\Http\Request;
use App\Models\Monsakti\TransaksiAset;

class TransaksiAsetController extends Controller
{
    public function gridData(Request $request)
    {
        $model = new TransaksiAset();
        $pagingParams = $request->only(['start', 'length']);
        $searchParams =  $request->only(['columns','kode_barang','nup','kdsatker','kduakpb']);
        $data = $model->getDataGrid($pagingParams, $searchParams);
        return response()->json([
            'data' => $data['data'],
            'recordsTotal' => $data['total'],
            'recordsFiltered' => $data['total'],
        ]);
    }
}
