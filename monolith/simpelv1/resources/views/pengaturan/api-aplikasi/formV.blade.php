@extends('layout.main')
@section('content')
@include('components.breadcums', $breadcums)
<div class="row">
    <div class="col-lg-8">
        <div class="card">
            <div class="card-header">
                <h5 class="card-title mb-0">{{ $isNew ? 'Tambah' : 'Edit' }} Aplikasi Integrasi API</h5>
            </div>
            <div class="card-body">
                <form action="{{ $isNew ? url('/pengaturan/api-aplikasi') : url('/pengaturan/api-aplikasi/'.$model->id) }}" method="POST" class="ajaxForm">
                    @csrf
                    @if(!$isNew)
                        @method('PUT')
                    @endif
                    <div class="mb-3">
                        <label for="name" class="form-label">Nama Aplikasi *</label>
                        <input class="form-control" id="name" name="name" value="{{ old('name', $model->name ?? '') }}" required>
                    </div>
                    <div class="mb-3">
                        <div class="form-check form-switch">
                            <input class="form-check-input" type="checkbox" id="is_active" name="is_active" value="1" {{ (old('is_active', $model->is_active ?? 1) ? 'checked' : '') }}>
                            <label class="form-check-label" for="is_active">Status Aktif</label>
                        </div>
                    </div>
                    @if(!$isNew)
                    <div class="mb-3">
                        <label class="form-label">Bearer Token</label>
                        <div class="input-group">
                            <input type="text" class="form-control" id="token" value="{{ $model->bearer_token }}" readonly>
                            <button class="btn btn-outline-secondary" type="button" onclick="copyToken()">Copy</button>
                            <button class="btn btn-outline-primary" type="button" onclick="generateToken({{ $model->id }})">Generate Baru</button>
                        </div>
                    </div>
                    @endif
                    <div class="hstack gap-2">
                        <a href="{{ url('/pengaturan/api-aplikasi') }}" class="btn btn-outline-primary">Kembali</a>
                        <button type="submit" class="btn btn-primary">Simpan</button>
                    </div>
                </form>
            </div>
        </div>
    </div>
</div>
@endsection
@section('js')
<script>
function copyToken() {
    const input = document.getElementById('token');
    input.select();
    input.setSelectionRange(0, 99999);
    document.execCommand('copy');
    alert('Token berhasil disalin!');
}
function generateToken(id) {
    if (!confirm('Generate token baru? Token lama akan diganti!')) return;
    $.post(`{{ url('/pengaturan/api-aplikasi') }}/${id}/generate-token`, {_token: '{{ csrf_token() }}'}, function(res) {
        document.getElementById('token').value = res.token;
        alert('Token baru: ' + res.token);
    });
}
</script>
@endsection 