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
                                <th>Nomor Permohonan @include('components.dtFilterInput',['index'=>0,'column' => 'Nama'])</th>
                                <th>Tanggal Permohonan @include('components.datepicker',['index'=>1,'name'=>'','className'=>'column-filter'])</th>
                                <th>Kategori @include('components.dtFilterInput',['index'=>2,'column'=>'kategori'])</th>
                                <th>Satker @include('components.dtFilterInput',['index' => 3,'column' => 'Satker'])</th>
                                <th>Aktifitas @include('components.dtFilterInput',['index' => 4,'column' => 'Aktifitas'])</th>
                                <th>File</th>
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
                <form action="{{ $controller }}" method="POST" class="ajaxForm" enctype="multipart/form-data">
                        @csrf
                        <div class="row">
                            <div class="col-lg-12">
                                <div class="mb-3">
                                    <label for="nama" class="form-label">Kategori</label>
                                    <select class="form-control" id="kategori" name="kategori">
                                        <option value="">-Pilih-</option>
                                        @foreach($kategori as $key=>$value)
                                            <option value="{{$key}}">{{$value}}</option>
                                        @endforeach
                                    </select>
                                </div>
                            </div>
                        </div>
                        <div class="row">
                            <div class="col-lg-12">
                                <div class="mb-3">
                                    <label for="nama" class="form-label">Jenis Barang</label>
                                    <input type="text" class="form-control" id="jenis_barang" name="jenis_barang" value="{{ $model['jenis_barang'] ?? '' }}"/>
                                </div>
                            </div>
                        </div>
                        <div class="row">
                            <div class="col-lg-12">
                                <div class="mb-3">
                                    <label for="nama" class="form-label">Nomor Surat Permohonan</label>
                                    <input type="text" class="form-control" id="no_surat_permohonan" name="no_surat_permohonan" value="{{ $model['no_surat_permohonan'] ?? '' }}"/>
                                </div>
                            </div>
                        </div>
                        <div class="row">
                            <div class="col-lg-12">
                                <div class="mb-3">
                                    <input type="hidden" id="id" name="id" value="" />
                                    @include('components.datepicker',[
                                        'value'=>'',
                                        'label'=>'Tanggal Surat Permohonan',
                                        'name'=>'tgl_surat_permohonan'
                                        ]
                                    )
                                </div>
                            </div>
                        </div>
                        <div class="row">
                            <div class="col-lg-12">
                                <div class="mb-3">
                                    <label for="nama" class="form-label">File Surat Permohonan</label>
                                    <input type="file" class="form-control" id="file_surat_permohonan" name="file_surat_permohonan" />
                                </div>
                            </div>
                        </div>
                        <div class="row mt-3">
                            <div class="col-lg-12">
                                <div class="hstack gap-2 justify-content-start">
                                    <button type="button" data-bs-dismiss="modal" class="btn btn-outline-primary">Kembali</a>
                                    <button type="submit" class="btn btn-primary"> Simpan </button>
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
@if(session('error'))
<script>
    notify({
            type: "warning",
            message: "{{ session('error') }}",
        });
</script>
@endif
    <script>
        const tableId = `{{ $tableId }}`;
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
                buttons: buttonPrintDt('Daftar Permohonan Penghapusan',"{{ URL::to($controller) }}"),
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
                columns: [
                    {
                        data: 'no_surat_permohonan'
                    },
                    {
                        data: 'tgl_surat_permohonan',
                        render :(data, type, row) =>{
                            return dateFormatIndo(row.tgl_surat_permohonan);
                        }
                    },
                    {
                        data: 'kategori'
                    },
                    {
                        data: 'inst_nama'
                    },
                    {
                        data: 'aktifitas'
                    },
                    {
                        data: 'id',
                        render: (data, type, row) => {
                            const url = `{{ $controller . '/downloadZip/' . '${data}' }}`;
                            return `<a href="${url}"><i class="ri-download-cloud-line"></i>Download</a>`;
                        }
                    },
                    {
                        data: 'id',
                        render: (data, type, row) => {
                            const url = `{{ $controller . '/' . '${data}' }}`;
                            if (`{{ $canCreate }}`) {
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

            // $('#simpan').on('click', function(){
            //     let id = $('#id').val();
            //     let kategori = $('#kategori').val();
            //     let tgl_pengajuan = $('#tgl_pengajuan').val();
            //     let nama = $('#nama').val();

            //     let data = {
            //         id,kategori,tgl_pengajuan,nama
            //     };

            //     $.ajax({
            //         method: "POST",
            //         url: `{{ $controller }}`,
            //         data: data,
            //         success: function(){
            //             notify({
            //                 type: "success",
            //                 message: "Data Berhasil Disimpan",
            //             });
            //         },
            //         error: showError,
            //     }).done(function( msg ) {
            //         dt.ajax.reload();
            //         $('#myModal').modal('hide');
            //     });
            // });

            $('#myModal').on('hidden.bs.modal', function(){
                $('#id').val("");
                $('#jenis_barang').val("");
                $('#kategori').val("");
                $('#no_surat_permohonan').val("");
                $('#tgl_surat_permohonan').val("");
                $('#file_surat_permohonan').val("");
                //$('#tgl_surat_permohonan').flatpickr().clear();
                $('input').val("");
            });
        })
    </script>
@endsection
