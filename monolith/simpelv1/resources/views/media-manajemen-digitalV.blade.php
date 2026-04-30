@extends('layout.main')
@section('content')
    @include('components.breadcums', $breadcums)
    <link rel="stylesheet" href="{{ url('assets/libs/glightbox/css/glightbox.min.css') }}">
    <div class="row">
        <div class="col-lg-12">
            <div class="">
                <div class="card-body">
                    <div class="row">
                        <div class="col-lg-12">
                            <div class="text-center">
                                <ul class="list-inline categories-filter animation-nav" id="filter">
                                    <li class="list-inline-item"><a class="categories {{ $aktif ? '' : 'active' }}"
                                            href="{{ url($controller) }}">All</a></li>
                                    @foreach ($kategories as $kategori)
                                        <li class="list-inline-item"><a
                                                class="categories {{ $aktif == $kategori->kategori_slug ? 'active' : '' }}"
                                                href="{{ url($controller . '/' . $kategori->kategori_slug) }}"
                                                data-filter=".{{ $kategori->kategori_slug }}">{{ $kategori->kategori }}</a>
                                        </li>
                                    @endforeach
                                </ul>
                            </div>
                            <div class="row gallery-wrapper">
                                @foreach ($fotos as $foto)
                                    <div class="element-item col-xxl-3 col-xl-4 col-sm-6 {{ $foto->kategori_slug }}"
                                        data-category="{{ $foto->kategori_slug }}">
                                        <div class="gallery-box card">
                                            <div class="gallery-container">
                                                @if ($foto->filetype == 'pdf')
                                                    <a href="{{ asset($foto->path) }}" target="_blank">
                                                        <div
                                                            class="file-details-box bg-light p-3 text-center rounded-3 border border-light mb-3">
                                                            <div class="display-4 file-icon">
                                                                <i class="ri-file-pdf-fill align-bottom text-danger"></i>
                                                            </div>
                                                        </div>
                                                    </a>
                                                @else
                                                    <a class="image-popup" href="{{ asset($foto->path) }}"
                                                        title="{{ $foto->filename }}">
                                                        <img class="gallery-img img-fluid mx-auto"
                                                            src="{{ asset($foto->path) }}" alt="{{ $foto->filename }}" />
                                                        <div class="gallery-overlay">
                                                            <h5 class="overlay-caption">{{ $foto->filename }}</h5>
                                                        </div>
                                                    </a>
                                                @endif
                                            </div>

                                            <div class="box-content">
                                                <div class="d-flex align-items-center mt-1">
                                                    <div class="flex-grow-1 text-muted">by <a href=""
                                                            class="text-body text-truncate">{{ $foto->name }}</a>
                                                    </div>
                                                    <div class="flex-shrink-0">
                                                        <div class="d-flex gap-3">
                                                            <button type="button"
                                                                class="btn btn-sm fs-12 btn-link text-body text-decoration-none px-0">
                                                                <i class="ri-time-fill text-muted align-bottom me-1"></i>
                                                                {{ MyHelper::dateFormat($foto->created_at) }}
                                                            </button>
                                                        </div>
                                                    </div>
                                                </div>
                                            </div>
                                        </div>
                                    </div>
                                @endforeach
                            </div>
                            {!! $paginationElms !!}
                        </div>
                    </div>
                    <!-- end row -->
                </div>
                <!-- ene card body -->
            </div>
            <!-- end card -->
        </div>
        <!-- end col -->
    </div>
@endsection

@section('js')
    <script src="{{ url('assets/libs/isotope-layout/isotope.pkgd.min.js') }}"></script>
    <script src="{{ url('assets/libs/glightbox/js/glightbox.min.js') }}"></script>
    <script>
        $(function() {
            var GalleryWrapper = document.querySelector('.gallery-wrapper');
            if (GalleryWrapper) {
                var iso = new Isotope('.gallery-wrapper', {
                    itemSelector: '.element-item',
                    layoutMode: 'fitRows'
                });
            }

        });
        // bind filter button click

        var lightbox = GLightbox({
            selector: '.image-popup',
            title: false,
        });
    </script>
@endsection
