@extends('layout.main')
@section('content')
    @include('components.breadcums', $breadcums)
    <div class="row">
        <div class="col-lg-12">
            <div class="card">
                <div class="card-header">
                    <div class="d-flex align-items-center">
                        <div class="flex-grow-1">
                            <h5 class="card-title mb-0">Pengajuan SK Penetapan Status Penggunaan</h5>
                        </div>
                        @if ($canChange)
                            <div class="flex-shrink-0">
                                <a href="{{ URL::to('/bmn/penetapan/penetapansk/create') }}" type="button"
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
                dom: dtLayoutPrint,
                lengthMenu: [
                    [10, 25, 50, 100, -1],
                    [10, 25, 50, 100, 'Semua']
                ],
                buttons: buttonPrintDt('Daftar Penetapan SK',"{{ URL::to($controller) }}"),
                language: {
                    url: `{{ url('/assets/js/datatable_bahasa.json') }}`,
                },
                ajax: {
                    url: "{{ URL::to($controller.'/gridData') }}",
                    dataSrc: 'data',
                },
                responsive :false,
                scrollX:true,
                "initComplete": function(settings, json) {
                    for(i=0;i<(dt.columns().header().length)-1;i++){
                        if($.inArray( i, defColumn )==-1){
                            let column = dt.column(i);
                            column.visible(false);
                        }
                    }
                },
                columns: [{
                        data: 'inst_nama',
                    },
                    {
                        data: 'sp_no'
                    },
                    {
                        data: 'sp_tgl',
                        render: (data, type, row) => {
                            return dateFormatIndo(row.sp_tgl);
                        }
                    },
                    {
                        "data": 'sp_file',
                        className: "text-center",
                        render: (data, type, row) => {
                            const fileSK = row.sp_file ?
                                `<a href="{{ url('${row.sp_file}') }}" class="text-nowrap" download terget="_blank"> <i class="ri-download-cloud-line"></i> File Lampiran</a><br/>` :
                                '-';
                            let file = fileSK;
                            return file;
                        }
                    },
                    {
                        data: 'aktifitas_nama',
                        className: "text-center",
                    },
                    {
                        data: 'sk_no'
                    },
                    {
                        data: 'sk_tgl',
                        render: (data, type, row) => {
                            return dateFormatIndo(row.sk_tgl);
                        }
                    },
                    {
                        "data": 'sk_file',
                        className: "text-center",
                        render: (data, type, row) => {
                            const fileSK = row.sk_file ?
                                `<a href="{{ url('${row.sk_file}') }}" class="text-nowrap" download terget="_blank"> <i class="ri-download-cloud-line"></i> File SK</a><br/>` :
                                '-';
                            let file = fileSK;
                            return file;
                        },
                    },
                    {
                        data: 'id',
                        className: "text-center",
                        render: (data, type, row) => {
                            const url = `{{ url('/bmn/penetapan/penetapansk/${data}') }}`;
                            if (`{{ $canChange }}` == 1) {
                                return `
                                <div class="btn-group" role="group">
                                    @include('components.updateBtn', [
                                        'url' => '${url}/edit',
                                        'className' => 'btn-icon',
                                    ])
                                    @include('components.deleteBtn', [
                                        'url' => '${url}',
                                        'title' => '${row.id}',
                                        'tableId' => '${tableId}',
                                        'className' => 'btn-icon',
                                    ])
                                </div>
                            `;
                            } else {
                                return `
                                <div class="text-center btn-group" role="group">
                                    @include('components.showBtn', [
                                        'url' => '${url}',
                                        'className' => 'btn-icon',
                                    ])
                                </div>`;
                            }
                            //const urlCetak = `{{ url('/asset/tik/cetakLabel/${data}') }}`;
                        },
                        searchable: false
                    }
                ]

            });
        });
    </script>
@endsection
