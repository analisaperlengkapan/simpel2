@extends('layout.main')
@section('content')
@include('components.breadcums', $breadcums)
    <div class="row">
        <div class="col-lg-12">
            <div class="card">
                <div class="card-header">
                    <div class="d-flex align-items-center">
                        <div class="flex-grow-1">
                            <h5 class="card-title mb-0">Konfirmasi Penerimaan</h5>
                        </div>
                    </div>
                </div>
                <div class="card-body">
                    <form action="/pengadaan/distribusi/konfirmasi-penerimaan" method="POST" class="ajaxForm" enctype="multipart/form-data">
                        @csrf
                        @if (!$isNew)
                            <input type="hidden" id="id" name="id" value="{{ $model['id'] }}">
                        @endif
                        <div class="row">
                            <div class="col-lg-6">
                                <div class="mb-3">
                                    <label for="nip" class="form-label">Nomor Kontrak *</label>
                                    <input type="hidden" id="isNew" name="isNew" value="{{ $isNew }}">
                                    <input type="text" class="form-control" id="no_kontrak" name="no_kontrak" value="{{ $model['no_kontrak'] ?? '' }}" readonly>
                                </div>
                            </div>
                            <div class=" col-lg-6">
                                <div class="mb-3">
                                    <label for="name" class="form-label">Tanggal Kontrak *</label>
                                    <input type="date" class="form-control" id="tgl_kontrak" name="tgl_kontrak" value="{{ $model['tgl_kontrak'] ?? '' }}" readonly>
                                </div>
                            </div>
                        </div>
                        <div class="row">
                            <div class="col-lg-4">
                                <div class="mb-3">
                                    <label for="nip" class="form-label">Nama Barang *</label>
                                    <input class="form-control" id="nm_barang" name="nm_barang" value="{{ $model['nm_barang'] ?? '' }}" readonly>
                                </div>
                            </div>
                            <div class=" col-lg-4">
                                <div class="mb-3">
                                    <label for="name" class="form-label">Jumlah Barang *</label>
                                    <input type="text" class="form-control" id="jml_barang" name="jml_barang" value="{{ $model['jml_barang'] ?? '' }}" readonly>
                                </div>
                            </div>
                            <div class=" col-lg-4">
                                <div class="mb-3">
                                    <label for="name" class="form-label">Nilai Barang *</label>
                                    <input type="number" class="form-control" id="nilai_barang" name="nilai_barang" value="{{ $model['nilai_barang'] ?? '' }}" readonly>
                                </div>
                            </div>
                        </div>
                        <div class="row">
                            <div class="col-lg-5">
                                <div class="mb-3">
                                    <label for="kdsatker_keu" class="form-label">Satker Tujuan *</label>
                                    <select class="form-control" data-choices data-choices-text-disabled-true name="kdsatker_tujuan" id="kdsatker_tujuan">
                                        <option value="">Pilih Satker</option>
                                        {!! $satkerOptions !!}
                                    </select>
                                </div>
                            </div>
                        </div>
                        <div class="row" >
                            <div class="col-lg-3">
                                <div class="mb-3">
                                    <label for="nip" class="form-label">File SPK *</label>
                                    <br/>
                                    @if(isset($model['file_spk']))
                                    <a href="{{ url($model['file_spk']) }}" download terget="_blank">
                                        <i class="ri-download-cloud-line"></i> Download File
                                    </a>
                                    @endif
                                </div>
                            </div>
                            <div class="col-lg-3">
                                <div class="mb-3">
                                    <label for="name" class="form-label">File BAST *</label>
                                    <input type="file" class="form-control" id="file_bast" name="file_bast" value="{{ $model['file_bast'] ?? '' }}">
                                </div>
                            </div>
                            <div class=" col-lg-3">
                                <div class="mb-3">
                                    <label for="name" class="form-label">File Foto *</label>
                                    <input type="file" class="form-control" id="file_foto" name="file_foto" value="{{ $model['file_foto'] ?? '' }}">
                                </div>
                            </div>
                        </div>
                        <div class="row">
                            <div class="col-lg-4">
                                <div class="mb-3">
                                    <label for="kdsatker_keu" class="form-label">Apakah barang masuk gudang terlebih dahulu?</label>
                                    <input type="text" value="{{ $model['is_gudang']==1? 'Ya':'Tidak' }}" class="form-control" readonly>
                                    <input type="hidden" name="is_gudang" id="is_gudang" value="{{ $model['is_gudang'] }}" class="form-control">
                                </div>
                            </div>
                        </div>
                        <hr/>
                        <div class="row">
                            <div class="col-lg-12">
                                <div class="hstack gap-2">
                                    <a href="{{ url('pengadaan/distribusi/konfirmasi-penerimaan') }}" class="btn btn-outline-primary">Kembali</a>
                                    @if(!$readOnly)
                                    <button type="submit" class="btn btn-primary">Simpan</button>
                                    @endif
                                </div>
                            </div>
                        </div>
                    </form>
                </div>
            </div>
        </div>
    </div>
@endsection

@section('js')
    <script>
        $(function() {
            var isReadOnly = '{{ $readOnly }}';
            if(isReadOnly){
                $('input, select').prop('readonly', true);
            }
        })
    </script>
@endsection
