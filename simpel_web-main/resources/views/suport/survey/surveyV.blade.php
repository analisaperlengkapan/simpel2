@extends('layout.main')
@section('content')
@include('components.breadcums', $breadcums)
    <div class="row">
        <div class="col-lg-12">
            <div class="card">
                <div class="card-header">
                    <div class="d-flex align-items-center">
                        <div class="flex-grow-1">
                            <h5 class="card-title mb-0">{{$kategoriJudul}}</h5>
                        </div>
						@if ($canCreate)
							<div class="flex-shrink-0">
								<a href="{{ URL::to('/suport/survey/create') }}" type="button" class="btn btn-success btn-label waves-effect waves-light"><i class="ri-add-line label-icon align-middle fs-16 me-2"></i>
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
                        data: 'tgl_survey',
                        render :(data, type, row) =>{
                            return dateFormatIndo(row.tgl_survey);
                        }
                    },				
                {
                    data: 'created_by',searchable: true
                },
                {
                    data: 'pertanyaan',searchable: true
                },
                {
                    data: 'deskripsi',searchable: true
                },                
                {
                    data: 'status',searchable: true
                },
                {
                    data: 'id',
                    render: (data, type, row) => {
                        const url = `{{ url('/suport/survey/${data}') }}`;
                        if (`{{ $candelete }}`) {
                            if (row.status !== 'Close') {
                                return `
                                <div>
                                    
                                    @include('components.updateBtn', [
                                        'url' => '${url}/edit',
                                    ])                                
                                </div>`;
                            }else{ 
                                return `
                                <div>
                                    @include('components.showBtn', [
                                        'url' => '${url}',
                                    ])  
                                                                   
                                </div>`;   
                            }
                            
                        } else {
                                if (row.status !== 'Close') {
                                return `
                                    <div class="text-center btn-group" role="group">
                                         @include('components.showBtn', [
                                            'url' => '${url}',
                                        ])
                                        @include('components.updateBtn', [
                                            'url' => '${url}/edit',
                                        ])
                                        @include('components.deleteBtn', [
                                            'url' => '${url}',
                                            'title' => '${row.judul}',
                                            'tableId'=>'${tableId}'
                                        ])
                                    </div>`;
                                }else{
                                    return `
                                    <div class="text-center btn-group" role="group">
                                        
                                        @include('components.showBtn', [
                                            'url' => '${url}',
                                        ])
                                      
                                    </div>`;
                                }
                            }
                    },searchable: false
                }
            ]

        })
    })
</script>
@endsection
