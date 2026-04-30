@extends('layout.main')
@section('content')
    @include('components.breadcums', $breadcums)
    <form action="{{ url($controller) }}" method="POST" class="ajaxForm">
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
                                <h5 class="card-title mb-">Kirim Notifikasi</h5>
                            </div>
                        </div>
                    </div>
                    <div class="card-body">
                        @csrf
                        <div class="row">
                            <div class=" col-lg-5">
                                <div class="mb-3">
                                    <label for="judul" class="form-label">Judul</label>
                                    <input class="form-control" id="judul" name="judul" required placeholder="Judul"
                                        value="{{ $model['judul'] ?? '' }}">
                                </div>
                            </div>
                        </div>
                        <div class="row">
                            <div class=" col-lg-5">
                                <div class="mb-3">
                                    @include('components.texteditor', [
                                        'label' => 'Isi',
                                        'name' => 'isi',
                                        'value' => $model['isi'] ?? '',
                                    ])
                                </div>
                            </div>
                        </div>
                        <div class="row">
                            <div class=" col-lg-5">
                                <div class="mb-3">
                                    <label for="url" class="form-label">Role Target Notifikasi</label>
                                    <select class="form-control" data-choices data-choices-removeItem multiple
                                        name="ms_role_id[]">
                                        {!! $roleOptions !!}
                                    </select>

                                </div>
                            </div>
                        </div>
                        <div class="row">
                            <div class=" col-lg-5">
                                <div class="mb-3">
                                    <div class="custom-control custom-checkbox">
                                        <input type="checkbox" class="custom-control-input" id="is_active" name="is_active"
                                            {{ $model['is_active'] ?? '0' == '1' ? 'checked' : '' }} value="1">
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
                            <a href="{{ url($controller) }}" class="btn btn-outline-primary">Kembali</a>
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
