@extends('layout.main')
@section('content')
    @include('components.breadcums', $breadcums)
    <form action="{{ $controller }}" method="POST" class="ajaxForm" enctype="multipart/form-data">
        @csrf
        @if (!$isNew)
            <input type="hidden" name="id" value="{{ $model['id'] }}">
        @endif
        <div class="row">
            <div class="col-lg-12">
                <div class="card">
                    <div class="card-header">
                        <div class="d-flex align-items-center">
                            <div class="flex-grow-1">
                                <h5 class="card-title mb-0">{{ $title }}</h5>
                            </div>
                        </div>
                    </div>
                    <div class="card-body">
                        <div class="row">
                            <div class=" col-lg-6">
                                <div class="mb-3">
                                    <label for="kategori" class="form-label">Kategori</label>
                                    <div class="input-group">
                                        <select class="form-select" id="kategori" name="kategori" required
                                            aria-label="Example select with button addon">
                                            <option selected>Pilih</option>
                                            {!! $kategoriOptions !!}
                                        </select>
                                        <button class="btn btn-outline-secondary" type="button"
                                            id="addKategori">Tambah</button>
                                    </div>
                                </div>
                            </div>
                        </div>
                        <div class="row">
                            <div class=" col-lg-6">
                                <div class="mb-3">
                                    <label for="platform" class="form-label">Platform</label>
                                    <select class="form-select" id="platform" name="platform" required>
                                        {!! $platformOptions !!}
                                    </select>
                                </div>
                            </div>
                        </div>
                        <div class="row">
                            <div class=" col-lg-6">
                                <div class="mb-3">
                                    <label for="judul" class="form-label">Judul</label>
                                    <input class="form-control" id="judul" name="judul" required
                                        placeholder="Judul Panduan" value="{{ $model['judul'] ?? '' }}">
                                </div>
                            </div>
                        </div>
                        <div class="row">
                            <div class="col-lg-6">
                                <div class="mb-3">
                                    <label for="file" class="form-label">File Panduan</label>
                                    <input type="file" class="form-control" id="file" name="file" accept=".pdf"
                                        @if ($isNew) required @endif
                                        value="{{ $model['path'] ?? '' }}">
                                    @if (isset($model['path']))
                                        <a href="{{ asset($model['path']) }}" target="_blank">
                                            <i class="ri-download-cloud-line"></i> Download File
                                        </a>
                                    @endif
                                </div>
                            </div>
                        </div>
                    </div>
                </div>


                <div class="row" style="margin-bottom: 10px">
                    <div class="col-lg-12">
                        <div class="hstack gap-2 justify-content-left">
                            <a href="{{ $controller }}" class="btn btn-outline-primary">Kembali</a>
                            <button type="submit" class="btn btn-primary">Simpan</button>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    </form>
    <div id="addKategoriModal" class="modal fade zoomIn" tabindex="-1" aria-labelledby="zoomInModalLabel"
        aria-hidden="true" style="display: none;">
        <div class="modal-dialog modal-dialog-centered">
            <div class="modal-content">
                <div class="modal-header">
                    <h5 class="modal-title" id="zoomInModalLabel">Tambah Kategori</h5>
                    <button type="button" class="btn-close" data-bs-dismiss="modal" aria-label="Close"></button>
                </div>
                <div class="modal-body">
                    <div class="row">
                        <div class=" col-lg-12">
                            <div class="mb-3">
                                <label for="newKategori" class="form-label">Nama Kategori</label>
                                <input type="newKategori" class="form-control" id="newKategori" required
                                    placeholder="Nama Kategori">
                            </div>
                        </div>
                    </div>
                </div>
                <div class="modal-footer">
                    <button type="button" class="btn btn-light" data-bs-dismiss="modal">Tutup</button>
                    <button type="button" class="btn btn-primary" id="saveKategori">Simpan</button>
                </div>
            </div>
        </div>
    </div>
@endsection

@section('js')
    <script>
        $(function() {
            $('#addKategori').on('click', function() {
                $('#addKategoriModal').modal('show');
            })

            $('#saveKategori').on('click', function() {
                const value = $('#newKategori').val();
                if (value == '') return;
                const newOption = $('<option>', {
                    value,
                    text: value
                })
                $('#kategori').append(newOption).val(value).trigger('change');
                $('#addKategoriModal').modal('hide');
            })
        })
    </script>
@endsection
