@extends('layout.main')
@section('content')
    @include('components.breadcums', $breadcums)
    <div class="row">
        <div class="col-lg-12">
            <div class="card ">
                <div class="card-header">
                    <div class="d-flex align-items-center">
                        <div class="flex-grow-1">
                            <h5 class="card-title mb-0">{{ $title }}</h5>
                        </div>
                    </div>
                </div>
                <div class="card-body p-4">
                    <form method="POST" action="{{ $controller }}" class="ajaxForm">
                        @csrf
                        @foreach ($ukurans as $jenis => $ukuran)
                            <div class="row">
                                <div class="col-lg-6">
                                    <div class="mb-3">
                                        <label for="{{ $jenis }}" class="form-label">Ukuran
                                            {{ $jenis }}</label>
                                        <select class="form-control" required name="ukuran_{{ strtolower($jenis) }}">
                                            <option value="" selected>Pilih Ukuran</option>
                                            {!! $ukuran !!}
                                        </select>
                                    </div>
                                </div>
                            </div>
                        @endforeach
                        @if ($default['jenis_kelamin'] ?? null == 'P')
                            <div class="row" id="div-hijab">
                                <div class="col-lg-12">
                                    <div class="form-check">
                                        <label class="form-check-label" for="modal-with-hijab">Pakaian Muslimah</label>
                                        <input class="form-check-input" type="checkbox" name="with_hijab" value="1"
                                            {{ $default['with_hijab'] == '1' ? 'checked' : '' }}>
                                    </div>
                                </div>
                            </div>
                        @endif
                        <div class="col-lg-12 mt-4">
                            <div class="hstack gap-2 justify-content-left">
                                <button type="submit" class="btn btn-primary">Simpan</button>
                            </div>
                        </div>
                    </form>
                </div>
            </div>
        </div>
    </div>
@endsection
