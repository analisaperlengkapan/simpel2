@extends('layout.main')
@section('content')
@include('components.breadcums', $breadcums)
<div class="row">
    <div class="col-lg-12">
        <div class="card">
            <div class="card-header">
                <div class="d-flex align-items-center">
                    <div class="flex-grow-1">
                        <h5 class="card-title mb-0">Endpoint API untuk Aplikasi: <b>{{ $application->name }}</b></h5>
                    </div>
                    <div class="flex-shrink-0">
                        <a href="{{ url('/pengaturan/api-endpoint/create?application_id='.$application->id) }}" class="btn btn-success btn-label waves-effect waves-light">
                            <i class="ri-add-line label-icon align-middle fs-16 me-2"></i>Tambah Endpoint
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
                            <th>Path</th>
                            <th>Method</th>
                            <th>Status</th>
                            <th>Keterangan</th>
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
            url: "{{ url('/pengaturan/api-endpoint/gridData') }}",
            dataSrc: 'data',
            data: d => { d.application_id = {{ $application->id }}; }
        },
        columns: [
            { data: null, render: (data, type, row, meta) => meta.row + 1 },
            { data: 'name' },
            { data: 'path' },
            { data: 'method' },
            { data: 'is_active', render: d => d ? '<span class="badge bg-success">Aktif</span>' : '<span class="badge bg-danger">Nonaktif</span>' },
            { data: 'description', render: d => d ? d : '-' },
            { data: 'id', render: function(data, type, row) {
                const url = `{{ url('/pengaturan/api-endpoint') }}/${data}`;
                return `<div class='text-center'>
                    <a href='${url}' class='btn btn-warning btn-sm'>Edit</a>
                    <button class='btn btn-danger btn-sm' onclick='deleteEndpoint(${data})'>Hapus</button>
                </div>`;
            }, searchable: false }
        ]
    });
    window.deleteEndpoint = function(id) {
        if (!confirm('Hapus endpoint ini?')) return;
        $.ajax({
            url: `{{ url('/pengaturan/api-endpoint') }}/${id}`,
            type: 'DELETE',
            data: {_token: '{{ csrf_token() }}'},
            success: function() { dt.ajax.reload(null, false); },
            error: function() { alert('Gagal menghapus!'); }
        });
    }
});
</script>
@endsection 