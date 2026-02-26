@extends('layout.main')
@section('content')
@include('components.breadcums', $breadcums)
    <div class="row">
        <div class="col-lg-12">
            <div class="card">
                <div class="card-header">
                    <div class="d-flex align-items-center">
                        <div class="flex-grow-1">
                            <h5 class="card-title mb-0">{{ $judul }} Kritik dan Saran</h5>
                        </div>
                    </div>
                </div>
                <div class="card-body">
                    <form action="/suport/kritik" method="POST" class="ajaxForm" >
                        @csrf
                        @if (!$isNew)
                            <input type="hidden" id="id" name="id" value="{{ $model['id'] }}">
                        @endif
                        <!--<div class="row">
                            <div class="col-lg-3">
                                <div class="mb-3">
                                    <label for="nip" class="form-label">Kode Tiket *</label>
                                    <input class="form-control" id="kode_tiket" name="kode_tiket" required value="{{ $model['kode_tiket'] ?? '' }}">
                                </div>
                            </div>
                            <div class=" col-lg-6">
                                <div class="mb-3">
                                    <label for="name" class="form-label">Judul *</label>
                                    <input type="text" class="form-control" id="judul" name="judul" required value="{{ $model['judul'] ?? '' }}">
                                </div>
                            </div>
                            <div class=" col-lg-3">
                                <div class="mb-3">
                                    <label for="name" class="form-label">Status </label>
                                    <select class="form-control" data-choices data-choices-search-false name="status" id="status">
                                        <option value="">Pilih Status</option>
                                        {!! $statusOptions !!}
                                    </select>
                                </div>
                            </div>
                           
                        </div>-->
                        <div class="row">
                        <div class=" col-lg-6">
                                <div class="mb-3">
                                    <label for="kritik" class="form-label">Kritik *</label>
                                    <!--<input type="text" class="form-control" id="deskripsi" name="deskripsi" required value="{{ $model['deskripsi'] ?? '' }}">-->
                                    <textarea class="form-control" id="kritik" name="kritik" required  rows="3">{{ $model['kritik'] ?? '' }}</textarea>
                                </div>
                            </div>
                            <div class=" col-lg-6">
                                <div class="mb-3">
                                    <label for="saran" class="form-label">saran *</label>
                                    <!--<input type="text" class="form-control" id="deskripsi" name="deskripsi" required value="{{ $model['deskripsi'] ?? '' }}">-->
                                    <textarea class="form-control" id="saran" name="saran" required  rows="3">{{ $model['saran'] ?? '' }}</textarea>
                                </div>
                            </div>
                            
                        </div>
                        
                        <div class="row">
                            <div class="col-lg-12">
                                <div class="hstack gap-2 justify-content-end">
                                    <a href="{{ url('suport/kritik') }}" class="btn btn-outline-primary">Kembali</a>
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
