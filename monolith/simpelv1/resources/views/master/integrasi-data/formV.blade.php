@extends('layout.main')
@section('content')
@include('components.breadcums', $breadcums)
    <div class="row">
        <div class="col-lg-12">
            <div class="card">
                <div class="card-header">
                    <div class="d-flex align-items-center">
                        <div class="flex-grow-1">
                            <h5 class="card-title mb-0">{{ $judul }} Integrasi Data</h5>
                        </div>
                    </div>
                </div>
                <div class="card-body">
                    <form action="/pengaturan/integrasi-data" method="POST" class="ajaxForm">
                        @csrf
                        @if (!$isNew)
                            <input type="hidden" id="id" name="id" value="{{ $model['id'] }}">
                        @endif
                        <div class="row">
                            <div class="col-lg-9">
                                <div class="mb-3">
                                    <label for="kdsatker_keu" class="form-label">Host *</label>
                                    <input class="form-control" id="host" name="host" value="{{ $model['host'] ?? '' }}">
                                </div>
                            </div>
                        </div>
                        <div class="row">
                            <div class="col-lg-9">
                                <div class="mb-3">
                                    <label for="kdsatker_keu" class="form-label">Username *</label>
                                    <input class="form-control" id="username" name="username" value="{{ $model['username'] ?? '' }}">
                                </div>
                            </div>
                        </div>
                        <div class="row">
                            <div class="col-lg-9">
                                <div class="mb-3">
                                    <label for="kdsatker_keu" class="form-label">Password *</label>
                                    <input class="form-control" id="password" name="password" value="{{ $model['password'] ?? '' }}">
                                </div>
                            </div>
                        </div>
                        <div class="row">
                            <div class="col-lg-9">
                                <div class="mb-3">
                                    <label for="kdsatker_keu" class="form-label">Nama Aplikasi *</label>
                                    <input class="form-control" id="nama_aplikasi" name="nama_aplikasi" value="{{ $model['nama_aplikasi'] ?? '' }}">
                                </div>
                            </div>
                        </div>
                        <div class="row">
                            <div class="col-lg-12">
                                <div class="hstack gap-2">
                                    <a href="{{ url('pengaturan/integrasi-data') }}" class="btn btn-outline-primary">Kembali</a>
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
        })
    </script>
@endsection
