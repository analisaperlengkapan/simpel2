@extends('layout.main')
@section('content')
<style>
    .kondisi-chart-container {
        min-height: 80px;
        display: flex;
        align-items: center;
        justify-content: center;
    }
    .kondisi-stats {
        font-size: 11px;
        line-height: 1.2;
    }
    .kondisi-stats .text-success {
        color: #28a745 !important;
    }
    .kondisi-stats .text-warning {
        color: #ffc107 !important;
    }
    .kondisi-stats .text-danger {
        color: #dc3545 !important;
    }
</style>
    <div class="row" id="frame">
        <div class="col-12">
            <div class="page-title-box d-sm-flex align-items-center justify-content-between">
                <h4 class="mb-sm-0">Dashboard Data SIMPEL- KEJAKSAAN RI </h4>

                {{-- <div class="page-title-right">
                <ol class="breadcrumb m-0">
                    <li class="breadcrumb-item"><a href="javascript: void(0);">Pages</a></li>
                    <li class="breadcrumb-item active">Starter</li>
                </ol>
            </div> --}}

            </div>
        </div>
    </div>

    <!-- Analisis Kebutuhan BMN Section -->
    <div class="row mb-4">
        <div class="col-12">
            <div class="card">
                <div class="card-header">
                    <h4 class="card-title mb-0">
                        <i class="ri-analysis-line text-warning me-2"></i>
                        Analisis Kebutuhan BMN
                    </h4>
                    <p class="text-muted mb-0">Prediksi kebutuhan penggantian aset berdasarkan nilai buku</p>
                </div>
                <div class="card-body">
                    <div class="row">
                        @php
                        // Sort analisis_kebutuhan_bmn by tahun_prediksi ascending (as integer)
                        $analisis_kebutuhan_bmn = collect($analisis_kebutuhan_bmn)->sortBy(function($item) {
                            return (int)$item->tahun_prediksi;
                        })->values();
                        @endphp
                        @foreach($analisis_kebutuhan_bmn as $analisis)
                        <div class="col-xl-4 col-md-6">
                            <div class="card card-animate border-0 shadow-sm">
                                <div class="card-body">
                                    <div class="d-flex align-items-center">
                                        <div class="flex-shrink-0">
                                            <div class="avatar-sm rounded">
                                                <div class="avatar-title bg-{{ $analisis->tahun_prediksi == '2025' ? 'danger' : ($analisis->tahun_prediksi == '2026' ? 'warning' : 'info') }}-subtle text-{{ $analisis->tahun_prediksi == '2025' ? 'danger' : ($analisis->tahun_prediksi == '2026' ? 'warning' : 'info') }} fs-20">
                                                    <i class="ri-calendar-line"></i>
                                                </div>
                                            </div>
                                        </div>
                                        <div class="flex-grow-1 ms-3">
                                            <h6 class="mb-1">{{ $analisis->tahun_prediksi }}</h6>
                                            <p class="text-muted mb-0 fs-12">{{ $analisis->keterangan }}</p>
                                        </div>
                                    </div>
                                    <div class="mt-3">
                                        <div class="row text-center">
                                            <div class="col-6">
                                                <div class="border-end">
                                                    <h5 class="mb-1 text-primary">{{ number_format($analisis->jumlah_aset, 0, ',', '.') }}</h5>
                                                    <p class="text-muted mb-0 fs-12">Jumlah Aset</p>
                                                </div>
                                            </div>
                                            <div class="col-6">
                                                <div>
                                                    <h5 class="mb-1 text-success">Rp {{ number_format($analisis->total_nilai_penggantian, 0, ',', '.') }}</h5>
                                                    <p class="text-muted mb-0 fs-12">Nilai Penggantian</p>
                                                </div>
                                            </div>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        </div>
                        @endforeach
                    </div>
                </div>
            </div>
        </div>
    </div>
    
    <!-- Chart Analisis Kebutuhan BMN -->
    <div class="row mb-4">
        <div class="col-12">
            <div class="card">
                <div class="card-header">
                    <h4 class="card-title mb-0">
                        <i class="ri-bar-chart-line text-info me-2"></i>
                        Grafik Prediksi Kebutuhan Penggantian Aset
                    </h4>
                </div>
                <div class="card-body">
                    <div class="row">
                        <div class="col-md-8">
                            <div id="analisis_kebutuhan_chart" style="height: 300px;"></div>
                        </div>
                        <div class="col-md-4">
                            <div class="table-responsive">
                                <table class="table table-sm table-borderless">
                                    <thead>
                                        <tr>
                                            <th class="text-center">Tahun</th>
                                            <th class="text-center">Jumlah Aset</th>
                                            <th class="text-center">Nilai (Miliar)</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        @foreach($analisis_kebutuhan_bmn as $analisis)
                                        <tr>
                                            <td class="text-center">
                                                <span class="badge bg-{{ $analisis->tahun_prediksi == '2025' ? 'danger' : ($analisis->tahun_prediksi == '2026' ? 'warning' : 'info') }}-subtle text-{{ $analisis->tahun_prediksi == '2025' ? 'danger' : ($analisis->tahun_prediksi == '2026' ? 'warning' : 'info') }}">
                                                    {{ $analisis->tahun_prediksi }}
                                                </span>
                                            </td>
                                            <td class="text-center fw-bold">{{ number_format($analisis->jumlah_aset, 0, ',', '.') }}</td>
                                            <td class="text-center fw-bold">Rp {{ number_format($analisis->total_nilai_penggantian / 1000000000, 1, ',', '.') }} M</td>
                                        </tr>
                                        @endforeach
                                    </tbody>
                                </table>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    </div>
    <!-- End Chart Analisis Kebutuhan BMN -->
    <!-- End Analisis Kebutuhan BMN Section -->

    @if (session('userData.current_role.ms_satker_id') == '00' && session('userData.current_role.ms_role_id') == '22')
    <div class="row">
        <div class="col-12">
            <div class="card">
                <div class="card-header">
                    <h4 class="mb-sm-0">Filter</h4>
                </div>
                <div class="card-body">
                    <form action="{{ url('dashboard') }}" method="POST" >
                        @csrf
                        <div class="row mb-12">
                            <div class="col-md-2">
                                <label for="nama" class="form-label">Level Laporan</label>
                            </div>
                            <div class="col-md-2">
                                <select class="form-control" data-choices data-choices-removeItem name="level_laporan" id="level_laporan">
                                    {!! $jenisOptions !!}
                                </select>
                            </div>

                            <div class="col-md-2 wilayah" style="display:none;">
                                <label for="nama" class="form-label">Wilayah</label>
                            </div>
                            <div class="col-lg-4 wilayah" style="display:none;">
                                <select class="form-control" data-choices data-choices-removeItem name="wilayah" id="wilayah">
                                    <option value="">Pilih Wilayah</option>
                                    {!! $wilayahOptions !!}
                                </select>
                            </div>

                            <div class="col-md-2 satker" style="display:none;">
                                <label for="nama" class="form-label">Satker</label>
                            </div>
                            <div class="col-lg-4 satker" style="display:none;">
                                <select class="form-control" data-choices data-choices-removeItem name="satker" id="satker">
                                    <option value="">Pilih Satker</option>
                                    {!! $satkerOptions !!}
                                </select>
                            </div>

                            <div class="col-md-2">
                                <button type="submit" class="btn btn-primary">Cari</button>
                                &nbsp;
                                <a href="{{ url('dashboard') }}" class="btn btn-danger">Reset</a>
                            </div>
                        </div>
                    </form>
                </div>
            </div>
        </div>
    </div>
    @endif

    @if (session('userData.current_role.ms_role_id') == '3')
    <div class="row">
        <div class="col-12">
            <div class="card">
                <div class="card-header">
                    <h4 class="mb-sm-0">Filter</h4>
                </div>
                <div class="card-body">
                    <form action="{{ url('dashboard') }}" method="POST" >
                        @csrf
                        <div class="row mb-12">
                            <div class="col-md-2">
                                <label for="nama" class="form-label">Level Laporan</label>
                            </div>
                            <div class="col-md-2">
                                <select class="form-control" data-choices data-choices-removeItem name="level_laporan" id="level_laporan">
                                    {!! $jenisOptions !!}
                                </select>
                            </div>

                            <div class="col-md-2 wilayah" style="display:none;">
                                <label for="nama" class="form-label">Wilayah</label>
                            </div>
                            <div class="col-lg-4 wilayah" style="display:none;">
                                <select class="form-control" data-choices data-choices-removeItem name="wilayah" id="wilayah">
                                    <option value="">Pilih Wilayah</option>
                                    {!! $wilayahOptions !!}
                                </select>
                            </div>

                            <div class="col-md-2 satker" style="display:none;">
                                <label for="nama" class="form-label">Satker</label>
                            </div>
                            <div class="col-lg-4 satker" style="display:none;">
                                <select class="form-control" data-choices data-choices-removeItem name="satker" id="satker">
                                    <option value="">Pilih Satker</option>
                                    {!! $satkerOptions !!}
                                </select>
                            </div>

                            <div class="col-md-2">
                                <button type="submit" class="btn btn-primary">Cari</button>
                                &nbsp;
                                <a href="{{ url('dashboard') }}" class="btn btn-danger">Reset</a>
                            </div>
                        </div>
                    </form>
                </div>
            </div>
        </div>
    </div>                
    @endif

    <div class="row">
        <div class="col-12">
            <div class="card">
                <div class="card-body">
                    <ul class="nav nav-tabs nav-justified mb-3" role="tablist">
                        <li class="nav-item">
                            <a class="nav-link active" data-bs-toggle="tab" href="#grafik" role="tab" aria-selected="true">
                                DASHBOARD DATA SUMMARY
                            </a>
                        </li>
                        <li class="nav-item" style="display:none;">
                            <a class="nav-link active" data-bs-toggle="tab" href="#rekap" role="tab" aria-selected="false">
                                DATA REKAPITULASI
                            </a>
                        </li>
                        <li class="nav-item" style="display:none;">
                            <a class="nav-link" data-bs-toggle="tab" href="#kepegawaian" role="tab" aria-selected="false">
                                KEPEGAWAIAN
                            </a>
                        </li>
                    </ul>
                    <div class="tab-content  text-muted">
                        <div class="tab-pane active" id="grafik" role="tabpanel">
                            
                        <div class="row">
                            <div class="col-12">
                                <!-- <h5 class="card-title mb-0 text-center">SUMMARY DATA ASET</h5> <br/> -->
                                <div class="row">

                                    @foreach ($statistik as $rows)
                                        <div class="row">
                                            <div class="col-xl-12">
                                                <div class="card crm-widget">
                                                    <div class="card-body p-0">

                                                        <div class="row">
                                                            <div class="col-xl-12">
                                                                <!-- <h5 class="card-title mb-0 text-start">{{ strtoupper($rows->kategori) }}</h5> -->

                                                                <div class="alert alert-secondary alert-dismissible alert-label-icon rounded-label fade show" role="alert" style="padding-left:20px !important;">
                                                                    <div class="row">
                                                                        <div class="col-xl-10">
                                                                            <!-- <i class="ri-chat-check-fill label-icon"></i> -->
                                                                            <strong>{{ strtoupper($rows->kategori) }}</strong>    
                                                                        </div>
                                                                        <div class="col-xl-2 text-end">
                                                                            <a href="javascript:void(0);" onClick="detaildashboard('{{ $rows->tipe }}');" class="btn btn-sm btn-success bg-gradient waves-effect waves-light">DETAIL</a>
                                                                        </div>
                                                                    </div>
                                                                    
                                                                </div>

                                                            </div>
                                                        </div>
                                                        <div class="row row-cols-xxl-4 row-cols-md-4 row-cols-1 g-0">
                                                            <div class="col">
                                                                <div class="py-4 px-3">
                                                                    <h5 class="text-muted text-uppercase fs-13">Total NUP <i class="ri-briefcase-line text-success fs-18 float-end align-middle"></i></h5>
                                                                    <div class="d-flex align-items-center">
                                                                        <div class="flex-shrink-0">
                                                                            <i class="ri-stack-fill  display-6 text-muted"></i>
                                                                        </div>
                                                                        <div class="flex-grow-1 ms-3">
                                                                            <h4 class="mb-0"><span class="counter-value" data-target="{{ $rows->total }}">0</span></h4>
                                                                        </div>
                                                                    </div>
                                                                </div>
                                                            </div>
                                                            <div class="col">
                                                                <div class="py-4 px-3">
                                                                    <h5 class="text-muted text-uppercase fs-13">Total Nilai Perolehan <i class="ri-book-mark-fill text-success fs-18 float-end align-middle"></i></h5>
                                                                    <div class="d-flex align-items-center">
                                                                        <div class="flex-shrink-0">
                                                                            <i class="bx bx-money display-6 text-muted"></i>
                                                                        </div>
                                                                        <div class="flex-grow-1 ms-3">
                                                                            <h4 class="mb-0"><span class="counter-value" data-target="{{ $rows->nilai_perolehan }}">0</span></h4>
                                                                        </div>
                                                                    </div>
                                                                </div>
                                                            </div>
                                                            @if ($rows->total_luas != -1)
                                                                <div class="col">
                                                                    <div class="py-4 px-3">
                                                                        <h5 class="text-muted text-uppercase fs-13">Total Luas Keseluruhan <i class="ri-home-3-line text-success fs-18 float-end align-middle"></i></h5>
                                                                        <div class="d-flex align-items-center">
                                                                            <div class="flex-shrink-0">
                                                                                <i class="ri-building-line display-6 text-muted"></i>
                                                                            </div>
                                                                            <div class="flex-grow-1 ms-3">
                                                                                <h4 class="mb-0"><span class="counter-value" data-target="{{ $rows->total_luas }}">0</span></h4>
                                                                            </div>
                                                                        </div>
                                                                    </div>
                                                                </div>
                                                            @endif
                                                            <div class="col">
                                                                <div class="py-4 px-3">
                                                                    <h5 class="text-muted text-uppercase fs-13">Kondisi Aset <i class="ri-pie-chart-line text-info fs-18 float-end align-middle"></i></h5>
                                                                    <div class="d-flex align-items-center">
                                                                        <div class="flex-shrink-0 kondisi-chart-container">
                                                                            <div id="kondisi_chart_{{ $rows->tipe }}" style="width: 80px; height: 80px;"></div>
                                                                        </div>
                                                                        <div class="flex-grow-1 ms-3">
                                                                            <div class="d-flex flex-column kondisi-stats">
                                                                                <small class="text-success mb-1">Baik: <span id="baik_{{ $rows->tipe }}">0</span>%</small>
                                                                                <small class="text-warning mb-1">Rusak Ringan: <span id="ringan_{{ $rows->tipe }}">0</span>%</small>
                                                                                <small class="text-danger">Rusak Berat: <span id="berat_{{ $rows->tipe }}">0</span>%</small>
                                                                            </div>
                                                                        </div>
                                                                    </div>
                                                                </div>
                                                            </div>
                                                        </div>

                                                    </div>
                                                </div>
                                            </div>
                                        </div>
                                    @endforeach

                                    {{--
                                    @foreach ($statistik as $rows)
                                    <div class="col-xl-3 col-md-3">
                                        <div class="card card-animate overflow-hidden">
                                            <div class="position-absolute start-0" style="z-index: 0;">
                                                <svg version="1.2" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 200 120" width="200" height="120">
                                                    <style>
                                                        .s0 {
                                                            opacity: .05;
                                                            fill: var(--vz-success)
                                                        }
                                                    </style>
                                                    <path id="Shape 8" class="s0" d="m189.5-25.8c0 0 20.1 46.2-26.7 71.4 0 0-60 15.4-62.3 65.3-2.2 49.8-50.6 59.3-57.8 61.5-7.2 2.3-60.8 0-60.8 0l-11.9-199.4z" />
                                                </svg>
                                            </div>
                                            <div class="card-body">
                                            <a href="javascript:void(0);" onClick="detaildashboard('{{ $rows->tipe }}');">
                                                <div class="d-flex align-items-center">
                                                    <div class="flex-grow-1 overflow-hidden">
                                                        <p class="text-uppercase fw-medium text-muted text-truncate mb-3"> {{ $rows->kategori }} </p>
                                                        <h4 class="fs-22 fw-semibold ff-secondary mb-0"><span class="counter-value" data-target="{{ $rows->total }}">0</span></h4>
                                                        <!-- <a href="javascript:void(0);" class="btn btn-light btn-sm">Detail</a> -->
                                                    </div>
                                                    <div class="flex-shrink-0">
                                                        <div id="total_jobs" data-colors='["--vz-success"]' class="apex-charts" dir="ltr"></div>
                                                    </div>
                                                </div>
                                            </a>
                                            </div><!-- end card body -->
                                        </div><!-- end card -->
                                    </div><!--end col-->
                                    @endforeach
                                    --}}
                                </div>
                            </div>
                            {{--
                            <div class="col-12">
                                <hr/>
                            </div>
                            <div class="col-6">
                                <br/>
                                <h5 class="card-title mb-0 text-center">JUMLAH ASET PER TAHUN</h5>
                                <br>
                                <div id="tahun_asset_charts" data-colors='["--vz-info", "--vz-info", "--vz-info", "--vz-info", "--vz-danger", "--vz-info", "--vz-info", "--vz-info", "--vz-info", "--vz-info"]' class="apex-charts" dir="ltr"></div>
                            </div>
                            <div class="col-6">
                                <br/>
                                <table class="table table-bordered table-nowrap">
                                    <tbody>
                                        @foreach ($data_tahun_asset as $rows)
                                            <tr>
                                                <td class="text-center" style="background-color:#F8F8F8;">{{ $rows->tahun }}</td>
                                                <td style="text-align:right;">{{ $rows->total }}</td>
                                            </tr>
                                        @endforeach
                                    </tbody>
                                </table>
                            </div>
                            --}}
                            <div class="col-12">
                                <hr/>
                            </div>
                            <div class="col-6">
                                <br/>    
                                <table class="table table-bordered table-nowrap">
                                    <tbody>
                                        @foreach ($data_psp_asset as $rows)
                                            <tr>
                                                <td class="text-center" style="background-color:#F8F8F8;">{{ $rows->tipe }}</td>
                                                <td style="text-align:right;">{{ $rows->total }}</td>
                                            </tr>
                                        @endforeach
                                    </tbody>
                                </table>
                            </div>
                            <div class="col-6">
                                <br/>
                                <h5 class="card-title mb-0 text-center">JUMLAH ASET BERDASARKAN STATUS PSP</h5>
                                <br>
                                <div id="psp_asset_charts" data-colors='["--vz-info", "--vz-info", "--vz-info", "--vz-info", "--vz-danger", "--vz-info", "--vz-info", "--vz-info", "--vz-info", "--vz-info"]' class="apex-charts" dir="ltr"></div>
                            </div>
                            
                        </div>

                        </div>
                       

                    </div>
                </div>
            </div>
        </div>
    </div>
    
    <div id="mapsModal" class="modal fade" tabindex="-1" data-bs-focus="false" aria-labelledby="mapsModalLabel" aria-hidden="true" style="display: none;">
        <div class="modal-dialog modal-xl">
            <div class="modal-content">
                <div class="modal-header">
                    <button type="button" class="btn-close" data-bs-dismiss="modal" aria-label="Close"> </button>
                </div>
                <div class="modal-body" id="modalencuk"></div>
            </div><!-- /.modal-content -->
        </div><!-- /.modal-dialog -->
    </div><!-- /.modal -->

     {{--
    <div class="row">
        <div class="col-8">
            <div class="card">
                <div class="card-body">
                    <!-- <h4>STATISTIK JUMLAH ASSET BMN KEJAKSAAN RI</h4>
                    <hr/> -->
                    <div class="row">
                        
                    </div>

                </div>
            </div>
        </div>
        <div class="col-4">
            <div class="card card-height-100">
                <div class="card-body">
                    

                    <div class="row">
                        <div class="col-md-12">
                            <div class="mt-3 pt-2">
                                <div class="progress progress-lg rounded-pill">
                                    <div class="progress-bar bg-primary" role="progressbar" style="width: 25%" aria-valuenow="25" aria-valuemin="0" aria-valuemax="100"></div>
                                    <div class="progress-bar bg-info" role="progressbar" style="width: 18%" aria-valuenow="18" aria-valuemin="0" aria-valuemax="100"></div>
                                    <div class="progress-bar bg-success" role="progressbar" style="width: 22%" aria-valuenow="22" aria-valuemin="0" aria-valuemax="100"></div>
                                    <div class="progress-bar bg-warning" role="progressbar" style="width: 16%" aria-valuenow="16" aria-valuemin="0" aria-valuemax="100"></div>
                                    <div class="progress-bar bg-danger" role="progressbar" style="width: 19%" aria-valuenow="19" aria-valuemin="0" aria-valuemax="100"></div>
                                </div>
                            </div><!-- end -->

                            <div class="mt-3 pt-2">
                                <div class="d-flex mb-2">
                                    <div class="flex-grow-1">
                                        <p class="text-truncate text-muted fs-14 mb-0"><i class="mdi mdi-circle align-middle text-primary me-2"></i>Pengadaan Pakaian Dinas </p>
                                    </div>
                                    <div class="flex-shrink-0">
                                        <p class="mb-0">24</p>
                                    </div>
                                </div><!-- end -->
                                <div class="d-flex mb-2">
                                    <div class="flex-grow-1">
                                        <p class="text-truncate text-muted fs-14 mb-0"><i class="mdi mdi-circle align-middle text-info me-2"></i>SDM Pengadaan </p>
                                    </div>
                                    <div class="flex-shrink-0">
                                        <p class="mb-0">1751</p>
                                    </div>
                                </div><!-- end -->
                                <div class="d-flex mb-2">
                                    <div class="flex-grow-1">
                                        <p class="text-truncate text-muted fs-14 mb-0"><i class="mdi mdi-circle align-middle text-success me-2"></i>Rencana Pengadaan Langsung </p>
                                    </div>
                                    <div class="flex-shrink-0">
                                        <p class="mb-0">239</p>
                                    </div>
                                </div><!-- end -->
                                <div class="d-flex mb-2">
                                    <div class="flex-grow-1">
                                        <p class="text-truncate text-muted fs-14 mb-0"><i class="mdi mdi-circle align-middle text-warning me-2"></i>Aset TIK </p>
                                    </div>
                                    <div class="flex-shrink-0">
                                        <p class="mb-0">122</p>
                                    </div>
                                </div><!-- end -->
                                <div class="d-flex">
                                    <div class="flex-grow-1">
                                        <p class="text-truncate text-muted fs-14 mb-0"><i class="mdi mdi-circle align-middle text-danger me-2"></i>Langganan Jasa IT </p>
                                    </div>
                                    <div class="flex-shrink-0">
                                        <p class="mb-0">178</p>
                                    </div>
                                </div><!-- end -->
                            </div><!-- end -->
                        </div><!-- end -->
                    </div><!-- end -->

                </div>
            </div>
        </div>
    </div>
    --}}                                                   
