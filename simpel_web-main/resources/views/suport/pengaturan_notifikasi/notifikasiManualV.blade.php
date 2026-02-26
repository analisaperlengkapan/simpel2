@extends('layout.main')
@section('content')
    @include('components.breadcums', $breadcums)
    <div class="row">
        <div class="col-lg-12">
            <div class="card">
                <div class="card-header">
                    <div class="d-flex align-items-center">
                        <div class="flex-grow-1">
                            <h5 class="card-title mb-0">Kirim Notifikasi Manual</h5>
                        </div>
                        @if ($canCreate)
                            <div class="flex-shrink-0">
                                <a href="{{ URL::to($controller . '/create') }}" type="button"
                                    class="btn btn-success btn-label waves-effect waves-light"><i
                                        class="ri-add-line label-icon align-middle fs-16 me-2"></i>
                                    Tambah
                                </a>
                            </div>
                        @endif
                    </div>
                </div>
                <div class="card-body">
                    @include('components.dtFilterBox', [
                        'columns' => $columns,
                        'selected' => $defColumns,
                    ])
                    <table id="{{ $tableId }}" class="display table table-bordered dt-responsive my-dt"
                        style="width:100%">
                        <thead>
                            <tr>
                                @foreach ($columns as $index => $column)
                                    <th>
                                        {{ $column }}
                                        @include('components.dtFilterInput', [
                                            'index' => $index,
                                            'column' => $column,
                                        ])
                                    </th>
                                @endforeach
                                <th>Aksi</th>
                            </tr>
                        </thead>
                    </table>
                </div>
            </div>
        </div>
    </div>
    <style>
        .dataTables_length {
            width: auto;
            float: right;
        }
    </style>
@endsection

@section('js')
    <script>
        const tableId = `{{ $tableId }}`;
        const defColumn = @json($defColumns);
        $(function() {
            const dt = $('#' + tableId).DataTable({
                processing: true,
                serverSide: true,
                ordering: false,
                "deferRender": true,
                responsive: false,
                dom: dtLayoutFilter,
                language: {
                    url: `{{ url('/assets/js/datatable_bahasa.json') }}`,
                },
                ajax: {
                    url: "{{ URL::to($controller . '/gridData') }}",
                    dataSrc: 'data',
                },
                scrollX: true,
                "initComplete": function(settings, json) {
                    for (i = 0; i < (dt.columns().header().length) - 1; i++) {
                        if ($.inArray(i, defColumn) == -1) {
                            let column = dt.column(i);
                            column.visible(false);
                        }
                    }
                },
                columns: [{
                        data: 'created_by',
                        className: 'text-center',
                    },
                    {
                        data: 'judul',
                        className: 'text-center',
                    },
                    {
                        data: 'created_at',
                        className: 'text-center',
                        render: (data, type, row) => {
                            return dateFormatIndo(data);
                        }
                    },
                    {
                        data: 'isi',
                        className: 'text-center',
                    },
                    {
                        data: 'target_role_id',
                        className: 'text-center',
                    },
                    {
                        data: 'is_active',
                        className: 'text-center',
                        render: (data) => data == 1 ? 'Aktif' : 'Non Aktif'
                    },
                    {
                        data: 'id',
                        className: "text-center",
                        render: (data, type, row) => {
                            const url = `{{ url($controller . '/${data}') }}`;
                            return `
                                <div class="btn-group" role="group">
                                @include('components.updateBtn', [
                                    'url' => '${url}',
                                    'className' => 'btn-icon',
                                ])
                                </div>
                            `;
                        },
                        searchable: false
                    }
                ]

            });

        });
    </script>
@endsection
