@extends('layout.main')
@section('content')
    @include('components.breadcums', $breadcums)
    <form action="/{{ $controller }}" method="POST" class="ajaxForm" enctype="multipart/form-data">
        @csrf
        <input type="hidden" name="id" value="{{ $model->id }}">
        <input type="hidden" name="parent" value="{{ $model->parent }}">
        <div class="row">
            <div class="col-lg-12">
                <div class="card">
                    <div class="card-header">
                        <div class="d-flex align-items-center">
                            <div class="flex-grow-1">
                                <h5 class="card-title mb-0">Atur Menu</h5>
                            </div>
                        </div>
                    </div>
                    <div class="card-body">
                        <div class="row">
                            <div class=" col-lg-5">
                                <div class="mb-3">
                                    <label for="name" class="form-label">Menu</label>
                                    <input class="form-control" id="name" name="name" required placeholder="name"
                                        value="{{ $model['name'] ?? '' }}">
                                </div>
                            </div>
                        </div>
                        <div class="row">
                            <div class=" col-lg-5">
                                <div class="mb-3">
                                    <label for="ururan" class="form-label">Urutan</label>
                                    <select name="urutan" id="urutan" class="form-control" data-choices
                                        data-choices-sorting-false>
                                        {!! $urutanOptions !!}
                                    </select>
                                </div>
                            </div>
                        </div>
                        <div class="row">
                            <div class=" col-lg-5">
                                <div class="mb-3">
                                    <label for="route" class="form-label">URL</label>
                                    <input class="form-control" placeholder="isi" id="route" disabled
                                        value="{{ $model['route'] ?? '' }}" />
                                </div>
                            </div>
                        </div>
                        <div class="row">
                            <div class=" col-lg-5">
                                <div class="mb-3">
                                    <div class="custom-control custom-checkbox">
                                        <input type="checkbox" class="custom-control-input" id="is_active" name="is_active"
                                            {{ $model['is_active'] == '1' ? 'checked' : '' }} value="1">
                                        <label class="custom-control-label" for="is_active">Aktif</label>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>


                <div class="row" style="margin-bottom: 10px">
                    <div class="col-lg-12">
                        <div class="hstack gap-2 justify-content-left">
                            <a href="/{{ $controller }}" class="btn btn-outline-primary">Kembali</a>
                            <button type="submit" class="btn btn-primary">Simpan</button>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    </form>
@endsection

@section('js')
    <script></script>
@endsection
