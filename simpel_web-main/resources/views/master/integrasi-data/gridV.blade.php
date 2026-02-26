@extends('layout.main')
@section('content')
@include('components.breadcums', $breadcums)
    <div class="row">
        <div class="col-lg-12">
            <div class="card">
                <div class="card-header">
                    <div class="d-flex align-items-center">
                        <div class="flex-grow-1">
                            <h5 class="card-title mb-0">Integrasi Data</h5>
                        </div>
                        <div class="flex-shrink-0">
                            <a href="{{ URL::to('/pengaturan/integrasi-data/create') }}" type="button" class="btn btn-success btn-label waves-effect waves-light"><i class="ri-add-line label-icon align-middle fs-16 me-2"></i>
                                Tambah
                            </a>
                        </div>
                    </div>
                    <div class="row mt-3">
                        @include('components.dtSearchBox', [
                            'tableId' => $tableId,
                            'filterBy' => [
                                ['text' => 'Host', 'value' => 'host'],
                                ['text' => 'Username', 'value' => 'username'],
                                ['text' => 'Nama Aplikasi', 'value' => 'nama_aplikasi'],
                            ],
                        ])
                    </div>
                </div>
                <div class="card-body">
                    <table id="{{ $tableId }}" class="display table table-bordered dt-responsive" style="width:100%">
                        <thead>
                            <tr>
                                <th>No</th>
                                <th>Host</th>
                                <th>Username</th>
                                <th>Password</th>
                                <th>Nama Aplikasi</th>
                                <th>Aksi</th>
                            </tr>
                        </thead>
                    </table>
                </div>
            </div>
        </div>
        <!--end col-->
    </div>
<style>
    #{{ $tableId }} thead th {
        background-color: #405189;
        color: #ffffff;
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
            "deferRender": true,
            dom: dtLayout,
            language: {
                url: `{{ url('/assets/js/datatable_bahasa.json') }}`,
            },
            ajax: {
                url: "{{ URL::to('/pengaturan/integrasi-data/gridData') }}",
                dataSrc: 'data',
                data: (data) => {
                        const filterType = $('#dt-filter-by').data('filterby');
                        if (filterType.trim() !== '') {
                            data.filterBy = filterType;
                        }
                    }
            },
            columns: [
                {"data": function (row, data, index, display) {
                        return (display.row+1);
                }},
                {
                    data: 'host'
                },
                {
                    data: 'username'
                },
                {
                    data: 'password'
                },
                {
                    data: 'nama_aplikasi'
                },
                {
                    data: 'id',
                    render: (data, type, row) => {
                        const url = `{{ url('/pengaturan/integrasi-data/${data}') }}`;
                        return `
                            <div class="text-center" role="group">
                                @include('components.updateBtn', [
                                    'url' => '${url}',
                                    'className'=>'btn-icon'
                                ])
                                @include('components.deleteBtn', [
                                    'url' => '${url}',
                                    'title' => '${row.nama}',
                                    'tableId'=>'${tableId}',
                                    'className'=>'btn-icon'
                                ])
                            </div>
                        `;
                    },searchable: false
                }
            ]

        })
    })
</script>
@endsection
