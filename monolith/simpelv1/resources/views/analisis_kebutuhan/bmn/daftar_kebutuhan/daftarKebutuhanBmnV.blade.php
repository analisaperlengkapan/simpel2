@extends('layout.main')
@section('content')
    @include('components.breadcums', $breadcums)
    <div class="col-lg-12">
        <div class="card">
            <div class="card-header">
                <div class="d-flex align-items-center">
                    <div class="flex-grow-1">
                        <h5 class="card-title mb-0">Daftar Kebutuhan BMN</h5>
                    </div>
                </div>
            </div>
            <div class="card-body">
                <table id="{{ $tableId }}" class="display table table-bordered dt-responsive my-dt" style="width:100%">
                    <thead>
                        <tr>
                            <th>Tahun
                            @include('components.dtFilterInput',[
                                'index' => 0,
                                'column' => 'Tahun',
                                ]
                            )
                            </th>
                            <th>Satker
                                @include('components.dtFilterInput',[
                                    'index' => 1,
                                    'column' => 'Satker',
                                    ]
                                )
                            </th>
                            <th>Nama
                                @include('components.dtFilterInput',[
                                    'index' => 2,
                                    'column' => 'Nama',
                                    ]
                                )
                            </th>
                            <th>Deskripsi
                                @include('components.dtFilterInput',[
                                    'index' => 3,
                                    'column' => 'Deskripsi',
                                    ]
                                )
                            </th>
                            <th>Status
                                @include('components.dtFilterInput',[
                                    'index' => 4,
                                    'column' => 'Status',
                                    ]
                                )
                            </th>
                            <th>Aksi</th>
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
                    url: "{{ $controller . '/gridDataSatker?id='.($id ?? '') }}",
                    dataSrc: 'data',
                },
                columns: [{
                        data: 'tahun'
                    },
                    {
                        data: 'satker'
                    },
                    {
                        data: 'nama'
                    },
                    {
                        data: 'deskripsi'
                    },
                    {
                        data: 'status'
                    },
                    {
                        data: 'id',
                        render: (data, type, row) => {
                            const url = `{{ $controller . '/' . '${data}'.'/edit' }}`;
                            return `
                                <div class="d-flex justify-content-center gap-2">
                                    @include('components.updateBtn', [
                                        'url' => '${url}',
                                    ])
                                </div>`;
                        }
                    },
                ]
            })
        })
    </script>
@endsection
