@extends('layout.main')
@section('content')
@include('components.breadcums', $breadcums)
    <div class="row">
        <div class="col-lg-12">
            <div class="card">
                <div class="card-header">
                    <div class="d-flex align-items-center">
                        <div class="flex-grow-1">
                            <h5 class="card-title mb-0">{{ $judul }} Panduan</h5>
                        </div>
                    </div>
                </div>
                <div class="card-body">
                    <form action="/suport/bantuan" method="POST" class="ajaxForm" enctype="multipart/form-data">
                        @csrf
                        @if (!$isNew)
                            <input type="hidden" id="id" name="id" value="{{ $model['id'] }}">
                        @endif
                        <div class="row">
                            <div class=" col-lg-6">
                                <div class="mb-3">
                                    <label for="Judul" class="form-label">Judul/Tentang *</label>
                                    <textarea class="form-control" id="judul" name="judul"  rows="3">{{ $model['judul'] ?? '' }}</textarea>
                                    <input type="hidden" id="isNew" name="isNew" value="{{ $isNew }}">
                                </div>
                            </div>
                            <div class="col-lg-3">
                                <div class="mb-3">
                                    <label for="bantuan" class="form-label">File Lampiran *</label>
                                    <input type="file" class="form-control" id="file_panduan" name="file_panduan" value="{{ $model['file_panduan'] ?? '' }}">
                                    @if(isset($model['file_panduan']))
                                    <a href="{{ url($model['file_panduan']) }}" download terget="_blank">
                                        <i class="ri-download-cloud-line"></i> Download File
                                    </a>
                                    @endif
                                </div>
                            </div>
                            
                        </div>
                        
                        <div class="row">
                            <div class="col-lg-12">
                                <div class="hstack gap-2 justify-content-end">
                                    <a href="{{ url('suport/bantuan') }}" class="btn btn-outline-primary">Kembali</a>
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
