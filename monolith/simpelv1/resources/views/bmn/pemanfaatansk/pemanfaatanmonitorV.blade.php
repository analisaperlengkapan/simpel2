@extends('layout.main')
@section('content')
@include('components.breadcums', $breadcums)
    <div class="row">
        <div class="col-lg-12">
            <div class="card">
                <div class="card-header">
                    <div class="d-flex align-items-center">
                        <div class="flex-grow-1">
                            <h5 class="card-title mb-0">Monitoring Pemanfaatan BMN</h5>
                        </div>
                        <!--div class="flex-shrink-0">
                            <a href="{{ URL::to('/bmn/pemanfaatan/pemanfaatansk/create') }}" type="button" class="btn btn-success btn-label waves-effect waves-light"><i class="ri-add-line label-icon align-middle fs-16 me-2"></i>
                                Tambah
                            </a>
                        </div>-->
                    </div>
                    <div class="card-body">
                    <table id="{{ $tableId }}" class="display table table-bordered dt-responsive my-dt"
                        style="width:100%">
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
                                <!-- <th>File SK</th> -->
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
                url: "{{ URL::to('/bmn/pemanfaatan/pemanfaatanmonitor/gridData') }}",
                dataSrc: 'data',
                /* data: (data) => {
                        const filterType = $('#dt-filter-by').data('filterby');
                        if (filterType.trim() !== '') {
                            data.filterBy = filterType;
                        }
                    } */
            },
            columns: [
                {
                    data: 'inst_nama',searchable: true
                },
                {
                    data: 'nm_barang',searchable: true
                },
                {
                        data: 'tgl_awal',
                        render :(data, type, row) =>{
                            return dateFormatIndo(row.tgl_awal);
                        }
                },
                {
                        data: 'tgl_akhir',
                        render :(data, type, row) =>{
                            return dateFormatIndo(row.tgl_akhir);
                        }
                },
                // {
                //     "data": function (row, data, index, display) {
                //         const fileSK = row.file_sk ? `<a href="{{ url('${row.file_sk}') }}" class="text-nowrap" download terget="_blank"> <i class="ri-download-cloud-line"></i> File SK</a><br/>`:'-';
                //         let file = fileSK;
                //         return file;
                //     }
                // },
                {
                    data: 'id',
                    classname: "text-center",
                    render: (data, type, row) => {
                        const url = `{{ url('/bmn/pemanfaatan/pemanfaatanmonitor/${data}') }}`;
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
            ]

        });
    });
</script>
@endsection
