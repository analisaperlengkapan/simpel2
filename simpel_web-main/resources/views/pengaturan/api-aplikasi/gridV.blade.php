@extends('layout.main')
@section('content')
@include('components.breadcums', $breadcums)
<div class="row">
    <div class="col-lg-12">
        <div class="card">
            <div class="card-header">
                <div class="d-flex align-items-center">
                    <div class="flex-grow-1">
                        <h5 class="card-title mb-0">Manajemen Aplikasi Integrasi API</h5>
                    </div>
                    <div class="flex-shrink-0">
                        <a href="{{ url('/pengaturan/api-aplikasi/create') }}" class="btn btn-success btn-label waves-effect waves-light">
                            <i class="ri-add-line label-icon align-middle fs-16 me-2"></i>Tambah
                        </a>
                    </div>
                </div>
            </div>
            <div class="card-body">
                <table id="{{ $tableId }}" class="display table table-bordered dt-responsive" style="width:100%">
                    <thead>
                        <tr>
                            <th>No</th>
                            <th>Nama</th>
                            <th>Bearer Token</th>
                            <th>Status</th>
                            <th>Aksi</th>
                        </tr>
                    </thead>
                </table>
            </div>
        </div>
    </div>
</div>
<style>
    #{{ $tableId }} thead th {
        background-color: #405189;
        color: #fff;
        text-align: center;
        text-transform: uppercase;
    }
</style>
@endsection
@section('js')
<script>
$(function() {
    const tableId = `{{ $tableId }}`;
    const dt = $('#' + tableId).DataTable({
        processing: true,
        serverSide: true,
        ordering: false,
        deferRender: true,
        dom: dtLayout,
        language: {
            url: `{{ url('/assets/js/datatable_bahasa.json') }}`,
        },
        ajax: {
            url: "{{ url('/pengaturan/api-aplikasi/gridData') }}",
            dataSrc: 'data',
        },
        columns: [
            { data: null, render: (data, type, row, meta) => meta.row + 1 },
            { data: 'name' },
            { data: 'bearer_token', render: function(data, type, row) {
                return `<div class='input-group'>
                    <input type='text' class='form-control form-control-sm' value='${data}' readonly id='token-${row.id}'>
                    <button class='btn btn-outline-secondary btn-sm' type='button' onclick='copyToken(${row.id})'>Copy</button>
                    <button class='btn btn-outline-primary btn-sm' type='button' onclick='generateToken(${row.id})'>Generate</button>
                </div>`;
            }},
            { data: 'is_active', render: d => d ? '<span class="badge bg-success">Aktif</span>' : '<span class="badge bg-danger">Nonaktif</span>' },
            { data: 'id', render: function(data, type, row) {
                const url = `{{ url('/pengaturan/api-aplikasi') }}/${data}`;
                const endpointUrl = `{{ url('/pengaturan/api-endpoint') }}?application_id=${data}`;
                const logUrl = `{{ url('/pengaturan/api-log') }}?application_id=${data}`;
                return `<div class='text-center'>
                    <a href='${url}' class='btn btn-warning btn-sm'>Edit</a>
                    <a href='${endpointUrl}' class='btn btn-info btn-sm'>Endpoint</a>
                    <a href='${logUrl}' class='btn btn-secondary btn-sm'>Log</a>
                    <button class='btn btn-danger btn-sm' onclick='deleteApp(${data})'>Hapus</button>
                </div>`;
            }, searchable: false }
        ]
    });
    window.copyToken = function(id) {
        const input = document.getElementById('token-' + id);
        input.select();
        input.setSelectionRange(0, 99999);
        document.execCommand('copy');
        alert('Token berhasil disalin!');
    }
    window.generateToken = function(id) {
        if (!confirm('Generate token baru? Token lama akan diganti!')) return;
        $.post(`{{ url('/pengaturan/api-aplikasi') }}/${id}/generate-token`, {_token: '{{ csrf_token() }}'}, function(res) {
            dt.ajax.reload(null, false);
            alert('Token baru: ' + res.token);
        });
    }
    window.deleteApp = function(id) {
        if (!confirm('Hapus aplikasi ini?')) return;
        $.ajax({
            url: `{{ url('/pengaturan/api-aplikasi') }}/${id}`,
            type: 'DELETE',
            data: {_token: '{{ csrf_token() }}'},
            success: function() { dt.ajax.reload(null, false); },
            error: function() { alert('Gagal menghapus!'); }
        });
    }
});
</script>
@endsection 