@extends('layout.main')
@section('content')
@include('components.breadcums', $breadcums)
    <div class="row">
        <div class="col-lg-12">
            <div class="card">
                <div class="card-header">
                    <div class="d-flex align-items-center">
                        <div class="flex-grow-1">
                            <h5 class="card-title mb-0">Laporan Wasdal - Hapus PP BMN</h5>
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
    .dataTables_length {
        width: 30%;
        float: left;
        padding-top:10px;
    }

    .dataTables_paginate {
        width: 70%;
        float: right;
        padding-top:10px;
    }
</style>
@endsection

@section('js')
<script>
    const tableId = `{{ $tableId }}`;
    const defColumn = @json($defColumns);
    $(function() {
        $('#combo_satker_all').select2();

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
            buttons: buttonPrintDt('Daftar Hapus PP BMN',"{{ URL::to($controller) }}"),
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
            columns: [
                {
                    data: 'nama_satker',
                    width: '10%',
                    className:'dt-center'
                },
                {
                    data: 'no_sk',
                    width: '15%',
                    className:'dt-center'
                },
                {
                    data: 'tgl_sk',
                    className:'dt-center',
                    render: (data, type, row) =>
                            `${dateFormatIndo(row.tgl_sk)}`
                },
                {
                    data: 'jenis_aset',
                    width: '15%',
                },
                {
                    data: 'total_bmn',
                    className:'dt-right',
                },
                {
                    data: 'nilai_penetapan',
                    className:'dt-right',
                    render: $.fn.dataTable.render.number('.', '.', 0, '')
                },
                {
                    data: 'uraian_keputusan',
                    width: '20%',
                },
                {
                    data: 'tgl_rekam',
                    className:'dt-center',
                    render: (data, type, row) =>
                            `${dateFormatIndo(row.tgl_rekam)}`
                },
                {
                    data: 'id',
                    classname: "text-center",
                    render: (data, type, row) => {
                        const url = `{{ url('/bmn/wasdal/hapusppbmn/${data}') }}`;
                        //const urlCetak = `{{ url('/asset/tik/cetakLabel/${data}') }}`;
                        return `
                            <div class="text-center btn-group" role="group">
                                @include('components.showBtn', [
                                    'url' => '${url}',
                                    'className'=>'btn-icon'
                                ])

                            </div>
                        `;
                    },searchable: false
                }
            ],
        });
    });
</script>
@endsection
