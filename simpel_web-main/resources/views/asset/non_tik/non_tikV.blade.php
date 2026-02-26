@extends('layout.main')
@section('content')
@include('components.breadcums', $breadcums)
    <div class="row">
        <div class="col-lg-12">
            <div class="card">
                <div class="card-header">
                    <div class="d-flex align-items-center">
                        <div class="flex-grow-1">
                            <h5 class="card-title mb-0">Aset Peralatan dan Mesin Non TIK</h5>
                        </div>
                    </div>
                </div>
                <div class="card-body">
                    @include('components.dtFilterBox', [
                        'columns' => $columns,
                        'selected'=> $defColumns
                    ])
                    <table id="{{ $tableId }}" class="display table table-bordered dt-responsive" style="width:100%">
                        <thead>
                            <tr>
                                @foreach ($columns as $index => $column)
                                <th>
                                    {{ $column }}
                                    @include('components.dtFilterInput',[
                                        'index' => $index,
                                        'column' => $column,
                                        ]
                                    )
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
        width: auto;
        float: right;
    }
    .dataTables_info {
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
    $('#' + tableId+' thead th').css({"background-color": "#405189", "color": "#ffffff","text-align":"center","text-transform":"uppercase"});
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
            buttons: buttonPrintDt('Daftar Aset Peralatan dan Mesin Non TIK',"{{ URL::to($controller) }}"),
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
                    data: 'id_satker_keu'
                },
                {
                    data: 'nama_satker'
                },
                {
                    data: 'kode_barang',searchable: true
                },
                {
                     data: 'nama_barang'
                },
                {
                    data: 'nup',searchable: false
                },
                {
                    data: 'kondisi',searchable: true
                },
                {
                    data: 'merk',searchable: false
                },
                {
                    data: 'tgl_rekam_pertama',
                    render :(data, type, row) =>{
                        return dateFormatIndo(row.tgl_rekam_pertama);
                    }
                },
                {
                    data: 'tgl_perolehan',
                    render :(data, type, row) =>{
                        return dateFormatIndo(row.tgl_perolehan);
                    }
                },
                {
                    data: 'nilai_perolehan_pertama',searchable: false
                },
                {
                    data: 'nilai_mutasi',searchable: false
                },
                {
                    data: 'nilai_perolehan',searchable: false
                },
                {
                    data: 'nilai_penyusutan',searchable: false
                },
                {
                    data: 'nilai_buku',searchable: false
                },
                {
                    data: 'kuantitas',searchable: false
                },
                {
                    data: 'jumlah_foto'
                },
                {
                    data: 'status_penggunaan',searchable: false
                },
                {
                    data: 'status_pengelolaan',searchable: false
                },
                {
                    data: 'no_psp',searchable: false
                },
                {
                    data: 'tgl_psp',searchable: false
                },
                {
                    data: 'jumlah_kib'
                },
                {
                    data: 'id',
                    render: (data, type, row) => {
                        const url = `{{ url('/asset/non_tik/${data}') }}`;
                        const urlCetak = `{{ url($controller.'/cetakLabel/${data}') }}`;
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
            ]

        })
    })
</script>
@endsection
