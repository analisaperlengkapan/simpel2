@extends('layout.main')
@section('content')
@include('components.breadcums', $breadcums)
    <div class="row">
        <div class="col-lg-12">
            <div class="card">
                <div class="card-header">
                    <div class="d-flex align-items-center">
                        <div class="flex-grow-1">
                            <h5 class="card-title mb-0">Referensi - Kode Barang - Sub Sub Kelompok</h5>
                        </div>
                        <div class="flex-shrink-0">
                            
                        </div>
                    </div>
                </div>
                <div class="card-body">
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
        const tableId = `{{ $tableId }}`;
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
                    data: 'kdbrg',searchable: true
                },
                {
                    data: 'deskripsi',searchable: true
                },
                {
                    data: 'masa_manfaat_brg_baru',searchable: true
                },
                {
                    data: 'satuan',searchable: true
                }
            ]

        })
    })
</script>
@endsection
