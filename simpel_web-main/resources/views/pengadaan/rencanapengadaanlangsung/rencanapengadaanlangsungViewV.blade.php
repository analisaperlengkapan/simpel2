@extends('layout.main')
@section('content')
@include('components.breadcums', $breadcums)
    <div class="row">
        <div class="col-lg-12">
            <div class="card">
                <div class="card-header">
                    <div class="d-flex align-items-center">
                        <div class="flex-grow-1">
                            <h5 class="card-title mb-0">{{ $judul }}</h5>
                        </div>
                    </div>
                </div>
                <div class="card-body">
                    <div class="row">
                        <div class="col-lg-6">
                            <div class="mb-3">
                                <label for="kdsatker_keu" class="form-label">Satker </label>
                                <input type="text" class="form-control-plaintext" value="{{ $model['inst_nama'] ?? '' }}" readonly/>
                            </div>
                        </div>
                        <div class="col-lg-6">
                            <div class="mb-3">
                                <label for="kdsatker_keu" class="form-label">Tahun Anggaran</label>
                                <input type="text" class="form-control-plaintext" value="{{ $model['thn_ang'] ?? '' }}" readonly/>
                            </div>
                        </div>
                    </div>
                    <div class="row">
                        <div class="col-lg-6">
                            <div class="mb-3">
                                <label for="kdsatker_keu" class="form-label">No Kontrak </label>
                                <input type="text" class="form-control-plaintext" value="{{ $model['no_kontrak'] ?? '' }}" readonly/>
                            </div>
                        </div>
                        <div class="col-lg-6">
                            <div class="mb-3">
                                <label for="kdsatker_keu" class="form-label">Tgl Kontrak</label>
                                <input type="text" class="form-control-plaintext" value="{{ $model['tanggal_kontrak'] ?? '' }}" readonly/>
                            </div>
                        </div>
                    </div>
                    <div class="row">
                        <div class="col-lg-6">
                            <div class="mb-3">
                                <label for="kdsatker_keu" class="form-label">Tgl Mulai </label>
                                <input type="text" class="form-control-plaintext" value="{{ $model['tanggal_mulai_pelaksanaan'] ?? '' }}" readonly/>
                            </div>
                        </div>
                        <div class="col-lg-6">
                            <div class="mb-3">
                                <label for="kdsatker_keu" class="form-label">Tgl Selesai</label>
                                <input type="text" class="form-control-plaintext" value="{{ $model['tanggal_selesai_pelaksanaan'] ?? '' }}" readonly/>
                            </div>
                        </div>
                    </div>
                    <div class="row">
                        <div class="col-lg-6">
                            <div class="mb-3">
                                <label for="kdsatker_keu" class="form-label">Nilai Kontrak</label>
                                <input type="text" class="form-control-plaintext angka" value="{{ $model['nilai_kontrak'] ?? '' }}" readonly/>
                            </div>
                        </div>
                        <div class="col-lg-6">
                            <div class="mb-3">
                                <label for="kdsatker_keu" class="form-label">Jenis Kontrak</label>
                                <input type="text" class="form-control-plaintext" value="{{ $model['jenis_kontrak'] ?? '' }}" readonly/>
                            </div>
                        </div>
                    </div>
                    <div class="row">
                        <div class="col-lg-6">
                            <div class="mb-3">
                                <label for="kdsatker_keu" class="form-label">Uraian Kontrak</label>
                                <textarea type="text" class="form-control-plaintext" readonly>{{ $model['uraian_kontrak'] ?? '' }}</textarea>
                            </div>
                        </div>
                        <div class="col-lg-6">
                            <div class="mb-3">
                                <label for="kdsatker_keu" class="form-label">Nama Suplier</label>
                                <input type="text" class="form-control-plaintext" value="{{ $model['nama_supplier'] ?? '' }}" readonly/>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    </div>
    <div class="row">
        <div class="col-lg-12">
            <div class="card">
                <div class="card-header">
                    <div class="d-flex align-items-center">
                        <div class="flex-grow-1">
                            <h5 class="card-title mb-0">Kontrak Line</h5>
                        </div>
                    </div>
                </div>
                <div class="card-body">
                    <table id="tb-monsakti-traset" class="display table table-bordered dt-responsive" style="width:100%">
                        <thead>
                            <tr>
                                <th>Deskripsi</th>
                                <th>Cara Tarik</th>
                                <th>Tipe Line</th>
                                <th>Nilai Line</th>
                            </tr>
                        </thead>
                        <tbody>
                            @foreach($dataLine as $data)
                                <tr>
                                    <td>{{$data->deskripsi_line}}</td>
                                    <td>{{$data->cara_tarik}}</td>
                                    <td width="15%" class="text-center">{{$data->tipe_line}}</td>
                                    <td class="angka" style="text-align: right;">{{$data->nilai_line}}</td>
                                </tr>
                            @endforeach
                        </tbody>
                    </table>
                </div>
            </div>
        </div>
        <!--end col-->
    </div>
    <div class="row">
        <div class="col-lg-12">
            <div class="card">
                <div class="card-header">
                    <div class="d-flex align-items-center">
                        <div class="flex-grow-1">
                            <h5 class="card-title mb-0">Kontrak Termin</h5>
                        </div>
                    </div>
                </div>
                <div class="card-body">
                    <table id="tb-monsakti-termin" class="display table table-bordered dt-responsive" style="width:100%">
                        <thead>
                            <tr>
                                <th width="10%">Termin Ke</th>
                                <th>Deskripsi Termin</th>
                                <th width="15%">Tgl Termin</th>
                                <th>Nilai Termin</th>
                            </tr>
                        </thead>
                        <tbody>
                            @foreach($dataTermin as $data)
                                <tr>
                                    <td class="text-center">{{$data->termin_ke}}</td>
                                    <td>{{$data->deskripsi_termin}}</td>
                                    <td class="text-center">{{$data->tanggal_termin}}</td>
                                    <td class="angka" style="text-align: right;">{{$data->nilai_termin}}</td>
                                </tr>
                            @endforeach
                        </tbody>
                    </table>
                </div>
            </div>
        </div>
        <!--end col-->
    </div>
    <div class="row">
        <div class="col-lg-12">
            <div class="card">
                <div class="card-header">
                    <div class="d-flex align-items-center">
                        <div class="flex-grow-1">
                            <h5 class="card-title mb-0">BAST Kontrak</h5>
                        </div>
                    </div>
                </div>
                <div class="card-body">
                    <div class="row">
                        <div class="col-lg-6">
                            <div class="mb-3">
                                <label for="kdsatker_keu" class="form-label">No. BAST </label>
                                <input type="text" class="form-control-plaintext" value="{{ $dataBast['no_bast'] ?? '' }}" readonly/>
                            </div>
                        </div>
                        <div class="col-lg-6">
                            <div class="mb-3">
                                <label for="kdsatker_keu" class="form-label">Tgl BAST</label>
                                <input type="text" class="form-control-plaintext" value="{{ $dataBast['tgl_bast'] ?? '' }}" readonly/>
                            </div>
                        </div>
                    </div>
                    <div class="row">
                        <div class="col-lg-6">
                            <div class="mb-3">
                                <label for="kdsatker_keu" class="form-label">Kategori BAST </label>
                                <input type="text" class="form-control-plaintext" value="{{ $dataBast['no_bast'] ?? '' }}" readonly/>
                            </div>
                        </div>
                        <div class="col-lg-6">
                            <div class="mb-3">
                                <label for="kdsatker_keu" class="form-label">Nilai BAST</label>
                                <input type="text" class="form-control-plaintext angka" value="{{ $dataBast['nilai_bast'] ?? '' }}" readonly/>
                            </div>
                        </div>
                    </div>
                    <div class="row">
                        <div class="col-lg-6">
                            <div class="mb-3">
                                <label for="kdsatker_keu" class="form-label">No. Status SPP </label>
                                <input type="text" class="form-control-plaintext" value="{{ $dataBast['no_status_spp'] ?? '' }}" readonly/>
                            </div>
                        </div>
                        <div class="col-lg-6">
                            <div class="mb-3">
                                <label for="kdsatker_keu" class="form-label">Jenis SPP</label>
                                <input type="text" class="form-control-plaintext" value="{{ $dataBast['jns_spp'] ?? '' }}" readonly/>
                            </div>
                        </div>
                    </div>

                </div>
            </div>
        </div>
        <!--end col-->
    </div>

    <div class="row">
        <div class="col-lg-12">
            <div class="card">
                <div class="card-header">
                    <div class="d-flex align-items-center">
                        <div class="flex-grow-1">
                            <h5 class="card-title mb-0">BAST Kontrak Detail</h5>
                        </div>
                    </div>
                </div>
                <div class="card-body">
                    <table id="tb-monsakti-termin" class="display table table-bordered dt-responsive" style="width:100%">
                        <thead>
                            <tr>
                                <th width="15%">Kode Barang</th>
                                <th>Nama Barang</th>
                                <th width="15%">Jml. Barang</th>
                                <th>Nilai Total Barang</th>
                            </tr>
                        </thead>
                        <tbody>
                            @foreach($dataBastDetail as $data)
                                <tr>
                                    <td class="text-center">{{$data->kode_barang}}</td>
                                    <td>{{$data->nama_barang}}</td>
                                    <td class="text-center">{{$data->jml_barang}}</td>
                                    <td class="angka" style="text-align: right;">{{$data->nilai_total_barang}}</td>
                                </tr>
                            @endforeach
                        </tbody>
                    </table>
                </div>
            </div>
        </div>
        <!--end col-->
    </div>


    <div class="row">
        <div class="col-lg-12">
            <div class="card">
                <div class="card-body">
                    <div class="hstack gap-2">
                        <a href="{{ url($controller) }}" class="btn btn-outline-primary">Kembali</a>
                    </div>
                </div>
            </div>
        </div>
    </div>
    <style>
    #tb-monsakti-traset thead th,
    #tb-monsakti-termin thead th {
        background-color: #405189;
        color: #ffffff;
        text-align: center;
        text-transform: uppercase;
    }
    .dataTables_length {
        width: auto;
        float: right;
    }
</style>
@endsection

@section('js')
    <script>
        $(function() {
            $('.angka').autoNumeric('init', {
                aSep : '.',
                aDec: ',',
                mDec: '0'
            });
        })
    </script>
@endsection
