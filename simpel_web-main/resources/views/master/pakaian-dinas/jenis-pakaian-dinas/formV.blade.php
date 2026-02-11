@extends('layout.main')
@section('content')
@include('components.breadcums', $breadcums)
    <div class="row">
        <div class="col-lg-12">
            <div class="card">
                <div class="card-header">
                    <div class="d-flex align-items-center">
                        <div class="flex-grow-1">
                            <h5 class="card-title mb-0">{{ $judul }} Jenis Pakaian Dinas</h5>
                        </div>
                    </div>
                </div>
                <div class="card-body">
                    <form action="/master/pakaian-dinas/jenis-pakaian-dinas" method="POST" class="ajaxForm">
                        @csrf
                        @if (!$isNew)
                            <input type="hidden" id="id" name="id" value="{{ $model['id'] }}">
                        @endif
                        <div class="row">
                            <div class="col-lg-9">
                                <div class="mb-3">
                                    <label for="kdsatker_keu" class="form-label">Nama *</label>
                                    <input class="form-control" id="nama" name="nama" value="{{ $model['nama'] ?? '' }}">
                                </div>
                            </div>
                        </div>
                        <div class="row">
                            <div class="col-lg-12">
                                <div class="hstack gap-2">
                                    <a href="{{ url('master/pakaian-dinas/jenis-pakaian-dinas') }}" class="btn btn-outline-primary">Kembali</a>
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
