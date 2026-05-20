<?php

namespace App\Http\Controllers\Pengguna;

use App\Http\Controllers\Controller;
use App\Models\Pengguna\Pengguna;
use App\Models\Pengguna\Review;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\DB;

class ReviewController extends Controller
{
    /**
     * Display a listing of the resource.
     */
    protected $breadcums = ['Pengguna', 'Penilai Aplikasi'];

    protected $controller = 'pengguna/review';

    public function index()
    {

        $columns = ['Username', 'Rating', 'Review', 'Tanggal', 'Platform'];
        $defColumns = [0, 1, 2, 3, 4, 5];

        return view('pengguna.review.reviewV', [
            'columns' => $columns,
            'defColumns' => $defColumns,
            'tableId' => 'dt-survey',
            'controller' => $this->controller,
            'breadcums' => $this->breadcums,
        ]);
    }

    public function gridData(Request $request)
    {
        $model = new Review;
        $pagingParams = $request->only(['start', 'length']);
        $searchParams = $request->only(['columns']);
        $data = $model->getDataGrid($pagingParams, $searchParams);

        return response()->json([
            'data' => $data['data'],
            'recordsTotal' => $data['total'],
            'recordsFiltered' => $data['total'],
        ]);
    }

    public function getData($id = null, $readOnly = false) {}

    public function create() {}

    public function store(Request $request)
    {
        $request->validate([
            'rating' => 'required',
        ]);
        $data = $request->input();
        $data['username'] = session('userData.username');

        Review::create($data);
        Pengguna::where('username', session('userData.username'))->update(['has_review' => 1]);

        return $this->resSuccess();
    }

    public function show(string $id)
    {
        $model = [];
        $model = Review::where('id', $id)->first();

        $data = [
            'model' => $model,
            'breadcums' => $this->breadcums,
        ];

        return view('pengguna.review.detailV', $data);
    }

    public function edit(string $id) {}

    public function update(Request $request)
    {
        //
    }

    public function destroy(string $id)
    {
        try {
            DB::beginTransaction();
            Review::destroy($id);
            DB::commit();

            return $this->resSuccess('Berhasil Dihapus!');
        } catch (\Throwable $th) {
            DB::rollBack();
        }
    }
}
