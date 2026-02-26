@extends('layout.main')
@section('content')
    @include('components.breadcums', $breadcums)
    <div class="col-lg-12">
        <div class="card">
            <div class="card-header">
                <div class="d-flex align-items-center">
                    <div class="flex-grow-1">
                        <h5 class="card-title mb-0">Aset Rusak Berat</h5>
                    </div>
                </div>
            </div>
            <div class="card-body">
                <table id="{{ $tableId }}" class="display table table-bordered dt-responsive my-dt" style="width:100%">
                    <thead>
                        <tr>
                            <th>Kode Satker @include('components.dtFilterInput',['index' => 0,'column' => 'Kode Satker'])</th>
                            <th>Nama Satker @include('components.dtFilterInput',['index' => 1,'column' => 'Nama Satker'])</th>
                            <th>Kode Barang @include('components.dtFilterInput',['index' => 2,'column' => 'Kode Barang'])</th>
                            <th>Nama Barang @include('components.dtFilterInput',['index' => 3,'column' => 'Nama Barang'])</th>
                            <th>NUP @include('components.dtFilterInput',['index' => 4,'column' => 'NUP'])</th>
                            <th>Kondisi @include('components.dtFilterInput',['index' => 5,'column' => 'Kondisi'])</th>
                            <th>Status @include('components.dtFilterInput',['index' => 6,'column' => 'Kondisi'])</th>
                        </tr>
                    </thead>
                </table>
            </div>
        </div>
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
        const tableId = `{{ $tableId }}`;
        $(function() {
            const dt = $('#' + tableId).DataTable({
                serverSide: true,
                processing: true,
                deferRender: true,
                ordering: false,
                dom: dtLayout,
                language: {
                    url: `{{ url('/assets/js/datatable_bahasa.json') }}`,
                },
                ajax: {
                    url: "{{ $controller . '/gridData' }}",
                    dataSrc: 'data',
                },
                columns: [
                    {
                        data: 'kdsatker_keu'
                    },
                    {
                        data: 'nm_satker'
                    },
                    {
                        data: 'kode_barang'
                    },
                    {
                        data: 'nm_barang'
                    },
                    {
                        data: 'nup'
                    },
                    {
                        data: 'kondisi'
                    },
                    {
                        data: 'status'
                    },
                ]
            })
        })
    </script>
@endsection
