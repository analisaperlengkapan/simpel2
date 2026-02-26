@extends('layout.main')
@section('content')
@include('components.breadcums', $breadcums)
    <div class="row">
        <div class="col-lg-12">
            <div class="card">
                <div class="card-header">
                    <div class="d-flex align-items-center">
                        <div class="flex-grow-1">
                            <h5 class="card-title mb-0">Monitoring Penerimaan Hibah</h5>
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
                                <!-- <th>File SK</th> 
                                <th>Status</th>-->
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
                url: "{{ URL::to('/bmn/hibah/hibah/gridData') }}",
                dataSrc: 'data',
            },
            columns: [
                {
                    data: 'inst_nama',searchable: true
                },
                {
                    data: 'jenis_hibah',searchable: true
                },
                {
                    data: 'kategori',searchable: true
                },
                {
                    data: 'tgl_register',
                        render :(data, type, row) =>{
                            return dateFormatIndo(row.tgl_register);
                        }
                },
                {
                    data: 'hibah_ke',searchable: true
                },
                {
                    data: 'nilai',searchable: true
                },
                // {
                //     "data": function (row, data, index, display) {
                //         const fileSK = row.file_hibah ? `<a href="{{ url('${row.file_hibah}') }}" class="text-nowrap" download terget="_blank"> <i class="ri-download-cloud-line"></i> File SK</a><br/>`:'-';
                //         let file = fileSK;
                //         return file;
                //     }
                // },
                //{
                //    data: 'status',searchable: true
                //},
            ],
            columnDefs: [ 
            {
                targets: 5,
                className: 'dt-body-right',
                render: $.fn.dataTable.render.number('.', '.', 0, '')
            }
        ],

        });
    });
</script>
@endsection
