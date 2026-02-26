@extends('layout.main')
@section('content')
@include('components.breadcums', $breadcums)
<div class="row">
    <div class="row">
        <div class="col-lg-12">
            <div class="card">
                <div class="card-header">
                    <div class="d-flex align-items-center">
                        <div class="flex-grow-1">
                            <h5 class="card-title mb-0">Master Pegawai</h5>
                        </div>
                        <!--div class="flex-shrink-0">
                            <a href="{{ URL::to('/master/pegawai/create') }}" type="button" class="btn btn-success btn-label waves-effect waves-light"><i class="ri-add-line label-icon align-middle fs-16 me-2"></i>
                                Tambah
                            </a>
                        </div>-->
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
                            </tr>
                        </thead>
                    </table>
                </div>
            </div>
        </div>
        <!--end col-->
    </div>
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
            buttons: buttonPrintDt('Daftar Master Pegawai',"{{ URL::to($controller) }}"),
            language: {
                url: `{{ url('/assets/js/datatable_bahasa.json') }}`,
            },
            ajax: {
                url: "{{ URL::to($controller.'/gridData') }}",
                dataSrc: 'data',
                type:'POST'
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
                    data: 'peg_nip_baru',searchable: true
                },
                {
                    data: 'nama',searchable: true
                },
                {
                    data: 'pns_mail',searchable: true
                },
                {
                    data: 'pangkat',searchable: true
                },
                {
                    data: 'jabatan',searchable: true
                },
                {
                    data: 'alamat',searchable: false
                },
                {
                    data: 'satker',searchable: false
                },
                {
                    data: 'jenis_kelamin',searchable: true
                },
                {
                    data: 'agama',searchable: true
                },
                {
                    data: 'tempat_lahir',searchable: true
                },
                {
                    data: 'tgl_lahir',searchable: true
                },
                {
                    data: 'jenis',searchable: true
                },
                {
                    data: 'unitkerja_nama',searchable: true
                },
            ]

        })
    })
</script>
@endsection
