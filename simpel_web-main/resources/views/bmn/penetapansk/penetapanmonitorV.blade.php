@extends('layout.main')
@section('content')
    @include('components.breadcums', $breadcums)
    <div class="row">
        <div class="col-lg-12">
            <div class="card">
                <div class="card-header">
                    <div class="d-flex align-items-center">
                        <div class="flex-grow-1">
                            <h5 class="card-title mb-0">Monitoring PSP BMN</h5>
                        </div>
                        <div class="flex-shrink-0">
                            <button id="exportExcel" type="button"
                                class="btn btn-primary btn-label waves-effect waves-light"><i
                                    class="ri-download-line label-icon align-middle fs-16 me-2"></i>
                                Export Excel
                            </button>

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
                                <!-- <th>Aksi</th> -->
                            </tr>
                        </thead>
                    </table>
                </div>
            </div>
        </div>
        <!--end col-->
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
                dom: dtLayout,
                language: {
                    url: `{{ url('/assets/js/datatable_bahasa.json') }}`,
                },
                ajax: {
                    url: "{{ URL::to('/bmn/penetapan/penetapanmonitor/gridData') }}",
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
                        data: 'nm_aset',
                    },
                    {
                        data: 'nm_barang',
                    },
                    {
                        data: 'nup',
                        className: 'text-center',
                    },
                    {
                        data: 'nilai_perolehan',
                        className: 'text-end',
                        render: (data) => int2money(data)
                    },
                    {
                        data: 'no_psp',
                        className: 'text-center',
                    },
                    {
                        data: 'tgl_psp',
                        className: 'text-center',
                    },
                    {
                        data: 'no_siman',
                        className: 'text-center',
                        render: (data) => data ? 'Sudah' : "Belum"
                    },
                    {
                        data: 'inst_nama',
                        className: 'text-center',
                    },
                    /*
                    {
                        data: 'bmn_penetapan_id',
                        render: (data, _, row) => {
                            const url = `{{ url('/bmn/penetapan/penetapansk') }}/${data}`;
                            if (data) {
                                return `
                                <div class="text-center btn-group" role="group">
                                    @include('components.showBtn', [
                                        'url' => '${url}',
                                        'className' => 'btn-icon',
                                    ])
                                </div>`;
                            } else {
                                return '';
                            }
                        }
                    }
                    */
                ]

            });

            $('#exportExcel').on('click', function() {
                const data = dt.ajax.params();
                const qParams = $.param(data);
                const url = `{{ url('/bmn/penetapan/penetapanmonitor') . '/exportExcel' }}?${qParams}`;
                window.open(url, '_blank');
            })
        });
    </script>
@endsection
