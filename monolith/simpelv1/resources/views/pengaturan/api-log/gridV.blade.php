@extends('layout.main')
@section('content')
@include('components.breadcums', $breadcums)
<div class="row">
    <div class="col-lg-12">
        <div class="card">
            <div class="card-header">
                <h5 class="card-title mb-0">Log API</h5>
                <div class="row mt-3">
                    <div class="col-md-2">
                        <select id="filterApp" class="form-select form-select-sm">
                            <option value="">Semua Aplikasi</option>
                            @foreach($apps as $app)
                                <option value="{{ $app->id }}">{{ $app->name }}</option>
                            @endforeach
                        </select>
                    </div>
                    <div class="col-md-2">
                        <select id="filterEndpoint" class="form-select form-select-sm">
                            <option value="">Semua Endpoint</option>
                            @foreach($endpoints as $ep)
                                <option value="{{ $ep->id }}">{{ $ep->name }}</option>
                            @endforeach
                        </select>
                    </div>
                    <div class="col-md-2">
                        <input type="text" id="filterStatus" class="form-control form-control-sm" placeholder="Status Code">
                    </div>
                    <div class="col-md-2">
                        <input type="date" id="filterDate" class="form-control form-control-sm">
                    </div>
                </div>
            </div>
            <div class="card-body">
                <table id="{{ $tableId }}" class="display table table-bordered dt-responsive" style="width:100%">
                    <thead>
                        <tr>
                            <th>Waktu</th>
                            <th>Aplikasi</th>
                            <th>Endpoint</th>
                            <th>Method</th>
                            <th>Status</th>
                            <th>IP</th>
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
            url: "{{ url('/pengaturan/api-log/gridData') }}",
            dataSrc: 'data',
            data: function(d) {
                d.application_id = $('#filterApp').val();
                d.endpoint_id = $('#filterEndpoint').val();
                d.status_code = $('#filterStatus').val();
                d.date = $('#filterDate').val();
            }
        },
        columns: [
            { data: 'created_at', render: d => d ? d.replace('T', ' ').substring(0, 19) : '-' },
            { data: 'application_id', render: function(d) {
                const app = @json($apps).find(a => a.id == d);
                return app ? app.name : '-';
            }},
            { data: 'endpoint_id', render: function(d) {
                const ep = @json($endpoints).find(e => e.id == d);
                return ep ? ep.name : '-';
            }},
            { data: 'request_method' },
            { data: 'status_code', render: d => d ? `<span class='badge bg-${d >= 200 && d < 300 ? 'success' : (d >= 400 ? 'danger' : 'warning')}'>${d}</span>` : '-' },
            { data: 'requester_ip' },
            { data: 'id', render: function(data) {
                const url = `{{ url('/pengaturan/api-log') }}/${data}`;
                return `<a href='${url}' class='btn btn-info btn-sm'>Detail</a>`;
            }, searchable: false }
        ]
    });
    $('#filterApp, #filterEndpoint, #filterStatus, #filterDate').on('change keyup', function() {
        dt.ajax.reload();
    });
});
</script>
@endsection 