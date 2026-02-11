<?php

namespace App\Http\Controllers\AssetTik;

use App\Helpers\MyHelper;
use App\Models\Files;
use App\Http\Controllers\Controller;
use App\Models\AssetTik\Hakcipta;
use App\Models\Master;
use App\Models\Master\MsSatker;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\DB;
use Symfony\Component\HttpKernel\Exception\NotFoundHttpException;

class HakciptaController extends Controller
{
    /**
     * Display a listing of the resource.
     */
    protected $breadcums = ['Asset TIK', 'Daftar Hak Cipta & Lisensi Khusus TIK'];
    protected $controller = '/sdm/timakuntansibarang';
    protected $columns = ['Nama Satker', 'Merk', 'No. Lisensi', 'Jangka Waktu(Bln)', 'Nilai'];
    protected $defColumns = [0, 1, 2, 3, 4, 5];

    protected function canCreate()
    {
        // return true;
        return in_array(session('userData.current_role.ms_role_id'), config('constants.pelaksana_role'));
    }

    public function index()
    {
        //print_r(session('userData.current_role'));exit;

        $data = ['tableId' => 'dt-hakcipta', 'breadcums' => $this->breadcums, 'columns' => $this->columns, 'defColumns' => $this->defColumns, 'controller' => $this->controller, 'canCreate' => $this->canCreate()];
        return view('asset_tik.hakcipta.hakciptaV', $data);
    }

    public function gridData(Request $request)
    {
        $model = new Hakcipta();
        $pagingParams = $request->only(['start', 'length']);
        $searchParams =  $request->only(['columns']);
        $data = $model->getDataGrid($pagingParams, $searchParams);
        return response()->json([
            'data' => $data['data'],
            'recordsTotal' => $data['total'],
            'recordsFiltered' => $data['total'],
        ]);
    }

    function getData($id = null, $readOnly = false)
    {
        $model = [];
        $isNew = true;
        $breadcum = 'Tambah';
        if ($id) {
            $breadcum = 'Ubah';
            $model = Hakcipta::where('id', $id)->first();
            if (!$model) {
                throw new NotFoundHttpException('Data Tidak Ditemukan');
            }

            $model = $model->toArray();
            $isNew = false;

            $ms_satker_id = $model['id_satker'];
        } else {
            $currentRole = session('userData.current_role');
            $ms_satker_id = $currentRole['ms_satker_id'] ?? $model['id_satker'];
        }

        if ($readOnly) {
            $breadcum = 'Detail';
        }
        $satkers = Master::getSatkersKeu();
        $nama_satker = MsSatker::where('inst_satkerkd', $ms_satker_id)->first();
        $tipe_beli = ['BELI BARU', 'PERPANJANGAN'];
        $is_lifetime = ['TIDAK', 'YA'];
        $is_pnbp = ['TIDAK', 'YA'];

        $model['kdsatker_keu'] = $nama_satker['kdsatker_keu'];
        $model['inst_nama'] = MsSatker::where('inst_satkerkd', $ms_satker_id)->first()['inst_nama'];

        $data = [
            'model' => $model,
            'isNew' => $isNew,
            'breadcums' => array_merge($this->breadcums, [$breadcum]),
            'judul' => $breadcum,
            'readOnly' => $readOnly,
            'satkerOptions' => MyHelper::generateSelectOptions([
                'data' => $satkers,
                'text' => 'inst_nama',
                'textkode' => 'kdsatker_keu',
                'value' => 'kdsatker_keu',
                'selected' => $model['kdsatker_keu'] ?? null,
            ]),
            'tipebeliOptions' => MyHelper::generateSelectOptions([
                'data' => $tipe_beli,
                'text' => 'tipe_beli',
                'value' => null,
                'selected' => $model['tipe_beli'] ?? null,
            ]),
            'lifetimeOptions' => MyHelper::generateSelectOptions([
                'data' => $is_lifetime,
                'text' => 'is_lifetime',
                'value' => null,
                'selected' => $model['is_lifetime'] ?? null,
            ]),
            'pnbpOptions' => MyHelper::generateSelectOptions([
                'data' => $is_pnbp,
                'text' => 'is_pnbp',
                'value' => null,
                'selected' => $model['is_pnbp'] ?? null,
            ]),
        ];
        return $data;
    }

