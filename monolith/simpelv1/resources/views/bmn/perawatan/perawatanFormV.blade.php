@extends('layout.main')
@section('content')
@include('components.breadcums', $breadcums)
    <div class="row">
        <div class="col-lg-12">
            <div class="card">
                <div class="card-header">
                    <div class="d-flex align-items-center">
                        <div class="flex-grow-1">
                            <h5 class="card-title mb-0">{{ $judul }}</h5>
                        </div>
                    </div>
                </div>
                <div class="card-body">
                    <form action="/bmn/perawatan/perawatan" method="POST" class="ajaxForm" >
                        @csrf
                        @if (!$isNew)
                            <input type="hidden" id="id" name="id" value="{{ $model['id'] }}">
                        @endif
                        <div class="row">
                            <div class="col-lg-6">
                                <div class="mb-3">
                                    <label for="kdsatker_keu" class="form-label">Kode Satker *</label>
                                    <select class="form-control" data-choices data-choices-sorting-false name="kdsatker_keu" id="kdsatker_keu">
                                        <option value="">Pilih Satker</option>
                                        {!! $satkerOptions !!}
                                    </select>
                                </div>
                            </div>
                        </div>
                        <div class="row">
                            <div class="col-lg-3">
                                <div class="mb-3">
                                    <label for="jns_perawatan" class="form-label">Jenis Perawatan *</label>
                                    <select class="form-control" data-choices data-choices-sorting-false name="jns_perawatan" id="jns_perawatan">
                                        <option value="">Pilih Jenis</option>
                                        {!! $jns_perawatan !!}
                                    </select>
                                </div>
                            </div>
                            <div class="col-lg-3">
                                <div class="mb-3">
                                <label for="kode_barang" class="form-label">Kode Barang *</label>
                                    <select class="form-control" data-choices data-choices-sorting-false name="kode_barang" id="kode_barang">
                                        <option value="">Pilih Barang</option>
                                        {!! $kode_barang !!}
                                    </select>
                                </div>
                            </div>
                            <div class=" col-lg-3">
                                <div class="mb-3">
                                    <!--<label for="name" class="form-label">Tanggal Perawatan *</label>
                                    <input type="date" class="form-control" id="tgl_perawatan" name="tgl_perawatan" value="{{ $model['tgl_perawatan'] ?? '' }}">-->
                                    @include('components.datepicker',[
                                        'value'=>$model['tgl_perawatan'] ?? '',
                                        'label'=>'Tanggal Perawatan',
                                        'name'=>'tgl_perawatan'
                                        ]
                                    )
                                </div>
                            </div>                            
                            <div class="col-lg-3">
                                <div class="mb-3">
                                    <label for="nip" class="form-label">Pelaksana</label>
                                    <input class="form-control" id="pelaksana" name="pelaksana" value="{{ $model['pelaksana'] ?? '' }}">
                                </div>
                            </div>
                        </div>
                        <div class="row">
                            <div class="col-lg-3">
                                <div class="mb-3">
                                    <label for="nip" class="form-label">Spesifikasi</label>
                                    <input class="form-control" id="spesifikasi" name="spesifikasi" value="{{ $model['spesifikasi'] ?? '' }}">
                                </div>
                            </div>
                            <div class="col-lg-3">
                                <div class="mb-3">
                                    <label for="nip" class="form-label">Biaya</label>
                                    <input class="form-control" id="biaya" name="biaya" value="{{ $model['biaya'] ?? '' }}">
                                </div>
                            </div>
                        </div>
                        
                        <hr/>
                        <div class="row">
                            <div class="col-lg-12">
                                <div class="hstack gap-2">
                                    <a href="{{ url('bmn/perawatan/perawatan') }}" class="btn btn-outline-primary">Kembali</a>
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
