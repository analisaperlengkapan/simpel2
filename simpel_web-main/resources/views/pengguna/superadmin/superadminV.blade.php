@extends('layout.main')
@section('content')
    @include('components.breadcums', $breadcums)
    <div class="row">
        <div class="col-lg-12">
            <div class="card">
                <div class="card-header">
                    <div class="d-flex align-items-center">
                        <div class="flex-grow-1">
                            <h5 class="card-title mb-0">Superadmin</h5>
                        </div>
                        <div class="flex-shrink-0">
                            <a href="{{ $controller . '/create' }}" type="button"
                                class="btn btn-success btn-label waves-effect waves-light"><i
                                    class="ri-add-line label-icon align-middle fs-16 me-2"></i>
                                Tambah
                            </a>
                        </div>
                    </div>
                </div>
                <div class="card-body">
                    <table id="dt-user" class="display table table-bordered dt-responsive my-dt" style="width:100%">
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
        <!--end col-->
    </div>
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
                dom: dtLayout,
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
                        data: 'username'
                    },
                    {
                        data: 'name'
                    },
                    {
                        data: 'email'
                    },
                    {
                        data: 'id',
                        render: (data, type, row) => {
                            const url = `{{ $controller . '/${data}' }}`;
                            return `
                            <div class="d-flex justify-content-center gap-2">
                                @include('components.updateBtn', [
                                    'url' => '${url}',
                                ])
                                @include('components.deleteBtn', [
                                    'url' => '${url}',
                                    'title' => '${row.name}',
                                    'tableId' => '${tableId}',
                                ])
                            </div>
                        `;
                        },
                    }
                ]
            })
        })
    </script>
@endsection