@endsection

@section('js')
<script>
    function getChartColorsArray(e) {
        if (null !== document.getElementById(e)) {
            var o = document.getElementById(e).getAttribute("data-colors");
            if (o) return (o = JSON.parse(o)).map(function(e) {
                var o = e.replace(" ", "");
                return -1 === o.indexOf(",") ? getComputedStyle(document.documentElement).getPropertyValue(o) || o : 2 == (e = e.split(",")).length ? "rgba(" + getComputedStyle(document.documentElement).getPropertyValue(e[0]) + "," + e[1] + ")" : o
            });
            console.warn("data-colors atributes not found on", e)
        }
    }

    $(document).ready(function(){
        $('#level_laporan').trigger('change');
    });

    $('#level_laporan').on('change', function(){
        var valnya = $(this).val();
        if(valnya == 'K/L'){
            $('.wilayah').hide();
            $('.satker').hide();
        }else if(valnya == 'WILAYAH'){
            $('.wilayah').show();
            $('.satker').hide();
        }else if(valnya == 'SATKER'){
            $('.wilayah').hide();
            $('.satker').show();
        }
    });

    function detaildashboard(tipe){
        $.LoadingOverlay("show");
        $('#modalencuk').html('');
        $('#mapsModal').modal('show');
        $.ajax({
            type: "POST",
            url: `{{ url('/dashboarddetail') }}`,
            data: {
                'tipe' : tipe,
                'level_laporan':$('#level_laporan').val(),
                'wilayah':$('#wilayah').val(),
                'satker':$('#satker').val(),
            },
            success: function(resp){
                $('#modalencuk').html(resp);
                $.LoadingOverlay("hide", true);
            }
        });

        /*
        $('#mapsModal').on('shown.bs.modal', function (e) {
            e.preventDefault();
            $.LoadingOverlay("show");
            $('#modalencuk').html('');
        });
        
        $(this).off('shown.bs.modal');
        */
    }

    /*
    var options = {
        series: [{
            data: @json($totalAsset),
            name: "Sessions"
        }],
        chart: {
            type: "bar",
            height: 436,
            toolbar: {
                show: !1
            }
        },
        plotOptions: {
            bar: {
                borderRadius: 4,
                horizontal: !0,
                distributed: !0,
                dataLabels: {
                    position: "top"
                }
            }
        },
        //colors: barchartCountriesColors,
        dataLabels: {
            enabled: !0,
            offsetX: 32,
            style: {
                fontSize: "12px",
                fontWeight: 400,
                colors: ["#adb5bd"]
            }
        },
        legend: {
            show: !1
        },
        grid: {
            show: !1
        },
        xaxis: {
            categories: @json($tahun)
        }
    };
    */
    var options = {
          series: [{
          name: 'Tahun',
          data: @json($totalAsset)
        }],
          annotations: {
          points: [{
            x: 'Bananas',
            seriesIndex: 0,
            label: {
              borderColor: '#775DD0',
              offsetY: 0,
              style: {
                color: '#fff',
                background: '#775DD0',
              },
              text: 'Bananas are good',
            }
          }]
        },
        chart: {
          height: 350,
          type: 'bar',
        },
        plotOptions: {
          bar: {
            borderRadius: 10,
            columnWidth: '50%',
          }
        },
        dataLabels: {
          enabled: false
        },
        stroke: {
          width: 0
        },
        grid: {
          row: {
            colors: ['#fff', '#f2f2f2']
          }
        },
        xaxis: {
            labels: {
                rotate: -45
            },
          categories: @json($tahun),
          //tickPlacement: 'on'
        },
        yaxis: {
          title: {
            text: 'Total Aset',
          },
        },
        fill: {
          type: 'gradient',
          gradient: {
            shade: 'light',
            type: "horizontal",
            shadeIntensity: 0.25,
            gradientToColors: undefined,
            inverseColors: true,
            opacityFrom: 0.85,
            opacityTo: 0.85,
            stops: [50, 0, 100]
          },
        }
        };

    (chart = new ApexCharts(document.querySelector("#tahun_asset_charts"), options)).render();

    var options_psp = {
          series: @json($totalpsp),
          chart: {
          width: 380,
          height: 350,
          type: 'pie',
        },
        labels: @json($psp),
        responsive: [{
          breakpoint: 480,
          options: {
            chart: {
              width: 200,
              height: 350,
            },
            legend: {
              position: 'bottom'
            }
          }
        }]
        };

        ( chart_psp = new ApexCharts(document.querySelector("#psp_asset_charts"), options_psp)).render();

    // Render kondisi charts for each asset type
    @foreach ($statistik as $rows)
        @php
            $kondisiData = [];
            $baik = 0;
            $ringan = 0;
            $berat = 0;
            $total = 0;
            
            switch($rows->tipe) {
                case 'lain':
                    $kondisiData = $kondisi_aset_lainnya ?? [];
                    break;
                case 'renovasi':
                    $kondisiData = $kondisi_renovasi ?? [];
                    break;
                case 'konstruksi':
                    $kondisiData = $kondisi_konstruksi ?? [];
                    break;
                case 'jalan_jembatan':
                    $kondisiData = $kondisi_jalan_jembatan ?? [];
                    break;
                case 'bangunan_air':
                    $kondisiData = $kondisi_bangunan_air ?? [];
                    break;
                case 'rumah':
                    $kondisiData = $kondisi_rumah ?? [];
                    break;
                case 'jaringan':
                    $kondisiData = $kondisi_instalasi_jaringan ?? [];
                    break;
                case 'wujud':
                    $kondisiData = $kondisi_tak_berwujud ?? [];
                    break;
                case 'tik':
                    $kondisiData = $kondisi_tik ?? [];
                    break;
                case 'non_tik':
                    $kondisiData = $kondisi_nontik ?? [];
                    break;
                case 'angkutan':
                    $kondisiData = $kondisi_kendaraan ?? [];
                    break;
                case 'alat_besar':
                    $kondisiData = $kondisi_alat_berat ?? [];
                    break;
                case 'gedung':
                    $kondisiData = $kondisi_gedung ?? [];
                    break;
                case 'tanah':
                    $kondisiData = $kondisi_tanah ?? [];
                    break;
            }
            
            $kondisiLabels = [];
            foreach($kondisiData as $kondisi) {
                $kondisiLabels[] = $kondisi->judul;
                if(strpos(strtolower($kondisi->judul), 'baik') !== false) {
                    $baik = $kondisi->total;
                } elseif(strpos(strtolower($kondisi->judul), 'ringan') !== false) {
                    $ringan = $kondisi->total;
                } elseif(strpos(strtolower($kondisi->judul), 'berat') !== false) {
                    $berat = $kondisi->total;
                }
                $total += $kondisi->total;
            }
            
            // Calculate percentages
            $baik_persen = $total > 0 ? round($baik / $total * 100, 1) : 0;
            $ringan_persen = $total > 0 ? round($ringan / $total * 100, 1) : 0;
            $berat_persen = $total > 0 ? round($berat / $total * 100, 1) : 0;
            
            // Prepare series data for pie chart
            $kondisiSeries = [];
            if(count($kondisiData) > 0) {
                // Reorder data to match color order: Baik (green), Rusak Ringan (yellow), Rusak Berat (red)
                $baikData = 0;
                $ringanData = 0;
                $beratData = 0;
                
                foreach($kondisiData as $kondisi) {
                    if(strpos(strtolower($kondisi->judul), 'baik') !== false) {
                        $baikData = $kondisi->total;
                    } elseif(strpos(strtolower($kondisi->judul), 'ringan') !== false) {
                        $ringanData = $kondisi->total;
                    } elseif(strpos(strtolower($kondisi->judul), 'berat') !== false) {
                        $beratData = $kondisi->total;
                    }
                }
                
                $kondisiSeries = [$baikData, $ringanData, $beratData];
                $kondisiLabels = ['Baik', 'Rusak Ringan', 'Rusak Berat'];
            }
        @endphp
        
        // Update kondisi counts
        $('#baik_{{ $rows->tipe }}').text('{{ $baik_persen }}');
        $('#ringan_{{ $rows->tipe }}').text('{{ $ringan_persen }}');
        $('#berat_{{ $rows->tipe }}').text('{{ $berat_persen }}');
        
        // Create kondisi chart
        @if(count($kondisiSeries) > 0 && array_sum($kondisiSeries) > 0)
        var options_kondisi_{{ $rows->tipe }} = {
            series: @json($kondisiSeries),
            chart: {
                width: 80,
                height: 80,
                type: 'pie',
                sparkline: {
                    enabled: true
                }
            },
            labels: @json($kondisiLabels),
            colors: ['#28a745', '#ffc107', '#dc3545'],
            stroke: {
                width: 0
            },
            legend: {
                show: false
            },
            dataLabels: {
                enabled: false
            },
            tooltip: {
                enabled: true,
                y: {
                    formatter: function(val) {
                        return val + " unit";
                    }
                }
            }
        };
        
        new ApexCharts(document.querySelector("#kondisi_chart_{{ $rows->tipe }}"), options_kondisi_{{ $rows->tipe }}).render();
        @else
        // Show empty state
        $('#kondisi_chart_{{ $rows->tipe }}').html('<div class="text-muted text-center" style="line-height: 80px; font-size: 12px;">No Data</div>');
        @endif
    @endforeach

    // Analisis Kebutuhan BMN Chart
    @php
        $tahunAnalisis = [];
        $jumlahAset = [];
        $nilaiPenggantian = [];
        $colors = [];
        
        foreach($analisis_kebutuhan_bmn as $analisis) {
            $tahunAnalisis[] = $analisis->tahun_prediksi;
            $jumlahAset[] = $analisis->jumlah_aset;
            $nilaiPenggantian[] = round($analisis->total_nilai_penggantian / 1000000000, 1); // Convert to billions
            
            // Set colors based on year
            if($analisis->tahun_prediksi == '2025') {
                $colors[] = '#dc3545'; // Red for urgent
            } elseif($analisis->tahun_prediksi == '2026') {
                $colors[] = '#ffc107'; // Yellow for warning
            } else {
                $colors[] = '#17a2b8'; // Blue for info
            }
        }
    @endphp
    
    var optionsAnalisisKebutuhan = {
        series: [{
            name: 'Jumlah Aset',
            data: @json($jumlahAset)
        }, {
            name: 'Nilai Penggantian (Miliar)',
            data: @json($nilaiPenggantian)
        }],
        chart: {
            type: 'bar',
            height: 300,
            stacked: false,
        },
        plotOptions: {
            bar: {
                horizontal: false,
                columnWidth: '55%',
                endingShape: 'rounded'
            },
        },
        dataLabels: {
            enabled: false
        },
        stroke: {
            show: true,
            width: 2,
            colors: ['transparent']
        },
        xaxis: {
            categories: @json($tahunAnalisis),
            labels: {
                style: {
                    fontSize: '12px',
                },
                rotate: -20,
                trim: true,
                hideOverlappingLabels: false,
                showDuplicates: false,
                minHeight: 20,
                maxHeight: 40
            },
        },
        yaxis: [{
            title: {
                text: 'Jumlah Aset',
            },
        }, {
            opposite: true,
            title: {
                text: 'Nilai Penggantian (Miliar Rp)',
            },
        }],
        colors: @json($colors),
        tooltip: {
            y: [{
                formatter: function (val) {
                    return val + " unit"
                }
            }, {
                formatter: function (val) {
                    return "Rp " + val + " Miliar"
                }
            }]
        },
        legend: {
            position: 'top',
            horizontalAlign: 'left',
        }
    };

    var chartAnalisisKebutuhan = new ApexCharts(document.querySelector("#analisis_kebutuhan_chart"), optionsAnalisisKebutuhan);
    chartAnalisisKebutuhan.render();

    // var linechartcustomerColors = getChartColorsArray("customer_impression_charts");
    // var options_bawah = {
	// 	series: [ {
	// 		name: "Aktif",
	// 		type: "area",
	// 		data: [34, 65, 46, 68, 49, 61, 42, 44, 78, 52, 63, 67]
	// 	}, {
	// 		name: "Tidak Aktif",
	// 		type: "bar",
	// 		data: [89.25, 98.58, 68.74, 108.87, 77.54, 84.03, 51.24, 28.57, 92.57, 42.36, 88.51, 36.57]
	// 	}, {
	// 		name: "Bermasalah",
	// 		type: "line",
	// 		data: [8, 12, 7, 17, 21, 11, 5, 9, 7, 29, 12, 35]
	// 	}],
	// 	chart: {
	// 		height: 370,
	// 		type: "line",
	// 		toolbar: {
	// 			show: !1
	// 		}
	// 	},
	// 	stroke: {
	// 		curve: "straight",
	// 		dashArray: [0, 0, 8],
	// 		width: [2, 0, 2.2]
	// 	},
	// 	fill: {
	// 		opacity: [.1, .9, 1]
	// 	},
	// 	markers: {
	// 		size: [0, 0, 0],
	// 		strokeWidth: 2,
	// 		hover: {
	// 			size: 4
	// 		}
	// 	},
	// 	xaxis: {
	// 		categories: ["Kejati DKI Jakarta", "Kejati Jawa Barat", "Kejati Jawa Timur", "Kejati Banten", "Kejati Bali", "Kejati NTB", "Kejati NTT", "Kejati Lampung", "Kejati Sulsel", "Kejati Sulut", "Kejati Sulteng", "Kejati Sultra"],
	// 		axisTicks: {
	// 			show: !1
	// 		},
	// 		axisBorder: {
	// 			show: !1
	// 		}
	// 	},
	// 	grid: {
	// 		show: !0,
	// 		xaxis: {
	// 			lines: {
	// 				show: !0
	// 			}
	// 		},
	// 		yaxis: {
	// 			lines: {
	// 				show: !1
	// 			}
	// 		},
	// 		padding: {
	// 			top: 0,
	// 			right: -2,
	// 			bottom: 15,
	// 			left: 10
	// 		}
	// 	},
	// 	legend: {
	// 		show: !0,
	// 		horizontalAlign: "center",
	// 		offsetX: 0,
	// 		offsetY: -5,
	// 		markers: {
	// 			width: 9,
	// 			height: 9,
	// 			radius: 6
	// 		},
	// 		itemMargin: {
	// 			horizontal: 10,
	// 			vertical: 0
	// 		}
	// 	},
	// 	plotOptions: {
	// 		bar: {
	// 			columnWidth: "30%",
	// 			barHeight: "70%"
	// 		}
	// 	},
	// 	colors: linechartcustomerColors,
	// 	tooltip: {
	// 		shared: !0,
	// 		y: [{
	// 			formatter: function(e) {
	// 				return void 0 !== e ? e.toFixed(0) : e
	// 			}
	// 		}, {
	// 			formatter: function(e) {
	// 				return void 0 !== e ? "$" + e.toFixed(2) : e
	// 			}
	// 		}, {
	// 			formatter: function(e) {
	// 				return void 0 !== e ? e.toFixed(0) : e
	// 			}
	// 		}]
	// 	}
	// };
    // (chart = new ApexCharts(document.querySelector("#customer_impression_charts"), options_bawah)).render();


    // const tableId = `tb-pegawai`;
    // const dt = $('#tb-pegawai').DataTable({
    //     serverSide: true,
    //     processing: true,
    //     deferRender: true,
    //     ordering: false,
    //     dom: dtLayout,
    //     language: {
    //         url: `{{ url('/assets/js/datatable_bahasa.json') }}`,
    //     },
    //     ajax: {
    //         url: "{{ 'dashboard/gridDataPegawaiDashboard' }}",
    //         dataSrc: 'data',
    //     },
    //     columns: [{
    //             data: 'inst_nama'
    //         },
    //         {
    //             data: 'jml_laki'
    //         },
    //         {
    //             data: 'jml_perempuan',
    //         },
    //         {
    //             data: 'jml_jaksa',
    //         },
    //         {
    //             data: 'jml_tu',
    //         },
    //         {
    //             data: 'total',
    //         },
    //     ],
    //     columnDefs: [
    //         {
    //             targets: [1,2,3,4,5],
    //             className: 'dt-body-right'
    //         }
    //     ]
    // });
</script>
@endsection
