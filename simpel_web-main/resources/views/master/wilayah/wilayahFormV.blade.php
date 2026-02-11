@extends('layout.main')
@section('content')
    <div class="row">
        <div class="col-12">
            <div class="page-title-box d-sm-flex align-items-center justify-content-between">
                <div class="page-title-right">
                    <ol class="breadcrumb m-0">
                        <li class="breadcrumb-item"><a href="javascript: void(0);">Master</a></li>
                        <li class="breadcrumb-item">Wilayah</li>
                        <li class="breadcrumb-item active">Tambah</li>
                    </ol>
                </div>
            </div>
        </div>
        <div class="col-xxl-9">
            <div class="card mt-xxl-n5">
                <div class="card-header">
                    <div class="d-flex align-items-center">
                        <div class="flex-grow-1">
                            <h5 class="card-title mb-0">Tambah Master Wilayah</h5>
                        </div>
                    </div>
                </div>
                <div class="card-body">
                    <form action="/master/wilayah" method="POST" class="ajaxForm">
                        @csrf
                        @if (!$isNew)
                            <input type="hidden" id="id" name="id" value="{{ $model['id'] }}">
                        @endif
                        <div class="row">
                        <div class="col-lg-3">
                                <div class="mb-3">
                                    <label for="nip" class="form-label">Kode Induk Wilayah *</label>
                                    <input class="form-control" id="inst_satkerinduk" name="inst_satkerinduk" required value="{{ $model['inst_satkerinduk'] ?? '' }}">
                                </div>
                            </div>
                            <div class="col-lg-3">
                                <div class="mb-3">
                                    <label for="nip" class="form-label">Kode Wilayah *</label>
                                    <input class="form-control" id="inst_satkerkd" name="inst_satkerkd" required value="{{ $model['inst_satkerkd'] ?? '' }}">
                                </div>
                            </div>
                            <div class=" col-lg-6">
                                <div class="mb-3">
                                    <label for="name" class="form-label">Nama Wilayah *</label>
                                    <input type="text" class="form-control" id="inst_nama" name="inst_nama" required value="{{ $model['inst_nama'] ?? '' }}">
                                </div>
                            </div>
                        </div>
                        <div class="row">
                            
                            <div class=" col-lg-6">
                                <div class="mb-3">
                                    <label for="name" class="form-label">Nama Akronim *</label>
                                    <input type="text" class="form-control" id="inst_akronim" name="inst_akronim" required value="{{ $model['inst_akronim'] ?? '' }}">
                                </div>
                            </div>
                            <div class=" col-lg-6">
                                <div class="mb-3">
                                    <label for="name" class="form-label">Nama Kepala </label>
                                    <input type="text" class="form-control" id="inst_kepala" name="inst_kepala" required value="{{ $model['inst_kepala'] ?? '' }}">
                                </div>
                            </div>
                        </div>
                        <div class="row">
                            <div class="col-lg-12">
                                <div class="mb-3">
                                    <label for="nip" class="form-label">Alamat</label>
                                    <input type="text" class="form-control" id="inst_alamat" name="inst_alamat" value="{{ $model['inst_alamat'] ?? '' }}">
                                </div>
                            </div>
                            
                        </div>
                        <div class="row">
                            <div class=" col-lg-3">
                                <div class="mb-3">
                                    <label for="name" class="form-label">Telepon</label>
                                    <input type="text" class="form-control" id="inst_telepon" name="inst_telepon" value="{{ $model['inst_telepon'] ?? '' }}">
                                </div>
                            </div>
                            <div class=" col-lg-3">
                                <div class="mb-3">
                                    <label for="name" class="form-label">Fax</label>
                                    <input type="text" class="form-control" id="inst_fax" name="inst_fax" value="{{ $model['inst_fax'] ?? '' }}">
                                </div>
                            </div>
                            <div class=" col-lg-3">
                                <div class="mb-3">
                                    <label for="name" class="form-label">Jenis *</label>
                                    <input type="text" class="form-control" id="inst_jenis" name="inst_jenis" value="{{ $model['inst_jenis'] ?? '' }}">
                                </div>
                            </div>
                            
                            <div class=" col-lg-3">
                                <div class="mb-3">
                                    <label for="name" class="form-label">Level</label>
                                    <input type="number" class="form-control" id="inst_level" name="inst_level" value="{{ $model['inst_level'] ?? '' }}">
                                </div>
                            </div>
                        </div>
                        <div class="row">
                            <div class="col-lg-12">
                                <div class="hstack gap-2 justify-content-end">
                                    <a href="{{ url('master/wilayah') }}" class="btn btn-outline-primary">Kembali</a>
                                    <button type="submit" class="btn btn-primary">
                                        {{ $isNew ? 'Simpan' : 'Ubah' }}
                                    </button>
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
