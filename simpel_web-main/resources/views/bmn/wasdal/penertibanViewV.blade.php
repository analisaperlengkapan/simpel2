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
                                <label for="kdsatker_keu" class="form-label">Satuan Kerja </label>
                                <input type="text" class="form-control-plaintext" value="{{ $model['nama_satker'] ?? '-' }}" readonly/>
                            </div>
                        </div>
                        <div class="col-lg-6">
                            <div class="mb-3">
                                <label for="kdsatker_keu" class="form-label">Tahun Anggaran</label>
                                <input type="text" class="form-control-plaintext" value="{{ $model['tahun_anggaran'] ?? '-' }}" readonly/>
                            </div>
                        </div>
                    </div>
                    <div class="row">
                        <div class="col-lg-3">
                            <div class="mb-3">
                                <label for="kdsatker_keu" class="form-label">Kode Barang </label>
                                <input type="text" class="form-control-plaintext" value="{{ $model['kode_barang'] ?? '-' }}" readonly/>
                            </div>
                        </div>
                        <div class="col-lg-3">
                            <div class="mb-3">
                                <label for="kdsatker_keu" class="form-label">NUP </label>
                                <input type="text" class="form-control-plaintext" value="{{ $model['nup'] ?? '-' }}" readonly/>
                            </div>
                        </div>
                        <div class="col-lg-6">
                            <div class="mb-3">
                                <label for="kdsatker_keu" class="form-label">Nama Barang</label>
                                <input type="text" class="form-control-plaintext" value="{{ $model['nama_barang'] ?? '-' }}" readonly/>
                            </div>
                        </div>
                    </div>
                    <div class="row">
                        <div class="col-lg-6">
                            <div class="mb-3">
                                <label for="kdsatker_keu" class="form-label">No. Register </label>
                                <input type="text" class="form-control-plaintext" value="{{ $model['noreg'] ?? '-' }}" readonly/>
                            </div>
                        </div>
                        <div class="col-lg-6">
                            <div class="mb-3">
                                <label for="kdsatker_keu" class="form-label">Dasar Penertiban</label>
                                <input type="text" class="form-control-plaintext" value="{{ $model['dasar_penertiban'] ?? '-' }}" readonly/>
                            </div>
                        </div>
                    </div>
                    <div class="row">
                        <div class="col-lg-6">
                            <div class="mb-3">
                                <label for="kdsatker_keu" class="form-label">No. Laporan</label>
                                <input type="text" class="form-control-plaintext" value="{{ $model['no_laporan'] ?? '-' }}" readonly/>
                            </div>
                        </div>
                        <div class="col-lg-6">
                            <div class="mb-3">
                                <label for="kdsatker_keu" class="form-label">Tgl. Laporan</label>
                                <input type="text" class="form-control-plaintext" value="{{ $model['tgl_laporan'] ?? '-' }}" readonly/>
                            </div>
                        </div>
                    </div>
                    <div class="row">
                        <div class="col-lg-6">
                            <div class="mb-3">
                                <label for="kdsatker_keu" class="form-label">Bentuk Penertiban</label>
                                <textarea type="text" class="form-control-plaintext" readonly>{{ $model['bentuk_penertiban'] ?? '-' }}</textarea>
                            </div>
                        </div>
                        <div class="col-lg-3">
                            <div class="mb-3">
                                <label for="kdsatker_keu" class="form-label">No. Surat Penertiban</label>
                                <input type="text" class="form-control-plaintext" value="{{ $model['no_surat_penertiban'] ?? '-' }}" readonly/>
                            </div>
                        </div>
                        <div class="col-lg-3">
                            <div class="mb-3">
                                <label for="kdsatker_keu" class="form-label">Tgl. Surat Penertiban</label>
                                <input type="text" class="form-control-plaintext" value="{{ $model['tgl_surat_penertiban'] ?? '-' }}" readonly/>
                            </div>
                        </div>
                    </div>
                    <div class="row">
                        <div class="col-lg-6">
                            <div class="mb-3">
                                <label for="kdsatker_keu" class="form-label">Uraian Penertiban</label>
                                <input type="text" class="form-control-plaintext" value="{{ $model['uraian_penertiban'] ?? '-' }}" readonly/>
                            </div>
                        </div>
                        <div class="col-lg-6">
                            <div class="mb-3">
                                <label for="kdsatker_keu" class="form-label">Tindak Lanjut</label>
                                <input type="text" class="form-control-plaintext" value="{{ $model['tindak_lanjut'] ?? '-' }}" readonly/>
                            </div>
                        </div>
                    </div>
                    <div class="row">
                        <div class="col-lg-3">
                            <div class="mb-3">
                                <label for="kdsatker_keu" class="form-label">Tgl. Tarik Data SIMAN</label>
                                <input type="text" class="form-control-plaintext" value="{{ $model['tgl_tarik'] ?? '-' }}" readonly/>
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
                <div class="card-body">
                    <div class="hstack gap-2">
                        <a href="{{ url($controller) }}" class="btn btn-outline-primary">Kembali</a>
                    </div>
                </div>
            </div>
        </div>
    </div>
    
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
