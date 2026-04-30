@extends('layout.main')
@section('content')
@include('components.breadcums', $breadcums)
<div class="row">
    <div class="col-lg-8">
        <div class="card">
            <div class="card-header">
                <h5 class="card-title mb-0">{{ $isNew ? 'Tambah' : 'Edit' }} Endpoint API untuk Aplikasi: <b>{{ $application->name }}</b></h5>
            </div>
            <div class="card-body">
                <form action="{{ $isNew ? url('/pengaturan/api-endpoint') : url('/pengaturan/api-endpoint/'.$model->id) }}" method="POST" class="ajaxForm">
                    @csrf
                    @if(!$isNew)
                        @method('PUT')
                    @endif
                    <input type="hidden" name="application_id" value="{{ $application->id }}">
                    <div class="mb-3">
                        <label for="name" class="form-label">Nama Endpoint *</label>
                        <input class="form-control" id="name" name="name" value="{{ old('name', $model->name ?? '') }}" required>
                    </div>
                    <div class="mb-3">
                        <label for="path" class="form-label">Path *</label>
                        <input class="form-control" id="path" name="path" value="{{ old('path', $model->path ?? '') }}" required placeholder="/api/endpoint">
                    </div>
                    <div class="mb-3">
                        <label for="method" class="form-label">Method *</label>
                        <select class="form-select" id="method" name="method" required>
                            @foreach(['GET','POST','PUT','DELETE'] as $m)
                                <option value="{{ $m }}" {{ (old('method', $model->method ?? 'GET') == $m) ? 'selected' : '' }}>{{ $m }}</option>
                            @endforeach
                        </select>
                    </div>
                    <div class="mb-3">
                        <div class="form-check form-switch">
                            <input class="form-check-input" type="checkbox" id="is_active" name="is_active" value="1" {{ (old('is_active', $model->is_active ?? 1) ? 'checked' : '') }}>
                            <label class="form-check-label" for="is_active">Status Aktif</label>
                        </div>
                    </div>
                    <div class="mb-3">
                        <label for="description" class="form-label">Keterangan</label>
                        <textarea class="form-control" id="description" name="description" rows="2">{{ old('description', $model->description ?? '') }}</textarea>
                    </div>
                    <div class="hstack gap-2">
                        <a href="{{ url('/pengaturan/api-endpoint?application_id='.$application->id) }}" class="btn btn-outline-primary">Kembali</a>
                        <button type="submit" class="btn btn-primary">Simpan</button>
                    </div>
                </form>
            </div>
        </div>
    </div>
</div>
@endsection 