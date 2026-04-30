@extends('layout.main')
@section('content')
@include('components.breadcums', $breadcums)
    <div class="row">
        <div class="col-lg-12">
            <div class="card">
                <div class="card-header">
                    <div class="d-flex align-items-center">
                        <div class="flex-grow-1">
                            <h5 class="card-title mb-0">Monitoring Penghapusan BMN</h5>
                        </div>
                        <!--div class="flex-shrink-0">
                            <a href="{{ URL::to('/bmn/penghapusan/penghapusansk/create') }}" type="button" class="btn btn-success btn-label waves-effect waves-light"><i class="ri-add-line label-icon align-middle fs-16 me-2"></i>
                                Tambah
                            </a>
                        </div>-->
                    </div>
                    <div class="card-body">
                    <table id="{{ $tableId }}" class="display table table-bordered dt-responsive my-dt"
                        style="width:100%">
                        <thead>
                            <tr>
                                <th>Nama Satker @include('components.dtFilterInput',['index' => 0,'column' => 'inst_nama'])</th>
                                <!-- <th>Jenis SK @include('components.dtFilterInput',['index' => 1,'column' => 'jenis_sk'])</th>
                                <th>No. Surat @include('components.dtFilterInput',['index' => 1,'column' => 'no_surat'])</th> -->
                                <th>Tgl. Surat</th>
                                <th>File SK</th>
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
    $(function() {

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
                url: "{{ URL::to('/bmn/penghapusan/penghapusansk/gridData') }}",
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
                // {
                //     data: 'jenis_sk',searchable: true
                // },
                // {
                //     data: 'no_surat',searchable: true
                // },
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
                        const url = `{{ url('/bmn/penghapusan/penghapusanmonitor/${data}') }}`;
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
