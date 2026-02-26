@extends('layout.main')
@section('content')
    @include('components.breadcums', $breadcums)
    <div class="row">
        <div class="col-lg-12">
            <div class="card">
                <div class="card-header">
                    <div class="d-flex align-items-center">
                        <div class="flex-grow-1">
                            <h5 class="card-title mb-0">{{ $title }}</h5>
                        </div>
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
                            </tr>
                        </thead>
                    </table>
                </div>
            </div>
        </div>
        <!--end col-->
    </div>
@endsection

@section('js')
    <script>
        const tableId = `{{ $tableId }}`;
        const defColumn = @json($defColumns);
        $(function() {
            const tableId = `{{ $tableId }}`;
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
                        data: 'created_at',
                        render: (data) => dateFormatIndo(data)
                    },
                    {
                        data: 'app',
                        className: 'text-center'
                    },
                    {
                        data: 'endpoint',
                        className: 'text-center'
                    },
                    {
                        data: 'description',
                        className: 'text-center'
                    },
                    {
                        data: 'total_data',
                        className: 'text-center'
                    },
                    // {
                    //     data: 'duration',
                    //     className: 'text-center',
                    //     render: (data) => data ? `${(data / 1000).toFixed()} Detik` : '-'
                    // },

                ]
            })
        })
    </script>
@endsection
