@extends('layout.main')
@section('content')
@include('components.breadcums', $breadcums)
    <div class="row">
        <div class="col-lg-12">
            <div class="card">
                <div class="card-header">
                    <div class="d-flex align-items-center">
                        <div class="flex-grow-1">
                            <h5 class="card-title mb-0">Data Pengadaan Barang dan Jasa</h5>
                        </div>
                        
                        <div class="flex-shrink-0">
                            <a href="{{ URL::to('/pengadaan/barang-jasa/create') }}" type="button" class="btn btn-success btn-label waves-effect waves-light"><i class="ri-add-line label-icon align-middle fs-16 me-2"></i>
                                Tambah
                            </a>
                        </div>
                        
                    </div>
                </div>
                <div class="card-body">
                    <table id="{{ $tableId }}" class="display table table-bordered dt-responsive" style="width:100%">
                        <thead>
                            <tr>
                                <!-- <th>Nama Satker @include('components.dtFilterInput',['index' => 0,'column' => 'Satker'])</th> -->
                                <!-- <th>Jenis @include('components.dtFilterInput',['index' => 1,'column' => 'Satker'])</th> -->

                                <th>Nama Pengadaan @include('components.dtFilterInput',['index' => 1,'column' => 'Satker'])</th>
                                <th>Jenis Pengadaan @include('components.dtFilterInput',['index' => 2,'column' => ''])</th>
                                <th>Kode Barang @include('components.dtFilterInput',['index'=>3,'name'=>'', 'column' => '', 'className'=>'column-filter'])</th>
                                <th>Nilai Pengadaan</th>
                                <th width="15%">Aksi</th>
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
                url: "{{ URL::to('/pengadaan/barang-jasa/gridData') }}",
                dataSrc: 'data',
            },
            columns: [
                // {
                //     data: 'inst_nama',searchable: true
                // },
                // {
                //     data: 'jenis_kontrak',searchable: true
                // },
                {
                    data: 'nama_pengadaan',searchable: true,
                    width: '40%',
                },
                {
                    data: 'jenis_pengadaan',
                    width: '20%',
                    className: "dt-center",
                    render :(data, type, row) =>{
                        if(row.jenis_pengadaan == 1){
                            return "Pengadaan lebih dari 200jt";
                        }else{
                            return "Pengadaan s.d 200jt";
                        }
                        //return dateFormatIndo(row.tgl_kontrak);
                    }
                },
                {
                    data: 'kode_barang',searchable: true,
                    className: "dt-center",
                    width: '15%',
                },
                {
                    data: 'nilai',searchable: true,
                    width: '15%',
                },
                {
                    data: 'id',
                    className: "dt-center",
                    width: '15%',
                    render: (data, type, row) => {
                        const url = `{{ url('/pengadaan/barang-jasa/${data}') }}`;
                        //const urlCetak = `{{ url('/asset/tik/cetakLabel/${data}') }}`;
                        return `
                            <div class="text-center btn-group" role="group">
                                @include('components.showBtn', [
                                    'url' => '${url}',
                                    'className'=>'btn-icon'
                                ])
                                &nbsp;
                                @include('components.updateBtn', [
                                    'url' => '${url}/edit',
                                    'className'=>'btn-icon'
                                ])
                                &nbsp;
                                @include('components.deleteBtn', [
                                    'url' => '${url}',
                                    'title' => '${row.id}',
                                    'tableId'=>'${tableId}',
                                    'className'=>'btn-icon'
                                ])
                                
                            </div>
                        `;
                    },searchable: false
                }
            ],
            columnDefs: [
                {
                    targets: 3,
                    className: 'dt-body-right',
                    render: $.fn.dataTable.render.number('.', '.', 0, '')
                }
            ]

        });
    });
</script>
@endsection
