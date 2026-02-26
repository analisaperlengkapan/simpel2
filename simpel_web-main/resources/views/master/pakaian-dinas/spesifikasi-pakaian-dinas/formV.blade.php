@extends('layout.main')
@section('content')
    @include('components.breadcums', $breadcums)
    <link rel="stylesheet" href="{{ url('assets/libs/glightbox/css/glightbox.min.css') }}">
    <div class="row">
        <div class="col-lg-12">
            <div class="card">
                <div class="card-header">
                    <div class="d-flex align-items-center">
                        <div class="flex-grow-1">
                            <h5 class="card-title mb-0">{{ $judul }} Spesifikasi Pakaian Dinas</h5>
                        </div>
                    </div>
                </div>
                <div class="card-body">
                    <form action="/master/pakaian-dinas/spesifikasi-pakaian-dinas" method="POST" class="ajaxForm">
                        @csrf
                        @if (!$isNew)
                            <input type="hidden" id="id" name="id" value="{{ $model['id'] }}">
                        @endif
                        <div class="row">
                            <div class="col-lg-9">
                                <div class="mb-3">
                                    <label for="kdsatker_keu" class="form-label">Jenis Pakaian *</label>
                                    <select class="form-control" data-choices data-choices-sorting-false
                                        name="ms_jenis_pakaian_dinas_id" id="ms_jenis_pakaian_dinas_id">
                                        <option value="">Pilih Jenis Pakaian</option>
                                        {!! $jenisPakaianOptions !!}
                                    </select>
                                </div>
                            </div>
                        </div>
                        <div class="row">
                            <div class="col-lg-9">
                                <div class="mb-3">
                                    <label for="gender" class="form-label">Jenis Kelamin*</label>
                                    <select class="form-control" data-choices data-choices-sorting-false name="gender"
                                        id="gender">
                                        {!! $genderOptions !!}
                                    </select>
                                </div>
                            </div>
                        </div>
                        <div class="row">
                            <div class="col-lg-9">
                                <div class="mb-3">
                                    <label for="kdsatker_keu" class="form-label">Spesifikasi *</label>
                                    <input class="form-control" id="nama" name="nama"
                                        value="{{ $model['nama'] ?? '' }}">
                                </div>
                            </div>
                        </div>
                        <div class="row">
                            <div class="col-lg-9">
                                <div class="mb-3">
                                    <label for="kdsatker_keu" class="form-label">Jenis Ukuran Digunakan</label>
                                    <select class="form-control" data-choices data-choices-sorting-false
                                        name="ms_ukuran_group" id="ms_ukuran_group">
                                        <option value="">Pilih Jenis Ukuran</option>
                                        {!! $jenisUkuranOptions !!}
                                    </select>
                                </div>
                            </div>
                        </div>
                        <div class="row">
                            <div class="col-lg-9">
                                <div class="mb-3">
                                    <label class="form-label">Foto</label>
                                    <input type="file" class="form-control" name="foto[]" multiple
                                        accept=".jpg, .jpeg, .png, .gif, .pdf" />
                                </div>
                            </div>
                        </div>
                        <div class="row gallery-wrapper">
                            @foreach ($fotos as $foto)
                                <div class="element-item col-xxl-3 col-xl-4 col-sm-6">
                                    <div class="gallery-box card">
                                        <div class="gallery-container">
                                            <a class="image-popup" href="{{ asset($foto->path) }}"
                                                data-title="{{ $foto->filename }}">
                                                <img class="gallery-img img-fluid mx-auto" src="{{ asset($foto->path) }}"
                                                    alt="" />
                                                <div class="gallery-overlay">
                                                    <h5 class="overlay-caption">{{ $foto->filename }}</h5>
                                                </div>
                                            </a>
                                        </div>

                                        <div class="box-content">
                                            <div class="d-flex align-items-center justify-content-center mt-1">
                                                <button type="button" data-id="{{ $foto->id }}"
                                                    class="btn btn-danger waves-effect waves-light btn-icon remove-foto"
                                                    data-title="Sepatu Pantofel">
                                                    <i class="ri-delete-bin-5-fill"></i>
                                                </button>
                                            </div>
                                        </div>
                                    </div>
                                </div>
                            @endforeach
                        </div>
                        <div id="deleted">
                        </div>
                        <div class="row">
                            <div class="col-lg-12">
                                <div class="hstack gap-2">
                                    <a href="{{ url('master/pakaian-dinas/spesifikasi-pakaian-dinas') }}"
                                        class="btn btn-outline-primary">Kembali</a>
                                    @if (!$readOnly)
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
    <script src="{{ url('assets/libs/isotope-layout/isotope.pkgd.min.js') }}"></script>
    <script src="{{ url('assets/libs/glightbox/js/glightbox.min.js') }}"></script>
    <script>
        $(function() {
            $('.remove-foto').on('click', function() {
                const id = $(this).data('id');
                $(this).closest('.element-item').remove();
                $('#deleted').append(`<input type="hidden" name="deleted[]" value="${id}" />`);
            })
            var isReadOnly = '{{ $readOnly }}';
            if (isReadOnly) {
                $('input, select').prop('readonly', true);
            }

            var GalleryWrapper = document.querySelector('.gallery-wrapper');
            if (GalleryWrapper) {
                var iso = new Isotope('.gallery-wrapper', {
                    itemSelector: '.element-item',
                    layoutMode: 'fitRows'
                });
            }

            // bind filter button click
            var filtersElem = document.querySelector('.categories-filter');
            if (filtersElem) {
                filtersElem.addEventListener('click', function(event) {
                    // only work with buttons
                    if (!matchesSelector(event.target, 'li a')) {
                        return;
                    }
                    var filterValue = event.target.getAttribute('data-filter');
                    if (filterValue) {
                        // use matching filter function
                        iso.arrange({
                            filter: filterValue
                        });
                    }
                });
            }

            // change is-checked class on buttons
            var buttonGroups = document.querySelectorAll('.categories-filter');
            if (buttonGroups) {
                Array.from(buttonGroups).forEach(function(btnGroup) {
                    var buttonGroup = btnGroup;
                    radioButtonGroup(buttonGroup);
                });
            }

            function radioButtonGroup(buttonGroup) {
                buttonGroup.addEventListener('click', function(event) {
                    // only work with buttons
                    if (!matchesSelector(event.target, 'li a')) {
                        return;
                    }
                    buttonGroup.querySelector('.active').classList.remove('active');
                    event.target.classList.add('active');
                });
            }
        })
        var lightbox = GLightbox({
            selector: '.image-popup',
            title: false,
        });
    </script>
@endsection
