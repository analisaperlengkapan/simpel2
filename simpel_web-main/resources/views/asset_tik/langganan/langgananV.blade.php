@extends('layout.main')
@section('content')
@include('components.breadcums', $breadcums)
    <div class="row">
        <div class="col-lg-12">
            <div class="card">
                <div class="card-header">
                    <div class="d-flex align-items-center">
                        <div class="flex-grow-1">
                            <h5 class="card-title mb-0">Daftar Langganan Jasa Khusus TIK</h5>
                        </div>
                        @if ($canCreate)
                        <div class="flex-shrink-0">
                            <a href="{{ URL::to('/asset-tik/langganan/create') }}" type="button" class="btn btn-success btn-label waves-effect waves-light"><i class="ri-add-line label-icon align-middle fs-16 me-2"></i>
                                Tambah
                            </a>
                        </div>
                        @endif
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
                                <th>Nama Layanan</th>
                                <th>Penyedia</th>
                                <th>Tgl Mulai</th>
                                <th>Tgl Selesai</th>
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
                [10, 25, 50, 100],
                [10, 25, 50, 100]
            ],
            buttons: buttonPrintDt('Daftar Langganan Jasa Khusus TIK',"{{ URL::to($controller) }}"),
            language: {
                url: `{{ url('/assets/js/datatable_bahasa.json') }}`,
            },
            ajax: {
                url: "{{ URL::to('/asset-tik/langganan/gridData') }}",
                dataSrc: 'data',
                type: 'POST',
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
                    data: 'nm_layanan'
                },
                {
                    data: 'nm_perusahaan'
                },
                {
                    data: 'tgl_mulai',
                    classname: "dt-center",
                    render :(data, type, row) =>{
                        return dateFormatIndo(row.tgl_mulai)+' <br/> s/d <br/> '+dateFormatIndo(row.tgl_selesai);
                    }
                },
                {
                    className: "dt-right",
                    data: 'nilai',
                    render :(data, type, row) =>{
                        return int2money(row.nilai);
                    }
                },
                {
                    "data": function (row, data, index, display) {
                        const file_invoice = row.file_invoice ? `<a href="{{ url('${row.file_invoice}') }}" class="text-nowrap" download terget="_blank"> <i class="ri-download-cloud-line"></i> File Invoice</a><br/>`:'-';
                        let file = file_invoice;
                        return file;
                    }
                },
                {
                    data: 'id',
                    classname: "dt-center",
                    render: (data, type, row) => {
                        const url = `{{ url('/asset-tik/langganan/${data}') }}`;
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
                                    'title' => '${row.nm_lisensi}',
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
