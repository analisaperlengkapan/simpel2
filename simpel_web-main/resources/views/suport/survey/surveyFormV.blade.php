@extends('layout.main')
@section('content')
@include('components.breadcums', $breadcums)
    <div class="row">
        <div class="col-lg-12">
            <div class="card">
                <div class="card-header">
                    <div class="d-flex align-items-center">
                        <div class="flex-grow-1">
                            <h5 class="card-title mb-0">{{ $judul }} Survey</h5>
                        </div>
                    </div>
                </div>
                <div class="card-body">
                    <form action="/suport/survey" method="POST" class="ajaxForm" enctype="multipart/form-data">
                        @csrf
                        @if (!$isNew)
                            <input type="hidden" id="id" name="id" value="{{ $model['id'] }}">
                        @endif
                        <div class="row">
                            <!--<div class="col-lg-3">
                                <div class="mb-3">
                                    <label for="nip" class="form-label">Kode Tiket *</label>
                                    <input class="form-control" id="kode_tiket" name="kode_tiket" required value="{{ $model['kode_tiket'] ?? '' }}">
                                </div>
                            </div>-->
                            
							<div class=" col-lg-6">
                                <div class="mb-3">
                                    <label for="name" class="form-label">Pertanyaan *</label>
                                    <input type="text" class="form-control" id="pertanyaan" name="pertanyaan" required value="{{ $model['pertanyaan'] ?? '' }}">
                                </div>
                            </div>
                            <div class=" col-lg-3">
                                <!--<div class="mb-3">
                                    <label for="name" class="form-label">Tipe *</label>
                                    <select class="form-control" data-choices data-choices-search-false name="tipe_tiket" id="tipe_tiket">
                                        <option value="">Pilih Tipe</option>
                                        {!! $tipeOptions !!}
                                    </select>
                                </div>-->
								@include('components.datepicker',[
                                        'value'=>'',
                                        'label'=>'Tanggal Survey',
                                        'name'=>'tgl_survey'
                                        ]
                                    )
                            </div>
                            
                            
                        </div>
                        <div class="row">
                            
                            <div class=" col-lg-6">
                                <div class="mb-3">
                                    <label for="name" class="form-label">Deskripsi *</label>
                                    <!--<input type="text" class="form-control" id="deskripsi" name="deskripsi" required value="{{ $model['deskripsi'] ?? '' }}">-->
                                    <textarea class="form-control" id="deskripsi" name="deskripsi" required  rows="2">{{ $model['deskripsi'] ?? '' }}</textarea>
                                </div>
                            </div>
							<!-- <div class=" col-lg-3">
                                <div class="mb-3">
                                    <label for="name" class="form-label">Status </label>
                                    <select class="form-control" data-choices data-choices-search-false name="status" id="status">
                                        <option value="">Pilih Status</option>
                                        {!! $statusOptions !!}
                                    </select>
                                </div>
                            </div> -->
						</div>

                        <div class="row">
                            <div class="col-lg-12">
                                <div class="hstack gap-2 justify-content-end">
                                    <a href="{{ url('suport/survey') }}" class="btn btn-outline-primary">Kembali</a>
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
