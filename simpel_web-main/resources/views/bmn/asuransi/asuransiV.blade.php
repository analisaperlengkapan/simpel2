@extends('layout.main')
@section('content')
    @include('components.breadcums', $breadcums)
    <div class="row">
        <div class="col-lg-12">
            <div class="card">
                <div class="card-header">
                    <div class="d-flex align-items-center">
                        <div class="flex-grow-1">
                            <h5 class="card-title mb-0">Data Obyek Asuransi</h5>
                        </div>

                        @if ($canChange)
                            <div class="flex-shrink-0">
                                <button id="exportExcel" type="button"
                                    class="btn btn-primary btn-label waves-effect waves-light"><i
                                        class="ri-download-line label-icon align-middle fs-16 me-2"></i>
                                    Export Excel
                                </button>

                            </div>
                        @endif
                    </div>
                    @if ($canChange)
                        <hr>
                        <form action="{{ $controller }}/importExcel" method="POST" class="ajaxForm">
                            @csrf
                            <div class="row">
                                <div class="col-lg-6">
                                    <div class="mb-3">
                                        <input class="form-control" type="file" name="excel_file" accept=".xlsx, .xls">
                                    </div>
                                </div>
                            </div>
                            <div class="row">
                                <div class="col-lg-6">
                                    <div class="mb-3">
                                        <button class="btn btn-warning btn-label waves-effect waves-light"><i
                                                class="ri-upload-line label-icon align-middle fs-16 me-2"></i>
                                            Import Excel
                                        </button>
                                    </div>
                                </div>
                            </div>
                        </form>
                    @endif
                </div>
                <div class="card-body">
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
                dom: dtLayout,
                language: {
                    url: `{{ url('/assets/js/datatable_bahasa.json') }}`,
                },
                ajax: {
                    url: "{{ url($controller . '/gridData') }}",
                    dataSrc: 'data',
                },
                columns: [{
                        data: 'nm_satker',
                        searchable: true,
                    },
                    {
                        data: 'polis_no',
                        searchable: true,
                        render: (data, type, row) => {
                            if (data) {
                                return '<font color="green">Sudah Asuransi</font>';
                            } else {
                                return '<font color="red">Belum Asuransi</font>';
                            }
                        }
                    },
                    {
                        data: 'kode_barang',
                        searchable: true,
                    },
                    {
                        data: 'nup',
                        searchable: false,
                    },
                    {
                        data: 'nm_barang',
                        searchable: true,
                    },
                    {
                        data: 'tipe',
                        render: (data) => data.toUpperCase()
                    },
                    {
                        data: 'filename',
                        searchable: false,
                        render: (data, type, row) => data ?
                            `<a href="{{ url('${data}') }}" class="text-nowrap" download terget="_blank"> <i class="ri-download-cloud-line"></i> Download</a><br/>` :
                            '-'
                    },
                    {
                        data: 'id',
                        className: "dt-center",
                        render: (data, type, row) => {
                            const url =
                                `{{ url('/bmn/asuransi/obyekasuransi/${data}') }}`;
                            const canChange = `{{ $canChange }}` == 1;
                            //const urlCetak = `{{ url('/asset/tik/cetakLabel/${data}') }}`;
                            return canChange ?
                                `@include('components.updateBtn', [
                                    'url' => '${url}',
                                    'className' => 'btn-icon',
                                ])` :
                                `@include('components.showBtn', [
                                    'url' => '${url}',
                                    'className' => 'btn-icon',
                                ])`;
                        },
                        searchable: false
                    }
                ],
                // columnDefs: [
                //     {
                //         targets: 3,
                //         className: 'dt-body-right',
                //         render: $.fn.dataTable.render.number('.', '.', 0, '')
                //     }
                // ]

            });

            $('#exportExcel').on('click', function() {
                const data = dt.ajax.params();
                const qParams = $.param(data);
                const url = `{{ url($controller) . '/exportExcel' }}?${qParams}`;
                window.open(url, '_blank');
            })
        });
    </script>
@endsection