    /**
     * Show the form for creating a new resource.
     */
    public function create()
    {
        $data = $this->getData();
        return view('asset_tik.hakcipta.hakciptaFormV', $data);
    }

    /**
     * Store a newly created resource in storage.
     */
    public function store(Request $request)
    {
        $isNew = $request->input('isNew');
        $customMessages = [
            'kdsatker_keu.required' => 'Satker harus diisi',
            'no_lisensi.required' => 'No Lisensi harus diisi',
            'merk.required' => 'Merk harus diisi',
            'tipe_beli.required' => 'Tipe Beli harus diisi',
            'is_lifetime.required' => 'Lifetime harus diisi',
            // 'is_pnbp.required' => 'PNBP harus diisi',
        ];
        $validate = [
            'kdsatker_keu' => 'required',
            'no_lisensi' => 'required',
            'merk' => 'required',
            'tipe_beli' => 'required',
            'is_lifetime' => 'required',
            // 'is_pnbp' => 'required',
        ];
        $id = $request->input('id') ?? MyHelper::getPk(date('Ymd'), 'asset_tik_hakcipta_seq');
        if ($isNew) {
            $validate['file_invoice'] = 'required|mimes:jpeg,png,pdf|max:2048';
            $customMessages['file_invoice.required'] = 'File Invoice harus diupload';
        }
        $request->validate($validate, $customMessages);

        try {
            DB::beginTransaction();
            $currentRole = session('userData.current_role');
            $ms_satker_id = $currentRole['ms_satker_id'] ?? '';
            $ms_satker_id_keu = $currentRole['ms_satker_id_keu'] ?? '';
            $data = [
                'id_satker' => $ms_satker_id,
                'kdsatker_keu' => $ms_satker_id_keu,
                'no_lisensi' => $request->input('no_lisensi'),
                'merk' => $request->input('merk'),
                'tipe_beli' => $request->input('tipe_beli'),
                'is_lifetime' => $request->input('is_lifetime'),
                // 'is_pnbp' => $request->input('is_pnbp'),
                'jangka_waktu' => str_replace('.', '', $request->input('jangka_waktu')),
                'nilai' => str_replace('.', '', $request->input('nilai')),
            ];

            // if ($request->hasFile('file_invoice')) {
            //     $filepath = 'uploads/asset-tik/hakcipta';
            //     $file = $request->file('file_invoice');
            //     $fileName = $id.'_file_invoice'.'.'.$file->getClientOriginalExtension();
            //     $filesave = $filepath.'/'.$fileName;
            //     $file->move($filepath, $fileName);
            //     $data['file_invoice'] = $filesave;
            // }

            if ($request->hasFile('file_invoice')) {
                $params = [
                    'kategori' => 'Asset TIK - Hak Cipta',
                    'dir' => 'asset-tik/hakcipta',
                    'fileKey' => 'file_invoice',
                    'pkey' => session('userData.username'),
                ];
                $fotos = Files::upload($request, $params);
                $data['file_invoice'] = $fotos['path'];
            }

            //echo "<pre>"; print_r($data);exit;

            Hakcipta::updateOrCreate(['id' => $id], $data);

            DB::commit();
            return $this->resSuccess('Berhasil Disimpan!', [
                'type' => 'redirect',
                'url' => \URL::to('/asset-tik/hakcipta')
            ]);
        } catch (\Throwable $th) {
            DB::rollBack();
            $errorMessage = $th->getMessage();
            return $this->resError($errorMessage);
        }
    }

    /**
     * Display the specified resource.
     */
    public function show(string $id)
    {
        $data = $this->getData($id, true);
        return view('asset_tik.hakcipta.hakciptaFormV', $data);
    }

    /**
     * Show the form for editing the specified resource.
     */
    public function edit(string $id)
    {
        $data = $this->getData($id);
        return view('asset_tik.hakcipta.hakciptaFormV', $data);
    }

    /**
     * Update the specified resource in storage.
     */
    public function update(Request $request, Hakcipta $hakcipta)
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
            Hakcipta::destroy($id);
            DB::commit();
            return $this->resSuccess('Berhasil Dihapus!');
        } catch (\Throwable $th) {
            DB::rollBack();
        }
    }

    public function cetakLabel($id)
    {
        $data = Hakcipta::findOne($id);
        $data = (array) $data;
        $pdf = MyHelper::generateLabelBankAsset($data);
        return $pdf->stream('label-asset-tak-berwujud.pdf');
    }
}
