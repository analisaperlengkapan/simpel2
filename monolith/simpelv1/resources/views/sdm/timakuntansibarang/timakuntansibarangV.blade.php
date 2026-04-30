@extends('layout.main')
@section('content')
@include('components.breadcums', $breadcums)
    <div class="row">
        <div class="col-lg-12">
            <div class="card">
                <div class="card-header">
                    <div class="d-flex align-items-center">
                        <div class="flex-grow-1">
                            <h5 class="card-title mb-0">Pengajuan SK Tim Unit Akuntansi Barang</h5>
                        </div>
                        @if ($canCreate)
                        <div class="flex-shrink-0">
                            <a href="{{ URL::to('/sdm/timakuntansibarang/create') }}" type="button" class="btn btn-success btn-label waves-effect waves-light"><i class="ri-add-line label-icon align-middle fs-16 me-2"></i>
                                Tambah
                            </a>
                        </div>
                        @endif
                    </div>
                    <div class="row mt-3">
                        {{--
                        @include('components.dtSearchBox', [
                            'tableId' => $tableId,
                            'filterBy' => [
                                ['text' => 'Kode Satker', 'value' => 'a.kdsatker_keu'],
                                ['text' => 'Nama Satker', 'value' => 'b.inst_nama'],
                                ['text' => 'No. Surat', 'value' => 'a.no_surat'],
                            ],
                        ])
                        --}}
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
                                <th>File Pendukung</th>
                                <th>Aksi</th>

                                <!-- <th>Nama Satker</th>
                                <th>Jenis SK</th>
                                <th>No. Surat</th>
                                <th>Tgl. Surat</th>
                                <th>File SK</th>
                                <th width="15%">Aksi</th> -->
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
            dom: dtLayout,
            lengthMenu: [
                [10, 25, 50, 100, -1],
                [10, 25, 50, 100]
            ],
            buttons: buttonPrintDt('Daftar Pengajuan SK Tim Unit Akuntansi Barang',"{{ URL::to($controller) }}"),
            language: {
                url: `{{ url('/assets/js/datatable_bahasa.json') }}`,
            },
            ajax: {
                url: "{{ URL::to('/sdm/timakuntansibarang/gridData') }}",
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
            columns: [
                {
                    data: 'inst_nama'
                },
                {
                    data: 'jenis_sk'
                },
                {
                    data: 'no_surat'
                },
                {
                    data: 'tgl_surat',
                    render :(data, type, row) =>{
                        return dateFormatIndo(row.tgl_surat);
                    }
                },
                {
                    "data": function (row, data, index, display) {
                        const fileSK = row.file_sk ? `<a href="{{ url('${row.file_sk}') }}" class="text-nowrap" download terget="_blank"> <i class="ri-download-cloud-line"></i> File SK</a><br/>`:'-';
                        let file = fileSK;
                        return file;
                    }
                },
                {
                    data: 'id',
                    classname: "text-center",
                    render: (data, type, row) => {
                        const url = `{{ url('/sdm/timakuntansibarang/${data}') }}`;
                        //const urlCetak = `{{ url('/asset/tik/cetakLabel/${data}') }}`;
                        return `
                            <div class="text-center btn-group" role="group">
                                @include('components.showBtn', [
                                    'url' => '${url}',
                                    'className'=>'btn-icon'
                                ])
                                @if ($canCreate)
                                    @include('components.updateBtn', [
                                        'url' => '${url}/edit',
                                        'className'=>'btn-icon'
                                    ])
                                    @include('components.deleteBtn', [
                                        'url' => '${url}',
                                        'title' => '${row.no_surat}',
                                        'tableId'=>'${tableId}',
                                        'className'=>'btn-icon'
                                    ])
                                @endif
                            </div>
                        `;
                    },searchable: false
                }
            ]

        });
    });
</script>
@endsection
