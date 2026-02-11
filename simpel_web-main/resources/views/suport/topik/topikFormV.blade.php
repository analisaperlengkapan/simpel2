@extends('layout.main')
@section('content')
@include('components.breadcums', $breadcums)
    <div class="row">
        <div class="col-lg-12">
            <div class="card">
                <div class="card-header">
                    <div class="d-flex align-items-center">
                        <div class="flex-grow-1">
                            <h5 class="card-title mb-0">Bantuan</h5>
                        </div>
                    </div>
                </div>
                <div class="card-body">
                    <form action="/suport/topik" method="POST" class="ajaxForm" enctype="multipart/form-data">
                        @csrf
                        
                        @if (!$isNew)
                            <input type="hidden" id="id" name="id" value="{{ $model['id'] }}">
                        @endif
                       
                        <div class="row">
                           
                            
							<div class=" col-lg-6">
                                <div class="mb-3">
                                    <label for="name" class="form-label">Topik *</label>
                                    <input type="text" class="form-control" id="topik" name="topik" required value="{{ $model['topik'] ?? '' }}">
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
                            
                        </div>
						
                        <div class="row">
                            <div class="col-lg-12">
                                <div class="hstack gap-2 justify-content-end">
                                    <a href="{{ url('suport/topik') }}" class="btn btn-outline-primary">Kembali</a>
                                   
                                    <button type="submit" class="btn btn-primary">
                                    {{ $isNew ? 'Simpan' : 'Ubah' }}
                                    </button>
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
       
    </script>
@endsection
