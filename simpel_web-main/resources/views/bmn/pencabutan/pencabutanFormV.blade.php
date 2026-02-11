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
                    <form action="/bmn/pencabutan/pencabutan" method="POST" class="ajaxForm" enctype="multipart/form-data">
                        @csrf
                        @if (!$isNew)
                            <input type="hidden" id="id" name="id" value="{{ $model['id'] }}">
                        @endif
                        <div class="row">
                            <div class="col-lg-6">
                                <div class="mb-3">
                                    <label for="kdsatker_keu" class="form-label">Kode Satker *</label>
                                    <select class="form-control" data-choices data-choices-sorting-false name="kdsatker_keu" id="kdsatker_keu">
                                        <option value="">Pilih Satker</option>
                                        {!! $satkerOptions !!}
                                    </select>
                                </div>
                            </div>
                        </div>
                        <div class="row">
                            <div class="col-lg-2">
                                <div class="mb-3">
                                    <label for="kdsatker_keu" class="form-label">Jenis *</label>
                                    <select class="form-control" data-choices data-choices-sorting-false name="jenis_sk" id="jenis_sk">
                                        <option value="">Pilih Jenis</option>
                                        {!! $jenisOptions !!}
                                    </select>
                                </div>
                            </div>
                            <div class="col-lg-4">
                                <div class="mb-3">
                                    <label for="nip" class="form-label">Nomor Surat *</label>
                                    <input type="hidden" id="isNew" name="isNew" value="{{ $isNew }}">
                                    <input type="text" class="form-control" id="no_surat" name="no_surat" value="{{ $model['no_surat'] ?? '' }}">
                                </div>
                            </div>
                            <div class=" col-lg-2">
                                <div class="mb-3">
                                    <label for="name" class="form-label">Tanggal *</label>
                                    <input type="date" class="form-control" id="tgl_surat" name="tgl_surat" value="{{ $model['tgl_surat'] ?? '' }}">
                                </div>
                            </div>
                            <div class="col-lg-4">
                                <div class="mb-3">
                                    <label for="nip" class="form-label">Dikeluarkan Di</label>
                                    <input class="form-control" id="dikeluarkan_di" name="dikeluarkan_di" value="{{ $model['dikeluarkan_di'] ?? '' }}">
                                </div>
                            </div>
                        </div>
                        <div class="row" >
                            <div class="col-lg-3">
                                <div class="mb-3">
                                    <label for="nip" class="form-label">File *</label>
                                    <input type="file" class="form-control" id="file_sk" name="file_sk" value="{{ $model['file_sk'] ?? '' }}">
                                    @if(isset($model['file_sk']))
                                    <a href="{{ url($model['file_sk']) }}" download terget="_blank">
                                        <i class="ri-download-cloud-line"></i> Download File
                                    </a>
                                    @endif
                                </div>
                            </div>
                        </div>
                        
                        <hr/>
                        <div class="row">
                            <div class="col-lg-12">
                                <div class="hstack gap-2">
                                    <a href="{{ url('bmn/pencabutan/pencabutan') }}" class="btn btn-outline-primary">Kembali</a>
                                    @if(!$readOnly)
                                    <button type="submit" class="btn btn-primary">
                                        {{ $isNew ? 'Simpan' : 'Ubah' }}
                                    </button>
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
