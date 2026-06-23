<?php

namespace App\Http\Controllers\Master;

use App\Exports\ExportExcel;
use App\Http\Controllers\Controller;
use App\Models\Master\MsPegawai;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\DB;
use Maatwebsite\Excel\Facades\Excel;
use Mccarlosen\LaravelMpdf\Facades\LaravelMpdf;
use Symfony\Component\HttpKernel\Exception\NotFoundHttpException;

class PegawaiController extends Controller
{
    /**
     * Display a listing of the resource.
     */
    // protected $breadcums = ['Master', 'Pegawai'];
    protected $kategoriJudul = 'Pegawai';

    protected $controller = 'master/pegawai';

    protected $breadcums = ['Master'];

    protected $columns = ['NIP', 'Nama', 'Email', 'Pangkat', 'Jabatan', 'Alamat', 'Satker', 'Jenis Kelamin', 'Agama', 'Tempat Lahir', 'Tgl Lahir', 'Jenis', 'Unit Kerja'];

    protected $defColumns = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];

    public function __construct()
    {
        $this->breadcums = array_merge($this->breadcums, [['link' => $this->controller, 'title' => 'Pegawai']]);

    }

    public function index()
    {
        // return view('master.pegawai.pegawaiV', ['tableId' => 'dt-kritik', 'breadcums' => $this->breadcums]);
        return view('master.pegawai.pegawaiV', [
            'tableId' => 'dt-pegawai',
            'kategoriJudul' => $this->kategoriJudul,
            'breadcums' => $this->breadcums,
            'columns' => $this->columns,
            'defColumns' => $this->defColumns,
            'controller' => $this->controller,
        ]);
    }

    public function gridData(Request $request)
    {
        $model = new MsPegawai;
        $pagingParams = $request->only(['start', 'length']);
        // $searchParams =  $request->only(['search',  'filterBy']);
        $searchParams = $request->only(['columns']);
        $data = $model->getDataGrid($pagingParams, $searchParams);

        return response()->json([
            'data' => $data['data'],
            'recordsTotal' => $data['total'],
            'recordsFiltered' => $data['total'],
        ]);
    }

    public function getData($id = null)
    {
        $model = [];
        $isNew = true;

        if ($id) {
            $model = MsPegawai::where('id', $id)->first();
            if (! $model) {
                throw new NotFoundHttpException('Data Tidak Ditemukan');
            }

            $model = $model->toArray();
            $isNew = false;
        }

        $data = [
            'model' => $model,
            'isNew' => $isNew,
        ];

        return $data;
    }

    /**
     * Show the form for creating a new resource.
     */
    public function create()
    {
        $data = $this->getData();

        return view('master.pegawai.pegawaiFormV', $data);
    }

    /**
     * Store a newly created resource in storage.
     */
    public function store(Request $request)
    {
        $request->validate([
            // 'inst_nama' => 'required',
            // 'inst_jenis' => 'required'
        ]);

        $data = new MsPegawai;
        $id = isset($request->input()['id']) ? $request->input()['id'] : null;
        if ($id) {
            $data = MsPegawai::find($id);
        }
        $data->fill($request->input());
        $data->save();

        return $this->resSuccess();
    }

    /**
     * Display the specified resource.
     */
    public function show(string $id)
    {
        $data = $this->getData($id);

        return view('master.pegawai.pegawaiFormV', $data);
    }

    /**
     * Show the form for editing the specified resource.
     */
    public function edit(MsPegawai $pegawai)
    {
        //
    }

    /**
     * Update the specified resource in storage.
     */
    public function update(Request $request, MsPegawai $pegawai)
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
            MsPegawai::destroy($id);
            DB::commit();

            return $this->resSuccess('Berhasil Dihapus!');
        } catch (\Throwable $th) {
            DB::rollBack();
        }
    }

    public function cetakExcel(Request $request)
    {
        try {
            // Set timeout dan memory limit untuk server
            ini_set('memory_limit', '2G');
            set_time_limit(1800); // 30 menit timeout
            \Log::info('cetakExcel called with params:', $request->all());

            $model = new MsPegawai;
            $searchParams = $request->only(['columns']);
            $paging['length'] = -1;
            $paging['start'] = 1;
            $isKolom = $request->input('isKolom');
            $select = [];
            $selectView = [];

            \Log::info('isKolom:', ['isKolom' => $isKolom]);

            if ($isKolom == 'all') {
                // Get all columns from the table
                $columns = \DB::getSchemaBuilder()->getColumnListing((new MsPegawai)->getTable());
                // Get data without specific select
                $data = $model->getDataGrid($paging, $searchParams);
            } else {
                $visible = explode(',', $request->input('visible'));
                foreach ($visible as $key) {
                    if (isset($searchParams['columns'][$key]['data'])) {
                        $kolomSelect = 'a.'.$searchParams['columns'][$key]['data'];
                        $kolomView = $searchParams['columns'][$key]['data'];
                        array_push($select, $kolomSelect);
                        array_push($selectView, $kolomView);
                    }
                }
                $columns = $selectView;
                $data = $model->getDataGrid($paging, $searchParams, $select);
            }

            \Log::info('Data retrieved:', ['count' => count($data['data']), 'columns' => $columns]);

            // Check if data is large and use chunking
            if (count($data['data']) > 5000) {
                \Log::info('Large dataset detected, using chunking approach');

                return $this->generateExcelWithChunking($data['data'], $columns, $isKolom);
            }

            // Convert data to array format for Excel - use object properties
            $excelData = [];
            foreach ($data['data'] as $row) {
                $rowArray = [];
                foreach ($columns as $column) {
                    // Handle object properties correctly
                    $value = '';
                    if (is_object($row)) {
                        $value = $row->$column ?? '';
                    } elseif (is_array($row)) {
                        $value = $row[$column] ?? '';
                    }
                    $rowArray[] = $value;
                }
                $excelData[] = $rowArray;
            }

            \Log::info('Excel data prepared:', ['rows' => count($excelData), 'sample' => array_slice($excelData, 0, 2)]);

            $filename = 'master_pegawai_'.date('Y-m-d_H-i-s').'.xlsx';

            return Excel::download(
                new ExportExcel($excelData, $columns, 'Daftar Master Pegawai'),
                $filename,
                \Maatwebsite\Excel\Excel::XLSX,
                [
                    'Content-Type' => 'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet',
                    'Content-Disposition' => 'attachment; filename="'.$filename.'"',
                ]
            );
        } catch (\Exception $e) {
            \Log::error('Error in cetakExcel:', ['error' => $e->getMessage(), 'trace' => $e->getTraceAsString()]);

            return response()->json(['error' => $e->getMessage()], 500);
        }
    }

    private function generateExcelWithChunking($data, $columns, $isKolom)
    {
        try {
            \Log::info('Starting chunked Excel generation');
            $filename = 'master_pegawai_'.date('Y-m-d_H-i-s').'.xlsx';
            $chunkSize = 2000; // Smaller chunk size for better memory management
            $chunks = array_chunk($data->toArray(), $chunkSize);
            $excelData = [];

            foreach ($chunks as $chunkIndex => $chunk) {
                \Log::info('Processing chunk '.($chunkIndex + 1).' of '.count($chunks));

                foreach ($chunk as $row) {
                    $rowArray = [];
                    foreach ($columns as $column) {
                        $value = '';
                        if (is_object($row)) {
                            $value = $row->$column ?? '';
                        } elseif (is_array($row)) {
                            $value = $row[$column] ?? '';
                        }
                        $rowArray[] = $value;
                    }
                    $excelData[] = $rowArray;
                }

                // Memory cleanup after each chunk
                unset($chunk);
                gc_collect_cycles();

                // Small delay to prevent server overload
                usleep(100000); // 0.1 second delay
            }

            \Log::info('Chunked Excel data prepared:', ['rows' => count($excelData)]);

            return Excel::download(
                new ExportExcel($excelData, $columns, 'Daftar Master Pegawai'),
                $filename,
                \Maatwebsite\Excel\Excel::XLSX,
                [
                    'Content-Type' => 'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet',
                    'Content-Disposition' => 'attachment; filename="'.$filename.'"',
                ]
            );
        } catch (\Exception $e) {
            \Log::error('Error in generateExcelWithChunking:', ['error' => $e->getMessage()]);
            throw $e;
        }
    }

    public function cetakPdf(Request $request)
    {
        try {
            // Set timeout dan memory limit untuk server
            ini_set('memory_limit', '1G');
            set_time_limit(900); // 15 menit timeout untuk PDF
            \Log::info('cetakPdf called with params:', $request->all());

            $model = new MsPegawai;
            $searchParams = $request->only(['columns']);

            // Get data with pagination for PDF to avoid memory issues
            $paging['length'] = 500; // Reduce to 500 records per page for PDF
            $paging['start'] = 0;

            $data = $model->getDataGrid($paging, $searchParams);
            $selectedColumns = array_intersect_key($this->columns, array_flip($this->defColumns));

            \Log::info('PDF data retrieved:', ['count' => count($data['data']), 'columns' => $selectedColumns]);

            // Convert data to array format for PDF - use the actual data from database
            $pdfData = [];
            foreach ($data['data'] as $row) {
                $rowArray = [];
                // Use the actual column names from the database
                $rowArray[] = $row->peg_nip_baru ?? '';
                $rowArray[] = $row->nama ?? '';
                $rowArray[] = $row->pns_mail ?? '';
                $rowArray[] = $row->pangkat ?? '';
                $rowArray[] = $row->jabatan ?? '';
                $rowArray[] = $row->alamat ?? '';
                $rowArray[] = $row->satker ?? '';
                $rowArray[] = $row->jenis_kelamin ?? '';
                $rowArray[] = $row->agama ?? '';
                $rowArray[] = $row->tempat_lahir ?? '';
                $rowArray[] = $row->tgl_lahir ?? '';
                $rowArray[] = $row->jenis ?? '';
                $rowArray[] = $row->unitkerja_nama ?? '';
                $pdfData[] = $rowArray;
            }

            \Log::info('PDF data prepared:', ['rows' => count($pdfData), 'sample' => array_slice($pdfData, 0, 2)]);

            $pdf = $this->generateSimplePdf($selectedColumns, $pdfData, 'Daftar Master Pegawai');

            return $pdf;
        } catch (\Exception $e) {
            \Log::error('Error in cetakPdf:', ['error' => $e->getMessage(), 'trace' => $e->getTraceAsString()]);

            return response()->json([
                'error' => 'PDF generation failed',
                'message' => 'PDF tidak dapat dibuat. Silakan gunakan tombol Excel sebagai alternatif.',
                'data_count' => count($data['data'] ?? []),
                'columns' => $selectedColumns ?? [],
            ], 500);
        }
    }

    private function generateSimplePdf($selectedColumns, $pdfData, $judul)
    {
        try {
            $filename = str_replace(' ', '_', strtolower($judul)).'.pdf';

            // Use LaravelMpdf with minimal configuration
            $pdf = LaravelMpdf::loadView('exports.asset', [
                'headers' => $selectedColumns,
                'rows' => $pdfData,
                'judul' => $judul,
                'defColumns' => $this->defColumns,
            ], [
                'title' => $judul,
                'format' => 'A4-L',
                'orientation' => 'L',
                'mode' => 'utf-8',
                'margin_left' => 10,
                'margin_right' => 10,
                'margin_top' => 10,
                'margin_bottom' => 10,
            ]);

            return $pdf->stream($filename);
        } catch (\Exception $e) {
            \Log::error('Error in generateSimplePdf:', ['error' => $e->getMessage()]);

            // Fallback with even simpler configuration
            try {
                $pdf = LaravelMpdf::loadView('exports.asset', [
                    'headers' => $selectedColumns,
                    'rows' => $pdfData,
                    'judul' => $judul,
                    'defColumns' => $this->defColumns,
                ]);

                return $pdf->stream($filename);
            } catch (\Exception $e2) {
                \Log::error('Error in fallback PDF generation:', ['error' => $e2->getMessage()]);
                throw $e2;
            }
        }
    }
}
