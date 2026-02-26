@extends('layout.main')
@section('content')
    @include('components.breadcums', $breadcums)
    <div class="row">
        <div class="col-lg-12">
            <div class="card">
                <div class="card-header">
                    <div class="d-flex align-items-center">
                        <div class="flex-grow-1">
                            <h5 class="card-title mb-0">Monitoring Penghapusan BMN</h5>
                        </div>
                    </div>
                </div>
                <div class="card-body">
                    <table id="{{ $tableId }}" class="display table table-bordered dt-responsive my-dt"
                        style="width:100%">
                        <thead>
                            <tr>
                                <th>Tanggal Pengajuan @include('components.datepicker',['index'=>0,'name'=>'','className'=>'column-filter'])</th>
                                <th>Nama Pengajuan @include('components.dtFilterInput',['index'=>1,'column' => 'Nama'])</th>
                                <th>Satker @include('components.dtFilterInput',['index' => 2,'column' => 'Satker'])</th>
                                <th>Kode Barang @include('components.dtFilterInput',['index' => 3,'column' => 'Satker'])</th>
                                <th>Nama Barang @include('components.dtFilterInput',['index' => 4,'column' => 'Satker'])</th>
                                <th>Aktifitas @include('components.dtFilterInput',['index' => 5,'column' => 'Aktifitas'])</th>
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
                columns: [{
                        data: 'tgl_pengajuan',
                        render :(data, type, row) =>{
                            return dateFormatIndo(row.tgl_pengajuan);
                        }
                    },
                    {
                        data: 'nama'
                    },
                    {
                        data: 'inst_nama'
                    },
                    {
                        data: 'kode_barang'
                    },
                    {
                        data: 'nm_barang'
                    },
                    {
                        data: 'aktifitas'
                    },
                    {
                        data: 'id',
                        render: (data, type, row) => {
                            const url = `{{ $controller . '/' . '${data}' }}`;
                            return `
                                <div class="d-flex justify-content-center gap-2">
                                    @include('components.updateBtn', [
                                        'url' => '${url}',
                                    ])
                                </div>`;
                        },
                    }
                ]
            });


        })
    </script>
@endsection
