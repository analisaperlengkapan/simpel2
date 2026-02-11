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
                                <label for="kdsatker_keu" class="form-label">No Dokumen </label>
                                <input type="text" class="form-control-plaintext" value="{{ $model['no_dokumen'] ?? '' }}" readonly/>
                            </div>
                        </div>
                        <div class="col-lg-6">
                            <div class="mb-3">
                                <label for="kdsatker_keu" class="form-label">Tgl. BAST</label>
                                <input type="text" class="form-control-plaintext" value="{{ $model['tgl_bast'] ?? '' }}" readonly/>
                            </div>
                        </div>
                    </div>
                    <div class="row">
                        <div class="col-lg-6">
                            <div class="mb-3">
                                <label for="kdsatker_keu" class="form-label">Kategori BAST </label>
                                <input type="text" class="form-control-plaintext" value="{{ $model['kategori_bast'] ?? '' }}" readonly/>
                            </div>
                        </div>
                        <div class="col-lg-6">
                            <div class="mb-3">
                                <label for="kdsatker_keu" class="form-label">Uraian BAST</label>
                                <input type="text" class="form-control-plaintext" value="{{ $model['uraian_bast'] ?? '' }}" readonly/>
                            </div>
                        </div>
                    </div>
                    <div class="row">
                        <div class="col-lg-6">
                            <div class="mb-3">
                                <label for="kdsatker_keu" class="form-label">Nilai BAST</label>
                                <input type="text" class="form-control-plaintext angka" value="{{ $model['nilai_bast'] ?? '' }}" readonly/>
                            </div>
                        </div>
                        <div class="col-lg-6">
                            <div class="mb-3">
                                <label for="kdsatker_keu" class="form-label">Nama Suplier</label>
                                <input type="text" class="form-control-plaintext" value="{{ $model['nama_suplier'] ?? '' }}" readonly/>
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
                            <h5 class="card-title mb-0">Detail</h5>
                        </div>
                    </div>
                </div>
                <div class="card-body">
                    <table id="tb-monsakti-traset" class="display table table-bordered dt-responsive" style="width:100%">
                        <thead>
                            <tr>
                                <th>Kode barang</th>
                                <th>Nama Barang</th>
                                <th>Jumlah Barang</th>
                                <th>Nilai Total Barang</th>
                                <th>Status</th>
                            </tr>
                        </thead>
                        <tbody>
                            @foreach($dataDetail as $data)
                                <tr>
                                    <td>{{$data->kode_barang}}</td>
                                    <td>{{$data->nama_barang}}</td>
                                    <td>{{$data->jml_barang}}</td>
                                    <td class="angka" style="text-align: right;">{{$data->nilai_total_barang}}</td>
                                    <td>{{$data->status_penditailan}}</td>
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
            <div class="hstack gap-2">
                <a href="{{ url($controller) }}" class="btn btn-outline-primary">Kembali</a>
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
