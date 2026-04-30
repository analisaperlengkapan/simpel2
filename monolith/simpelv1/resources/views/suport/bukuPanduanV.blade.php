@extends('layout.main')
@section('content')
    @include('components.breadcums', $breadcums)
    <div class="row justify-content-center">
        <div class="col-lg-10">
            <div class="card">
                <div class="bg-warning-subtle position-relative">
                    <div class="card-body p-5">
                        <div class="text-center">
                            <h3>Buku Panduan</h3>
                            <p class="mb-0 text-muted">Last update: {{ MyHelper::dateFormatIndo($lastUpdate) }}</p>
                        </div>
                    </div>
                    <div class="shape">
                        <svg xmlns="http://www.w3.org/2000/svg" version="1.1" xmlns:xlink="http://www.w3.org/1999/xlink"
                            xmlns:svgjs="http://svgjs.com/svgjs" width="1440" height="60" preserveAspectRatio="none"
                            viewBox="0 0 1440 60">
                            <g mask="url(&quot;#SvgjsMask1001&quot;)" fill="none">
                                <path d="M 0,4 C 144,13 432,48 720,49 C 1008,50 1296,17 1440,9L1440 60L0 60z"
                                    style="fill: var(--vz-secondary-bg);"></path>
                            </g>
                            <defs>
                                <mask id="SvgjsMask1001">
                                    <rect width="1440" height="60" fill="#ffffff"></rect>
                                </mask>
                            </defs>
                        </svg>
                    </div>
                </div>
                <div class="card-body p-4">

                    @foreach ($panduans as $key => $panduan)
                        <div class="d-flex">
                            <div class="flex-shrink-0 me-3">
                                <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"
                                    fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"
                                    stroke-linejoin="round"
                                    class="feather feather-check-circle text-success icon-dual-success icon-xs">
                                    <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"></path>
                                    <polyline points="22 4 12 14.01 9 11.01"></polyline>
                                </svg>
                            </div>
                            <div class="flex-grow-1">
                                <h5>{{ $key }}</h5>
                                <ul class="text-muted vstack gap-2">
                                    @foreach ($panduan as $item)
                                        <li>
                                            <a href="{{ asset($item->path) }}" target="_blank">
                                                <i class="ri-download-cloud-line"></i> {{ $item->judul }}
                                            </a>
                                        </li>
                                    @endforeach

                                </ul>
                            </div>
                        </div>
                    @endforeach
                </div>
            </div>
        </div>
    </div>
@endsection
