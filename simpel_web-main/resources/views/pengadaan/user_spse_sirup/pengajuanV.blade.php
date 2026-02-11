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
                                <a href="#" type="button" data-bs-toggle="modal" data-bs-target="#myModal"
                                    class="btn btn-success btn-label waves-effect waves-light"><i
                                        class="ri-add-line label-icon align-middle fs-16 me-2"></i>
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
                    <table id="{{ $tableId }}" class="display table table-bordered dt-responsive my-dt"
                        style="width:100%">
                        <thead>
                            <tr>
                                <th>Tanggal Pengajuan @include('components.datepicker',['index'=>0,'name'=>'','className'=>'column-filter'])</th>
                                <th>Satker  @include('components.dtFilterInput',['index' => 1,'column' => 'Satker'])</th>
                                <th>Aktifitas @include('components.dtFilterInput',['index' => 2,'column' => 'Aktifitas'])</th>
                                <th>Aksi</th>
                            </tr>
                        </thead>
                    </table>
                </div>
            </div>
        </div>
    </div>
    <div id="myModal" class="modal fade" tabindex="-1" data-bs-focus="false" aria-labelledby="myModalLabel" aria-hidden="true" style="display: none;">
        <div class="modal-dialog">
            <div class="modal-content">
                <div class="modal-header">
                    <h5 class="modal-title" id="myModalLabel">Form Pengajuan</h5>
                    <button type="button" class="btn-close" data-bs-dismiss="modal" aria-label="Close"> </button>
                </div>
                <div class="modal-body">
                <form action="{{ $controller }}" method="POST" class="ajaxForm">
                        @csrf
                        <div class="row">
                            <div class="col-lg-12">
                                <div class="mb-12">
                                    <input type="hidden" id="id" name="id" value="">
                                    <input type="hidden" id="kategori" name="kategori" value="{{$kategori}}">
                                    @include('components.datepicker',[
                                        'value'=>'',
                                        'label'=>'Tanggal Pengajuan',
                                        'name'=>'tgl_pengajuan'
                                        ]
                                    )
                                </div>
                            </div>
                        </div>
                        <div class="row mt-3">
                            <div class="col-lg-12">
                                <div class="hstack gap-2 justify-content-start">
                                    <button type="button" data-bs-dismiss="modal" class="btn btn-outline-primary">Kembali</a>
                                    <button type="button" id="simpan" class="btn btn-primary"> Simpan </button>
                                </div>
                            </div>
                        </div>
                    </form>
                </div>
            </div><!-- /.modal-content -->
        </div><!-- /.modal-dialog -->
    </div><!-- /.modal -->
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
        const kategori = `{{ $kategori }}`;
        const defColumn = @json($defColumns);
        $(function() {
            const dt = $('#' + tableId).DataTable({
                serverSide: true,
                processing: true,
                deferRender: true,
                ordering: false,
                dom: dtLayoutPrint,
                lengthMenu: [
                    [10, 25, 50, 100, -1],
                    [10, 25, 50, 100, 'Semua']
                ],
                buttons: buttonPrintDt('Daftar Pengajuan User SPSE SIRUP',"{{ URL::to($controller) }}"),
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
                        data: 'tgl_pengajuan',
                        render :(data, type, row) =>{
                            return dateFormatIndo(row.tgl_pengajuan);
                        }
                    },
                    {
                        data: 'inst_nama'
                    },
                    {
                        data: 'aktifitas',
                        render :(data, type, row) =>{
                            const aktifitas = (row.ms_aktifitas_id==1008?data+' '+kategori.toUpperCase():data);
                            return data;
                        }
                    },
                    {
                        data: 'id',
                        render: (data, type, row) => {
                            const url = `{{ $controller . '/' . '${data}' }}`;
                            if (`{{ $canCreate }}`) {
                                if(row.ms_aktifitas_id == 1000 || row.ms_aktifitas_id == 1003 || row.ms_aktifitas_id == 1005 || row.ms_aktifitas_id == 1007){
                                    return `
                                        <div class="d-flex justify-content-center gap-2">
                                            @include('components.updateBtn', [
                                                'url' => '${url}',
                                            ])
                                            @include('components.deleteBtn', [
                                                'url' => '${url}',
                                                'title' => '',
                                                'tableId' => '${tableId}',
                                            ])
                                        </div>`;
                                }else if(row.ms_aktifitas_id == 1009){
                                    return `
                                        <div class="d-flex justify-content-center gap-2">
                                            @include('components.updateBtn', [
                                                'url' => '${url}',
                                            ])
                                        </div>`;
                                }else{
                                    return '';
                                }
                            } else {
                                return `
                                <div class="d-flex justify-content-center gap-2">
                                    @include('components.updateBtn', [
                                        'url' => '${url}',
                                    ])
                                </div>`;
                            }
                        },
                    }
                ]
            });

            $('#simpan').on('click', function(){
                let id = $('#id').val();
                let kategori = $('#kategori').val();
                let tgl_pengajuan = $('#tgl_pengajuan').val();

                let data = {
                    id,kategori,tgl_pengajuan
                };

                $.ajax({
                    method: "POST",
                    url: `{{ $controller }}`,
                    data: data,
                    success: function(){
                        notify({
                            type: "success",
                            message: "Data Berhasil Disimpan",
                        });
                    },
                    error: showError,
                }).done(function( msg ) {
                    dt.ajax.reload();
                    $('#myModal').modal('hide');
                });
            });

            $('#myModal').on('hidden.bs.modal', function(){
                $('#id').val("");
                $('#tgl_pengajuan').val("");
                $('#tgl_pengajuan').flatpickr().clear();
            });
        })
    </script>
@endsection
